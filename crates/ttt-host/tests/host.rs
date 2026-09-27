//! `ttt-host`, end to end: real sockets, a real HTTP listener, and peer stores served over
//! IPC by this test — one that keeps the store contract and one that does not.
//!
//! ```text
//! cargo test -p ttt-host --test host -- --nocapture
//! ```
//!
//! Hermetic: every socket is a fresh short path in the temp directory (macOS allows a socket
//! path 104 bytes, so never the scratchpad), and every HTTP listener binds `127.0.0.1:0`.

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ikigai_core::{
    ArgRef, EndpointSpace, FnEndpoint, Invocation, Iri, Kernel, ReprType, Representation, Request,
    UriTemplate, Verb,
};
use ikigai_resolve::Resolver;
use tic_tac_toe::{stored_name, stored_space, CellStore, STORED};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use ttt_host::{parse_args, Host, Options};

/// A fresh socket path, short enough for macOS.
fn socket_path() -> PathBuf {
    static N: AtomicUsize = AtomicUsize::new(0);
    let path = std::env::temp_dir().join(format!(
        "ttt-{}-{}.sock",
        std::process::id(),
        N.fetch_add(1, Ordering::SeqCst)
    ));
    assert!(path.as_os_str().len() < 100, "{}", path.display());
    path
}

/// Serve `kernel` on a fresh socket, on its own thread, and wait until it answers.
fn serve_peer(kernel: Kernel) -> PathBuf {
    let path = socket_path();
    let at = path.clone();
    std::thread::spawn(move || {
        let _ = ikigai_ipc::serve(kernel, &at);
    });
    for _ in 0..200 {
        if ikigai_ipc::connect(&path).is_ok() {
            return path;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    panic!("the peer on {} never answered", path.display());
}

fn options(args: &[&str]) -> Options {
    parse_args(args.iter().map(|a| a.to_string())).expect("valid arguments")
}

/// One HTTP exchange, `Connection: close` style, as `ikigai-web`'s own doctests do it.
async fn http(addr: SocketAddr, method: &str, path: &str) -> (u16, String) {
    let (status, _, body) = http_raw(addr, method, path).await;
    (status, body)
}

/// [`http`], with the response's head (status line and headers) too.
async fn http_raw(addr: SocketAddr, method: &str, path: &str) -> (u16, String, String) {
    let mut stream = tokio::net::TcpStream::connect(addr).await.expect("connect");
    let request = format!(
        "{method} {path} HTTP/1.1\r\nHost: test\r\nHX-Request: true\r\nContent-Length: 0\r\n\r\n"
    );
    stream.write_all(request.as_bytes()).await.expect("write");
    let mut out = Vec::new();
    stream.read_to_end(&mut out).await.expect("read");
    let text = String::from_utf8_lossy(&out).into_owned();
    let status = text[9..12].parse().expect("a status code");
    let (head, body) = text.split_once("\r\n\r\n").unwrap_or((&text, ""));
    (status, head.to_string(), body.to_string())
}

/// Serve `host` over HTTP on an ephemeral port.
async fn serve_http(host: Arc<Host>) -> SocketAddr {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let addr = listener.local_addr().expect("an address");
    tokio::spawn(async move {
        let _ = ttt_host::serve_http(&host, listener).await;
    });
    addr
}

// ANCHOR: http_move
/// A move posted to game `a`'s path re-renders game `a`'s board, and only game `a`'s: the
/// root and game `b` are other corridors on the same kernel.
#[tokio::test(flavor = "multi_thread")]
async fn a_move_posted_over_http_re_renders_that_games_board() {
    let host = Arc::new(Host::build(&options(&[])).expect("a host"));
    let addr = serve_http(host).await;

    let (status, reply) = http(addr, "POST", "/game/a/iki/tutorial/ttt/view/play/1/1").await;
    assert_eq!((status, reply.as_str()), (200, "X plays 1,1. O to play."));

    let (status, board) = http(addr, "GET", "/game/a/iki/tutorial/ttt/view/board").await;
    assert_eq!(status, 200);
    assert!(
        board.contains("aria-label=\"X at 1,1\">X</button>"),
        "{board}"
    );
    for other in [
        "/game/b/iki/tutorial/ttt/view/board",
        "/iki/tutorial/ttt/view/board",
    ] {
        let (_, board) = http(addr, "GET", other).await;
        assert!(!board.contains("X at 1,1"), "{other}: {board}");
    }
}
// ANCHOR_END: http_move

/// The pages: each game's at `/game/{id}/` with a `<base>` there, the shell being the game's
/// own `game` template; the files it loads; and what the edge does NOT serve.
#[tokio::test(flavor = "multi_thread")]
async fn the_edge_serves_the_pages_and_only_the_games_names() {
    let host = Arc::new(Host::build(&options(&["--game", "a"])).expect("a host"));
    let addr = serve_http(host).await;

    // No trailing slash: the page's `<base>` puts its relative paths under the game anyway,
    // and the page's CSP lets it (ikigai-web's default says `base-uri 'none'`).
    let (status, head, page) = http_raw(addr, "GET", "/game/a").await;
    assert_eq!(status, 200);
    assert!(head.contains("base-uri 'self'"), "{head}");
    assert!(page.contains("<base href=\"/game/a/\">"), "{page}");
    assert!(page.contains("aria-label=\"Game a\""), "{page}");
    assert!(
        page.contains("hx-get=\"iki/tutorial/ttt/view/status\""),
        "{page}"
    );
    let (_, root) = http(addr, "GET", "/").await;
    assert!(root.contains("<base href=\"/\">"), "{root}");

    let (status, htmx) = http(addr, "GET", "/static/htmx-2.0.4.min.js").await;
    assert_eq!(status, 200);
    assert!(htmx.starts_with("var htmx=function()"));
    assert_eq!(http(addr, "GET", "/static/ttt.css").await.0, 200);

    // A game the host does not have, and the store behind the rules, are not there.
    assert_eq!(http(addr, "GET", "/game/zz/").await.0, 404);
    assert_eq!(
        http(addr, "GET", "/game/zz/iki/tutorial/ttt/view/board")
            .await
            .0,
        404
    );
    assert_eq!(
        http(addr, "GET", "/game/a/iki/tutorial/ttt/stored/0/0")
            .await
            .0,
        404
    );
    assert_eq!(
        http(addr, "POST", "/iki/tutorial/ttt/stored/0/0").await.0,
        404
    );
    // A play is a write: reading it is refused by its declared verbs.
    assert_eq!(
        http(addr, "GET", "/game/a/iki/tutorial/ttt/view/play/1/1")
            .await
            .0,
        405
    );
}

// ANCHOR: peer_ok
/// A Rust store served over a real socket keeps the contract, so the host accepts it, and a
/// game played over HTTP lands in that other process's store.
#[tokio::test(flavor = "multi_thread")]
async fn a_peer_store_that_keeps_the_contract_plays_a_game() {
    let store = Arc::new(CellStore::default());
    let peer = serve_peer(Kernel::new(Arc::new(stored_space(store.clone()))));
    let args = ["--game".to_string(), format!("p={}", peer.display())];
    let host = Arc::new(Host::build(&parse_args(args).expect("valid")).expect("accepted"));
    let addr = serve_http(host).await;

    for (x, y) in [(0, 0), (1, 0), (1, 1), (2, 0), (2, 2)] {
        let (status, _) = http(
            addr,
            "POST",
            &format!("/game/p/iki/tutorial/ttt/view/play/{x}/{y}"),
        )
        .await;
        assert_eq!(status, 200);
    }
    let (_, status) = http(addr, "GET", "/game/p/iki/tutorial/ttt/view/status").await;
    assert_eq!(status, "X has won.");

    // The marks are in the peer, not in the host: ask the peer directly.
    let direct = ikigai_ipc::connect(&peer).expect("the peer");
    let read = Request::new(Verb::Source, Iri::parse(stored_name(1, 1)).unwrap());
    let (mark, _) = direct.issue(read).expect("a mark");
    assert_eq!(mark.bytes, b"X");
    assert!(store.reads.load(Ordering::SeqCst) > 0);
}
// ANCHOR_END: peer_ok

// ANCHOR: peer_refused
/// A peer that answers an unplayed square with the empty string breaks the contract, and
/// the host refuses to start with it, naming the clause.
#[test]
fn a_peer_store_that_breaks_the_contract_is_refused_at_startup() {
    let broken = EndpointSpace::new().bind(
        UriTemplate::parse(STORED).expect("parses"),
        FnEndpoint::new("broken-store", |_: &Invocation<'_>| {
            Ok(Representation::new(ReprType::new("text/plain"), Vec::new()).cacheable())
        }),
    );
    let peer = serve_peer(Kernel::new(Arc::new(broken)));
    let refusal = match Host::build(&options(&["--store", peer.to_str().unwrap()])) {
        Err(refusal) => refusal,
        Ok(_) => panic!("a broken store was accepted"),
    };
    assert!(refusal.contains("breaks the store contract"), "{refusal}");
    assert!(refusal.contains("NotFound"), "{refusal}");
}
// ANCHOR_END: peer_refused

/// A socket that nobody serves is refused too, before anything listens.
#[test]
fn a_peer_that_is_not_there_is_refused_at_startup() {
    let missing = socket_path();
    assert!(Host::build(&options(&["--game", &format!("x={}", missing.display())])).is_err());
}

/// The IPC face: the root game under its own names, and game `a` at its edge name — the
/// HTTP path spelled as an IRI.
#[test]
fn the_socket_serves_the_root_game_and_every_game_by_its_edge_name() {
    let host = Host::build(&options(&["--game", "a"])).expect("a host");
    let socket = serve_peer(host.ipc_kernel());
    let client = ikigai_ipc::connect(&socket).expect("the host");
    let issue = |verb: Verb, name: &str| -> String {
        let request = Request::new(verb, Iri::parse(name).unwrap())
            .with_arg("content", ArgRef::Inline(Vec::new()));
        let (answer, _) = client.issue(request).expect(name);
        String::from_utf8_lossy(&answer.bytes).into_owned()
    };
    assert_eq!(
        issue(Verb::Sink, "urn:iki:tutorial:ttt:move:1:1"),
        "X plays 1,1"
    );
    assert_eq!(
        issue(Verb::Source, "urn:iki:tutorial:ttt:board"),
        "---\n-X-\n---"
    );
    assert_eq!(
        issue(Verb::Source, "urn:game:a:iki:tutorial:ttt:board"),
        "---\n---\n---"
    );
    assert_eq!(
        issue(Verb::Source, "urn:game:a:iki:tutorial:ttt:view:status"),
        "X to play."
    );
    // The store is open over the socket (it is the host's own user), and it is the root's.
    assert_eq!(issue(Verb::Exists, &stored_name(1, 1)), "true");
    assert_eq!(
        issue(Verb::Exists, "urn:game:a:iki:tutorial:ttt:stored:1:1"),
        "false"
    );
}

#[test]
fn the_command_line_says_what_to_serve() {
    let defaults = options(&[]);
    assert_eq!(
        defaults.games,
        [("a".to_string(), None), ("b".to_string(), None)]
    );
    let custom = options(&[
        "--http",
        "127.0.0.1:9999",
        "--game",
        "py=/tmp/py.sock",
        "--game",
        "x",
        "--store",
        "/tmp/root.sock",
    ]);
    assert_eq!(custom.http, Some("127.0.0.1:9999".parse().unwrap()));
    assert_eq!(
        custom.store,
        Some(Path::new("/tmp/root.sock").to_path_buf())
    );
    assert_eq!(
        custom.games,
        [
            (
                "py".to_string(),
                Some(Path::new("/tmp/py.sock").to_path_buf())
            ),
            ("x".to_string(), None)
        ]
    );
    for bad in [
        vec!["--game", "a b"],
        vec!["--game", "a", "--game", "a"],
        vec!["--http", "nowhere"],
        vec!["--frob"],
        vec!["--game"],
    ] {
        assert!(
            parse_args(bad.iter().map(|a| a.to_string())).is_err(),
            "{bad:?}"
        );
    }
}
