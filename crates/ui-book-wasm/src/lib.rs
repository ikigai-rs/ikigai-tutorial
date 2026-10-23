//! **Building ikigai UIs' own kernel, in the page.**
//!
//! A chapter's runnable cell (`<div class="ikigai-run" data-cmd="…">`) is turned into a Run button
//! by `js/run.js`, the script The ikigai Book and The gonk Book use too (this book's `js/run.js`
//! is a symlink to it). It imports `wasm/book_wasm.js` beside the book's root, and for this book
//! `scripts/build-kernels.sh` binds THIS crate under that name.
//!
//! The kernel is [`inspector::kernel`], the kernel inspector the book builds: its views, its
//! artifacts compiled in, XSLT and RDF transreption, ikigai-ui's shell. So a cell that sources
//! `urn:iki:tutorial:inspect:fragment:cache` runs the same XSLT over the same readout the
//! inspector's HTTP host serves, in the reader's browser. One kernel per page, shared by every
//! cell on it: "write the note, then read the cache" is only worth showing against state that
//! persists between two Runs. Each page load starts again.
//!
//! The engine issues every line under ROOT, as the other books' page kernels do: the reader is
//! the only one asking, and it is their kernel. The inspector's HTTP host is where a viewer's
//! capability narrows what a view shows.
//!
//! Exports, by the names `js/run.js` calls: `evalLineAsync` and `boundNames`.

use std::rc::Rc;

use ikigai_core::{Kernel, Space};
use ikigai_engine::Engine;
use wasm_bindgen::prelude::*;

thread_local! {
    static PAGE: Rc<Engine> = Rc::new(page_engine());
}

/// The page's kernel: the inspector, compiled in.
pub fn page_kernel() -> Kernel {
    inspector::kernel(Vec::new())
}

/// The engine a cell's line is evaluated by, over [`page_kernel`].
pub fn page_engine() -> Engine {
    Engine::new(page_kernel())
}

/// One evaluated line as JSON for the page: `{ "kind", "text", "cache" }`, the shape `js/run.js`
/// renders.
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

/// A Rust panic reaches the browser console as a message rather than an opaque `unreachable`.
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

/// The names the inspector binds, one per line.
#[wasm_bindgen(js_name = boundNames)]
pub fn bound_names() -> String {
    inspector::space()
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
            // An empty output line is written `&#32;`, since a blank line would end the cell.
            .replace("&#32;", " ")
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

    /// What a cell prints, as `js/run.js` renders a reply.
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

    /// Every cell of a page, run natively in PAGE ORDER against one fresh page kernel, compared
    /// with what the page says it prints.
    fn run_page_in_order(file: &str, expected_cells: usize) {
        let path = format!("{}/../../books/ui/src/{file}", env!("CARGO_MANIFEST_DIR"));
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
    fn a_page_as_a_composed_resource_answers_as_the_page_says() {
        run_page_in_order("model/composed.md", CELLS_COMPOSED);
    }

    #[test]
    fn graph_to_page_answers_as_the_page_says() {
        run_page_in_order("model/xslt.md", CELLS_XSLT);
    }

    #[test]
    fn live_data_answers_as_the_page_says() {
        run_page_in_order("model/live.md", CELLS_LIVE);
    }

    #[test]
    fn views_from_json_answer_as_the_page_says() {
        run_page_in_order("model/views-from-json.md", CELLS_JSON);
    }

    #[test]
    fn views_from_csv_answer_as_the_page_says() {
        run_page_in_order("model/views-from-csv.md", CELLS_CSV);
    }

    const CELLS_JSON: usize = 9;
    const CELLS_CSV: usize = 9;
    const CELLS_COMPOSED: usize = 7;
    const CELLS_XSLT: usize = 8;
    const CELLS_LIVE: usize = 5;

    #[test]
    fn the_page_kernel_binds_the_inspector() {
        let names = bound_names();
        assert!(
            names.contains("urn:iki:tutorial:inspect:fragment:{view}"),
            "{names}"
        );
        assert!(
            names.contains("urn:iki:tutorial:inspect:demo:note"),
            "{names}"
        );
    }
}
