//! The book's claims about OTHER repositories, as data — and the probe that re-checks them.
//!
//! `book_urns.rs` checks names; this checks sentences. The status chapters assert facts
//! about `ikigai-module`, `ikigai-cli` and the browser demo — "the host enforces what a
//! module declares", "no host embeds wasmtime", "an MCP session with no grant runs as
//! root" — and nothing this workspace compiles can prove one of those. They are claims
//! about code on another release schedule, and they rot exactly the way a version pin
//! that nobody re-reads rots: silently, while the sentence stays confident.
//!
//! Two rules make them checkable. Every such claim in the prose carries **the version it
//! was true at**, and this file holds the same claims as a table — crate, version, the
//! file inside the published crate, and a symbol that must be present (or absent) there.
//!
//! * The always-on test checks the prose still carries each stamp the table names, so the
//!   table and the chapters cannot drift apart.
//! * The `#[ignore]`d probe downloads each crate from crates.io at exactly that version
//!   and greps for the symbol. It needs the network, so — like the CLI vocabulary probe —
//!   it is ignored rather than silently skipped, and it says so in the test output.
//!
//!     cargo test -p book-urns --test book_claims -- --ignored --nocapture
//!
//! When a probe fails, the claim is not "wrong": it was true at the stamped version and
//! stopped being true at a later one. The fix is to re-stamp and rewrite the sentence —
//! which is what happened when `ikigai-module` 0.2.0 started enforcing a module endpoint's
//! declared capability and the chapter that said "the host does not enforce it" became
//! false. This file exists so that is a diff, not a discovery.

use std::path::{Path, PathBuf};
use std::process::Command;

/// Where a claim's evidence lives.
#[derive(Debug, Clone, Copy)]
enum Source {
    /// A crate on crates.io, at an exact version.
    Crate {
        name: &'static str,
        version: &'static str,
    },
    /// A GitHub repository at an exact commit — for the browser demo, which is not a crate.
    GitHub {
        repo: &'static str,
        sha: &'static str,
    },
}

impl Source {
    fn label(&self) -> String {
        match self {
            Source::Crate { name, version } => format!("{name} {version}"),
            Source::GitHub { repo, sha } => format!("{repo}@{}", &sha[..7]),
        }
    }

    fn url(&self) -> String {
        match self {
            Source::Crate { name, version } => {
                format!("https://static.crates.io/crates/{name}/{name}-{version}.crate")
            }
            Source::GitHub { repo, sha } => {
                format!("https://github.com/ikigai-rs/{repo}/archive/{sha}.tar.gz")
            }
        }
    }
}

/// One sentence the book asserts about somebody else's code.
struct Claim {
    /// The chapter, relative to `books/ikigai/src`.
    chapter: &'static str,
    /// The stamp the prose must carry, verbatim.
    stamp: &'static str,
    /// What the sentence says.
    says: &'static str,
    /// Where the evidence is.
    source: Source,
    /// The file inside the unpacked source (after its top-level directory).
    file: &'static str,
    /// The symbol or phrase to look for.
    needle: &'static str,
    /// Whether the claim is that it is there, or that it is not.
    present: bool,
}

/// The status chapter's stamp: re-stamped from 0.3.1 on 2026-10-10 (ledger #1110).
const MODULE: Source = Source::Crate {
    name: "ikigai-module",
    version: "0.3.2",
};
/// The machine-client chapter's stamp, where `ikigai mcp` with no grant warns.
const CLI_028: Source = Source::Crate {
    name: "ikigai-cli",
    version: "0.1.28",
};
/// The status chapter's manifest check: re-stamped from 0.1.28 on 2026-10-10 (ledger #1110).
const CLI_045: Source = Source::Crate {
    name: "ikigai-cli",
    version: "0.1.45",
};
/// The CLI the configuration chapter is stamped at: re-stamped from 0.1.28 when the passkey
/// relying party stopped being an environment variable (ledger #926).
const EMBEDDED_043: Source = Source::Crate {
    name: "ikigai-embedded",
    version: "0.1.43",
};
const CLI_043: Source = Source::Crate {
    name: "ikigai-cli",
    version: "0.1.43",
};
/// The status chapter's time-budget sentence (2026-10-10, ledger #1110).
const XSLT_030: Source = Source::Crate {
    name: "ikigai-xslt",
    version: "0.3.0",
};
const LISP: Source = Source::Crate {
    name: "ikigai-lisp",
    version: "0.2.0",
};
/// The browser demo's `main` on 2026-10-10 (ledger #1110); 4ae3903 before that.
const WEB_DEMO: Source = Source::GitHub {
    repo: "ikigai-web-demo",
    sha: "24a13fb445b26dd877224530919e84726063cb35",
};

/// The claims. The stamp is what the prose must say; the rest is what the probe checks.
const CLAIMS: &[Claim] = &[
    Claim {
        chapter: "modules/status.md",
        stamp: "ikigai-module` 0.3.2's own first line still calls it",
        says: "the crate's first line still says Phase 1: in-process proof",
        source: MODULE,
        file: "src/lib.rs",
        needle: "Phase 1: in-process proof",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "**enforced** since 0.2.0",
        says: "a module endpoint's declared requires is enforced across the boundary",
        source: MODULE,
        file: "src/lib.rs",
        needle: "fn a_module_endpoints_declared_requirement_is_enforced_like_a_linked_ones",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`ModuleFloor` on the mount",
        says: "ModuleSpace::new takes the mount's floor",
        source: MODULE,
        file: "src/lib.rs",
        needle: "pub struct ModuleFloor",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`UdsTransport` + `serve`",
        says: "an out-of-process Unix-socket transport is built",
        source: MODULE,
        file: "src/lib.rs",
        needle: "pub struct UdsTransport",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "no `wasmtime` in its manifest",
        says: "ikigai-module does not embed wasmtime",
        source: MODULE,
        file: "Cargo.toml",
        needle: "wasmtime",
        present: false,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "the word \"fuel\" appears in none of them",
        says: "ikigai-module sets no execution budget",
        source: MODULE,
        file: "src/lib.rs",
        needle: "fuel",
        present: false,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`ModuleReply::ErrorTyped`",
        says: "the module wire carries the error type (0.3.0)",
        source: MODULE,
        file: "src/lib.rs",
        needle: "ErrorTyped",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`ModuleReply::ResolvedThreaded`",
        says: "the module wire carries a result's declared golden threads (0.3.0)",
        source: MODULE,
        file: "src/lib.rs",
        needle: "ResolvedThreaded",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "a callback that **failed** is a dependency on the wasm path too",
        says: "a failed out-of-band callback is recorded by core's rule (0.3.2)",
        source: MODULE,
        file: "src/lib.rs",
        needle: "fn record_failure(&mut self, requested: &Iri, error: &Error)",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`ModuleCall::Cards`",
        says: "the module's cards cross at connect (0.3.0)",
        source: MODULE,
        file: "src/lib.rs",
        needle: "Cards",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`ikigai-cli` 0.1.45 by manifest",
        says: "the CLI does not embed wasmtime",
        source: CLI_045,
        file: "Cargo.toml",
        needle: "wasmtime",
        present: false,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`WasmModuleSpace`, as of 2026-10-10",
        says: "the browser demo loads modules",
        source: WEB_DEMO,
        file: "src/lib.rs",
        needle: "WasmModuleSpace",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`ikigai-xslt` 0.3.0) — the caller is released",
        says: "ikigai-xslt answers within a time budget and abandons the work past it",
        source: XSLT_030,
        file: "src/limits.rs",
        needle: "The work is **abandoned, not cancelled**", // spelling: quote (ikigai-xslt's own words)
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "since `ikigai-xslt` 0.2.1 every transform is",
        says: "the time budget arrived in ikigai-xslt 0.2.1",
        source: XSLT_030,
        file: "src/lib.rs",
        needle: "Since 0.2.1 it is answered within [`limits::DEFAULT_TIME_BUDGET`]",
        present: true,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "within a time budget, five seconds",
        says: "ikigai-xslt's default time budget is five seconds",
        source: XSLT_030,
        file: "src/limits.rs",
        needle: "pub const DEFAULT_TIME_BUDGET: Duration = Duration::from_secs(5);",
        present: true,
    },
    Claim {
        chapter: "beyond/agent.md",
        stamp: "runs as root, loudly** (`ikigai-cli` 0.1.28)",
        says: "ikigai mcp with no grant prints that it runs unrestricted",
        source: CLI_028,
        file: "src/main.rs",
        needle: "running UNRESTRICTED (root)",
        present: true,
    },
    Claim {
        chapter: "getting-started/configuration.md",
        stamp: "(`ikigai-cli` 0.1.43) reads a set of `IKIGAI_*` variables",
        says: "IKIGAI_FILES is read by the embedded host",
        source: EMBEDDED_043,
        file: "src/lib.rs",
        needle: "IKIGAI_FILES",
        present: true,
    },
    Claim {
        chapter: "getting-started/configuration.md",
        stamp: "`decide(flag, config, env)`",
        says: "the scheduler is chosen by a three-way precedence function",
        // In the embedded host crate, not the binary's — the first run of this probe
        // said so, which is the probe doing its job on its own author.
        source: EMBEDDED_043,
        file: "src/scheduling.rs",
        needle: "fn decide(flag: Option<&str>, config: Option<&str>, env: Option<&str>)",
        present: true,
    },
    Claim {
        chapter: "getting-started/configuration.md",
        stamp: "relying party was a variable until `ikigai-cli` 0.1.39",
        says: "the passkey origin is no longer read from the environment",
        // Where it was read until ikigai-cli PR 401 removed it.
        source: EMBEDDED_043,
        file: "src/passkey.rs",
        needle: "IKIGAI_PASSKEY_ORIGIN",
        present: false,
    },
    Claim {
        chapter: "getting-started/configuration.md",
        stamp: "`--passkey-rp-id <domain>` and `--passkey-origin <url>`",
        says: "serve takes the passkey relying party as flags",
        source: CLI_043,
        file: "src/main.rs",
        needle: "--passkey-rp-id <domain> [--passkey-origin <url>]",
        present: true,
    },
    Claim {
        chapter: "beyond/identity.md",
        stamp: "Since `ikigai-resolve` 0.1.40 that default **refuses** any capability",
        says: "a mount carries a narrowed capability to its peer, and the default refuses one",
        source: Source::Crate {
            name: "ikigai-resolve",
            version: "0.1.40",
        },
        file: "src/lib.rs",
        needle: "fn a_mount_carries_a_narrowed_capability_to_its_peer",
        present: true,
    },
    Claim {
        chapter: "getting-started/golden-threads.md",
        stamp: "In the REPL (`ikigai-cli` 0.1.43 and later) the same question is",
        says: "the REPL has a `dependents <thread>` command",
        source: Source::Crate {
            name: "ikigai-engine",
            version: "0.1.43",
        },
        file: "src/engine.rs",
        needle: r#""dependents" => output(self, self.run_dependents(rest).await),"#,
        present: true,
    },
    Claim {
        chapter: "front-door/grammar.md",
        stamp: "(`ikigai-lisp` 0.2.0)",
        says: "a program sees an allowlist, and every other global is refused",
        source: LISP,
        file: "src/sandbox.rs",
        needle: "pub(crate) const PROGRAM_NAMES: &[&str] = &[",
        present: true,
    },
    Claim {
        chapter: "front-door/grammar.md",
        stamp: "`require`, `defmacro`",
        says: "require and defmacro are refused before compiling",
        source: LISP,
        file: "src/sandbox.rs",
        needle: r#"const REFUSED_FORMS: &[&str] = &["require", "require-builtin", "defmacro", "begin-for-syntax"];"#,
        present: true,
    },
    Claim {
        chapter: "modules/dual-mode.md",
        stamp: "`ikigai-xslt` 0.1.3",
        says: "ikigai-xslt has a module feature",
        source: Source::Crate {
            name: "ikigai-xslt",
            version: "0.1.3",
        },
        file: "Cargo.toml",
        needle: "\nmodule = [",
        present: true,
    },
    Claim {
        chapter: "modules/dual-mode.md",
        stamp: "`ikigai-jsonld` 0.1.2",
        says: "ikigai-jsonld has a module feature",
        source: Source::Crate {
            name: "ikigai-jsonld",
            version: "0.1.2",
        },
        file: "Cargo.toml",
        needle: "\nmodule = [",
        present: true,
    },
    Claim {
        chapter: "modules/dual-mode.md",
        stamp: "`ikigai-shacl` 0.1.2",
        says: "ikigai-shacl has a module feature",
        source: Source::Crate {
            name: "ikigai-shacl",
            version: "0.1.2",
        },
        file: "Cargo.toml",
        needle: "\nmodule = [",
        present: true,
    },
];

fn book_src() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../books/ikigai/src")
        .canonicalize()
        .expect("the book source is two levels up")
}

/// The prose carries every stamp the table names — so the two cannot drift apart.
#[test]
fn every_stamped_claim_is_still_in_the_prose() {
    let mut missing = Vec::new();
    for claim in CLAIMS {
        let text = std::fs::read_to_string(book_src().join(claim.chapter))
            .unwrap_or_else(|e| panic!("reading {}: {e}", claim.chapter));
        if !text.contains(claim.stamp) {
            missing.push(format!(
                "{}: the stamp {:?} for \"{}\" ({}) is not in the prose",
                claim.chapter,
                claim.stamp,
                claim.says,
                claim.source.label()
            ));
        }
    }
    assert!(
        missing.is_empty(),
        "claims and prose have drifted:\n  {}\n\nEither the sentence was reworded (update \
         the stamp here) or the claim was dropped (delete it here).",
        missing.join("\n  ")
    );
    assert!(
        CLAIMS.len() >= 10,
        "the table is not reading its own claims"
    );
}

/// The version the workspace's `Cargo.lock` resolves for `name`. The book is one workspace
/// with one lock, so each ikigai crate appears there at exactly one version.
fn locked_version(name: &str) -> String {
    let lock = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.lock");
    let text = std::fs::read_to_string(&lock)
        .unwrap_or_else(|e| panic!("reading {}: {e}", lock.display()));
    let header = format!("name = \"{name}\"\nversion = \"");
    let versions: Vec<&str> = text
        .match_indices(&header)
        .map(|(at, _)| {
            let rest = &text[at + header.len()..];
            &rest[..rest.find('"').expect("a closing quote")]
        })
        .collect();
    assert_eq!(
        versions.len(),
        1,
        "{name} should be locked at exactly one version, found {versions:?}"
    );
    versions[0].to_string()
}

/// The status chapter's opening names the versions the book builds against, and those are
/// facts about THIS workspace, so they are checked against it rather than stamped: it said
/// core 0.1.78 for two weeks after the lock had moved to 0.1.93, and nothing noticed
/// (ledger #1110). Moving a pin without moving the sentence now fails here.
#[test]
fn the_status_chapter_names_the_versions_the_book_builds_against() {
    let text = std::fs::read_to_string(book_src().join("modules/status.md"))
        .expect("reading modules/status.md");
    let cli_pin =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../books/ikigai/ikigai-cli.version");
    let cli = std::fs::read_to_string(&cli_pin)
        .unwrap_or_else(|e| panic!("reading {}: {e}", cli_pin.display()));
    let expected = [
        format!("`ikigai-core` **{}**", locked_version("ikigai-core")),
        format!("`ikigai-module` **{}**", locked_version("ikigai-module")),
        format!("`ikigai-cli` **{}**", cli.trim()),
    ];
    let missing: Vec<&String> = expected
        .iter()
        .filter(|e| !text.contains(e.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "modules/status.md does not say what the book builds against: missing {missing:?}"
    );
}

/// Download and unpack one source into `into`, returning its top-level directory.
fn fetch(source: &Source, into: &Path) -> Result<PathBuf, String> {
    let dir = into.join(source.label().replace(['/', ' ', '@'], "_"));
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let archive = dir.join("archive.tgz");
    let curl = Command::new("curl")
        .args(["-sSfL", "-o"])
        .arg(&archive)
        .arg(source.url())
        .status()
        .map_err(|e| format!("curl: {e}"))?;
    if !curl.success() {
        return Err(format!("could not download {}", source.url()));
    }
    let tar = Command::new("tar")
        .args(["-xzf"])
        .arg(&archive)
        .arg("-C")
        .arg(&dir)
        .status()
        .map_err(|e| format!("tar: {e}"))?;
    if !tar.success() {
        return Err(format!("could not unpack {}", archive.display()));
    }
    // Both a crate tarball and a GitHub archive unpack to one top-level directory.
    let top = std::fs::read_dir(&dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .find(|p| p.is_dir())
        .ok_or_else(|| format!("nothing unpacked from {}", source.url()))?;
    Ok(top)
}

/// The instrument: fetch each source at its stamped version and grep for the evidence.
#[test]
#[ignore = "needs the network (crates.io and GitHub); run it by hand when a sibling crate releases"]
fn every_stamped_claim_holds_at_the_version_it_names() {
    let scratch = std::env::temp_dir().join(format!("ikigai-book-claims-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("a scratch directory");

    let mut unpacked: Vec<(String, PathBuf)> = Vec::new();
    let mut failures = Vec::new();
    for claim in CLAIMS {
        let label = claim.source.label();
        let top = match unpacked.iter().find(|(l, _)| *l == label) {
            Some((_, top)) => top.clone(),
            None => match fetch(&claim.source, &scratch) {
                Ok(top) => {
                    unpacked.push((label.clone(), top.clone()));
                    top
                }
                Err(e) => {
                    failures.push(format!("{label}: {e}"));
                    continue;
                }
            },
        };
        let path = top.join(claim.file);
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(e) => {
                failures.push(format!("{label}: reading {}: {e}", path.display()));
                continue;
            }
        };
        let found = text.contains(claim.needle);
        let ok = found == claim.present;
        println!(
            "{} {label} {} — {:?} {} in {}",
            if ok { "ok  " } else { "MISS" },
            claim.says,
            claim.needle,
            if found { "present" } else { "absent" },
            claim.file
        );
        if !ok {
            failures.push(format!(
                "{}: \"{}\" — {label} {} does {}contain {:?}",
                claim.chapter,
                claim.says,
                claim.file,
                if claim.present { "not " } else { "" },
                claim.needle
            ));
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);

    assert!(
        failures.is_empty(),
        "stamped claims that do not hold at the version they name:\n  {}\n\nA claim that \
         was true at its stamp and is false later is a sentence to re-stamp and rewrite, \
         not a test to delete.",
        failures.join("\n  ")
    );
}
