//! **`inspector-htmx`**: the kernel inspector delivered as server-rendered HTML.
//!
//! A thin renderer crate (*Building ikigai UIs*, Part II): everything the pages show is the
//! `inspector` crate's resources. This crate adds only what a delivery target owns:
//!
//! - **the route table** (UI conventions D10): each page and fragment at a URL, ikigai-ui's
//!   artifacts at `/ui/{name}`, and nothing else (`routes_only`, so the kernel's own names are
//!   not public by accident);
//! - **the door** (D6): `EdgeConfig::html_door`, the strict page CSP, the Origin check on every
//!   unsafe method and the security headers;
//! - **the push stream** (D9) at `/_ikigai/push`, which the inspector's `live.js` listens to;
//! - **who is asking** (the `CapFn`): a local viewer gets [`inspector::VIEWER_SCOPES`], so every
//!   view is read under a capability the kernel checks, never under root;
//! - **the directory delivery** (D17): `--artifacts DIR` serves the inspector's artifacts from
//!   DIR, file by file, ahead of the compiled-in ones.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ikigai_core::{Capability, EndpointSpace, Kernel, Space, UriTemplate};
use ikigai_web::{CapFn, EdgeConfig, HttpRequest, PushConfig, Route, RouteTable};

/// The HTTP address when `--http` is not given.
pub const DEFAULT_HTTP: &str = "127.0.0.1:8071";

/// The usage line.
pub const USAGE: &str = "usage: inspector-htmx [--http <addr>] [--artifacts <dir>]\n\
  --http <addr>       serve on this address (default 127.0.0.1:8071; port 0 picks one)\n\
  --artifacts <dir>   serve the inspector's artifacts from <dir> (a file there replaces the\n\
                      compiled-in artifact of the same name; see crates/inspector/artifacts)";

/// What to serve, and where.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// `--http`.
    pub http: SocketAddr,
    /// `--artifacts`.
    pub artifacts: Option<PathBuf>,
}

/// Read the command line (without the program name).
pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> Result<Options, String> {
    let mut options = Options {
        http: DEFAULT_HTTP.parse().expect("a valid default"),
        artifacts: None,
    };
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| format!("{flag} needs a value\n{USAGE}"))
        };
        match flag.as_str() {
            "--http" => {
                let text = value()?;
                options.http = text
                    .parse()
                    .map_err(|e| format!("--http {text}: not an address ({e})"))?;
            }
            "--artifacts" => options.artifacts = Some(PathBuf::from(value()?)),
            "-h" | "--help" => return Err(USAGE.to_string()),
            other => return Err(format!("unknown argument `{other}`\n{USAGE}")),
        }
    }
    Ok(options)
}

fn route(pattern: &str, iri: &str) -> Route {
    Route {
        pattern: pattern.to_string(),
        iri_template: iri.to_string(),
        cap: None,
        cors: None,
        csp: None,
        redirect: None,
    }
}

// ANCHOR: routes
/// The route table: every URL this door answers, and the resource behind it. First match
/// wins, so the fixed paths come before the patterns that would also fit them.
pub fn routes() -> RouteTable {
    const B: &str = "urn:iki:tutorial:inspect:";
    RouteTable::new(vec![
        route("/", &format!("{B}page:cache")),
        route("/ui/{name}", "urn:iki:ui:{name}"),
        route("/inspect/live.js", &format!("{B}live.js")),
        route("/inspect/inspector.css", &format!("{B}inspector.css")),
        route("/inspect/card", &format!("{B}demo:card")),
        route("/inspect/note/write", &format!("{B}demo:note")),
        route("/inspect/entry/{id}", &format!("{B}page:entry:{{id}}")),
        route(
            "/inspect/thread/{+name}",
            &format!("{B}page:thread:{{+name}}"),
        ),
        route(
            "/inspect/fragment/entry/{id}",
            &format!("{B}fragment:entry:{{id}}"),
        ),
        route(
            "/inspect/fragment/thread/{+name}",
            &format!("{B}fragment:thread:{{+name}}"),
        ),
        route("/inspect/fragment/{view}", &format!("{B}fragment:{{view}}")),
        route("/inspect/{view}", &format!("{B}page:{{view}}")),
    ])
}
// ANCHOR_END: routes

/// The inspector's kernel, with `artifacts` (if any) bound at [`inspector::DIR`].
pub fn kernel(artifacts: Option<&Path>) -> Kernel {
    let mut extra: Vec<Arc<dyn Space>> = Vec::new();
    if let Some(dir) = artifacts {
        extra.push(Arc::new(EndpointSpace::new().bind(
            UriTemplate::parse(format!("{}{{path}}", inspector::DIR)).expect("a template"),
            ikigai_fs::FileEndpoint::new(dir),
        )));
    }
    inspector::kernel(extra)
}

/// The scopes a viewer of this door holds: [`inspector::VIEWER_SCOPES`], and, when the
/// artifacts come from a directory, reading that directory (ikigai-fs checks it on every read).
pub fn viewer_scopes(artifacts: Option<&Path>) -> Vec<String> {
    let mut scopes: Vec<String> = inspector::VIEWER_SCOPES
        .iter()
        .map(|s| s.to_string())
        .collect();
    if let Some(dir) = artifacts {
        let dir = std::fs::canonicalize(dir).unwrap_or_else(|_| dir.to_path_buf());
        scopes.push(format!("urn:cap:fs:read:{}", dir.display()));
    }
    scopes
}

/// Who is asking: every request gets the viewer's scopes, nothing more.
pub fn cap_fn(scopes: Vec<String>) -> CapFn {
    let cap = Capability::root().attenuate(scopes.iter().map(String::as_str));
    Arc::new(move |_req: &HttpRequest| cap.clone())
}

/// The door: the HTML door default for `origin`, this crate's routes and nothing else, and the
/// push stream.
pub fn config(origin: &str) -> Result<EdgeConfig, String> {
    let mut config = EdgeConfig::html_door([origin]).map_err(|e| format!("{origin}: {e}"))?;
    config.routes = routes();
    config.routes_only = true;
    config.push = Some(PushConfig::default());
    Ok(config)
}

/// Serve on an already-bound listener until the process ends.
pub async fn serve(
    listener: tokio::net::TcpListener,
    artifacts: Option<PathBuf>,
) -> std::io::Result<()> {
    let addr = listener.local_addr()?;
    let origin = format!("http://{addr}");
    let config = config(&origin).map_err(std::io::Error::other)?;
    let kernel = Arc::new(kernel(artifacts.as_deref()));
    let caps = cap_fn(viewer_scopes(artifacts.as_deref()));
    ikigai_web::serve_with_listener(kernel, caps, listener, config).await
}
