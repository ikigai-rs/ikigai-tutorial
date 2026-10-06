//! **The gonk Book's own kernel, in the page.**
//!
//! A chapter's runnable cell — `<div class="ikigai-run" data-cmd="…">` in the markdown — is
//! turned into a Run button by `js/run.js`, the SAME script The ikigai Book uses (the gonk
//! book's `js/run.js` is a symlink to it). That script imports `wasm/book_wasm.js` beside the
//! book's root; for this book, `pages.yml` binds THIS crate under that file name, so one
//! script serves two books whose kernels share nothing.
//!
//! The kernel is [`ledger_host::kernel`]: `ikigai-ledger` over an in-memory `ikigai-store`,
//! with the story clock — the same composition the chapter's Rust examples build and its
//! tests run. Nothing is mocked and nothing is a recording: Run files an item, lists it and
//! ranks it in the reader's browser, under the CLI's own engine, so `sink`, `source`,
//! `item=2` and a quoted body mean exactly what they mean at `ikigai`.
//!
//! One kernel per page, in a thread-local, shared by every cell on it — a ledger is state, and
//! "file it, then list it" is only demonstrable against state that persists between two
//! Runs. Each page load starts an empty ledger.
//!
//! Exports, by the names `js/run.js` calls: `evalLineAsync` and `boundNames`. The others the
//! ikigai book's kernel exports (games, scenarios, clocks, push) are not here, because no
//! page of this book places a widget that calls them; `pages.yml` asserts the two that are.

use std::rc::Rc;

use ikigai_core::{Kernel, Space};
use ikigai_engine::Engine;
use wasm_bindgen::prelude::*;

thread_local! {
    // `Rc` so an async eval can own a handle across `.await` points — a thread-local borrow
    // cannot span an await.
    static PAGE: Rc<Engine> = Rc::new(page_engine());
}

/// The page's kernel: the chapter's own host, [`ledger_host::kernel`].
pub fn page_kernel() -> Kernel {
    ledger_host::kernel()
}

/// The engine a cell's line is evaluated by, over [`page_kernel`].
pub fn page_engine() -> Engine {
    Engine::new(page_kernel())
}

/// One evaluated line as JSON for the page: `{ "kind", "text", "cache" }`, the shape
/// `js/run.js` renders (and `crates/book-wasm` produces).
pub fn action_to_json(action: ikigai_engine::Action) -> String {
    use ikigai_engine::Action;
    let (kind, text, cache) = match action {
        Action::Output(entry) => match entry.result {
            Ok(out) => ("output", out, entry.cache.label().unwrap_or_default()),
            Err(err) => ("error", err, String::new()),
        },
        Action::Help => ("help", ikigai_engine::HELP.to_string(), String::new()),
        Action::Clear => ("clear", String::new(), String::new()),
        Action::Quit => ("quit", String::new(), String::new()),
        Action::Noop => ("noop", String::new(), String::new()),
    };
    serde_json::json!({ "kind": kind, "text": text, "cache": cache }).to_string()
}

/// Called by wasm-bindgen when the module is instantiated: a Rust panic reaches the browser
/// console as a message rather than an opaque `unreachable`.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Evaluate one REPL line against the page's kernel; resolves to [`action_to_json`]'s JSON.
#[wasm_bindgen(js_name = evalLineAsync)]
pub fn eval_line_async(line: String) -> js_sys::Promise {
    let engine = PAGE.with(Rc::clone);
    wasm_bindgen_futures::future_to_promise(async move {
        Ok(JsValue::from_str(&action_to_json(
            engine.eval_async(&line).await,
        )))
    })
}

/// The names the page's kernel binds, one per line.
#[wasm_bindgen(js_name = boundNames)]
pub fn bound_names() -> String {
    ledger_host::space()
        .entries()
        .unwrap_or_default()
        .into_iter()
        .map(|entry| entry.pattern)
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use book_transcripts::{matches, normalize};

    /// One runnable cell: its line, and the output the page shows under "Expected output".
    #[derive(Debug)]
    struct Cell {
        cmd: String,
        expected: String,
    }

    fn unescape(text: &str) -> String {
        text.replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&amp;", "&")
    }

    /// The cells a page places, in page order.
    fn cells(page: &str) -> Vec<Cell> {
        let mut found = Vec::new();
        let mut rest = page;
        while let Some(at) = rest.find("data-cmd='") {
            let after = &rest[at + "data-cmd='".len()..];
            let end = after.find('\'').expect("a closed data-cmd");
            let cmd = unescape(&after[..end]);
            let open = "<pre class=\"ikigai-run-expected\">";
            let from = after
                .find(open)
                .expect("every cell shows its expected output");
            let body = &after[from + open.len()..];
            let close = body.find("</pre>").expect("a closed expected output");
            found.push(Cell {
                cmd,
                expected: unescape(&body[..close]),
            });
            rest = &body[close..];
        }
        found
    }

    /// What a cell prints, as `js/run.js` renders a reply: the text, then the cache verdict
    /// in brackets; an error as `error: …`.
    fn render(json: &str) -> String {
        let reply: serde_json::Value = serde_json::from_str(json).expect("the reply is JSON");
        let mut out = String::new();
        if reply["kind"] == "error" {
            out.push_str(&format!(
                "error: {}\n",
                reply["text"].as_str().unwrap_or_default()
            ));
        } else {
            out.push_str(reply["text"].as_str().unwrap_or_default());
            if !out.ends_with('\n') {
                out.push('\n');
            }
        }
        if let Some(cache) = reply["cache"].as_str().filter(|c| !c.is_empty()) {
            out.push_str(&format!("[{cache}]\n"));
        }
        out
    }

    /// Every cell of a page, run natively in PAGE ORDER against one fresh page kernel — what
    /// the page does — and compared with the output the page shows. `…` in the page matches
    /// what the kernel mints (an item's id), as it does in a transcript.
    fn run_page_in_order(file: &str, expected_cells: usize) {
        let path = format!("{}/../../books/gonk/src/{file}", env!("CARGO_MANIFEST_DIR"));
        let page = std::fs::read_to_string(&path).expect("the page is readable");
        let cells = cells(&page);
        assert_eq!(cells.len(), expected_cells, "{file}'s cells: {cells:#?}");
        let engine = page_engine();
        let mut misses = Vec::new();
        for (i, cell) in cells.iter().enumerate() {
            let printed = render(&action_to_json(futures::executor::block_on(
                engine.eval_async(&cell.cmd),
            )));
            let expected = normalize(cell.expected.lines());
            let actual = normalize(printed.lines());
            if !matches(&expected, &actual) {
                misses.push(format!(
                    "cell {i}: {}\n  the page says:\n    {}\n  the kernel printed:\n    {}",
                    cell.cmd,
                    expected.join("\n    "),
                    actual.join("\n    ")
                ));
            }
        }
        assert!(misses.is_empty(), "{file}:\n{}", misses.join("\n"));
    }

    #[test]
    fn a_ledger_of_your_own_answers_as_the_page_says_in_page_order() {
        run_page_in_order("ledger/your-own.md", 10);
    }

    #[test]
    fn the_page_kernel_binds_the_ledger_and_the_store() {
        let names = bound_names();
        assert!(names.contains("urn:iki:ledger:"), "{names}");
        assert!(names.contains("urn:iki:store:"), "{names}");
    }
}
