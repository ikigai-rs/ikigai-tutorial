//! The door, for real: the binary on port 0, spoken to over TCP.

use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::process::{Child, Command, Stdio};
use std::time::Duration;

struct Host {
    child: Child,
    addr: String,
}

impl Drop for Host {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

/// Start the binary on port 0 and read the address it bound from its banner (field guide 9h:
/// never pick a port by bind-and-release).
fn start(args: &[&str]) -> Host {
    let mut child = Command::new(env!("CARGO_BIN_EXE_inspector-htmx"))
        .args(["--http", "127.0.0.1:0"])
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .expect("the binary starts");
    let mut line = String::new();
    BufReader::new(child.stdout.take().unwrap())
        .read_line(&mut line)
        .unwrap();
    let addr = line
        .trim()
        .strip_prefix("inspector: http://")
        .and_then(|a| a.strip_suffix('/'))
        .unwrap_or_else(|| panic!("a banner, not {line:?}"))
        .to_string();
    Host { child, addr }
}

fn send(host: &Host, raw: &str) -> String {
    let mut c = TcpStream::connect(&host.addr).unwrap();
    c.set_read_timeout(Some(Duration::from_secs(10))).unwrap();
    c.write_all(raw.as_bytes()).unwrap();
    let mut out = Vec::new();
    let _ = c.read_to_end(&mut out);
    String::from_utf8_lossy(&out).into_owned()
}

fn get(host: &Host, path: &str) -> String {
    send(
        host,
        &format!("GET {path} HTTP/1.1\r\nHost: {}\r\n\r\n", host.addr),
    )
}

fn post(host: &Host, path: &str, origin: &str, body: &str) -> String {
    send(
        host,
        &format!(
            "POST {path} HTTP/1.1\r\nHost: {}\r\nOrigin: {origin}\r\n\
             Content-Type: application/x-www-form-urlencoded\r\nContent-Length: {}\r\n\r\n{body}",
            host.addr,
            body.len()
        ),
    )
}

#[test]
fn every_page_and_fragment_answers_under_the_html_door_defaults() {
    let host = start(&[]);
    for path in [
        "/",
        "/inspect/cache",
        "/inspect/threads",
        "/inspect/topology",
        "/inspect/catalog",
        "/inspect/scheduler",
        "/inspect/note",
        "/inspect/fragment/cache",
        "/inspect/fragment/scheduler",
        "/ui/tokens.css",
        "/ui/shell.js",
        "/ui/htmx",
        "/inspect/live.js",
        "/inspect/inspector.css",
    ] {
        let answer = get(&host, path);
        assert!(answer.starts_with("HTTP/1.1 200"), "{path}: {answer}");
        assert!(
            answer.contains("\r\nX-Content-Type-Options: nosniff\r\n"),
            "{path}"
        );
        assert!(
            answer.contains("script-src 'self'; style-src 'self'"),
            "{path}: the page CSP"
        );
    }
    // Only the routes are public: the kernel's own names are not reachable by the mechanical rule.
    assert!(get(&host, "/kernel/cache").starts_with("HTTP/1.1 404"));
}

#[test]
fn a_detail_page_is_reached_by_the_link_the_cache_view_prints() {
    let host = start(&[]);
    get(&host, "/inspect/card");
    let cache = get(&host, "/inspect/fragment/cache");
    let at = cache
        .find(">urn:iki:tutorial:inspect:demo:card</a>")
        .expect("the card is listed");
    let href = &cache[cache[..at].rfind("href='").unwrap() + 6..];
    let href = &href[..href.find('\'').unwrap()];
    let page = get(&host, href);
    assert!(page.starts_with("HTTP/1.1 200"), "{href}: {page}");
    assert!(
        page.contains("<h1>Cache entry for <code>urn:iki:tutorial:inspect:demo:card</code></h1>"),
        "{page}"
    );
    let thread = get(
        &host,
        "/inspect/thread/urn%3Aiki%3Atutorial%3Ainspect%3Ademo%3Anote",
    );
    assert!(thread.starts_with("HTTP/1.1 200"), "{thread}");
    assert!(thread
        .contains("<title>Golden thread urn:iki:tutorial:inspect:demo:note · Inspector</title>"));
}

#[test]
fn a_write_from_another_origin_is_refused_and_one_from_this_origin_is_heard_on_the_push_stream() {
    let host = start(&[]);
    let origin = format!("http://{}", host.addr);
    get(&host, "/inspect/card");
    let refused = post(
        &host,
        "/inspect/note/write",
        "https://evil.example",
        "content=x",
    );
    assert!(refused.starts_with("HTTP/1.1 403"), "{refused}");

    // Listen, then write: the stream names the cut and the card it invalidated.
    let mut stream = TcpStream::connect(&host.addr).unwrap();
    stream
        .set_read_timeout(Some(Duration::from_secs(10)))
        .unwrap();
    write!(
        stream,
        "GET /_ikigai/push HTTP/1.1\r\nHost: {}\r\n\r\n",
        host.addr
    )
    .unwrap();
    let mut reader = BufReader::new(stream);
    let mut seen = String::new();
    while !seen.contains("event: ready") {
        reader.read_line(&mut seen).unwrap();
    }
    let ok = post(&host, "/inspect/note/write", &origin, "content=hello+there");
    assert!(
        ok.starts_with("HTTP/1.1 200") && ok.ends_with("written"),
        "{ok}"
    );
    while !seen.contains("event: cut") {
        assert!(
            reader.read_line(&mut seen).unwrap() > 0,
            "the stream ended: {seen}"
        );
    }
    reader.read_line(&mut seen).unwrap();
    reader.read_line(&mut seen).unwrap();
    assert!(
        seen.contains("urn:iki:tutorial:inspect:demo:card"),
        "{seen}"
    );
    let card = get(&host, "/inspect/card");
    assert!(card.contains("<p>hello there</p>"), "{card}");
}

#[test]
fn the_artifacts_can_come_from_a_directory() {
    let dir = std::env::temp_dir().join(format!("inspector-htmx-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("header.html"),
        "<nav aria-label=\"Inspector\">from the directory</nav>",
    )
    .unwrap();
    let host = start(&["--artifacts", dir.to_str().unwrap()]);
    let page = get(&host, "/inspect/scheduler");
    assert!(page.contains(">from the directory</nav>"), "{page}");
    assert!(
        page.contains("<h1>Scheduler</h1>"),
        "the rest stays compiled in: {page}"
    );
    drop(host);
    std::fs::remove_dir_all(&dir).unwrap();
}
