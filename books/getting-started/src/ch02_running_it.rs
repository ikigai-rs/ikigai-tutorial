//! # 2 · Running it
//!
//! Read a chapter, then have a kernel in front of you. Three ways, cheapest first.
//!
//! ## The tutorial binary (this repo, nothing else)
//!
//! ```text
//! cargo run -p ikigai-book-getting-started -- "resource oriented computing"
//! ```
//!
//! ```text
//! in  resource oriented computing
//! out resourceOrientedComputing
//! ```
//!
//! That is a complete ikigai host: a root space, a kernel around it, one resolution. It
//! is about thirty lines and you will have read all of them by the end of chapter 5.
//!
//! ## The CLI
//!
//! The full host lives in the `ikigai-cli` repository and installs a binary called
//! `ikigai`:
//!
//! ```text
//! cargo install --path crates/ikigai-cli
//! ```
//!
//! One-shot resolutions take `-c`, and `--plain` drops the decoration so output is
//! pipeable:
//!
//! ```text
//! ikigai --plain -c 'source urn:fn:toUpper in="hello"'
//! ```
//!
//! Run it with no arguments and you get a REPL with the same grammar: pipes (`|`), map
//! (`..`), named arguments, `compose`, `cache`, `cap`, and `list`. The REPL, the one-shot
//! flag and the page-assembling browser demo all drive *the same engine* — worth knowing
//! early, because it means anything you learn in one place transfers.
//!
//! Two commands worth running on your first day, because they show the system describing
//! itself rather than doing work:
//!
//! ```text
//! ikigai --plain -c 'source urn:kernel:catalog'
//! ikigai --plain -c 'source urn:kernel:actions'
//! ```
//!
//! The first is everything resolvable. The second is everything *you* may invoke, given
//! the capability you are holding — the same list an agent would be handed.
//!
//! ## The browser demo
//!
//! <https://ikigai-rs.github.io/ikigai-web-demo/> runs the kernel as WebAssembly, in the
//! page, with no server: the whole page is one resource that the in-browser kernel
//! composed. The **Control** tab shows the scheduler, the cache and its golden threads
//! updating live; the **Demo** tab is a set of runnable walkthroughs.
//!
//! It is also the only host that loads **modules** rather than linking everything in —
//! see [`super::ch08_whats_next`].
//!
//! Next: [`super::ch03_hello_resource`].
