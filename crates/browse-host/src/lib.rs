//! **Explain, with a mounted model** and **One-off review** — the code The gonk Book's
//! browser and review chapters teach.
//!
//! gonk serves the repositories it browses with explanations and reviews that a MODEL writes,
//! and it does not link a model: it mounts one, at `urn:llm:`, over a socket or QUIC. That is
//! what makes this crate possible. A kernel that binds `urn:llm:` to a stub is, to browse,
//! indistinguishable from one that mounts a real model, so every rule about the archive, the
//! version tag and the findings can be shown and checked without a network or a GPU.
//!
//! ## The resource map, as built
//!
//! * **Sets.** The roots (one here, `demo`); in a root, its files (`urn:repo:demo:file:{path}`).
//! * **Atoms that hold state.** The files themselves (the working tree, read by browse), and
//!   browse's store: the archive of derived explanations and review passes, the pending
//!   findings, and the published annotations. The store is an in-memory `oxigraph` dataset
//!   here; in gonk it is one graph of the durable store.
//! * **The model.** `urn:llm:stub:ask` and `urn:llm:stub:model`: an ATOM in the sense that
//!   matters (an input nothing here can derive), and the one piece of code this crate writes
//!   that answers a request. It is a stand-in for a mount, and says so.
//! * **Compositions.** `explain`, `review` and every finding decision are browse's, over the
//!   file, the model and the store.
//! * **Views.** `explain-status`, `explain-versions`, `review-status`, `findings` and
//!   `annotations`: read-only faces of the store that never call the model.
//!
//! Resource ratio: two resources written (the stub's two names), browse's family composed.
//!
//! Read the chapters: `mdbook serve books/gonk --open`.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use ikigai_browse::{ExplainConfig, Mount};
use ikigai_core::{
    Description, EndpointSpace, Exact, Fallback, FnEndpoint, Kernel, ReprType, Representation,
    Space, Verb,
};
use oxigraph::store::Store;

pub use ledger_host::{ask, ask_as};

/// The one file the chapters' repository holds, `src/lib.rs`, as first committed. It is
/// `demo/src/lib.rs` in this crate, which the chapters include from and a transcript writes out;
/// not a target of this crate, so it is never compiled here.
pub const INITIALS_RS: &str = include_str!("../demo/src/lib.rs");

// ANCHOR: stub
/// The stub model's provider: browse asks `urn:llm:{name}:ask` and reads the model's identity
/// at `urn:llm:{name}:model`, so a model is two names, and this crate binds both.
pub const PROVIDER: &str = "urn:llm:stub:ask";

/// What the stub says its model is. It becomes the `@…` half of every version tag.
pub const MODEL: &str = "stub-1";

/// The stub's explanation, whatever it is asked to explain.
pub const EXPLANATION: &str = "Defines `initials`, which joins the first letter of each \
space-separated word of a name into one string.";

// ANCHOR: review
/// The stub's review, in the grammar browse parses: three lines per finding, and a note that
/// ends in a period (without one, the last finding reads as cut off and is dropped).
pub const REVIEW: &str = "\
QUOTE: name.split(' ').map(|word| &word[..1]).collect()
SEVERITY: major
NOTE: Slicing the first byte panics on an empty word, so a name with two spaces in a row, or an empty name, crashes the caller.
QUOTE: pub fn initials(name: &str) -> String {
SEVERITY: minor
NOTE: Returning a String allocates on every call; return a &str instead.
";
// ANCHOR_END: review

/// A model that is not a model: a space binding [`PROVIDER`] and its `:model` name, answering
/// [`REVIEW`] when browse asks for a review and [`EXPLANATION`] otherwise, and counting every
/// ask in `calls` — which is how a test can say "no model call" and mean it.
pub fn stub_model(calls: Arc<AtomicUsize>) -> EndpointSpace {
    let ask = FnEndpoint::new("stub-model", move |inv| {
        calls.fetch_add(1, Ordering::SeqCst);
        // Browse's two kinds of ask differ in their system prompt; a review's says so.
        let system = inv.inline_str("system").unwrap_or("");
        let answer = if system.contains("reviewing") {
            REVIEW
        } else {
            EXPLANATION
        };
        Ok(text(answer))
    })
    .with_description(
        // The capability a real `urn:llm:` module declares: asking a model is network use.
        Description::new("stub-model")
            .verb(Verb::Source)
            .requires("urn:cap:net:*")
            .output("text/plain"),
    );
    let model = FnEndpoint::new("stub-model-id", |_| Ok(text(MODEL))).with_description(
        Description::new("stub-model-id")
            .verb(Verb::Source)
            .output("text/plain"),
    );
    EndpointSpace::new()
        .bind(Exact::new(PROVIDER), ask)
        .bind(Exact::new("urn:llm:stub:model"), model)
}
// ANCHOR_END: stub

fn text(body: &str) -> Representation {
    Representation::new(
        ReprType::new("text/plain").with_param("charset", "utf-8"),
        body.as_bytes().to_vec(),
    )
}

// ANCHOR: kernel
/// The host: browse over one root, `demo`, with its model-derived layers bound to the stub.
pub fn kernel(root: &Path, calls: Arc<AtomicUsize>) -> Kernel {
    // Where browse keeps the archive, the findings and the annotations.
    let store = Arc::new(Store::new().expect("an in-memory store opens"));
    let config = ExplainConfig::new(store)
        .file_provider(PROVIDER)
        .dir_provider(PROVIDER)
        .review_provider(PROVIDER)
        // A critical or major finding is otherwise sent to a second model, the judge, for a
        // verdict; this chapter's model is a stub, and a judge of a stub would judge nothing.
        .no_judge();
    let space = Mount::new([("demo".to_string(), root.to_path_buf())])
        // No config home: built-in styles, no files read from this machine.
        .config_home(None)
        .explain(config)
        .space();
    let spaces: Vec<Arc<dyn Space>> = vec![Arc::new(space), Arc::new(stub_model(calls))];
    Kernel::with_meta_renderer(
        Arc::new(Fallback::new(spaces)),
        Arc::new(ikigai_vocab::TurtleRenderer),
    )
    .with_clock(Arc::new(ledger_host::StoryClock::default()))
}
// ANCHOR_END: kernel

/// A repository of the book's own, in a fresh temporary directory, removed when dropped.
///
/// browse reads a root's working tree from the file system; only `urn:repo:{root}:state`
/// asks git. So for these chapters a directory is enough, and nothing needs git installed.
pub struct Scratch {
    root: PathBuf,
}

impl Scratch {
    /// A new directory holding `src/lib.rs` = [`INITIALS_RS`].
    pub fn new() -> Scratch {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "gonk-book-browse-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        let scratch = Scratch { root };
        scratch.write("src/lib.rs", INITIALS_RS);
        scratch
    }

    /// The directory, for [`kernel`].
    pub fn path(&self) -> &Path {
        &self.root
    }

    /// Write `text` to `rel` under the directory, creating what is missing.
    pub fn write(&self, rel: &str, text: &str) {
        let path = self.root.join(rel);
        std::fs::create_dir_all(path.parent().expect("a file has a parent"))
            .expect("the scratch directory can be created");
        std::fs::write(path, text).expect("the scratch file can be written");
    }
}

impl Default for Scratch {
    fn default() -> Self {
        Scratch::new()
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.root).ok();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ikigai_core::Verb;

    #[test]
    fn an_explanation_is_derived_once_and_then_read_from_the_archive() {
        let scratch = Scratch::new();
        let calls = Arc::new(AtomicUsize::new(0));
        let kernel = kernel(scratch.path(), calls.clone());
        let status = |k: &Kernel| {
            ask(
                k,
                Verb::Source,
                "urn:repo:demo:explain-status:src/lib.rs",
                &[],
            )
            .unwrap()
        };
        assert!(
            status(&kernel).starts_with("derive\t"),
            "{}",
            status(&kernel)
        );
        let first = ask(
            &kernel,
            Verb::Source,
            "urn:repo:demo:explain:src/lib.rs",
            &[],
        )
        .unwrap();
        assert_eq!(first.trim(), EXPLANATION);
        assert_eq!(calls.load(Ordering::SeqCst), 1);
        assert!(
            status(&kernel).starts_with("archived\t"),
            "{}",
            status(&kernel)
        );
        ask(
            &kernel,
            Verb::Source,
            "urn:repo:demo:explain:src/lib.rs",
            &[],
        )
        .unwrap();
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
