//! **The book's own kernel, in the page.**
//!
//! A chapter's runnable cell — `<div class="ikigai-run" data-cmd="…">` in the markdown,
//! rendered by `books/ikigai/js/run.js` — sends its line here, and the answer comes from
//! [`page_kernel()`]: Part I's space (`hello_camel::space()`) and the applied chapter's
//! game (`tic_tac_toe::space()`) under one Meta renderer — the same spaces, bindings and
//! renderer the chapters' tests run against. Nothing is mocked and nothing is a recording.
//! `camel-case`, `title`, `camel-title`, everything `ikigai-fn` binds and the game's cells
//! resolve in the reader's browser, under the CLI's own engine, so `source a | b`,
//! `cache`, `trace` and `describe … text/turtle` all mean exactly what they mean at a
//! shell.
//!
//! One kernel per page, in a thread-local, shared by every cell on it. That is deliberate:
//! the cache and the golden threads live in the kernel, and "cached the second time" is only
//! demonstrable against state that persists between two runs.
//!
//! Built for `wasm32-unknown-unknown` by `pages.yml` and bound with `wasm-bindgen --target
//! web`; the output lands in the built book under `wasm/` and is never committed. The crate
//! also builds natively (an `rlib`), which is how the workspace gates lint and test it.

use std::rc::Rc;
use std::sync::Arc;

use ikigai_core::{Fallback, Kernel, Space};
use wasm_bindgen::prelude::*;

thread_local! {
    // `Rc` so an async eval can own a handle across `.await` points — a thread-local
    // borrow cannot span an await.
    static ENGINE: Rc<ikigai_engine::Engine> = Rc::new(ikigai_engine::Engine::new(page_kernel()));
}

/// Every name a cell may resolve: Part I's space first, then the applied chapter's game.
///
/// One space for the whole book rather than one per chapter, because the two families
/// are disjoint (`urn:iki:tutorial:{camel-case,title,camel-title}` and `urn:iki:fn:*`
/// against `urn:iki:tutorial:ttt:*`), so the order decides nothing and every cell Part I
/// ships answers exactly as it did. What a cell *changes* is still per page: each page
/// load instantiates the wasm afresh, so a game played on one chapter's page is not on
/// the next one's. The URN gate asks this same space which names a cell may use.
pub fn page_space() -> Fallback {
    Fallback::new(vec![
        Arc::new(hello_camel::space()) as Arc<dyn Space>,
        Arc::new(tic_tac_toe::space()),
    ])
}

/// The page's kernel: [`page_space`] with the Meta renderer `hello_camel::kernel()` uses,
/// so `describe` and `urn:kernel:catalog` answer in the page.
pub fn page_kernel() -> Kernel {
    Kernel::with_meta_renderer(
        Arc::new(page_space()),
        Arc::new(ikigai_vocab::TurtleRenderer),
    )
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
    page_space()
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
        let engine = ikigai_engine::Engine::new(page_kernel());
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

    /// One evaluated line as `js/run.js` renders it: the text, then the verdict in
    /// brackets on its own line; an error as `error: …`.
    fn render(json: &str) -> String {
        let reply: serde_json::Value = serde_json::from_str(json).unwrap();
        let (kind, text, cache) = (
            reply["kind"].as_str().unwrap(),
            reply["text"].as_str().unwrap(),
            reply["cache"].as_str().unwrap(),
        );
        let mut out = String::new();
        if kind == "error" {
            out.push_str(&format!("error: {text}\n"));
        } else if !text.is_empty() {
            out.push_str(text);
            if !text.ends_with('\n') {
                out.push('\n');
            }
        }
        if !cache.is_empty() {
            out.push_str(&format!("[{cache}]\n"));
        }
        out
    }

    /// The runnable cells of a chapter, in page order: each one's command and the text of
    /// its expected-output `<pre>`, with the HTML escapes the markup needs undone.
    fn cells(chapter: &str) -> Vec<(String, String)> {
        let unescape = |s: &str| {
            s.replace("&#32;", "")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&amp;", "&")
        };
        let mut found = Vec::new();
        let mut rest = chapter;
        while let Some(at) = rest.find("data-cmd='") {
            let after = &rest[at + "data-cmd='".len()..];
            let end = after.find('\'').expect("a closed data-cmd");
            let command = after[..end].to_string();
            let open = "<pre class=\"ikigai-run-expected\">";
            let pre = after.find(open).expect("an expected <pre> after the cell") + open.len();
            let close = after[pre..].find("</pre>").expect("a closed <pre>") + pre;
            found.push((command, unescape(&after[pre..close])));
            rest = &after[close..];
        }
        found
    }

    /// Every cell of the applied chapter, run natively in PAGE ORDER against one kernel
    /// — which is what the page does — and compared with the output the chapter shows.
    /// The chapter's expected output is therefore not a claim about the kernel but a
    /// reading of it: an edit to either that the other does not follow fails here.
    #[test]
    fn the_tic_tac_toe_cells_answer_as_the_chapter_says_in_page_order() {
        let chapter = std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../books/ikigai/src/applied/tic-tac-toe-1.md"
        ))
        .expect("the chapter is readable");
        let cells = cells(&chapter);
        assert_eq!(cells.len(), 5, "the chapter's cells: {cells:#?}");

        let engine = ikigai_engine::Engine::new(page_kernel());
        let mut transcript = Vec::new();
        for (command, expected) in &cells {
            let got: String = command
                .lines()
                .map(|line| render(&action_to_json(engine.eval(line))))
                .collect();
            let tidy = |s: &str| -> Vec<String> {
                s.trim_end()
                    .lines()
                    .map(|l| l.trim_end().to_string())
                    .collect()
            };
            assert_eq!(
                tidy(&got),
                tidy(expected),
                "the cell `{command}` answers differently from the chapter.\n--- got ---\n{got}"
            );
            transcript.push(got);
        }

        // And the lesson itself, stated rather than only matched: empty computed then
        // served; the store's own miss is NotFound, not Unresolved; the Sink to the
        // STORED cell un-caches the PLATONIC one; the Delete brings the empty cell back.
        assert_eq!(transcript[0], "-\n[computed]\n-\n[cached]\n");
        assert!(
            transcript[1].starts_with("error: not found:"),
            "{}",
            transcript[1]
        );
        assert_eq!(
            transcript[2],
            "cached\nok\n[uncacheable]\nnot cached\nX\n[computed]\n"
        );
        assert_eq!(
            transcript[3],
            "ok\n[uncacheable]\n-\n[computed]\n-\n[cached]\n"
        );
        assert!(
            transcript[4].contains("<urn:ikigai:endpoint:ttt-cell> a ik:Endpoint"),
            "{}",
            transcript[4]
        );
    }

    #[test]
    fn part_one_and_the_game_share_the_page_without_overlapping() {
        // Each family resolves to its own endpoint: the Fallback's order decides nothing.
        let names = bound_names();
        for name in [
            "urn:iki:tutorial:ttt:stored:{x}:{y}",
            "urn:iki:tutorial:ttt:cell:{x}:{y}",
        ] {
            assert!(
                names.lines().any(|n| n == name),
                "{name} missing from:\n{names}"
            );
        }
        use ikigai_core::{Iri, Request, Resolution, Scope, Verb};
        let game = tic_tac_toe::space();
        let part_one = hello_camel::space();
        for name in ["urn:iki:tutorial:title", "urn:iki:tutorial:camel-case"] {
            let request = Request::new(Verb::Source, Iri::parse(name).unwrap());
            assert!(matches!(
                game.resolve(&request, &Scope::empty()),
                Resolution::Miss
            ));
        }
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:ttt:cell:0:0").unwrap(),
        );
        assert!(matches!(
            part_one.resolve(&request, &Scope::empty()),
            Resolution::Miss
        ));
    }

    #[test]
    fn an_unknown_command_is_an_error_not_a_panic() {
        let engine = ikigai_engine::Engine::new(page_kernel());
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
