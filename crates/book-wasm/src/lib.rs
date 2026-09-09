//! **The book's own kernel, in the page.**
//!
//! A chapter's runnable cell — `<div class="ikigai-run" data-cmd="…">` in the markdown,
//! rendered by `books/ikigai/js/run.js` — sends its line here, and the answer comes from
//! `hello_camel::kernel()`: the same space, the same bindings, the same Meta renderer the
//! chapter's tests run against. Nothing is mocked and nothing is a recording. `camel-case`,
//! `title`, `camel-title` and everything `ikigai-fn` binds resolve in the reader's browser,
//! under the CLI's own engine, so `source a | b`, `cache`, `trace` and
//! `describe … text/turtle` all mean exactly what they mean at a shell.
//!
//! One kernel per page, in a thread-local, shared by every cell on it. That is deliberate:
//! the cache and the golden threads live in the kernel, and "cached the second time" is only
//! demonstrable against state that persists between two runs.
//!
//! Built for `wasm32-unknown-unknown` by `pages.yml` and bound with `wasm-bindgen --target
//! web`; the output lands in the built book under `wasm/` and is never committed. The crate
//! also builds natively (an `rlib`), which is how the workspace gates lint and test it.

use std::rc::Rc;

use wasm_bindgen::prelude::*;

thread_local! {
    // `Rc` so an async eval can own a handle across `.await` points — a thread-local
    // borrow cannot span an await.
    static ENGINE: Rc<ikigai_engine::Engine> = Rc::new(ikigai_engine::Engine::new(hello_camel::kernel()));
}

/// One evaluated line as JSON for the page: `{ "kind", "text", "cache" }`.
///
/// `kind` is `output` | `error` | `help` | `clear` | `quit` | `noop`; `cache` is the engine's
/// verdict on the resolutions the line performed — `computed`, `cached`, `uncacheable`, or
/// a tally like `1 cached · 1 computed` for a pipeline — and empty when nothing resolved.
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

/// Called by wasm-bindgen when the module is instantiated: route a Rust panic to the
/// browser console with a message rather than an opaque `unreachable`.
#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
}

/// Evaluate one REPL line against the page's kernel; resolves to the JSON of
/// [`action_to_json`]. Async so a resolution runs on the JS event loop instead of
/// blocking the page — every line the book's cells send is in-memory and returns at
/// once, but the shape is the one that stays correct if a cell ever awaits anything.
#[wasm_bindgen(js_name = evalLineAsync)]
pub fn eval_line_async(line: String) -> js_sys::Promise {
    let engine = ENGINE.with(Rc::clone);
    wasm_bindgen_futures::future_to_promise(async move {
        Ok(JsValue::from_str(&action_to_json(
            engine.eval_async(&line).await,
        )))
    })
}

/// The names the in-page kernel binds, one per line — what a cell may name. The URN gate
/// asks the same space natively; this is the same answer, from inside the page.
#[wasm_bindgen(js_name = boundNames)]
pub fn bound_names() -> String {
    use ikigai_core::Space;
    hello_camel::space()
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

    /// The four lines the payoff chapter's cells send, run natively through the same
    /// engine and kernel the wasm wraps. A cell's static fallback output is what these
    /// return, so the test is what keeps the fallback honest.
    #[test]
    fn the_payoff_cells_answer_as_the_chapter_says() {
        let engine = ikigai_engine::Engine::new(hello_camel::kernel());
        let run = |line: &str| -> (String, String) {
            let json: serde_json::Value =
                serde_json::from_str(&action_to_json(engine.eval(line))).unwrap();
            (
                json["text"].as_str().unwrap().to_string(),
                json["cache"].as_str().unwrap().to_string(),
            )
        };

        // Cached once.
        assert_eq!(
            run("source urn:iki:tutorial:title"),
            ("resource oriented computing".into(), "computed".into())
        );
        assert_eq!(
            run("source urn:iki:tutorial:title"),
            ("resource oriented computing".into(), "cached".into())
        );

        // A golden thread, cut.
        assert_eq!(
            run("source urn:iki:tutorial:camel-title"),
            ("resourceOrientedComputing".into(), "computed".into())
        );
        assert_eq!(run("cache urn:iki:tutorial:camel-title").0, "cached");
        assert_eq!(
            run("sink urn:iki:tutorial:title golden threads cut").0,
            "ok"
        );
        assert_eq!(run("cache urn:iki:tutorial:camel-title").0, "not cached");
        assert_eq!(
            run("source urn:iki:tutorial:camel-title"),
            ("goldenThreadsCut".into(), "computed".into())
        );

        // Traced. The composite is cached from the lines above, and a trace of a cache
        // hit is one node with no children — true, and not the tree the chapter wants to
        // show — so the cell cuts the thread by hand first, exactly as a reader would.
        assert_eq!(
            run("sink urn:kernel:cut urn:iki:tutorial:title").0,
            "cut urn:iki:tutorial:title\n"
        );
        let (trace, _) = run("trace urn:iki:tutorial:camel-title");
        println!("{trace}");
        let nodes: Vec<&str> = trace
            .lines()
            .filter(|line| {
                line.starts_with("urn:")
                    || line.starts_with("  urn:")
                    || line.starts_with("└")
                    || line.contains("urn:iki:tutorial:title ")
            })
            .collect();
        assert!(trace.contains("urn:iki:tutorial:camel-title"), "{trace}");
        assert!(
            trace.contains("urn:iki:tutorial:title") && nodes.len() >= 2,
            "the sub-resolution should be a node of its own:\n{trace}"
        );
        assert!(
            trace.contains("computed"),
            "a fresh resolution, not a hit:\n{trace}"
        );

        // Described.
        let (turtle, _) = run("describe urn:iki:tutorial:camel-case text/turtle");
        assert!(
            turtle.contains("<urn:ikigai:endpoint:camel-case> a ik:Endpoint"),
            "{turtle}"
        );
    }

    #[test]
    fn an_unknown_command_is_an_error_not_a_panic() {
        let engine = ikigai_engine::Engine::new(hello_camel::kernel());
        let json: serde_json::Value =
            serde_json::from_str(&action_to_json(engine.eval("frobnicate"))).unwrap();
        assert_eq!(json["kind"], "error");
    }

    #[test]
    fn bound_names_include_the_chapters_three() {
        let names = bound_names();
        for name in [
            "urn:iki:tutorial:camel-case",
            "urn:iki:tutorial:title",
            "urn:iki:tutorial:camel-title",
        ] {
            assert!(
                names.lines().any(|n| n == name),
                "{name} missing from:\n{names}"
            );
        }
    }
}
