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

const MODULE: Source = Source::Crate {
    name: "ikigai-module",
    version: "0.2.0",
};
const CLI: Source = Source::Crate {
    name: "ikigai-cli",
    version: "0.1.18",
};
const EMBEDDED: Source = Source::Crate {
    name: "ikigai-embedded",
    version: "0.1.18",
};
const WEB_DEMO: Source = Source::GitHub {
    repo: "ikigai-web-demo",
    sha: "4f855d999ca1f9c3750471bbceeac9892d0e7d1b",
};

/// The claims. The stamp is what the prose must say; the rest is what the probe checks.
const CLAIMS: &[Claim] = &[
    Claim {
        chapter: "modules/status.md",
        stamp: "ikigai-module` 0.2.0's own first line still calls it",
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
        stamp: "`ikigai-cli` 0.1.18 by manifest",
        says: "the CLI does not embed wasmtime",
        source: CLI,
        file: "Cargo.toml",
        needle: "wasmtime",
        present: false,
    },
    Claim {
        chapter: "modules/status.md",
        stamp: "`WasmModuleSpace`, as of 2026-09-08",
        says: "the browser demo loads modules",
        source: WEB_DEMO,
        file: "src/lib.rs",
        needle: "WasmModuleSpace",
        present: true,
    },
    Claim {
        chapter: "beyond/agent.md",
        stamp: "runs as root, loudly** (`ikigai-cli` 0.1.18)",
        says: "ikigai mcp with no grant prints that it runs unrestricted",
        source: CLI,
        file: "src/main.rs",
        needle: "running UNRESTRICTED (root)",
        present: true,
    },
    Claim {
        chapter: "getting-started/configuration.md",
        stamp: "(`ikigai-cli` 0.1.18) reads a set of `IKIGAI_*` variables",
        says: "IKIGAI_FILES is read by the embedded host",
        source: EMBEDDED,
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
        source: EMBEDDED,
        file: "src/scheduling.rs",
        needle: "fn decide(flag: Option<&str>, config: Option<&str>, env: Option<&str>)",
        present: true,
    },
    Claim {
        chapter: "modules/dual-mode.md",
        stamp: "`ikigai-xslt` 0.1.1",
        says: "ikigai-xslt has a module feature",
        source: Source::Crate {
            name: "ikigai-xslt",
            version: "0.1.1",
        },
        file: "Cargo.toml",
        needle: "\nmodule = [",
        present: true,
    },
    Claim {
        chapter: "modules/dual-mode.md",
        stamp: "`ikigai-jsonld`\n0.1.1",
        says: "ikigai-jsonld has a module feature",
        source: Source::Crate {
            name: "ikigai-jsonld",
            version: "0.1.1",
        },
        file: "Cargo.toml",
        needle: "\nmodule = [",
        present: true,
    },
    Claim {
        chapter: "modules/dual-mode.md",
        stamp: "`ikigai-shacl` 0.1.1",
        says: "ikigai-shacl has a module feature",
        source: Source::Crate {
            name: "ikigai-shacl",
            version: "0.1.1",
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
