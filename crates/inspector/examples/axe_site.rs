//! Write the inspector's pages as a static site for axe and the browser checks:
//! `cargo run -p inspector --example axe_site -- <dir>`.
//!
//! Every page in every theme at `/inspect/{name}-{theme}.html`, rendered under the viewer's
//! capability in the known state the snapshot tests use; the assets at the paths the pages link
//! them by; the list fragments at their URLs, so a browser can follow a nav link and swap; and
//! a push stream that says only "retry much later", since a static server has no cuts to send.
//! ikigai-ui's `.github/workflows/axe.yml` serves the directory and checks every `.html` in it.

#[path = "../tests/common/mod.rs"]
mod common;

use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("usage: cargo run -p inspector --example axe_site -- <dir>")?,
    );
    let names = common::pages(&common::kernel());
    let mut files: Vec<(String, String, Vec<u8>)> = Vec::new();
    let viewer = common::viewer();
    let kernel = common::kernel();
    for (path, iri) in [
        ("/ui/htmx", "urn:iki:ui:htmx"),
        ("/ui/shell.js", "urn:iki:ui:shell.js"),
        ("/ui/tokens.css", ikigai_ui::TOKENS_CSS),
        ("/inspect/live.js", "urn:iki:tutorial:inspect:live.js"),
        (
            "/inspect/inspector.css",
            "urn:iki:tutorial:inspect:inspector.css",
        ),
    ] {
        let rep = futures::executor::block_on(kernel.issue(
            ikigai_core::Request::new(ikigai_core::Verb::Source, ikigai_core::Iri::parse(iri)?),
            &viewer,
        ))?;
        files.push((path.to_string(), rep.repr_type.media_type, rep.bytes));
    }
    for view in inspector::VIEWS {
        let fragment = common::read(
            &common::kernel(),
            &format!("{}fragment:{view}", common::B),
            &viewer,
        )?;
        files.push((
            format!("/inspect/fragment/{view}"),
            "text/html; charset=utf-8".into(),
            fragment.into_bytes(),
        ));
    }
    for (path, iri) in &names {
        for theme in ikigai_ui::THEMES {
            let page = ikigai_ui::testkit::render::source_as(
                &common::kernel(),
                iri,
                &[("theme", theme)],
                &viewer,
            )?;
            let name = match path.split('/').nth(2) {
                Some("entry") => "entry-card".to_string(),
                Some("thread") => "thread-note".to_string(),
                _ => path.trim_start_matches("/inspect/").to_string(),
            };
            files.push((
                format!("/inspect/{name}-{theme}.html"),
                "text/html; charset=utf-8".into(),
                page.into_bytes(),
            ));
        }
    }
    files.push((
        "/_ikigai/push".into(),
        "text/event-stream".into(),
        b"retry: 3600000\n\n".to_vec(),
    ));
    let borrowed: Vec<(&str, &str, &[u8])> = files
        .iter()
        .map(|(p, t, b)| (p.as_str(), t.as_str(), b.as_slice()))
        .collect();
    ikigai_ui::testkit::site::write(&dir, &borrowed)?;
    println!("wrote {} files to {}", files.len(), dir.display());
    Ok(())
}
