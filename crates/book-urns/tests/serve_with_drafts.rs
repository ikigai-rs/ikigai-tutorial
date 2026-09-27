//! `scripts/serve-with-drafts.sh` checks the wasm32 standard library in the compiler the
//! BUILD will use — not in rustup's opinion of what is installed.
//!
//! The two disagreed on a Mac with Homebrew's Rust ahead of `~/.cargo/bin` on PATH
//! (ledger #590): rustup's toolchain had `wasm32-unknown-unknown`, the old check asked
//! rustup and passed, mdbook built the book, and then cargo compiled with Homebrew's rustc,
//! which ships no wasm32 `core` — E0463 on every crate, minutes later, telling the reader
//! to run the command they had already run.
//!
//! No real toolchain is involved here. Each case runs the script with a PATH of stubs in
//! front of `/usr/bin:/bin` and nothing else (the environment is cleared): a `rustc` that
//! answers `--print target-libdir` / `--print sysroot` / `--version` with directories this
//! test controls, where a wasm32 `libcore-*.rlib` either exists or does not; a `rustup`
//! whose `which rustc` names another stub (or the same one, when rustup is in charge);
//! and a `wasm-bindgen` at the pinned version, so the only thing wrong is the compiler.
//!
//! Two tripwires say how far the script got. `mktemp` is the first command after the
//! toolchain check (it makes the copy of the book), so a stub `mktemp` marks "the check
//! passed" and then fails, which stops the script before it copies or builds anything.
//! `mdbook` marks "the book was built", which a failed check must never reach.
//!
//! Shell, so Unix only; `/bin/sh` for the stubs and whatever `bash` `/usr/bin:/bin` holds
//! for the script — bash 3.2 on macOS, which is the one a reader without Homebrew's bash
//! runs it under.
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

const WASM: &str = "wasm32-unknown-unknown";

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/book-urns sits two levels below the repository root")
        .to_path_buf()
}

/// The `wasm-bindgen` version `crates/book-wasm/Cargo.toml` pins, read the way the script
/// reads it, so the stub always matches and the wasm-bindgen half of the check stays quiet.
fn wasm_bindgen_pin() -> String {
    let manifest = fs::read_to_string(repo_root().join("crates/book-wasm/Cargo.toml"))
        .expect("read crates/book-wasm/Cargo.toml");
    manifest
        .lines()
        .find_map(|line| {
            let rest = line.strip_prefix("wasm-bindgen")?.trim_start();
            let value = rest.strip_prefix('=')?.trim();
            let value = value.strip_prefix('"')?.trim_start_matches('=');
            Some(value.split('"').next()?.to_string())
        })
        .expect("crates/book-wasm/Cargo.toml pins wasm-bindgen")
}

/// A scratch directory holding the stubs and the fake sysroots. Removed on drop.
struct Scratch {
    root: PathBuf,
}

impl Scratch {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("swd-{}-{name}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("bin")).expect("create the stub bin directory");
        let scratch = Scratch { root };
        scratch.script(
            "bin/wasm-bindgen",
            &format!("echo 'wasm-bindgen {}'", wasm_bindgen_pin()),
        );
        let past = scratch.root.join("past-the-check");
        scratch.script("bin/mktemp", &format!("touch '{}'\nexit 1", past.display()));
        let mdbook = scratch.root.join("mdbook-ran");
        scratch.script(
            "bin/mdbook",
            &format!("touch '{}'\nexit 1", mdbook.display()),
        );
        scratch
    }

    fn bin(&self) -> PathBuf {
        self.root.join("bin")
    }

    fn script(&self, relative: &str, body: &str) -> PathBuf {
        let path = self.root.join(relative);
        fs::create_dir_all(path.parent().unwrap()).expect("create the stub's directory");
        fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("write a stub");
        fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).expect("chmod a stub");
        path
    }

    /// A stub rustc whose sysroot is `<root>/<name>`, reporting `version`. With `wasm_core`,
    /// that sysroot holds a wasm32 `libcore` rlib; without it, the directory does not exist.
    fn rustc(&self, relative: &str, name: &str, version: &str, wasm_core: bool) -> PathBuf {
        let sysroot = self.root.join(name);
        let libdir = sysroot.join(format!("lib/rustlib/{WASM}/lib"));
        if wasm_core {
            fs::create_dir_all(&libdir).expect("create the fake wasm32 libdir");
            fs::write(libdir.join("libcore-0123456789abcdef.rlib"), b"").expect("fake libcore");
        }
        self.script(
            relative,
            &format!(
                "case \"$*\" in\n\
                 *target-libdir*) echo '{}' ;;\n\
                 *sysroot*) echo '{}' ;;\n\
                 *--version*) echo '{version}' ;;\n\
                 esac",
                libdir.display(),
                sysroot.display(),
            ),
        )
    }

    /// A stub rustup whose `which rustc` names `rustc`, and whose `target list --installed`
    /// agrees with that rustc's libdir — so against the OLD check, which asked rustup, the
    /// shadowing case passes exactly as it did on bug.
    fn rustup(&self, rustc: &Path) {
        let rustc = rustc.display();
        self.script(
            "bin/rustup",
            &format!(
                "case \"$*\" in\n\
                 'which rustc') echo '{rustc}' ;;\n\
                 'target list --installed')\n\
                 d=\"$('{rustc}' --print target-libdir --target {WASM})\"\n\
                 if [ -d \"$d\" ]; then echo {WASM}; fi ;;\n\
                 esac"
            ),
        );
    }

    fn run(&self, rustc_env: Option<&Path>) -> Run {
        let mut command = Command::new(repo_root().join("scripts/serve-with-drafts.sh"));
        command
            .args(["build", &self.root.join("out").display().to_string()])
            .env_clear()
            .env("PATH", format!("{}:/usr/bin:/bin", self.bin().display()))
            // HOME is the scratch root, so the stub bin directory is `$HOME/bin`.
            .env("HOME", &self.root);
        if let Some(rustc) = rustc_env {
            command.env("RUSTC", rustc);
        }
        let output: Output = command.output().expect("run scripts/serve-with-drafts.sh");
        Run {
            code: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
            passed_check: self.root.join("past-the-check").exists(),
            built_book: self.root.join("mdbook-ran").exists(),
        }
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(Debug)]
struct Run {
    code: Option<i32>,
    stderr: String,
    passed_check: bool,
    built_book: bool,
}

impl Run {
    fn refused(&self) {
        assert_eq!(self.code, Some(1), "the check exits 1:\n{}", self.stderr);
        assert!(
            !self.passed_check,
            "refused, yet it went on:\n{}",
            self.stderr
        );
        assert!(
            !self.built_book,
            "refused, yet mdbook ran:\n{}",
            self.stderr
        );
        assert!(
            self.stderr.contains("--no-kernel"),
            "the prose-only way out is offered:\n{}",
            self.stderr
        );
    }

    fn says(&self, needle: &str) {
        assert!(
            self.stderr.contains(needle),
            "expected {needle:?} in:\n{}",
            self.stderr
        );
    }

    fn does_not_say(&self, needle: &str) {
        assert!(
            !self.stderr.contains(needle),
            "did not expect {needle:?} in:\n{}",
            self.stderr
        );
    }
}

/// The case from ledger #590: another Rust shadows rustup, and rustup HAS the target.
#[test]
fn a_rust_that_shadows_rustup_is_named_and_the_path_is_the_fix() {
    let s = Scratch::new("shadow");
    let brew = s.rustc("bin/rustc", "brew", "rustc 1.98.1 (Homebrew)", false);
    let rustups = s.rustc("toolchain/rustc", "rustup-toolchain", "rustc 1.97.1", true);
    s.rustup(&rustups);

    let run = s.run(None);
    run.refused();
    run.says(&brew.display().to_string());
    run.says("rustc 1.98.1 (Homebrew)");
    run.says("the first rustc on PATH");
    run.says("shadows it");
    // rustup's bin directory, named from where rustup really is; under HOME it is spelled
    // `$HOME/…` (the stub rustup is in `$HOME/bin` here), as `$HOME/.cargo/bin` would be.
    run.says("fish_add_path --move $HOME/bin");
    run.says("export PATH=\"$HOME/bin:$PATH\"");
    run.says("env PATH=\"$HOME/bin:$PATH\" ./scripts/serve-with-drafts.sh");
    // rustup already has the target: telling the reader to add it is the advice that
    // sent them round in a circle.
    run.says("changes nothing");
    run.does_not_say(&format!("rustup target add {WASM}"));
}

/// Shadowed AND rustup lacks the target: both fixes are needed, and both are given.
#[test]
fn a_shadowed_rustup_without_the_target_needs_both_fixes() {
    let s = Scratch::new("shadow-bare");
    s.rustc("bin/rustc", "brew", "rustc 1.98.1 (Homebrew)", false);
    let rustups = s.rustc("toolchain/rustc", "rustup-toolchain", "rustc 1.97.1", false);
    s.rustup(&rustups);

    let run = s.run(None);
    run.refused();
    run.says("shadows it");
    run.says("lacks the target as well");
    run.says(&format!("rustup target add {WASM}"));
}

/// rustup in charge (its `which rustc` has the build's sysroot), the target missing: the
/// plain case, and the plain advice.
#[test]
fn rustup_in_charge_without_the_target_is_told_to_add_it() {
    let s = Scratch::new("rustup-bare");
    let rustc = s.rustc("bin/rustc", "rustup-toolchain", "rustc 1.97.1", false);
    s.rustup(&rustc);

    let run = s.run(None);
    run.refused();
    run.says(&rustc.display().to_string());
    run.says(&format!("rustup target add {WASM}"));
    run.does_not_say("shadows");
}

/// No rustup at all, and a Rust without the target: say that rustup is the way.
#[test]
fn no_rustup_and_no_target_points_at_rustup() {
    let s = Scratch::new("no-rustup");
    s.rustc("bin/rustc", "distro", "rustc 1.90.0 (distro)", false);

    let run = s.run(None);
    run.refused();
    run.says("rustup is not installed");
    run.says("https://rustup.rs");
}

/// cargo compiles with `$RUSTC` when it is set, so the check asks that compiler — both
/// ways round.
#[test]
fn rustc_from_the_environment_is_the_one_checked() {
    let s = Scratch::new("rustc-env");
    let good = s.rustc("bin/rustc", "rustup-toolchain", "rustc 1.97.1", true);
    s.rustup(&good);
    let bad = s.rustc("elsewhere/rustc", "other", "rustc 1.98.1 (other)", false);

    let run = s.run(Some(&bad));
    run.refused();
    run.says(&bad.display().to_string());
    run.says("$RUSTC");
    run.says("Unset RUSTC");

    let s = Scratch::new("rustc-env-good");
    s.rustc("bin/rustc", "brew", "rustc 1.98.1 (Homebrew)", false);
    let good = s.rustc("elsewhere/rustc", "rustup-toolchain", "rustc 1.97.1", true);
    s.rustup(&good);
    let run = s.run(Some(&good));
    assert!(run.passed_check, "RUSTC has the target:\n{}", run.stderr);
}

/// The positive case: a compiler with a wasm32 `core` passes, and the script goes on.
#[test]
fn a_rustc_with_a_wasm32_core_passes_the_check() {
    let s = Scratch::new("good");
    let rustc = s.rustc("bin/rustc", "rustup-toolchain", "rustc 1.97.1", true);
    s.rustup(&rustc);

    let run = s.run(None);
    assert!(
        run.passed_check,
        "a rustc with the target was refused:\n{}",
        run.stderr
    );
    assert!(
        !run.stderr.contains("cannot be built"),
        "no complaint expected:\n{}",
        run.stderr
    );
}
