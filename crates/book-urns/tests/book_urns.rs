//! Every resource name the book prints must resolve somewhere real.
//!
//! Read `crates/book-urns/src/lib.rs` first: it explains the two authorities and why a
//! gate that cannot tell them apart is worse than no gate at all.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use book_urns::{Context, Exemption, Mention, Scan, Vocabulary};
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Request, Resolution, Scope, Space, Verb};

/// The repository root, from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .canonicalize()
        .expect("the workspace root is two levels up from this crate")
}

fn book_src() -> PathBuf {
    repo_root().join("books/ikigai/src")
}

fn scan() -> Scan {
    book_urns::scan_tree(&book_src()).expect("the book scans")
}

fn vocabulary() -> Vocabulary {
    let path = repo_root().join("books/ikigai/cli-vocabulary.txt");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("reading {}: {e}", path.display()));
    Vocabulary::parse(&text)
}

/// Is `iri` bound in this space? A `Source` resolution that hits is exactly the question
/// "does anything answer to this name", and it invokes nothing.
fn binds(space: &dyn Space, iri: &str) -> bool {
    let Ok(parsed) = Iri::parse(iri) else {
        return false;
    };
    let request = Request::new(Verb::Source, parsed);
    matches!(space.resolve(&request, &Scope::empty()), Resolution::Hit(_))
}

/// The hosts this workspace builds — the ones the book actually stands a reader in front
/// of: the two it quotes listings from, and the reader's own, which the exercises name.
fn book_hosts() -> Vec<(&'static str, Arc<dyn Space>)> {
    vec![
        (
            "hello_camel::space()",
            Arc::new(hello_camel::space()) as Arc<dyn Space>,
        ),
        (
            "loadable_module::host_space()",
            Arc::new(loadable_module::host_space()) as Arc<dyn Space>,
        ),
        (
            "your_endpoints::module::host_space()",
            Arc::new(your_endpoints::module::host_space()) as Arc<dyn Space>,
        ),
    ]
}

fn a_book_host_binds(iri: &str) -> Option<&'static str> {
    book_hosts()
        .into_iter()
        .find(|(_, space)| binds(space.as_ref(), iri))
        .map(|(name, _)| name)
        .or_else(|| the_kernel_answers(iri).then_some("hello_camel::kernel()"))
}

/// The third authority: names the **kernel** answers itself. `urn:kernel:*` is intercepted
/// before any space is consulted, so no `Space` binds it and [`binds`] cannot see it — a
/// Rust example naming `urn:kernel:catalog` would fail the gate while working perfectly.
/// So ask a real kernel: the tutorial host, which carries the renderer the catalog needs.
///
/// "Answers" means the kernel recognized the operation. An unknown `urn:kernel:` suffix
/// comes back `Unresolved`; anything else — a representation, a missing argument, a
/// denial — is the kernel saying the name is one of its own. `Source` is tried first, then
/// `Sink` for the write-only ones (`urn:kernel:cut`), which need a thread to name.
fn the_kernel_answers(iri: &str) -> bool {
    if !iri.starts_with("urn:kernel:") {
        return false;
    }
    let Ok(parsed) = Iri::parse(iri) else {
        return false;
    };
    let kernel = hello_camel::kernel();
    let root = Capability::root();
    let recognized = |request: Request| {
        !matches!(
            block_on(kernel.issue(request, &root)),
            Err(Error::Unresolved(_))
        )
    };
    recognized(Request::new(Verb::Source, parsed.clone()))
        || recognized(
            Request::new(Verb::Sink, parsed)
                .with_arg("thread", ArgRef::Inline(b"urn-gate-probe".to_vec())),
        )
}

/// A family is reachable if the book, the manifest, or a prefix-routed space can show
/// something under it. `urn:greet:` is routed by a `ModuleSpace`, so a probe under it
/// hits; `urn:iki:tutorial:` binds exactly, so the evidence is a sibling name instead.
fn family_is_reachable(prefix: &str, scan: &Scan, vocabulary: &Vocabulary) -> bool {
    if vocabulary.covers_family(prefix) {
        return true;
    }
    if a_book_host_binds(&format!("{prefix}urn-gate-probe")).is_some() {
        return true;
    }
    scan.mentions.iter().any(|m| {
        !m.is_prefix
            && m.urn.starts_with(prefix)
            && (vocabulary.covers(&m.urn) || a_book_host_binds(&m.urn).is_some())
    })
}

/// Capability scopes are authority, not resources: nothing resolves `urn:cap:fs:read:…`,
/// and the alias table is deliberately never applied to them. They are checked by the
/// crates that enforce them, not here.
fn is_capability_scope(urn: &str) -> bool {
    urn.starts_with("urn:cap:")
}

/// A node in the description graph — `<urn:ikigai:endpoint:camel-case:input:in>` — is a
/// name too, and the prose quotes several. Nothing *resolves* one: it is the skolem IRI
/// `ikigai-vocab` mints for an endpoint, an input or an action so the catalog has no
/// blank nodes. The authority for such a name is the catalog itself, so the check is
/// that the tutorial host's catalog really contains it, verbatim, in angle brackets.
/// A renamed skolem prefix or a renamed endpoint then fails here rather than leaving
/// the book quoting a graph that no longer exists.
fn is_graph_node(urn: &str) -> bool {
    urn.starts_with("urn:ikigai:endpoint:")
}

fn the_catalog_contains(node: &str) -> bool {
    use std::sync::OnceLock;
    static CATALOG: OnceLock<String> = OnceLock::new();
    let catalog = CATALOG.get_or_init(|| {
        let kernel = hello_camel::kernel();
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:kernel:catalog").expect("a constant IRI"),
        );
        let repr = block_on(kernel.issue(request, &Capability::root()))
            .expect("the tutorial host renders its catalog");
        String::from_utf8(repr.bytes).expect("Turtle is UTF-8")
    });
    catalog.contains(&format!("<{node}>"))
}

fn describe(mention: &Mention) -> String {
    let where_from = match mention.context {
        Context::Cli => "an `ikigai …` shell example",
        Context::Rust => "a Rust example",
        Context::Prose => "prose",
    };
    format!("{} ({where_from}) names {}", mention.site(), mention.urn)
}

#[test]
fn every_name_the_book_prints_resolves_somewhere_real() {
    let scan = scan();
    let vocabulary = vocabulary();
    let mut failures: Vec<String> = Vec::new();

    assert!(
        scan.mentions.len() > 20,
        "the scanner found only {} names in {} — it is not reading the book",
        scan.mentions.len(),
        book_src().display()
    );

    for mention in &scan.mentions {
        if is_capability_scope(&mention.urn) {
            continue;
        }
        if is_graph_node(&mention.urn) {
            if !the_catalog_contains(&mention.urn) {
                failures.push(format!(
                    "{}: quotes a catalog node the tutorial host's catalog does not \
                     contain.",
                    describe(mention)
                ));
            }
            continue;
        }

        match scan.exemption(&mention.urn).map(|d| d.kind) {
            Some(Exemption::Illustration) => continue,
            Some(Exemption::Unbound) => {
                if let Some(host) = a_book_host_binds(&mention.urn) {
                    failures.push(format!(
                        "{}: declared `unbound`, but {host} binds it. The prose says this \
                         name is gone; it is not.",
                        describe(mention)
                    ));
                }
                if vocabulary.covers(&mention.urn) {
                    failures.push(format!(
                        "{}: declared `unbound`, but books/ikigai/cli-vocabulary.txt \
                         claims the CLI resolves it.",
                        describe(mention)
                    ));
                }
                continue;
            }
            None => {}
        }

        if mention.is_prefix {
            if !family_is_reachable(&mention.urn, &scan, &vocabulary) {
                failures.push(format!(
                    "{}: nothing the book can reach lives under that prefix.",
                    describe(mention)
                ));
            }
            continue;
        }

        match mention.context {
            // A shell example runs against the CLI, which this workspace does not build.
            // Only the manifest can answer, and moving the goalposts to "some host, any
            // host" would let a name that exists only here look like a working command.
            Context::Cli => {
                if !vocabulary.covers(&mention.urn) {
                    failures.push(format!(
                        "{}: not in books/ikigai/cli-vocabulary.txt. If the CLI really \
                         resolves it, add it there and re-run the probe test; if it does \
                         not, the command in the book does not work.",
                        describe(mention)
                    ));
                }
            }
            // A Rust example runs in a host built right here.
            Context::Rust => {
                if a_book_host_binds(&mention.urn).is_none() {
                    failures.push(format!(
                        "{}: no host this workspace builds binds it.",
                        describe(mention)
                    ));
                }
            }
            // Prose may truthfully name anything the reader can reach either way.
            Context::Prose => {
                if a_book_host_binds(&mention.urn).is_none() && !vocabulary.covers(&mention.urn) {
                    failures.push(format!(
                        "{}: neither a host this workspace builds nor \
                         books/ikigai/cli-vocabulary.txt has it.",
                        describe(mention)
                    ));
                }
            }
        }
    }

    assert!(
        failures.is_empty(),
        "the book names {} resource{} nothing can resolve:\n  {}\n\nDeclare a deliberate \
         exception next to the prose that needs it:\n  <!-- urn-gate: unbound urn:x:y — \
         why -->\n  <!-- urn-gate: illustration urn:x:y — why -->",
        failures.len(),
        if failures.len() == 1 { "" } else { "s" },
        failures.join("\n  ")
    );
}

#[test]
fn no_exemption_outlives_the_prose_that_needed_it() {
    let scan = scan();
    let stale: Vec<String> = scan
        .directives
        .iter()
        .filter(|d| !scan.mentions.iter().any(|m| m.urn == d.urn))
        .map(|d| {
            format!(
                "{}:{} exempts {}, which the book no longer names",
                d.file, d.line, d.urn
            )
        })
        .collect();

    assert!(
        stale.is_empty(),
        "stale urn-gate directives — delete them:\n  {}",
        stale.join("\n  ")
    );
}

#[test]
fn the_book_still_teaches_the_name_this_workspace_binds() {
    // A canary for the scanner itself: if the classifier ever silently stopped finding
    // anything, every other assertion above would pass vacuously.
    let scan = scan();
    assert!(
        scan.mentions
            .iter()
            .any(|m| m.urn == "urn:iki:tutorial:camel-case" && m.context == Context::Rust),
        "no Rust example names urn:iki:tutorial:camel-case — either the book changed or \
         the scanner stopped seeing Rust fences"
    );
    assert!(
        scan.mentions.iter().any(|m| m.context == Context::Cli),
        "no `ikigai …` shell example was classified as CLI — the thinnest part of the \
         book's gate is unguarded again"
    );
}

/// A cross-reference must be checkable, and a chapter number is not.
///
/// The book's five stale numbers all pointed at the right file — merging two books
/// renumbered the second one, and the prose kept the old numbers — so every link resolved
/// and nothing complained. This is the check that would have complained.
///
/// Two rules, and they differ because only one of the two forms can be verified:
///
/// * `[chapter N](target)` is checked against `SUMMARY.md`. A number that disagrees with
///   the real one fails; so does one pointing at a chapter mdbook does not number.
/// * a bare `chapter N` is refused outright — there is no target, so there is nothing to
///   check it against, and it will be wrong after some future reorder with no way to know.
///
/// The fix for both is the same and it is why neither rule is a nuisance: name the
/// chapter. `[Where this actually stands](../modules/status.md)` says where to go and what
/// it is called, and a reorder cannot make either half false.
#[test]
fn a_cross_reference_names_a_chapter_rather_than_numbering_it() {
    let (linked, bare) = book_urns::chapter_numbers_in_tree(&book_src()).expect("the book scans");
    let summary = std::fs::read_to_string(book_src().join("SUMMARY.md")).expect("SUMMARY.md");
    let chapters = book_urns::summary_chapters(&summary);

    assert!(
        chapters.len() > 5,
        "SUMMARY.md parsed to {} numbered chapters — the parser is not reading it",
        chapters.len()
    );

    let mut failures: Vec<String> = Vec::new();
    for link in &linked {
        match book_urns::chapter_for(link, &chapters) {
            Some(chapter) if chapter.number == link.number => failures.push(format!(
                "{}:{}: `[chapter {}]` is right today and wrong after the next reorder. \
                 Write `[{}]({})`.",
                link.file, link.line, link.number, chapter.title, link.target
            )),
            Some(chapter) => failures.push(format!(
                "{}:{}: the prose says chapter {}, but {} is chapter {} — \"{}\". Write \
                 `[{}]({})`.",
                link.file,
                link.line,
                link.number,
                link.target,
                chapter.number,
                chapter.title,
                chapter.title,
                link.target
            )),
            None => failures.push(format!(
                "{}:{}: `[chapter {}]({})` points at something SUMMARY.md does not number \
                 (a prefix chapter, or a file it does not list).",
                link.file, link.line, link.number, link.target
            )),
        }
    }
    for number in &bare {
        failures.push(format!(
            "{}:{}: `{}` has no link, so nothing can check it. Name the chapter instead.",
            number.file, number.line, number.text
        ));
    }

    assert!(
        failures.is_empty(),
        "the book numbers a chapter where it could name one:\n  {}",
        failures.join("\n  ")
    );
}

/// The reader's crate is never quoted. `crates/your-endpoints` exists so the exercises
/// have a file to edit that is NOT one the chapters include listings from — doing exercise
/// 1 used to rewrite the chapter that set it. That property holds only as long as no
/// `{{#include}}` points there, and an include is one line anyone can add in good faith,
/// so this is checked rather than remembered.
#[test]
fn no_chapter_includes_a_listing_from_the_readers_crate() {
    let root = book_src();
    let mut files = Vec::new();
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).expect("the book source is readable") {
            let path = entry.expect("a directory entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                out.push(path);
            }
        }
    }
    walk(&root, &mut files);
    assert!(files.len() > 10, "the walk is not reading the book");

    let offenders: Vec<String> = files
        .iter()
        .flat_map(|path| {
            let text = std::fs::read_to_string(path).expect("a chapter is readable");
            let file = path
                .strip_prefix(&root)
                .expect("under the book root")
                .to_string_lossy()
                .into_owned();
            text.lines()
                .enumerate()
                .filter(|(_, line)| line.contains("#include") && line.contains("your-endpoints"))
                .map(|(index, _)| format!("{file}:{}", index + 1))
                .collect::<Vec<_>>()
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "a chapter includes a listing from crates/your-endpoints, the file the exercises \
         edit — put the listing in the crate the chapter teaches instead:\n  {}",
        offenders.join("\n  ")
    );
}

/// The instrument, not the gate: probe a real `ikigai` for every exact manifest entry.
///
/// This cannot be a gate. CI has no `ikigai` binary, and a check that silently does not
/// run is the failure mode the gate above exists to avoid — so rather than skipping
/// quietly it is `#[ignore]`d, which says so out loud in the test output.
///
///     cargo install ikigai-cli --locked
///     cargo test -p book-urns -- --ignored --nocapture
#[test]
#[ignore = "needs an `ikigai` binary on PATH; run it by hand when the manifest or the CLI changes"]
fn the_cli_vocabulary_matches_an_installed_binary() {
    let vocabulary = vocabulary();

    let version = Command::new("ikigai")
        .arg("--plain")
        .arg("-c")
        .arg("help")
        .output();
    let Ok(probe) = version else {
        panic!(
            "no `ikigai` on PATH. Install it with `cargo install ikigai-cli --locked`; \
             this test is the only thing that can tell you whether \
             books/ikigai/cli-vocabulary.txt is still true."
        );
    };
    assert!(probe.status.success(), "`ikigai -c help` failed");

    let mut missing = Vec::new();
    for entry in vocabulary.exact_entries() {
        // `describe` answers for anything bound in a space; the kernel's own resources
        // are intercepted before space resolution, so they answer to `source` instead —
        // and one of them, `urn:kernel:cut`, answers only to `sink`. A sink is a write,
        // so it is tried for the kernel's own family alone: cutting a thread in a
        // one-shot process that exits on the next line changes nothing, whereas a
        // probe that sank into `urn:file:` or `urn:llm:config` would.
        let mut commands = vec![format!("describe {entry}"), format!("source {entry}")];
        if entry.starts_with("urn:kernel:") {
            commands.push(format!("sink {entry} urn-gate-probe"));
        }
        let reachable = commands.iter().any(|command| {
            Command::new("ikigai")
                .arg("--plain")
                .arg("-c")
                .arg(command)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
        });
        println!("{} {entry}", if reachable { "ok  " } else { "MISS" });
        if !reachable {
            missing.push(entry.clone());
        }
    }

    for prefix in vocabulary.prefix_entries() {
        println!("skip {prefix}* — a templated family; probe a concrete name by hand");
    }

    assert!(
        missing.is_empty(),
        "books/ikigai/cli-vocabulary.txt claims names this binary does not resolve: {}",
        missing.join(", ")
    );
}
