//! The code taught by **Building endpoints** — the part between Getting started and
//! Loadable modules.
//!
//! Part I built one endpoint and showed what resolving it buys. This part is the rest of
//! the builder's toolkit, one chapter per module here, each pulled into the book by
//! anchor:
//!
//! * [`transreption`] — a transreptor of your own, and the kernel finding it;
//! * [`multi_verb`] — one endpoint, three verbs, three contracts, and a capability that
//!   is declared *and* enforced;
//! * [`graph`] — the catalog as a graph, queried with SPARQL;
//! * [`hermetic`] — an endpoint that reads the clock, tested without one;
//! * [`config`] — a host that reads a config file, from a home a test can choose.
//!
//! Read the book: `mdbook serve books/ikigai --open`

pub mod config;
pub mod graph;
pub mod hermetic;
pub mod multi_verb;
pub mod transreption;

use std::path::PathBuf;
use std::sync::Arc;

use ikigai_core::{EndpointSpace, Exact, Fallback, Kernel, ReprType, Space};
use ikigai_vocab::TurtleRenderer;

/// `text/plain; charset=utf-8` as a [`ReprType`].
pub fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

/// The same media type as a string, for `Description::output`.
pub const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";

// ANCHOR: space
/// This part's space: Part I's bindings, the published SPARQL face, and the four
/// endpoints these chapters build. `home` is where [`config`] looks for its file — a
/// test hands it a directory of its own; [`kernel`] hands it this machine's.
pub fn space(home: Option<PathBuf>) -> Fallback {
    let ours: EndpointSpace = hello_camel::space()
        .bind(
            Exact::new("urn:iki:tutorial:markdown"),
            transreption::markdown(),
        )
        .bind(
            Exact::new("urn:iki:tutorial:counter"),
            multi_verb::counter(),
        )
        .bind(Exact::new("urn:iki:tutorial:stamp"), hermetic::stamp())
        .bind(
            Exact::new("urn:iki:tutorial:banner"),
            config::banner(home, Some("yours")),
        );
    // Two spaces, first match wins: ours, then `urn:sparql:*` from the published crate.
    Fallback::new(vec![
        Arc::new(ours) as Arc<dyn Space>,
        Arc::new(ikigai_sparql::space()) as Arc<dyn Space>,
    ])
}
// ANCHOR_END: space

/// The host: [`space`] over this machine's config home, with the renderer.
pub fn kernel() -> Kernel {
    kernel_in(ikigai_core::config::config_home())
}

/// [`kernel`] with the config home stated — the form every test here uses.
pub fn kernel_in(home: Option<PathBuf>) -> Kernel {
    Kernel::with_meta_renderer(Arc::new(space(home)), Arc::new(TurtleRenderer))
}
