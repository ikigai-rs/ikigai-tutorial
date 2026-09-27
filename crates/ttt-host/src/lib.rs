//! **`ttt-host`** — the tutorial's tic-tac-toe, served.
//!
//! One kernel holds the game: `tic_tac_toe::space_with_store` over the root game's store, with
//! every other game a corridor ([`tic_tac_toe::game`]) over its own. Two transports face it:
//!
//! * **HTTP**, through `ikigai-web`: the game's own htmx markup, the same bytes the book's
//!   page shows. A page for game `a` lives at `/game/a/`, and its markup's relative paths
//!   (`iki/tutorial/ttt/view/board`) arrive as `/game/a/iki/tutorial/ttt/view/board`. The
//!   host takes `/game/a/` off the front and turns it into game `a`'s corridor — the game is
//!   where the page is — and maps the rest by `ikigai-web`'s mechanical rule, `a/b/c` →
//!   `urn:a:b:c`. `/` is the root game's page, and `/iki/tutorial/ttt/…` its names.
//! * **IPC**, through `ikigai-ipc`: the same names for a client in another process or
//!   language, the root game's under their own names and game `a`'s under
//!   `urn:game:a:iki:tutorial:ttt:…` — the HTTP path, spelled as an IRI.
//!
//! A game's store may be a PEER: another process serving `urn:iki:tutorial:ttt:stored:{x}:{y}`
//! on a socket (the Python and Deno faces' `examples/tictactoe_store.*`, or a Rust one). The
//! host mounts it the way `ikigai --override urn:iki:tutorial:ttt:stored:=<socket>` does, and
//! runs [`tic_tac_toe::check_store`] against it before accepting it: a peer that breaks the
//! contract is refused at startup, naming the clause it broke.
//!
//! ## Why each transport has a front kernel
//!
//! `ikigai-web` and `ikigai-ipc` both issue every request with `Kernel::issue` — the root
//! chain, no corridor. Injecting a corridor ahead of the root is `Kernel::issue_in`, which
//! only the holder of the kernel can call (core, `Invocation::confine`'s doc). So each
//! transport serves a small FRONT kernel whose spaces forward into the game's kernel: the
//! `Gateway` turns `urn:game:{id}:…` into `issue_in(…, game id's corridor)`, and
//! `Forward` passes the root game's names through. A forwarded answer is marked
//! uncacheable in the front, because the front never sees its dependencies — the game's
//! kernel caches it, and cuts it. A `ScopeFn` in `ikigai-web`'s `EdgeConfig`, the chain
//! twin of its `CapFn`, would make the HTTP front unnecessary.

use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use ikigai_core::{
    Bindings, Description, Endpoint, Error, Expiry, Fallback, Invocation, Iri, Kernel, ReprType,
    Representation, Request, Resolution, Resolved, Result, Scope, Space, SpaceEntry, Verb,
};
use ikigai_resolve::MountedRemote;
use ikigai_vocab::TurtleRenderer;
use ikigai_web::{EdgeConfig, Route, RouteTable};
use tic_tac_toe::{fill, Fill};

/// The game's names, as the root game has them.
const GAME_NAMES: &str = "urn:iki:tutorial:ttt:";

/// The stored cell's family: a peer store is mounted over exactly this prefix.
const STORED_PREFIX: &str = "urn:iki:tutorial:ttt:stored:";

/// The prefix a game's names have at the edge: `urn:game:{id}:iki:tutorial:ttt:…`.
const EDGE_GAMES: &str = "urn:game:";

/// The vendored htmx, byte for byte the book's (`books/ikigai/src/vendor/`).
const HTMX: &str = include_str!("../../../books/ikigai/src/vendor/htmx-2.0.4.min.js");

/// The board's stylesheet — the book's own, the one stylesheet for this markup.
const TTT_CSS: &str = include_str!("../../../books/ikigai/css/ttt.css");

/// This page's colours and layout, which the book's pages get from mdbook.
const HOST_CSS: &str = include_str!("../static/host.css");

// ANCHOR: options
/// What to serve, and where. Built by [`parse_args`] from the command line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    /// The IPC socket to serve on (`--socket`). macOS limits a socket path to 104 bytes.
    pub socket: Option<PathBuf>,
    /// The HTTP address to serve on (`--http`).
    pub http: Option<SocketAddr>,
    /// The root game's store: a peer's socket (`--store`), or in memory.
    pub store: Option<PathBuf>,
    /// Every other game: its id, and a peer's socket or `None` for in memory (`--game`).
    pub games: Vec<(String, Option<PathBuf>)>,
}
// ANCHOR_END: options

/// A page's Content-Security-Policy: `ikigai-web`'s default with `base-uri 'self'`.
pub const PAGE_CSP: &str =
    "default-src 'self'; frame-ancestors 'none'; base-uri 'self'; form-action 'self'";

/// The HTTP address when `--http` is not given.
pub const DEFAULT_HTTP: &str = "127.0.0.1:8070";

/// The usage line.
pub const USAGE: &str = "usage: ttt-host [--http <addr>] [--socket <path>] [--store <socket>] \
                         [--game <id>[=<socket>]]…\n\
  --http <addr>          serve the game's pages over HTTP (default 127.0.0.1:8070)\n\
  --socket <path>        serve the kernel over IPC (default: ttt-host.sock in the temp dir)\n\
  --store <socket>       the ROOT game's store is the peer at <socket>\n\
  --game <id>            a game with its store in memory, at /game/<id>/\n\
  --game <id>=<socket>   a game whose store is the peer at <socket>\n\
  with no --game, games a and b are served, in memory";

/// Read the command line (without the program name).
pub fn parse_args<I: IntoIterator<Item = String>>(args: I) -> std::result::Result<Options, String> {
    let mut options = Options {
        socket: None,
        http: None,
        store: None,
        games: Vec::new(),
    };
    let mut args = args.into_iter();
    while let Some(flag) = args.next() {
        let mut value = || {
            args.next()
                .ok_or_else(|| format!("{flag} needs a value\n{USAGE}"))
        };
        match flag.as_str() {
            "--socket" => options.socket = Some(PathBuf::from(value()?)),
            "--http" => {
                let text = value()?;
                options.http = Some(
                    text.parse()
                        .map_err(|e| format!("--http {text}: not an address ({e})"))?,
                );
            }
            "--store" => options.store = Some(PathBuf::from(value()?)),
            "--game" => {
                let text = value()?;
                let (id, socket) = match text.split_once('=') {
                    Some((id, socket)) => (id.to_string(), Some(PathBuf::from(socket))),
                    None => (text.clone(), None),
                };
                tic_tac_toe::game_name(&id).map_err(|e| format!("--game {text}: {e}"))?;
                if options.games.iter().any(|(known, _)| *known == id) {
                    return Err(format!("--game {id} is given twice"));
                }
                options.games.push((id, socket));
            }
            "--help" | "-h" => return Err(USAGE.to_string()),
            other => return Err(format!("unknown argument `{other}`\n{USAGE}")),
        }
    }
    if options.games.is_empty() {
        options.games = vec![("a".into(), None), ("b".into(), None)];
    }
    Ok(options)
}

// ANCHOR: peer
/// A peer store on `socket`, mounted over the stored cell's names and checked against the
/// store contract before it is accepted.
///
/// The mount is an OVERRIDE (`MountedRemote::overriding`): names are forwarded unchanged,
/// so the peer answers `urn:iki:tutorial:ttt:stored:1:1` as that name — what the Python and
/// Deno stores bind. The check WRITES a square off the board (`7,-3`) and clears it again.
pub fn peer_store(socket: &Path) -> std::result::Result<Arc<dyn Space>, String> {
    let resolver = ikigai_ipc::connect(socket)
        .map_err(|e| format!("the store at {}: {e}", socket.display()))?;
    let mount: Arc<dyn Space> = Arc::new(MountedRemote::overriding(
        Arc::new(resolver),
        STORED_PREFIX,
        format!("ipc:{}", socket.display()),
    ));
    futures::executor::block_on(tic_tac_toe::check_store(Arc::clone(&mount))).map_err(
        |clause| {
            format!(
                "the store at {} breaks the store contract: {clause}",
                socket.display()
            )
        },
    )?;
    Ok(mount)
}
// ANCHOR_END: peer

/// The game, built: one kernel, and a corridor per game.
pub struct Host {
    kernel: Arc<Kernel>,
    games: BTreeMap<String, Scope>,
}

impl Host {
    /// Build every store (checking each peer), then the kernel and the games' corridors.
    pub fn build(options: &Options) -> std::result::Result<Host, String> {
        let root_store: Arc<dyn Space> = match &options.store {
            Some(socket) => peer_store(socket)?,
            None => Arc::new(tic_tac_toe::stored_space(Arc::default())),
        };
        let kernel = Arc::new(Kernel::with_meta_renderer(
            Arc::new(tic_tac_toe::space_with_store(root_store)),
            Arc::new(TurtleRenderer),
        ));
        let mut games = BTreeMap::new();
        for (id, socket) in &options.games {
            let store: Arc<dyn Space> = match socket {
                Some(socket) => peer_store(socket)?,
                None => Arc::new(tic_tac_toe::stored_space(Arc::default())),
            };
            let scope = tic_tac_toe::game(id, store).map_err(|e| format!("game {id}: {e}"))?;
            games.insert(id.clone(), scope);
        }
        Ok(Host { kernel, games })
    }

    /// The games, by id.
    pub fn games(&self) -> impl Iterator<Item = &str> {
        self.games.keys().map(String::as_str)
    }

    /// The kernel the IPC socket serves: every game at its edge name, and the root game's
    /// names — the stored cell included, since a socket client is the host's own user.
    pub fn ipc_kernel(&self) -> Kernel {
        self.front(true, None)
    }

    /// The kernel the HTTP edge serves: the pages, every game at its edge name, and the root
    /// game's names — but not the stored cell, which is reached through the rules or not at
    /// all from a browser.
    pub fn http_kernel(&self) -> Kernel {
        self.front(false, Some(Arc::new(self.pages())))
    }

    fn front(&self, store_open: bool, pages: Option<Arc<dyn Space>>) -> Kernel {
        let mut spaces: Vec<Arc<dyn Space>> = pages.into_iter().collect();
        spaces.push(Arc::new(Gateway {
            kernel: Arc::clone(&self.kernel),
            games: self.games.clone(),
            store_open,
        }));
        spaces.push(Arc::new(Forward {
            kernel: Arc::clone(&self.kernel),
            store_open,
        }));
        Kernel::with_meta_renderer(Arc::new(Fallback::new(spaces)), Arc::new(TurtleRenderer))
    }

    fn pages(&self) -> Pages {
        Pages {
            kernel: Arc::clone(&self.kernel),
            games: self.games.keys().cloned().collect(),
        }
    }
}

// ANCHOR: routes
/// The HTTP edge's routes: the pages and the static files. Everything else — every game
/// resource — falls through to `ikigai-web`'s mechanical `a/b/c` → `urn:a:b:c`.
///
/// A page's route loosens ONE thing in `ikigai-web`'s default CSP: `base-uri 'self'` instead
/// of `'none'`, because the page's `<base href="/game/a/">` is what makes its relative paths
/// land under the game — the default blocks it, and then `/game/a` (no trailing slash) would
/// send every request to `/game/iki/…`.
pub fn edge_config() -> EdgeConfig {
    let route = |pattern: &str, iri: &str| Route {
        pattern: pattern.to_string(),
        iri_template: iri.to_string(),
        cap: None,
        cors: None,
        csp: None,
    };
    let page = |pattern: &str, iri: &str| Route {
        csp: Some(PAGE_CSP.to_string()),
        ..route(pattern, iri)
    };
    EdgeConfig {
        routes: RouteTable::new(vec![
            page("/", "urn:ttt-host:page:root"),
            page("/game/{id}", "urn:ttt-host:page:game:{id}"),
            route("/static/htmx-2.0.4.min.js", "urn:ttt-host:static:htmx"),
            route("/static/ttt.css", "urn:ttt-host:static:ttt-css"),
            route("/static/host.css", "urn:ttt-host:static:host-css"),
        ]),
        ..EdgeConfig::default()
    }
}
// ANCHOR_END: routes

/// Serve `host` over HTTP on an already-bound listener, until it fails.
pub async fn serve_http(host: &Host, listener: tokio::net::TcpListener) -> std::io::Result<()> {
    ikigai_web::serve_with_listener(
        Arc::new(host.http_kernel()),
        ikigai_web::public_cap(),
        listener,
        edge_config(),
    )
    .await
}

/// Serve `host` over IPC on `socket`, until it fails. Blocks: run it on its own thread.
pub fn serve_ipc(host: &Host, socket: &Path) -> std::io::Result<()> {
    ikigai_ipc::serve(host.ipc_kernel(), socket)
}

/// The socket path when `--socket` is not given: `ttt-host.sock` in the temp directory, if
/// it fits the 104 bytes macOS allows a socket path.
pub fn default_socket() -> std::result::Result<PathBuf, String> {
    let path = std::env::temp_dir().join("ttt-host.sock");
    if path.as_os_str().len() >= 100 {
        return Err(format!(
            "{} is too long for a socket path; pass --socket /tmp/ttt.sock",
            path.display()
        ));
    }
    Ok(path)
}

// ANCHOR: gateway
/// `urn:game:{id}:iki:tutorial:ttt:{rest}` → `urn:iki:tutorial:ttt:{rest}`, issued in game
/// `id`'s corridor on the game's kernel. The host's act, not the request's: the request
/// names a game the host already has, and the host decides what that game's corridor holds.
struct Gateway {
    kernel: Arc<Kernel>,
    games: BTreeMap<String, Scope>,
    store_open: bool,
}

impl Space for Gateway {
    fn resolve(&self, request: &Request, _scope: &Scope) -> Resolution {
        let Some(rest) = request.target.as_str().strip_prefix(EDGE_GAMES) else {
            return Resolution::Miss;
        };
        let Some((id, name)) = rest.split_once(':') else {
            return Resolution::Miss;
        };
        let (Some(scope), Some(target)) = (self.games.get(id), game_name(name, self.store_open))
        else {
            return Resolution::Miss;
        };
        forwarded(&self.kernel, target, scope.clone())
    }

    fn entries(&self) -> Option<Vec<SpaceEntry>> {
        let names = shown(&self.kernel, self.store_open);
        Some(
            self.games
                .keys()
                .flat_map(|id| {
                    names.iter().map(move |entry| {
                        let local = entry.pattern.trim_start_matches("urn:");
                        SpaceEntry::new(format!("{EDGE_GAMES}{id}:{local}"), &entry.endpoint)
                    })
                })
                .collect(),
        )
    }
}
// ANCHOR_END: gateway

/// The root game's names, passed through to the game's kernel as they are.
struct Forward {
    kernel: Arc<Kernel>,
    store_open: bool,
}

impl Space for Forward {
    fn resolve(&self, request: &Request, _scope: &Scope) -> Resolution {
        let name = request
            .target
            .as_str()
            .strip_prefix("urn:")
            .unwrap_or_default();
        match game_name(name, self.store_open) {
            Some(target) => forwarded(&self.kernel, target, Scope::empty()),
            None => Resolution::Miss,
        }
    }

    fn entries(&self) -> Option<Vec<SpaceEntry>> {
        Some(shown(&self.kernel, self.store_open))
    }
}

/// `urn:` + `name`, if it is one of the game's names the edge serves.
fn game_name(name: &str, store_open: bool) -> Option<Iri> {
    let iri = format!("urn:{name}");
    let served = iri.starts_with(GAME_NAMES) && (store_open || !iri.starts_with(STORED_PREFIX));
    served.then(|| Iri::parse(iri).ok()).flatten()
}

/// The game kernel's entries the edge serves.
fn shown(kernel: &Kernel, store_open: bool) -> Vec<SpaceEntry> {
    kernel
        .entries()
        .unwrap_or_default()
        .into_iter()
        .filter(|entry| {
            entry.pattern.starts_with(GAME_NAMES)
                && (store_open || !entry.pattern.starts_with(STORED_PREFIX))
        })
        .collect()
}

fn forwarded(kernel: &Arc<Kernel>, target: Iri, scope: Scope) -> Resolution {
    Resolution::Hit(Resolved::new(
        Arc::new(Forwarded {
            kernel: Arc::clone(kernel),
            target,
            scope,
        }),
        Bindings::new(),
    ))
}

/// One request, re-issued on the game's kernel in a chain the host chose.
struct Forwarded {
    kernel: Arc<Kernel>,
    target: Iri,
    scope: Scope,
}

#[async_trait::async_trait]
impl Endpoint for Forwarded {
    async fn invoke(&self, inv: &Invocation<'_>) -> Result<Representation> {
        let mut request = inv.request.clone();
        request.target = self.target.clone();
        let answer = self
            .kernel
            .issue_in(request, inv.capability, self.scope.clone())
            .await?;
        // The game's kernel caches this answer and cuts it when a move changes what it read.
        // This kernel saw none of those reads, so a copy cached here could never be cut.
        Ok(answer.with_expiry(Expiry::Always))
    }

    fn name(&self) -> &str {
        "ttt-host-forward"
    }

    fn describe(&self) -> Description {
        self.kernel
            .describe(&self.target)
            .unwrap_or_else(|| Description::new("ttt-host-forward"))
    }
}

/// The pages and static files the HTTP edge serves, beside the game.
struct Pages {
    kernel: Arc<Kernel>,
    games: Vec<String>,
}

impl Space for Pages {
    fn resolve(&self, request: &Request, _scope: &Scope) -> Resolution {
        let Some(what) = request.target.as_str().strip_prefix("urn:ttt-host:") else {
            return Resolution::Miss;
        };
        let page = match what {
            "page:root" => Page::Game(None),
            "static:htmx" => Page::Static("text/javascript", HTMX),
            "static:ttt-css" => Page::Static("text/css", TTT_CSS),
            "static:host-css" => Page::Static("text/css", HOST_CSS),
            other => match other.strip_prefix("page:game:") {
                Some(id) if self.games.iter().any(|g| g == id) => Page::Game(Some(id.to_string())),
                _ => return Resolution::Miss,
            },
        };
        Resolution::Hit(Resolved::new(
            Arc::new(PageEndpoint {
                kernel: Arc::clone(&self.kernel),
                games: self.games.clone(),
                page,
            }),
            Bindings::new(),
        ))
    }

    fn entries(&self) -> Option<Vec<SpaceEntry>> {
        Some(vec![
            SpaceEntry::new("urn:ttt-host:page:root", "ttt-host-page"),
            SpaceEntry::new("urn:ttt-host:page:game:{id}", "ttt-host-page"),
        ])
    }
}

enum Page {
    /// A game's page: the root's (`None`) or game `id`'s.
    Game(Option<String>),
    /// A file, with its media type.
    Static(&'static str, &'static str),
}

struct PageEndpoint {
    kernel: Arc<Kernel>,
    games: Vec<String>,
    page: Page,
}

#[async_trait::async_trait]
impl Endpoint for PageEndpoint {
    async fn invoke(&self, inv: &Invocation<'_>) -> Result<Representation> {
        if inv.request.verb != Verb::Source {
            return Err(Error::Endpoint("a page is read, not written".into()));
        }
        let (media, text) = match &self.page {
            Page::Static(media, text) => {
                // Named by its version, so it never changes under its URL.
                let answer = Representation::new(ReprType::new(*media), text.as_bytes().to_vec());
                return Ok(if media.ends_with("javascript") {
                    answer.cacheable()
                } else {
                    answer
                });
            }
            Page::Game(id) => ("text/html", self.game_page(id.as_deref()).await?),
        };
        Ok(Representation::new(
            ReprType::new(media).with_param("charset", "utf-8"),
            text.into_bytes(),
        ))
    }

    fn name(&self) -> &str {
        "ttt-host-page"
    }

    fn describe(&self) -> Description {
        Description::new("ttt-host-page")
            .title("A page")
            .summary("A game's page, or a file it loads.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
    }
}

impl PageEndpoint {
    // ANCHOR: page
    /// A game's page: the document around the game's own `game` template, with a `<base>`
    /// at the game's path so the markup's relative paths arrive under it.
    async fn game_page(&self, id: Option<&str>) -> Result<String> {
        let request = Request::new(
            Verb::Source,
            Iri::parse(tic_tac_toe::template_name("game"))
                .map_err(|e| Error::Endpoint(format!("the game template's name: {e}")))?,
        );
        let shell = self
            .kernel
            .issue(request, &ikigai_core::Capability::root())
            .await?;
        let label = id.unwrap_or("root");
        let game = fill(&String::from_utf8_lossy(&shell.bytes), |slot| {
            match slot.name.as_str() {
                "game" => Ok(Fill::Text(label.to_string())),
                other => Err(Error::Endpoint(format!(
                    "the game template has a slot `{other}` the host does not fill"
                ))),
            }
        })?;
        let base = id.map_or_else(|| "/".to_string(), |id| format!("/game/{id}/"));
        let mut links = vec![format!("<li><a href=\"/\">the root game</a></li>")];
        for other in &self.games {
            links.push(format!(
                "<li><a href=\"/game/{other}/\">game {other}</a></li>"
            ));
        }
        Ok(format!(
            "<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n\
             <meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n\
             <meta name=\"htmx-config\" content='{{\"allowEval\":false,\"includeIndicatorStyles\":false,\"historyEnabled\":false}}'>\n\
             <title>Tic-tac-toe: {title}</title>\n<base href=\"{base}\">\n\
             <link rel=\"stylesheet\" href=\"/static/host.css\">\n\
             <link rel=\"stylesheet\" href=\"/static/ttt.css\">\n\
             <script src=\"/static/htmx-2.0.4.min.js\"></script>\n</head>\n<body>\n<main>\n\
             <h1>Tic-tac-toe: {title}</h1>\n<div class=\"ttt-boards\"><div class=\"ttt-play\">\n{game}\n</div></div>\n\
             <nav aria-label=\"Games\"><h2>Games on this host</h2><ul>{links}</ul></nav>\n</main>\n</body>\n</html>\n",
            title = tic_tac_toe::escape(&if id.is_some() {
                format!("game {label}")
            } else {
                "the root game".to_string()
            }),
            links = links.join(""),
        ))
    }
    // ANCHOR_END: page
}
