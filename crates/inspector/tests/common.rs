//! What every test and the axe site share: a kernel in a known state, read as a viewer.
// Shared by the tests and the axe site example; each uses part of it.
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, EndpointSpace, Iri, Kernel, Request, Space, UriTemplate, Verb,
};

pub const B: &str = "urn:iki:tutorial:inspect:";

/// The viewer a door hands every request: inspect, listen, write the note. Never root.
pub fn viewer() -> Capability {
    Capability::root().attenuate(inspector::VIEWER_SCOPES)
}

/// The compiled-in kernel, with the note's card read once, so the cache has something in it.
pub fn kernel() -> Kernel {
    let kernel = inspector::kernel(Vec::new());
    read(&kernel, &format!("{B}demo:card"), &viewer()).expect("the card");
    kernel
}

/// The artifacts directory, as committed.
pub fn artifacts() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("artifacts")
}

/// A kernel whose artifacts come from `dir` (ikigai-fs at `inspector::DIR`), and the viewer
/// with read on that directory.
pub fn kernel_over(dir: &Path, cacheable: bool) -> (Kernel, Capability) {
    let file = ikigai_fs::FileEndpoint::new(dir);
    let file = if cacheable { file.cacheable() } else { file };
    let space: Arc<dyn Space> = Arc::new(EndpointSpace::new().bind(
        UriTemplate::parse(&format!("{}{{path}}", inspector::DIR)).unwrap(),
        file,
    ));
    let dir = std::fs::canonicalize(dir).unwrap();
    let mut scopes: Vec<String> = inspector::VIEWER_SCOPES
        .iter()
        .map(|s| s.to_string())
        .collect();
    scopes.push(format!("urn:cap:fs:read:{}", dir.display()));
    scopes.push(format!("urn:cap:fs:write:{}", dir.display()));
    (
        inspector::kernel(vec![space]),
        Capability::root().attenuate(scopes),
    )
}

pub fn read(kernel: &Kernel, iri: &str, cap: &Capability) -> ikigai_core::Result<String> {
    let rep = block_on(kernel.issue(Request::new(Verb::Source, Iri::parse(iri).unwrap()), cap))?;
    Ok(String::from_utf8(rep.bytes).unwrap())
}

pub fn write(
    kernel: &Kernel,
    iri: &str,
    content: &str,
    cap: &Capability,
) -> ikigai_core::Result<String> {
    let rep = block_on(
        kernel.issue(
            Request::new(Verb::Sink, Iri::parse(iri).unwrap())
                .with_arg("content", ArgRef::Inline(content.as_bytes().to_vec())),
            cap,
        ),
    )?;
    Ok(String::from_utf8(rep.bytes).unwrap())
}

/// The page path of the cache entry resolved from `iri`, as the cache view links it.
pub fn entry_page(kernel: &Kernel, iri: &str) -> String {
    let cache = read(kernel, &format!("{B}fragment:cache"), &viewer()).unwrap();
    let at = cache
        .find(&format!(">{iri}</a>"))
        .expect("the entry is listed");
    let href = cache[..at].rfind("href='").expect("a link") + "href='".len();
    let end = cache[href..].find('\'').unwrap();
    cache[href..href + end].to_string()
}

/// Every page the inspector serves in the known state: (site path, page IRI).
pub fn pages(kernel: &Kernel) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = inspector::VIEWS
        .iter()
        .map(|v| (format!("/inspect/{v}"), format!("{B}page:{v}")))
        .collect();
    let entry = entry_page(kernel, &format!("{B}demo:card"));
    let id = entry.trim_start_matches("/inspect/entry/").to_string();
    out.push((entry, format!("{B}page:entry:{id}")));
    out.push((
        "/inspect/thread/urn%3Aiki%3Atutorial%3Ainspect%3Ademo%3Anote".to_string(),
        format!("{B}page:thread:{B}demo:note"),
    ));
    out
}
