//! **The ikigai Architecture's own kernel, in the page.**
//!
//! A chapter's runnable cell (`<div class="ikigai-run" data-cmd="…">`) is turned into a Run
//! button by `js/run.js`, the script every book loads (this book's is a symlink to The ikigai
//! Book's). It imports `wasm/book_wasm.js` beside the book's root, and `scripts/build-kernels.sh`
//! binds THIS crate under that name for this book.
//!
//! Two kinds of cell, two kernels:
//!
//! - **The page's kernel** ([`page_kernel`]): [`ledger_host::kernel`], the work ledger over an
//!   in-memory store, the worked resource this book reads two ways ("The capability spine",
//!   "Golden threads and the cost of a read"). One per page, shared by every plain cell.
//! - **A resilience harness** (`data-harness='overlays'`, chain `harness:overlays`):
//!   [`resilience_harness::Harness`], a kernel of its own, started on first use and kept for
//!   the life of the page. Its cells, and the controls `js/stages.js` builds beside its
//!   stages, run there, and so do the lines that viewer reads after each step (the picture,
//!   `urn:diagram:state scheme=…`, and the board's words): nothing here knows about stages.
//!
//! Why a kernel of this book's own rather than The gonk Book's, which this book used to share:
//! the harness links `ikigai-throttle` and `ikigai-diagram`, which no page of The gonk Book
//! uses, so sharing would grow every gonk page's download for one chapter of this book.
//!
//! Exports, by the names the scripts call: `evalLineAsync`, `evalLineInChainAsync` and
//! `boundNames` (`js/run.js`, and `js/stages.js`, which runs its lines the same way);
//! `pages.yml` asserts all three.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use ikigai_core::{Kernel, Space};
use ikigai_engine::Engine;
use resilience_harness::Harness;
use wasm_bindgen::prelude::*;

/// A harness and the engine its lines run in.
pub struct HarnessHost {
    harness: Harness,
    engine: Rc<Engine>,
}

impl HarnessHost {
    /// A harness as it starts, with its engine.
    pub fn start() -> HarnessHost {
        let harness = Harness::new();
        let engine = Rc::new(Engine::new(std::sync::Arc::clone(harness.kernel())));
        HarnessHost { harness, engine }
    }

    /// The harness.
    pub fn harness(&self) -> &Harness {
        &self.harness
    }

    /// The engine its lines run in.
    pub fn engine(&self) -> Rc<Engine> {
        Rc::clone(&self.engine)
    }
}

/// The page: its kernel's engine, and the harnesses its cells have named.
pub struct Page {
    engine: Rc<Engine>,
    harnesses: RefCell<BTreeMap<String, Rc<HarnessHost>>>,
}

impl Default for Page {
    fn default() -> Self {
        Page {
            engine: Rc::new(page_engine()),
            harnesses: RefCell::default(),
        }
    }
}

impl Page {
    /// The harness `name` names (`harness:{name}` in a chain spec), started on first use and
    /// kept for the life of the page. A name is lowercase ASCII letters, digits and `-`.
    pub fn harness(&self, name: &str) -> Result<Rc<HarnessHost>, String> {
        if name.is_empty()
            || !name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(format!(
                "`{name}` is not a harness's name: lowercase letters, digits and -"
            ));
        }
        let mut harnesses = self.harnesses.borrow_mut();
        Ok(Rc::clone(
            harnesses
                .entry(name.to_string())
                .or_insert_with(|| Rc::new(HarnessHost::start())),
        ))
    }

    /// The engine a line in `chain` runs in: the page's for none, a harness's for
    /// `harness:{name}`. Any other chain is refused in words: this book's pages name no other.
    pub fn engine(&self, chain: Option<&str>) -> Result<Rc<Engine>, String> {
        match chain {
            None => Ok(Rc::clone(&self.engine)),
            Some(spec) => match spec.strip_prefix("harness:") {
                Some(name) => Ok(self.harness(name)?.engine()),
                None => Err(format!(
                    "this book's kernel has no chain `{spec}`: a cell runs on the page's \
                     kernel, or in a harness (data-harness)"
                )),
            },
        }
    }
}

thread_local! {
    // `Rc` so an async eval can own a handle across `.await` points: a thread-local borrow
    // cannot span an await.
    static PAGE: Rc<Page> = Rc::new(Page::default());
}

/// The page's kernel: the ledger and the store it is written in, [`ledger_host::kernel`].
pub fn page_kernel() -> Kernel {
    ledger_host::kernel()
}

/// The engine a plain cell's line is evaluated by, over [`page_kernel`].
pub fn page_engine() -> Engine {
    Engine::new(page_kernel())
}

/// One evaluated line as JSON for the page: `{ "kind", "text", "cache" }`, the shape
/// `js/run.js` renders (and the other books' kernels produce).
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

/// Evaluate `line` in `chain` on `page`: what [`eval_line_in_chain_async`] resolves to, and
/// what the native tests replay a chapter with.
pub async fn eval_on(page: &Page, chain: Option<&str>, line: &str) -> String {
    match page.engine(chain) {
        Ok(engine) => action_to_json(engine.eval_async(line).await),
        Err(refusal) => {
            serde_json::json!({ "kind": "error", "text": refusal, "cache": "" }).to_string()
        }
    }
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
    let page = PAGE.with(Rc::clone);
    wasm_bindgen_futures::future_to_promise(async move {
        Ok(JsValue::from_str(&eval_on(&page, None, &line).await))
    })
}

/// [`eval_line_async`] in chain `chain`: what a cell carrying `data-harness='…'` sends
/// (`harness:{name}`), and what `js/harness.js` sends for each of its controls.
#[wasm_bindgen(js_name = evalLineInChainAsync)]
pub fn eval_line_in_chain_async(chain: String, line: String) -> js_sys::Promise {
    let page = PAGE.with(Rc::clone);
    wasm_bindgen_futures::future_to_promise(async move {
        Ok(JsValue::from_str(
            &eval_on(&page, Some(&chain), &line).await,
        ))
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
