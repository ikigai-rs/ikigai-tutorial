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

/// The path ↔ IRI rule as `ikigai-web` 0.1.30's edge applies it — what the tic-tac-toe
/// README states and a Python or Deno app that answers every path the way this host does has
/// to copy. The markup only ever sends plain relative paths; this is everything else.
#[tokio::test(flavor = "multi_thread")]
async fn the_edge_decodes_a_path_segment_by_segment_and_refuses_a_malformed_escape() {
    let host = Arc::new(Host::build(&options(&["--game", "a"])).expect("a host"));
    let addr = serve_http(host).await;
    let board = "/iki/tutorial/ttt/view/board";
    let a = "/game/a";
    let play = "/game/a/iki/tutorial/ttt/view/play";

    for (method, path, status, says) in [
        // `+` in the PATH is a `+`: it reaches the game and the coordinate rule refuses it,
        // exactly as it refuses the escaped spelling.
        (
            "POST",
            format!("{play}/+1/0"),
            400,
            "`+1` is not an integer",
        ),
        (
            "POST",
            format!("{play}/%2B1/0"),
            400,
            "`+1` is not an integer",
        ),
        // `%2F` is data inside its segment, never a separator: the coordinate is `1/1`, and
        // `/game%2Fa/…` is the IRI `urn:game/a:…`, not game `a`.
        (
            "POST",
            format!("{play}/1%2F1/0"),
            400,
            "`1/1` is not an integer",
        ),
        ("GET", format!("/game%2Fa{board}"), 404, "urn:game/a:iki"),
        // A malformed escape is refused before anything resolves; so is a non-UTF-8 byte.
        (
            "GET",
            format!("{play}/%zz/0"),
            400,
            "malformed percent-escape",
        ),
        (
            "GET",
            format!("{a}{board}%"),
            400,
            "malformed percent-escape",
        ),
        (
            "GET",
            format!("{a}{board}%4"),
            400,
            "malformed percent-escape",
        ),
        ("GET", format!("{a}{board}%FF"), 400, "not UTF-8"),
        // Empty segments are dropped; `.` and `..` are segments like any other.
        ("GET", format!("{a}/{board}"), 200, "ttt-grid"),
        (
            "GET",
            format!("{a}/iki/tutorial/ttt/view/../board"),
            404,
            "view:..:board",
        ),
        // A `:` inside a segment is not escaped on the way to the IRI: segments are joined
        // with `:`, so it adds an IRI segment whether it came escaped or literal.
        (
            "GET",
            format!("{a}/iki%3Atutorial/ttt/view/board"),
            200,
            "ttt-grid",
        ),
        (
            "GET",
            format!("{a}/iki:tutorial/ttt/view/board"),
            200,
            "ttt-grid",
        ),
        ("GET", format!("/game/a:b{board}"), 404, "urn:game:a:b:iki"),
        // A well-formed escape of an ordinary character is that character.
        ("POST", format!("{play}/%31/0"), 200, "X plays 1,0."),
        // The query is form-encoded (`+` is a space there) and a malformed escape in it is
        // refused too; the views read no query.
        ("GET", format!("{a}{board}?x=a+b"), 200, "ttt-grid"),
        (
            "GET",
            format!("{a}{board}?x=%zz"),
            400,
            "malformed percent-escape",
        ),
    ] {
        let (got, body) = http(addr, method, &path).await;
        assert_eq!(got, status, "{method} {path}: {body}");
        assert!(body.contains(says), "{method} {path}: {body}");
    }

    // The deadlines and the connection cap are ikigai-web's defaults; the host keeps them.
    let edge = ttt_host::edge_config();
    assert_eq!(edge.header_timeout, Duration::from_secs(10));
    assert_eq!(edge.body_timeout, Duration::from_secs(30));
    assert_eq!(edge.write_timeout, Duration::from_secs(30));
    assert_eq!(edge.max_connections, 256);
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
    assert!(custom.commands.is_empty());
    assert_eq!(
        options(&["-c", "source urn:iki:tutorial:ttt:board", "-c", "cache x"]).commands,
        ["source urn:iki:tutorial:ttt:board", "cache x"]
    );
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
        vec!["-c"],
    ] {
        assert!(
            parse_args(bad.iter().map(|a| a.to_string())).is_err(),
            "{bad:?}"
        );
    }
}

/// The games are listed in one order — sorted — whatever order `--game` named them: the
/// startup list, every page's navigation, and the catalog's gateway names.
#[tokio::test(flavor = "multi_thread")]
async fn the_games_are_listed_sorted_whatever_order_the_flags_came_in() {
    let host = Arc::new(Host::build(&options(&["--game", "b", "--game", "a"])).expect("a host"));
    assert_eq!(host.games().collect::<Vec<_>>(), ["a", "b"]);

    let in_catalog: Vec<String> = host
        .ipc_kernel()
        .entries()
        .unwrap_or_default()
        .into_iter()
        .filter_map(|entry| {
            let rest = entry.pattern.strip_prefix("urn:game:")?;
            Some(rest.split_once(':')?.0.to_string())
        })
        .fold(Vec::new(), |mut seen, id| {
            if seen.last() != Some(&id) {
                seen.push(id);
            }
            seen
        });
    assert_eq!(in_catalog, ["a", "b"]);

    let addr = serve_http(host).await;
    let (_, page) = http(addr, "GET", "/game/b/").await;
    let nav = &page[page.find("<nav").expect("a navigation")..];
    let a = nav
        .find("href=\"/game/a/\"")
        .expect("game a in the navigation");
    let b = nav
        .find("href=\"/game/b/\"")
        .expect("game b in the navigation");
    assert!(a < b, "{nav}");
}

/// The root game has a gateway name too, so a client can spell every game one way:
/// `urn:game:root:…` over the socket, `/game/root/…` over HTTP. The plain names still work.
#[tokio::test(flavor = "multi_thread")]
async fn the_root_game_answers_at_its_gateway_name_too() {
    let host = Arc::new(Host::build(&options(&["--game", "a"])).expect("a host"));
    let socket = serve_peer(host.ipc_kernel());
    let client = ikigai_ipc::connect(&socket).expect("the host");
    let issue = |verb: Verb, name: &str| -> String {
        let request = Request::new(verb, Iri::parse(name).unwrap())
            .with_arg("content", ArgRef::Inline(Vec::new()));
        let (answer, _) = client.issue(request).expect(name);
        String::from_utf8_lossy(&answer.bytes).into_owned()
    };
    assert_eq!(
        issue(Verb::Sink, "urn:game:root:iki:tutorial:ttt:move:1:1"),
        "X plays 1,1"
    );
    assert_eq!(
        issue(Verb::Source, "urn:iki:tutorial:ttt:board"),
        "---\n-X-\n---"
    );
    assert_eq!(
        issue(Verb::Source, "urn:game:root:iki:tutorial:ttt:board"),
        "---\n-X-\n---"
    );
    assert_eq!(
        issue(Verb::Exists, "urn:game:root:iki:tutorial:ttt:stored:1:1"),
        "true"
    );
    assert_eq!(
        issue(Verb::Source, "urn:game:a:iki:tutorial:ttt:board"),
        "---\n---\n---"
    );
    // One entry per resource: the root's names are in the catalog under their plain names.
    let entries = host.ipc_kernel().entries().unwrap_or_default();
    assert!(entries
        .iter()
        .all(|e| !e.pattern.starts_with("urn:game:root:")));

    let addr = serve_http(host).await;
    let (status, board) = http(addr, "GET", "/game/root/iki/tutorial/ttt/view/board").await;
    assert_eq!(status, 200);
    assert!(board.contains("X at 1,1"), "{board}");
    let (status, page) = http(addr, "GET", "/game/root").await;
    assert_eq!(status, 200);
    assert!(page.contains("<base href=\"/game/root/\">"), "{page}");
    assert!(page.contains("aria-label=\"Game root\""), "{page}");
    assert_eq!(
        http(addr, "GET", "/game/root/iki/tutorial/ttt/stored/1/1")
            .await
            .0,
        404
    );
    // `root` is the root game's, so no `--game` can take it.
    assert!(parse_args(["--game".to_string(), "root".to_string()]).is_err());
}

/// `--http 127.0.0.1:0` binds a port the system chooses, and the startup line names THAT
/// port, so a script can find the host it started.
#[test]
fn the_startup_line_names_the_port_it_bound() {
    use std::io::{BufRead, BufReader, Read, Write};
    let socket = socket_path();
    let mut child = std::process::Command::new(env!("CARGO_BIN_EXE_ttt-host"))
        .args(["--http", "127.0.0.1:0", "--socket"])
        .arg(&socket)
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("ttt-host starts");
    // The reader is HELD until the child is killed. Dropped after the first line, it closes
    // the pipe while the host still has its second line (`IPC on …`) to write; `eprintln!`
    // then panics on EPIPE, the host dies, and the request below reads a connection reset —
    // intermittently, since it depends on which side gets there first (seen 3 runs in 5).
    let mut stderr = BufReader::new(child.stderr.take().expect("stderr"));
    let mut line = String::new();
    stderr.read_line(&mut line).expect("a startup line");
    let addr = line
        .strip_prefix("ttt-host: http://")
        .and_then(|rest| rest.split_once('/'))
        .map(|(addr, _)| addr.to_string())
        .unwrap_or_default();
    let port: u16 = addr
        .rsplit_once(':')
        .map_or(0, |(_, p)| p.parse().unwrap_or(0));
    let answered = std::net::TcpStream::connect(("127.0.0.1", port)).and_then(|mut stream| {
        stream.write_all(b"GET / HTTP/1.1\r\nHost: test\r\nConnection: close\r\n\r\n")?;
        let mut out = String::new();
        stream.read_to_string(&mut out)?;
        Ok(out)
    });
    let _ = child.kill();
    let _ = child.wait();
    drop(stderr);
    let _ = std::fs::remove_file(&socket);
    assert_ne!(port, 0, "{line}");
    assert!(
        answered
            .as_deref()
            .is_ok_and(|out| out.starts_with("HTTP/1.1 200")),
        "{line}: {answered:?}"
    );
}

// ANCHOR: run_lines
/// `-c`: the reader's own commands, run on the game's kernel in this process with the store
/// in ANOTHER process — the verdicts are that kernel's, `[computed]` then `[cached]`, and a
/// trace shows the peer's own span under the stored cell.
#[test]
fn run_lines_answer_with_the_game_kernels_verdicts_over_a_peer_store() {
    let peer = serve_peer(Kernel::new(Arc::new(stored_space(Arc::default()))));
    let args = ["--store".to_string(), peer.display().to_string()];
    let host = Host::build(&parse_args(args).expect("valid")).expect("accepted");
    let lines: Vec<String> = [
        "source urn:iki:tutorial:ttt:cell:1:1",
        "sink urn:iki:tutorial:ttt:move:1:1",
        "trace urn:iki:tutorial:ttt:cell:1:1",
        "source urn:iki:tutorial:ttt:board",
        "source urn:iki:tutorial:ttt:board",
        "sink urn:iki:tutorial:ttt:move:1:1",
    ]
    .map(String::from)
    .to_vec();
    let out = host.run(&lines);
    assert!(
        out.starts_with("-\n[computed]\nX plays 1,1\n[uncacheable]\n"),
        "{out}"
    );
    assert!(
        out.contains("---\n-X-\n---\n[computed]\n---\n-X-\n---\n[cached]\n"),
        "{out}"
    );
    // The stored cell twice: the mount's node in this kernel, and the peer's own under it.
    assert_eq!(
        out.matches("urn:iki:tutorial:ttt:stored:1:1 ").count(),
        2,
        "{out}"
    );
    assert!(out.ends_with("error: invalid argument `x, y`: 1,1 is taken — X played there\n"));
}
// ANCHOR_END: run_lines
