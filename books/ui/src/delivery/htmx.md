# Server-rendered HTML, with htmx and push

Part I built the inspector as resources and never said where they would be shown. This chapter
puts them in a browser: a host that serves the inspector's pages over HTTP, with htmx swapping
views in place and the push stream keeping them current. It is the first delivery target, and
it is deliberately thin. Everything a page shows is the `inspector` crate's; this crate adds
only what a door owns.

If you played the board in [Tic-tac-toe, V: a board you can play](../../applied/tic-tac-toe-5.html),
you have seen the pattern: htmx markup served as a resource, and a door that maps a URL to a
name. The inspector follows the same pattern, with the rules this book has collected.

## Running it

From a checkout of this book's repository:

```text
cargo run -p inspector-htmx
```

It prints the address it is serving on (`inspector: http://127.0.0.1:8071/`); open it. The
cache view is the front page. Open the note in a second tab, write it, and watch the card's
row in the first tab turn `cut`. Follow a thread's link, then press Back.

`--http 127.0.0.1:0` picks a free port and prints it. `--artifacts DIR` serves the inspector's
artifacts from a directory instead of the compiled-in copies: copy `crates/inspector/artifacts`
somewhere, change `cache.xsl`, and reload. The next chapter of Part I to be written,
[Artifacts: from a directory or compiled in](../model/artifacts.md), is about that.

## What the door owns

**The route table.** Each page and each fragment has a URL, ikigai-ui's shared artifacts are at
`/ui/{name}`, and nothing else is public: the table is the whole surface (`routes_only`), so the
kernel's own names are not reachable by accident.

```rust,ignore
{{#include ../../../../crates/inspector-htmx/src/lib.rs:routes}}
```

A thread's name is an IRI, so its route takes the segment raw (`{+name}`) and its link carries
it percent-encoded. Every other variable is strict: a value with a `:` or a `/` in it is refused
before it can name some other resource.

**The door's defaults.** `EdgeConfig::html_door` is the cli door library's one call for an
HTML door: a page CSP that allows scripts and styles from this origin only, never inline; an
Origin check that refuses any unsafe method (a form's POST) whose `Origin` is not this door's;
and the security headers (`nosniff`, no referrer, `Cross-Origin-Opener-Policy`). An artifact can
change what a page says, never what the door allows.

**Who is asking.** Every request is issued under the viewer's capability, never root:
inspect the kernel, hear its cuts, write the note. A door that grants less shows less, and the
kernel, not the inspector, enforces it.

**The push stream** at `/_ikigai/push`, which the inspector's `live.js` listens to.

## In Rust

The door, started in this process on a free port and asked for the front page, compiled and run
by `mdbook test`:

```rust
# extern crate inspector_htmx;
# extern crate tokio;
use std::io::{Read, Write};

let runtime = tokio::runtime::Runtime::new().unwrap();
let listener = runtime.block_on(tokio::net::TcpListener::bind("127.0.0.1:0")).unwrap();
let addr = listener.local_addr().unwrap();
runtime.spawn(inspector_htmx::serve(listener, None));

let mut c = std::net::TcpStream::connect(addr).unwrap();
write!(c, "GET / HTTP/1.1\r\nHost: {addr}\r\n\r\n").unwrap();
let mut answer = String::new();
c.read_to_string(&mut answer).unwrap();
assert!(answer.starts_with("HTTP/1.1 200"));
assert!(answer.contains("script-src 'self'; style-src 'self'"));
assert!(answer.contains("<title>Cache · Inspector</title>"));
```

## How it is checked

- **Snapshots.** Every page is held to a file in the repository (ikigai-ui's test kit), so a
  change to a page is a change someone reviews.
- **axe**, through ikigai-ui's reusable workflow: every page, in every theme, under light,
  dark and more-contrast media, served with the door's CSP, with no WCAG 2.2 A or AA violation,
  page error or console error.
- **In a real browser** (`a11y/inspector.mjs`): every page at 320 pixels wide without sideways
  scrolling; the first Tab reaches the skip link and the skip link reaches the view; a nav link
  swaps the view, pushes its URL and moves focus to the new heading; Back restores the view; and
  a cut refetches the view without moving focus or speaking in the status region.
- **The door itself** (`crates/inspector-htmx/tests`): the real binary, every route, a write
  from another origin refused, and a write from this one heard on the push stream.

## What it waits on

The form that writes the note works without a script, but answers a bare line of text:
redirecting to the note's page after a write is not something the door library does for a
plain form yet. And the push mapping lives in the inspector's own `live.js` until ikigai-ui's
shell script carries it.
