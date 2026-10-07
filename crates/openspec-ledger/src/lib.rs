//! **A spec in the repository** — the code The gonk Book's OpenSpec part teaches.
//!
//! [OpenSpec](https://github.com/Fission-AI/OpenSpec) keeps a project's requirements as Markdown
//! in an `openspec/` tree: current specs per capability, and changes in flight, each with a
//! proposal, delta specs and a task list. This crate lifts such a tree to RDF with
//! `ikigai-markdown`'s GENERIC lift and an OpenSpec MAPPING resource (`config/markdown/mappings/
//! openspec/mapping.ttl`, served at `urn:markdown:mapping:openspec`), loads it into the same store
//! the ledger lives in, and keeps each open task of each change as a ledger item ABOUT the
//! requirement it names.
//!
//! ## The resource map, as built
//!
//! * **Sets.** The tree's documents (one named graph each, `urn:markdown:doc:{path}`); the
//!   ledger's items.
//! * **Atoms that hold state.** The Markdown files (the tree is the source of truth, as OpenSpec
//!   says); the ledger's graph, held by the store. Nothing else: the spec graphs are DERIVED, and
//!   [`lift_tree`] replaces them wholesale from the files every time.
//! * **Compositions.** `urn:markdown:lift` (the generic lift, then the mapping's constructs) per
//!   document; `urn:iki:store:load` into one graph per document; the ledger's `append` and
//!   `close`.
//! * **Views.** The mapping's vocabulary over the union of those graphs, asked with
//!   `urn:iki:store:graph-select` ([`select`]); the ledger's `next`.
//! * **Code that is not a composition, and why.** [`sync_ledger`] decides WHICH tasks are open and
//!   which items are finished. That decision is a SPARQL query over the spec graphs and the
//!   ledger's graph together; acting on its rows is one keyed `append` or one `close` each, and
//!   there is no resource that turns a result set into ledger writes. [`lift_tree`] walks a directory,
//!   which is reading files outside the kernel; in a host they would be `urn:file:` resources.
//!
//! Resource ratio: no endpoint written; the store's, the ledger's and the lift's resources
//! composed, plus one mapping file and two queries.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use ikigai_core::{Error, Fallback, Kernel, Result, Space, Verb};
use ikigai_markdown::MappingHome;
use ikigai_store::DurableStore;

pub use ledger_host::ask;

/// The mapping's name as a resource: what `mapping=` is given.
pub const MAPPING: &str = "urn:markdown:mapping:openspec";

/// The ledger's graph, which the store keeps beside the spec graphs.
pub const LEDGER_GRAPH: &str = "urn:iki:ledger:graph:default";

/// Every spec graph's name starts with this: the lift's document prefix plus `openspec/`.
pub const SPEC_GRAPH_PREFIX: &str = "urn:markdown:doc:openspec/";

/// This crate's directory: it holds the committed `openspec/` tree and the config home.
pub fn crate_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

// ANCHOR: space
/// The store, the ledger, and the lift, as one space.
///
/// The lift finds mappings BY NAME under a config home, at
/// `<home>/markdown/mappings/<name>/mapping.ttl`. A host passes its own; this crate's is
/// `config/` beside it, so the book never reads the reader's `~/.config`.
pub fn space(mapping_home: &Path) -> Fallback {
    let store = DurableStore::in_memory().expect("an in-memory store opens");
    let home = Arc::new(MappingHome::new(Some(mapping_home.to_path_buf())));
    Fallback::new(vec![
        Arc::new(ikigai_store::space(store)) as Arc<dyn Space>,
        Arc::new(ikigai_ledger::space()) as Arc<dyn Space>,
        Arc::new(ikigai_markdown::space_with(home)) as Arc<dyn Space>,
    ])
}

/// The host: [`space`] over this crate's config home, a Meta renderer and the story clock.
pub fn kernel() -> Kernel {
    Kernel::with_meta_renderer(
        Arc::new(space(&crate_dir().join("config"))),
        Arc::new(ikigai_vocab::TurtleRenderer),
    )
    .with_clock(Arc::new(ledger_host::StoryClock::default()))
}
// ANCHOR_END: space

/// Every Markdown file under `<root>/openspec`, as `(path relative to root, text)`, sorted.
pub fn read_tree(root: &Path) -> Vec<(String, String)> {
    fn walk(dir: &Path, root: &Path, out: &mut Vec<(String, String)>) {
        let Ok(entries) = std::fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                walk(&path, root, out);
            } else if path.extension().is_some_and(|e| e == "md") {
                let rel = path.strip_prefix(root).expect("under root");
                let rel = rel.to_string_lossy().replace('\\', "/");
                let text = std::fs::read_to_string(&path).expect("a readable Markdown file");
                out.push((rel, text));
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.join("openspec"), root, &mut out);
    out.sort();
    out
}

// ANCHOR: lift
/// Make the store's spec graphs exactly the lift of `tree`: one graph per document, named by
/// its path, and no graph for a document that is gone (archiving a change MOVES its files, so
/// the old paths must disappear). Returns how many documents were lifted.
pub fn lift_tree(kernel: &Kernel, tree: &[(String, String)]) -> Result<usize> {
    for graph in spec_graphs(kernel)? {
        ask(
            kernel,
            Verb::Sink,
            "urn:iki:store:update",
            &[("content", &format!("DROP SILENT GRAPH <{graph}>"))],
        )?;
    }
    for (path, text) in tree {
        let quads = ask(
            kernel,
            Verb::Source,
            "urn:markdown:lift",
            &[("content", text), ("path", path), ("mapping", MAPPING)],
        )?;
        ask(
            kernel,
            Verb::Sink,
            "urn:iki:store:load",
            &[("content", &quads), ("format", "application/n-quads")],
        )?;
    }
    Ok(tree.len())
}
// ANCHOR_END: lift

/// The spec graphs the store holds now, sorted.
pub fn spec_graphs(kernel: &Kernel) -> Result<Vec<String>> {
    let graphs = ask(kernel, Verb::Source, "urn:iki:store:graphs", &[])?;
    Ok(graphs
        .lines()
        .map(str::trim)
        .filter(|g| g.starts_with(SPEC_GRAPH_PREFIX))
        .map(str::to_string)
        .collect())
}

// ANCHOR: select
/// A SPARQL SELECT over the union of every spec graph and the ledger's graph, answered as a
/// tab-separated table (one header line, terms in N-Triples spelling).
///
/// The union is the store's `graph=` set: one value, several IRIs, and the dataset becomes their
/// merge — so a query needs no `GRAPH` clause to join a task in `tasks.md` to the requirement it
/// names in a delta spec, or to the ledger item about it.
pub fn select(kernel: &Kernel, query: &str) -> Result<String> {
    let mut graphs = spec_graphs(kernel)?;
    graphs.push(LEDGER_GRAPH.to_string());
    ask(
        kernel,
        Verb::Source,
        "urn:iki:store:graph-select",
        &[
            ("query", query),
            ("graph", &graphs.join(" ")),
            ("as", "text/tab-separated-values"),
        ],
    )
}
// ANCHOR_END: select

/// The rows of a [`select`] answer, each a list of plain values: an IRI without its brackets, a
/// literal without its quotes (or its datatype), an unbound column as `""`.
pub fn rows(table: &str) -> Vec<Vec<String>> {
    table
        .lines()
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|line| line.split('\t').map(plain).collect())
        .collect()
}

fn plain(term: &str) -> String {
    let term = term.trim();
    if let Some(iri) = term.strip_prefix('<').and_then(|t| t.strip_suffix('>')) {
        return iri.to_string();
    }
    if let Some(rest) = term.strip_prefix('"') {
        let end = rest.rfind('"').unwrap_or(rest.len());
        return rest[..end].replace("\\\"", "\"").replace("\\\\", "\\");
    }
    term.to_string()
}

// ANCHOR: queries
/// Every open task of an unarchived change, with the requirement each names (if it names one the
/// change adds or modifies). Whether a task already HAS an item is not asked here: the ledger
/// answers that itself, when the task's IRI is the append's key.
pub const OPEN_TASKS: &str = r#"
PREFIX os: <http://example.org/openspec#>
SELECT DISTINCT ?task ?id ?number ?title ?requirement WHERE {
  ?task a os:Task ; os:change ?change ; os:number ?number ; os:title ?title ; os:done false .
  ?change os:id ?id .
  FILTER NOT EXISTS { ?change os:archivedOn ?archived }
  OPTIONAL {
    ?task os:cites ?name .
    ?change os:adds|os:modifies ?requirement .
    ?requirement os:name ?name .
  }
}
ORDER BY ?id ?number
"#;

/// Open ledger items about a task that is done, or whose change has been archived.
pub const FINISHED: &str = r#"
PREFIX os: <http://example.org/openspec#>
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
SELECT DISTINCT ?number ?id ?task ?why WHERE {
  ?item ledger:about ?task ; ledger:number ?number ; ledger:status ledger:open .
  ?task a os:Task ; os:change ?change .
  ?change os:id ?id .
  OPTIONAL { ?task os:done ?done }
  OPTIONAL { ?change os:archivedOn ?archived }
  FILTER(?done = true || BOUND(?archived))
  BIND(IF(BOUND(?archived), CONCAT("archived with ", ?id, " on ", STR(?archived)),
                            "checked off in tasks.md") AS ?why)
}
ORDER BY ?number
"#;
// ANCHOR_END: queries

// ANCHOR: sync
/// Bring the ledger in line with the tree: file an item for every open task, keyed by the task's
/// IRI and about the task and the requirement it names; close every item whose task is checked
/// off or whose change is archived. Idempotent, and safe to run twice at once: the key lets the
/// ledger file at most one item per task, in the same store update that checks for one.
/// Returns one line per write, in the ledger's own words.
pub fn sync_ledger(kernel: &Kernel) -> Result<Vec<String>> {
    let mut done = Vec::new();
    for row in rows(&select(kernel, OPEN_TASKS)?) {
        let [task, id, number, title, requirement] = &row[..] else {
            return Err(Error::Endpoint(format!(
                "an OPEN_TASKS row of {} columns",
                row.len()
            )));
        };
        let about = format!("{requirement} {task}");
        let answer = ask(
            kernel,
            Verb::Sink,
            "urn:iki:ledger:append",
            &[
                ("content", &format!("{id} {number}: {title}")),
                ("about", about.trim()),
                ("labels", &format!("openspec,{id}")),
                ("key", task),
            ],
        )?;
        // `#4 <iri>` when it filed; `#4 <iri> existing open` when an item already carries the
        // key, and nothing was written. Only a filing is a write.
        match answer.split_whitespace().nth(2) {
            None => done.push(format!("{} — {id} {number}", answer.trim())),
            Some("existing") => {}
            Some(other) => {
                return Err(Error::Endpoint(format!(
                    "an append answered `{}`: neither filed nor existing ({other})",
                    answer.trim()
                )))
            }
        }
    }
    for row in rows(&select(kernel, FINISHED)?) {
        let [number, _id, _task, why] = &row[..] else {
            return Err(Error::Endpoint(format!(
                "a FINISHED row of {} columns",
                row.len()
            )));
        };
        let closed = ask(
            kernel,
            Verb::Sink,
            "urn:iki:ledger:close",
            &[("item", number), ("content", why)],
        )?;
        done.push(format!("{} — {why}", closed.trim()));
    }
    Ok(done)
}
// ANCHOR_END: sync

/// A copy of this crate's `openspec/` tree in a fresh temporary directory, removed when
/// dropped — somewhere the chapters can check a box and archive a change.
pub struct Workspace {
    root: PathBuf,
}

impl Workspace {
    /// A copy of the committed tree.
    pub fn new() -> Workspace {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "gonk-book-openspec-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        for (path, text) in read_tree(&crate_dir()) {
            let to = root.join(&path);
            std::fs::create_dir_all(to.parent().expect("a parent")).expect("a scratch tree");
            std::fs::write(to, text).expect("a scratch file");
        }
        Workspace { root }
    }

    /// The directory holding `openspec/`.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The tree as it is now.
    pub fn tree(&self) -> Vec<(String, String)> {
        read_tree(&self.root)
    }

    /// Check off the task numbered `number` in `change`'s `tasks.md`.
    pub fn check(&self, change: &str, number: &str) {
        let path = self
            .root
            .join(format!("openspec/changes/{change}/tasks.md"));
        let text = std::fs::read_to_string(&path).expect("the change's tasks.md");
        let from = format!("- [ ] {number} ");
        assert!(text.contains(&from), "no open task {number} in {change}");
        std::fs::write(&path, text.replace(&from, &format!("- [x] {number} "))).expect("written");
    }

    /// Archive `change` as OpenSpec does, by MOVING its folder to
    /// `changes/archive/<date>-<change>/`. (OpenSpec's `archive` also merges the change's delta
    /// specs into the current ones; that half is OpenSpec's, and not repeated here.)
    pub fn archive(&self, change: &str, date: &str) {
        let changes = self.root.join("openspec/changes");
        std::fs::create_dir_all(changes.join("archive")).expect("an archive folder");
        std::fs::rename(
            changes.join(change),
            changes.join(format!("archive/{date}-{change}")),
        )
        .expect("the change folder moves");
    }
}

impl Default for Workspace {
    fn default() -> Self {
        Workspace::new()
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_committed_tree_lifts_and_files_three_tasks() {
        let kernel = kernel();
        assert_eq!(lift_tree(&kernel, &read_tree(&crate_dir())).unwrap(), 5);
        let filed = sync_ledger(&kernel).unwrap();
        assert_eq!(filed.len(), 3, "{filed:?}");
        assert!(sync_ledger(&kernel).unwrap().is_empty());
    }

    /// ★ The claim the chapter makes: syncs running at once file each task ONCE. With 0.3.0's
    /// check-then-append (a query for tasks no item is `about`, then an append) every thread
    /// could see a task unfiled before any of them filed it; the key makes the check and the
    /// filing one store update.
    #[test]
    fn concurrent_syncs_file_each_task_once() {
        let kernel = kernel();
        lift_tree(&kernel, &read_tree(&crate_dir())).unwrap();
        let barrier = std::sync::Barrier::new(4);
        let filed: usize = std::thread::scope(|scope| {
            let threads: Vec<_> = (0..4)
                .map(|_| {
                    scope.spawn(|| {
                        barrier.wait();
                        sync_ledger(&kernel).unwrap().len()
                    })
                })
                .collect();
            threads.into_iter().map(|t| t.join().unwrap()).sum()
        });
        assert_eq!(
            filed, 3,
            "three open tasks, filed once each across four syncs"
        );
        let all = ask(
            &kernel,
            Verb::Source,
            "urn:iki:ledger:items",
            &[("status", "all")],
        )
        .unwrap();
        assert!(all.contains("3 item(s)"), "{all}");
    }

    /// Part 5's sync as it was first written, before the key: ask which open tasks no item is
    /// `about`, then append each one with no key.
    fn check_then_append(kernel: &Kernel, between: impl FnOnce()) -> usize {
        let unfiled = OPEN_TASKS
            .replace(
                "PREFIX os: <http://example.org/openspec#>",
                "PREFIX os: <http://example.org/openspec#>\n\
                 PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>",
            )
            .replace(
                "FILTER NOT EXISTS { ?change os:archivedOn ?archived }",
                "FILTER NOT EXISTS { ?change os:archivedOn ?archived }\n  \
                 FILTER NOT EXISTS { ?item ledger:about ?task }",
            );
        let rows = rows(&select(kernel, &unfiled).unwrap());
        between();
        for row in &rows {
            let [task, id, number, title, _requirement] = &row[..] else {
                panic!("an unfiled-task row of {} columns", row.len());
            };
            ask(
                kernel,
                Verb::Sink,
                "urn:iki:ledger:append",
                &[
                    ("content", &format!("{id} {number}: {title}")),
                    ("about", task),
                ],
            )
            .unwrap();
        }
        rows.len()
    }

    /// ★ The chapter's figure for the sync without the key: four syncs started together, each
    /// asking first and appending after, file TWELVE items for three tasks. The barrier sits
    /// between every sync's question and its first append, which makes certain the interleaving
    /// the race only allows: every sync has seen every task unfiled before any of them files.
    /// The ledger pins the same shape for two callers and one key
    /// (`check_then_append_files_twice_when_two_callers_race`, in `ikigai-ledger`'s tests).
    #[test]
    fn check_then_append_syncs_file_every_task_once_per_sync() {
        let kernel = kernel();
        lift_tree(&kernel, &read_tree(&crate_dir())).unwrap();
        let barrier = std::sync::Barrier::new(4);
        std::thread::scope(|scope| {
            for _ in 0..4 {
                scope.spawn(|| {
                    check_then_append(&kernel, || {
                        barrier.wait();
                    })
                });
            }
        });
        let all = ask(
            &kernel,
            Verb::Source,
            "urn:iki:ledger:items",
            &[("status", "all")],
        )
        .unwrap();
        assert!(all.contains("12 item(s)"), "{all}");
    }

    /// The key is what the sync recognizes, not `about`: an item filed by hand ABOUT a task,
    /// without the task's IRI as its key, is not that task's item to the keyed sync, which files
    /// a second one.
    #[test]
    fn an_item_filed_about_a_task_without_its_key_is_filed_again() {
        let kernel = kernel();
        lift_tree(&kernel, &read_tree(&crate_dir())).unwrap();
        let task = "urn:openspec:change:add-undo:task:1.2";
        ask(
            &kernel,
            Verb::Sink,
            "urn:iki:ledger:append",
            &[("content", "Undo, filed by hand"), ("about", task)],
        )
        .unwrap();
        let filed = sync_ledger(&kernel).unwrap();
        assert_eq!(filed.len(), 3, "{filed:?}");
        let about = ask(
            &kernel,
            Verb::Source,
            "urn:iki:ledger:items",
            &[("about", task)],
        )
        .unwrap();
        assert!(about.contains("2 item(s)"), "{about}");
    }
}
