//! Every resource name the book prints must resolve somewhere real.
//!
//! Read `crates/book-urns/src/lib.rs` first: it explains the two authorities and why a
//! gate that cannot tell them apart is worse than no gate at all.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;

use book_urns::{Context, Exemption, Mention, Scan, Vocabulary};
use ikigai_core::{Iri, Request, Resolution, Scope, Space, Verb};

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

/// The hosts this workspace builds — the two the book actually stands a reader in front of.
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
    ]
}

fn a_book_host_binds(iri: &str) -> Option<&'static str> {
    book_hosts()
        .into_iter()
        .find(|(_, space)| binds(space.as_ref(), iri))
        .map(|(name, _)| name)
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
        // are intercepted before space resolution, so they answer to `source` instead.
        let reachable = ["describe", "source"].iter().any(|verb| {
            Command::new("ikigai")
                .arg("--plain")
                .arg("-c")
                .arg(format!("{verb} {entry}"))
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
