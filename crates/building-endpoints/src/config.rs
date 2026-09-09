//! **Configuration** — a host that reads a config file, from a home a test can choose.
//!
//! Two rules from `ikigai-core`'s design notes, both followed here. The *paths* come from
//! `ikigai_core::config::layered_paths_in`: a shared file and an app-scoped sibling that
//! overrides it key-wise, so every host in the ecosystem layers the same way. And the
//! *home* is taken at construction — `banner(home, app)` — never read from the
//! environment inside the endpoint: the injected form is the real one, and the ambient
//! read happens once, in the host, where it is a fact about the process.

use std::path::{Path, PathBuf};

use ikigai_core::config::layered_paths_in;
use ikigai_core::{Description, FnEndpoint, Invocation, Representation, Result, Verb};

use crate::{text_plain_utf8, TEXT_PLAIN_UTF8};

/// The file this host reads: `<config home>/tutorial.toml`, then
/// `<config home>/<app>.tutorial.toml` over it.
pub const STEM: &str = "tutorial.toml";

/// The banner when no file says otherwise.
pub const DEFAULT_BANNER: &str = "The ikigai Book";

// ANCHOR: settings
/// What the config file configures. One key, so the layering is visible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// `[banner] text = "…"`.
    pub banner: String,
}

impl Settings {
    /// Read and merge the layers under `home`. A missing file is a missing *layer*, not
    /// an error — the process is under-configured, which is legal — but a file that is
    /// present and is not TOML stops the read, because a silently ignored config is the
    /// operator believing one thing while the process does another.
    pub fn load_in(home: &Path, app: Option<&str>) -> Result<Settings> {
        let mut merged = toml::Table::new();
        for path in layered_paths_in(home, STEM, app) {
            let Ok(text) = std::fs::read_to_string(&path) else {
                continue;
            };
            let layer: toml::Table = toml::from_str(&text).map_err(|e| {
                ikigai_core::Error::Endpoint(format!("{}: not TOML: {e}", path.display()))
            })?;
            // Key-wise, later wins: a later layer overrides the keys it states and
            // leaves the rest alone.
            for (key, value) in layer {
                merged.insert(key, value);
            }
        }
        let banner = merged
            .get("banner")
            .and_then(|banner| banner.get("text"))
            .and_then(toml::Value::as_str)
            .unwrap_or(DEFAULT_BANNER)
            .to_string();
        Ok(Settings { banner })
    }
}
// ANCHOR_END: settings

// ANCHOR: endpoint
/// `banner`: the configured banner text.
///
/// The endpoint holds the *home*, not the parsed settings, and re-reads per resolution.
/// That is the right split for a value that lives in a file: the representation is
/// cacheable and depends on a golden thread named after each candidate file, so the
/// re-read costs nothing until something cuts one of those threads — a watcher, or a
/// hand on `kernel.cut(..)`. A handle holding parsed settings would serve the values the
/// process started with, forever, however many times the thread was cut.
///
/// `home == None` is the under-configured state, stated rather than guessed: no file is
/// read and the default banner answers. There is no `current_dir()` fallback.
pub fn banner(home: Option<PathBuf>, app: Option<&str>) -> FnEndpoint {
    let app = app.map(str::to_string);
    FnEndpoint::new("banner", move |_inv: &Invocation<'_>| {
        let mut repr = match &home {
            Some(home) => {
                let settings = Settings::load_in(home, app.as_deref())?;
                Representation::new(text_plain_utf8(), settings.banner.into_bytes()).cacheable()
            }
            None => Representation::new(text_plain_utf8(), DEFAULT_BANNER.as_bytes().to_vec())
                .cacheable(),
        };
        if let Some(home) = &home {
            for path in layered_paths_in(home, STEM, app.as_deref()) {
                repr = repr.depends_on(path.to_string_lossy().into_owned());
            }
        }
        Ok(repr)
    })
    .with_description(
        Description::new("banner")
            .title("Banner")
            .summary("The banner text this host was configured with.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: endpoint

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_in;
    use futures::executor::block_on;
    use ikigai_core::{Capability, Iri, Kernel, Request};

    /// A config home of this test's own. `cargo test` runs tests in one process, so a
    /// shared home would let two tests disagree about one file.
    fn scratch_home(label: &str) -> PathBuf {
        let home =
            std::env::temp_dir().join(format!("ikigai-book-config-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&home);
        std::fs::create_dir_all(&home).expect("a scratch config home");
        home
    }

    fn banner_of(kernel: &Kernel) -> String {
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:banner").expect("iri"),
        );
        let repr = block_on(kernel.issue(request, &Capability::root())).expect("resolves");
        String::from_utf8_lossy(&repr.bytes).into_owned()
    }

    // ANCHOR: layers
    /// Two layers, and the app-scoped one wins for the key it states.
    #[test]
    fn the_app_layer_overrides_the_shared_one() {
        let home = scratch_home("layers");
        std::fs::write(home.join("tutorial.toml"), "[banner]\ntext = \"shared\"\n").unwrap();

        let kernel = kernel_in(Some(home.clone()));
        assert_eq!(banner_of(&kernel), "shared");

        // The override appears. The cached answer is stale now — and this host has no
        // watcher on the config home, so the thread is cut by hand. Its name is the
        // file's path, which the endpoint declared.
        std::fs::write(
            home.join("yours.tutorial.toml"),
            "[banner]\ntext = \"yours\"\n",
        )
        .unwrap();
        assert_eq!(
            banner_of(&kernel),
            "shared",
            "served from cache: nothing cut a thread"
        );
        kernel.cut(
            home.join("yours.tutorial.toml")
                .to_string_lossy()
                .into_owned(),
        );
        assert_eq!(banner_of(&kernel), "yours");
    }
    // ANCHOR_END: layers

    #[test]
    fn no_file_is_the_default_and_no_home_is_the_default() {
        let home = scratch_home("empty");
        assert_eq!(banner_of(&kernel_in(Some(home))), DEFAULT_BANNER);
        assert_eq!(banner_of(&kernel_in(None)), DEFAULT_BANNER);
    }

    #[test]
    fn a_file_that_is_present_and_broken_stops_the_read() {
        let home = scratch_home("broken");
        std::fs::write(home.join("tutorial.toml"), "[banner\ntext = ").unwrap();
        let error = Settings::load_in(&home, Some("yours")).unwrap_err();
        assert!(error.to_string().contains("not TOML"), "{error}");
    }
}
