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
//! A cell may also name a **game** (`data-game='a'`, Part IV of the applied chapter). It then
//! runs in that game's resolution chain — [`tic_tac_toe::game`], one corridor binding the
//! stored cell to that game's own store — on the SAME kernel, through an engine whose
//! resolver ([`InGame`]) issues every request in the chain. Choosing the game is the host's
//! act, not the line's: the REPL grammar has no way to name a chain, and injecting one is
//! authority (whoever may push a corridor chooses what the names mean), so the page's host
//! code does it from markup the chapter wrote.
//!
//! Built for `wasm32-unknown-unknown` by `pages.yml` and bound with `wasm-bindgen --target
//! web`; the output lands in the built book under `wasm/` and is never committed. The crate
//! also builds natively (an `rlib`), which is how the workspace gates lint and test it.
//!
//! **A clock is the page's to give, and only one page asks for it.** The kernel has no
//! clock by default, so a trace prints `—` for a duration and `urn:iki:tutorial:time:now`
//! refuses. A page whose markup carries `data-clock` (the chapter *Time is a resource*) has
//! `js/run.js` call [`use_clock`] before anything resolves, and that page's kernel is built
//! with the browser's clock. It is per page rather than for the whole book because a clock
//! changes what every other page shows: every trace would print a duration in place of
//! the `—` its chapter explains.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use ikigai_core::{
    Capability, Clock, Error, Expiry, Fallback, Iri, Kernel, Provenance, Representation, Request,
    Scope, Space, SpaceEntry, Tracer, Verb,
};
use ikigai_engine::Engine;
use ikigai_resolve::{CacheStatus, Resolver};
use wasm_bindgen::prelude::*;

thread_local! {
    // The clock the page's kernel is built with, if the page asked for one before the
    // kernel was built (`useClock`), and whether it has been built yet.
    static CLOCK: RefCell<Option<Arc<dyn Clock>>> = const { RefCell::new(None) };
    static BUILT: Cell<bool> = const { Cell::new(false) };
    // `Rc` so an async eval can own a handle across `.await` points — a thread-local
    // borrow cannot span an await.
    static PAGE: Rc<Page> = {
        BUILT.with(|built| built.set(true));
        Rc::new(Page::with_clock(CLOCK.with(|clock| clock.borrow().clone())))
    };
}

/// Every name a cell may resolve: Part I's space first, then the applied chapter's game,
/// then the time chapter's clock.
///
/// One space for the whole book rather than one per chapter, because the families are
/// disjoint (`urn:iki:tutorial:{camel-case,title,camel-title}` and `urn:iki:fn:*` against
/// `urn:iki:tutorial:ttt:*` against `urn:iki:tutorial:time:*`), so the order decides
/// nothing and every cell Part I ships answers exactly as it did. What a cell *changes* is
/// still per page: each page load instantiates the wasm afresh, so a game played on one
/// chapter's page is not on the next one's. The URN gate asks this same space which names
/// a cell may use.
pub fn page_space() -> Fallback {
    Fallback::new(vec![
        Arc::new(hello_camel::space()) as Arc<dyn Space>,
        Arc::new(tic_tac_toe::space()),
        Arc::new(time_resource::space()),
    ])
}

/// The page's kernel: [`page_space`] with the Meta renderer `hello_camel::kernel()` uses,
/// so `describe` and `urn:kernel:catalog` answer in the page. No clock.
pub fn page_kernel() -> Kernel {
    page_kernel_with_clock(None)
}

// ANCHOR: page_kernel_with_clock
/// [`page_kernel`], with `clock` if there is one: the browser's on a page that asked for
/// it, a clock a test holds in a test.
pub fn page_kernel_with_clock(clock: Option<Arc<dyn Clock>>) -> Kernel {
    let kernel = Kernel::with_meta_renderer(
        Arc::new(page_space()),
        Arc::new(ikigai_vocab::TurtleRenderer),
    );
    match clock {
        Some(clock) => kernel.with_clock(clock),
        None => kernel,
    }
}
// ANCHOR_END: page_kernel_with_clock

// ANCHOR: browser_clock
/// The browser's clock: `Date.now()`, milliseconds since 1970. `std::time` has no clock on
/// `wasm32-unknown-unknown`, and ikigai-core reads none itself, so the page supplies one,
/// the way `ikigai-web-demo` does.
#[cfg(target_family = "wasm")]
struct BrowserClock;

#[cfg(target_family = "wasm")]
impl Clock for BrowserClock {
    fn now(&self) -> ikigai_core::Time {
        ikigai_core::Time::from_millis(js_sys::Date::now() as u64)
    }
}
// ANCHOR_END: browser_clock

/// The clock [`use_clock`] gives the page: the browser's in the page, the system's when this
/// crate is built natively (where a `js_sys` call would panic rather than answer).
fn host_clock() -> Arc<dyn Clock> {
    #[cfg(target_family = "wasm")]
    return Arc::new(BrowserClock);
    #[cfg(not(target_family = "wasm"))]
    return Arc::new(ikigai_core::SystemClock);
}

/// Build this page's kernel with the browser's clock. `js/run.js` calls it right after the
/// module loads, on a page that carries `data-clock`, and before anything resolves.
///
/// Refused once the kernel has been built: a kernel's clock is fixed when it is made, and
/// a page that asked too late would otherwise get a kernel with no clock and no word why.
#[wasm_bindgen(js_name = useClock)]
pub fn use_clock() -> Result<(), JsValue> {
    if BUILT.with(Cell::get) {
        return Err(JsValue::from_str(
            "the page's kernel is already built, without a clock",
        ));
    }
    CLOCK.with(|clock| *clock.borrow_mut() = Some(host_clock()));
    Ok(())
}

/// The page: one kernel, the engine a cell with no game runs in, and one engine per game a
/// cell has named — every engine over the SAME kernel, so one cache and one set of golden
/// threads serve them all, partitioned by the chain each request carries.
pub struct Page {
    kernel: Arc<Kernel>,
    root: Rc<Engine>,
    games: RefCell<BTreeMap<String, Game>>,
}

/// One game on the page: the engine its cells run in, and the chain it is played in — the
/// same chain, cloned, so a cell and a click in one game are one game to the cache.
struct Game {
    engine: Rc<Engine>,
    scope: Scope,
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

impl Page {
    /// A fresh page: [`page_kernel`], and no game played yet.
    pub fn new() -> Self {
        Self::with_clock(None)
    }

    /// A fresh page whose kernel has `clock`, if there is one.
    pub fn with_clock(clock: Option<Arc<dyn Clock>>) -> Self {
        let kernel = Arc::new(page_kernel_with_clock(clock));
        Self {
            root: Rc::new(Engine::new(Arc::clone(&kernel))),
            kernel,
            games: RefCell::default(),
        }
    }

    /// The engine for a cell: the root's when it names no game, else game `id`'s — created
    /// on first use with an empty in-memory store, and kept for the life of the page, since
    /// a game's corridor is built ONCE (its name is the cache's claim that it is one game).
    pub fn engine(&self, game: Option<&str>) -> Result<Rc<Engine>, String> {
        match game {
            None => Ok(Rc::clone(&self.root)),
            Some(id) => self.game(id, |game| Rc::clone(&game.engine)),
        }
    }

    /// Game `id`, made on first use, handed to `with`.
    fn game<T>(&self, id: &str, with: impl FnOnce(&Game) -> T) -> Result<T, String> {
        if let Some(game) = self.games.borrow().get(id) {
            return Ok(with(game));
        }
        let store = Arc::new(tic_tac_toe::stored_space(Arc::default()));
        let scope = tic_tac_toe::game(id, store).map_err(|e| e.to_string())?;
        let game = Game {
            engine: Rc::new(Engine::new(InGame::new(
                Arc::clone(&self.kernel),
                scope.clone(),
            ))),
            scope,
        };
        let answer = with(&game);
        self.games.borrow_mut().insert(id.to_string(), game);
        Ok(answer)
    }

    // ANCHOR: page_issue
    /// One request, straight to the kernel — not a line of the REPL — in game `game`'s
    /// chain, or the root's when it names none. What the page's htmx shim sends for a click:
    /// `GET` a view is a `Source`, `POST` a play is a `Sink`, in the game the board's markup
    /// names. Resolves to the representation, or the kernel's refusal as its message.
    pub fn issue(
        &self,
        game: Option<&str>,
        verb: Verb,
        target: &str,
    ) -> impl std::future::Future<Output = Result<Representation, String>> + 'static {
        let kernel = Arc::clone(&self.kernel);
        let scope = match game {
            None => Ok(Scope::empty()),
            Some(id) => self.game(id, |game| game.scope.clone()),
        };
        let target = Iri::parse(target).map_err(|e| format!("not a resource name: {e}"));
        async move {
            let request = Request::new(verb, target?);
            kernel
                .issue_in(request, &Capability::root(), scope?)
                .await
                .map_err(|e| e.to_string())
        }
    }
    // ANCHOR_END: page_issue

    /// Would [`issue`](Self::issue) be answered from the cache right now? A probe, which
    /// resolves nothing and runs nothing: what a page asks before a request so it can say
    /// afterwards whether the kernel did any work. A name that does not parse is not cached.
    pub fn is_cached(&self, game: Option<&str>, verb: Verb, target: &str) -> bool {
        let scope = match game {
            None => Scope::empty(),
            Some(id) => match self.game(id, |game| game.scope.clone()) {
                Ok(scope) => scope,
                Err(_) => return false,
            },
        };
        Iri::parse(target).is_ok_and(|target| {
            self.kernel
                .is_cached_in(&Request::new(verb, target), &Capability::root(), &scope)
        })
    }
}

// ANCHOR: in_game
/// A [`Resolver`] that issues every request in ONE resolution chain — a game's — on a shared
/// kernel. The engine takes any resolver, so an engine over `InGame` is the whole REPL
/// grammar (`source`, `sink`, `cache`, `trace`, pipes, maps) played in one game: every
/// stage, every contract fetch and every cache probe carries the chain, and the kernel hands
/// it on to every sub-request.
pub struct InGame {
    kernel: Arc<Kernel>,
    scope: Scope,
}

impl InGame {
    /// Resolve in `scope` on `kernel`.
    pub fn new(kernel: Arc<Kernel>, scope: Scope) -> Self {
        Self { kernel, scope }
    }

    /// How the cache served a resolution, from a probe taken before it.
    fn status(was_cached: bool, representation: &Representation) -> CacheStatus {
        match (representation.expiry == Expiry::Always, was_cached) {
            (true, _) => CacheStatus::Uncacheable,
            (false, true) => CacheStatus::Hit,
            (false, false) => CacheStatus::Miss,
        }
    }
}

#[async_trait::async_trait]
impl Resolver for InGame {
    fn issue(&self, request: Request) -> Result<(Representation, CacheStatus), Error> {
        self.issue_as(request, &Capability::root())
    }

    fn issue_as(
        &self,
        request: Request,
        capability: &Capability,
    ) -> Result<(Representation, CacheStatus), Error> {
        futures::executor::block_on(self.issue_as_async(request, capability))
    }

    async fn issue_as_async(
        &self,
        request: Request,
        capability: &Capability,
    ) -> Result<(Representation, CacheStatus), Error> {
        let was_cached = self.kernel.is_cached_in(&request, capability, &self.scope);
        let representation = self
            .kernel
            .issue_in(request, capability, self.scope.clone())
            .await?;
        let status = Self::status(was_cached, &representation);
        Ok((representation, status))
    }

    async fn issue_as_async_with_incoming(
        &self,
        request: Request,
        capability: &Capability,
        incoming: Provenance,
    ) -> Result<(Representation, CacheStatus), Error> {
        let was_cached = self.kernel.is_cached_in(&request, capability, &self.scope);
        let representation = self
            .kernel
            .issue_with_incoming_in(request, capability, incoming, self.scope.clone())
            .await?;
        let status = Self::status(was_cached, &representation);
        Ok((representation, status))
    }

    fn is_cached(&self, request: &Request, capability: &Capability) -> bool {
        self.kernel.is_cached_in(request, capability, &self.scope)
    }

    fn set_tracer(&self, tracer: Arc<dyn Tracer>) {
        self.kernel.set_tracer(tracer);
    }

    fn clear_tracer(&self) {
        self.kernel.clear_tracer();
    }

    fn entries(&self) -> Option<Vec<SpaceEntry>> {
        self.kernel.entries()
    }

    /// What `trace` prints as the transport — so a traced line says which game it ran in.
    fn transport(&self) -> String {
        format!("embedded · in-process · chain {}", self.scope)
    }
}
// ANCHOR_END: in_game

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
    eval_in(None, line)
}

/// [`eval_line_async`] in game `game` — what a cell carrying `data-game='…'` sends. The
/// game is created on first use, with an empty board, and kept for the life of the page.
#[wasm_bindgen(js_name = evalLineInGameAsync)]
pub fn eval_line_in_game_async(game: String, line: String) -> js_sys::Promise {
    eval_in(Some(game), line)
}

fn eval_in(game: Option<String>, line: String) -> js_sys::Promise {
    let engine = PAGE.with(|page| page.engine(game.as_deref()));
    wasm_bindgen_futures::future_to_promise(async move {
        let json = match engine {
            Ok(engine) => action_to_json(engine.eval_async(&line).await),
            Err(refusal) => {
                serde_json::json!({ "kind": "error", "text": refusal, "cache": "" }).to_string()
            }
        };
        Ok(JsValue::from_str(&json))
    })
}

/// One request to the page's kernel, as an htmx shim (`js/ttt.js`, `js/view.js`) sends it:
/// `verb` is `source`, `sink`, `exists` or `delete`; `game` is the game the board's markup
/// names, or empty for the root's. Resolves to the JSON of [`reply_to_json`], with the
/// cache's verdict on the request, so a page can count what its polling cost.
#[wasm_bindgen(js_name = issueAsync)]
pub fn issue_async(game: String, verb: String, target: String) -> js_sys::Promise {
    let reply = match verb_named(&verb) {
        Ok(verb) => {
            let game = (!game.is_empty()).then_some(game);
            Ok(PAGE.with(|page| {
                let was_cached = page.is_cached(game.as_deref(), verb, &target);
                (was_cached, page.issue(game.as_deref(), verb, &target))
            }))
        }
        Err(refusal) => Err(refusal),
    };
    wasm_bindgen_futures::future_to_promise(async move {
        let json = match reply {
            Ok((was_cached, issued)) => reply_to_json(issued.await, was_cached),
            Err(refusal) => reply_to_json(Err(refusal), false),
        };
        Ok(JsValue::from_str(&json))
    })
}

/// A verb by the name the shim sends — the four an htmx request can map to.
pub fn verb_named(name: &str) -> Result<Verb, String> {
    match name {
        "source" => Ok(Verb::Source),
        "sink" => Ok(Verb::Sink),
        "exists" => Ok(Verb::Exists),
        "delete" => Ok(Verb::Delete),
        other => Err(format!("`{other}` is not a verb the page sends")),
    }
}

/// One issued request as JSON for the shim: `{ "kind": "output", "type", "text", "cache" }`,
/// or `{ "kind": "error", "text" }` with the kernel's refusal. `cache` is `cached`,
/// `computed` or `uncacheable`, the words a cell prints, from a probe taken before the
/// request (`was_cached`) and the answer's expiry.
pub fn reply_to_json(reply: Result<Representation, String>, was_cached: bool) -> String {
    match reply {
        Ok(representation) => serde_json::json!({
            "kind": "output",
            "type": representation.repr_type.to_string(),
            "text": String::from_utf8_lossy(&representation.bytes),
            "cache": match InGame::status(was_cached, &representation) {
                CacheStatus::Uncacheable => "uncacheable",
                CacheStatus::Hit => "cached",
                _ => "computed",
            },
        }),
        Err(refusal) => serde_json::json!({ "kind": "error", "text": refusal }),
    }
    .to_string()
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

    /// The runnable cells of a chapter, in page order: the game each names (`data-game`, if
    /// any), its command, and the text of its expected-output `<pre>`, with the HTML escapes
    /// the markup needs undone.
    fn cells(chapter: &str) -> Vec<(Option<String>, String, String)> {
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
            // `data-game='…'` sits in the same tag, before `data-cmd`.
            let tag = &rest[rest[..at].rfind("<div").expect("a cell is a <div>")..at];
            let game = tag.find("data-game='").map(|g| {
                let value = &tag[g + "data-game='".len()..];
                value[..value.find('\'').expect("a closed data-game")].to_string()
            });
            let after = &rest[at + "data-cmd='".len()..];
            let end = after.find('\'').expect("a closed data-cmd");
            let command = after[..end].to_string();
            let open = "<pre class=\"ikigai-run-expected\">";
            let pre = after.find(open).expect("an expected <pre> after the cell") + open.len();
            let close = after[pre..].find("</pre>").expect("a closed <pre>") + pre;
            found.push((game, command, unescape(&after[pre..close])));
            rest = &after[close..];
        }
        found
    }

    /// The games of the playable boards a chapter places, in page order.
    fn boards(chapter: &str) -> Vec<String> {
        chapter
            .split("<div class=\"ttt-play\" data-game='")
            .skip(1)
            .map(|rest| rest[..rest.find('\'').expect("a closed data-game")].to_string())
            .collect()
    }

    /// Every cell of one applied chapter, run natively in PAGE ORDER against one fresh
    /// kernel — which is what the page does — and compared with the output the chapter
    /// shows. The chapter's expected output is therefore not a claim about the kernel but a
    /// reading of it: an edit to either that the other does not follow fails here.
    ///
    /// Returns what each cell printed, in order, for the caller's own assertions.
    fn run_chapter_in_page_order(file: &str, expected_cells: usize) -> Vec<String> {
        run_chapter_at(file, expected_cells, &[])
    }

    /// The shells the view hosts a chapter places load, in page order
    /// (`<div class="ikigai-view" data-shell='…'>`, `js/view.js`).
    fn view_shells(chapter: &str) -> Vec<String> {
        chapter
            .split("<div class=\"ikigai-view\"")
            .skip(1)
            .map(|rest| {
                let tag = &rest[..rest.find('>').expect("a closed tag")];
                let value = &tag[tag.find("data-shell='").expect("a view names its shell")
                    + "data-shell='".len()..];
                value[..value.find('\'').expect("a closed data-shell")].to_string()
            })
            .collect()
    }

    /// [`run_chapter_in_page_order`] for a chapter whose page has a clock (`data-clock`): the
    /// page's kernel reads a clock this test holds, and before cell `i` runs, the clock is
    /// stopped at the time `times` gives for `i`, if it gives one. Index 0's time is also the
    /// time the page loads at.
    fn run_chapter_at(file: &str, expected_cells: usize, times: &[(usize, u64)]) -> Vec<String> {
        let path = format!(
            "{}/../../books/ikigai/src/applied/{file}",
            env!("CARGO_MANIFEST_DIR")
        );
        let chapter = std::fs::read_to_string(&path).expect("the chapter is readable");
        let cells = cells(&chapter);
        assert_eq!(cells.len(), expected_cells, "{file}'s cells: {cells:#?}");
        let clocked = chapter.contains("data-clock");
        assert!(
            clocked || times.is_empty(),
            "{file} has no data-clock, so its page has no clock to set"
        );
        let clock = Arc::new(time_resource::ManualClock::default());
        let at = |i: usize| {
            if let Some((_, millis)) = times.iter().find(|(cell, _)| *cell == i) {
                clock.set(*millis);
            }
        };
        at(0);

        // A trace names the thread each node ran on. The page has one, which prints as
        // `ThreadId(1)`; a test runs on a thread named after the test (or `main`). Spell
        // this one the way the page does, so the chapter can show what a reader sees.
        let here = std::thread::current()
            .name()
            .map(|name| format!(" · {name} · "));

        // One page: one kernel, and an engine per game a cell names — as in the browser.
        let page = Page::with_clock(clocked.then(|| Arc::clone(&clock) as Arc<dyn Clock>));
        // A view host loads its shell when the kernel loads, before any cell can run; its
        // polling starts paused, so the shell is the only request it makes.
        for shell in view_shells(&chapter) {
            futures::executor::block_on(page.issue(None, Verb::Source, &shell))
                .unwrap_or_else(|e| panic!("the view shell {shell}: {e}"));
        }
        // A playable board (`<div class="ttt-play" data-game='…'>`, Part V) loads when the
        // kernel does, before any cell can run: `js/ttt.js` fetches the page shell, and the
        // shell's markup asks for the status and then the board. And after a cell runs in a
        // board's game, the shim refreshes that board the same way. Both are reads that fill
        // the cache, so the chapter's cells see them — replay them here, in the same order.
        let boards = boards(&chapter);
        let refresh = |game: &str| {
            for view in [
                "urn:iki:tutorial:ttt:view:status",
                "urn:iki:tutorial:ttt:view:board",
            ] {
                futures::executor::block_on(page.issue(Some(game), Verb::Source, view))
                    .unwrap_or_else(|e| panic!("board {game}: {view}: {e}"));
            }
        };
        for game in &boards {
            futures::executor::block_on(page.issue(
                Some(game),
                Verb::Source,
                "urn:iki:tutorial:ttt:template:game",
            ))
            .expect("the page shell");
            refresh(game);
        }
        let mut transcript = Vec::new();
        for (i, (game, command, expected)) in cells.iter().enumerate() {
            at(i);
            let engine = page.engine(game.as_deref()).expect("a valid game id");
            let mut got: String = command
                .lines()
                .map(|line| render(&action_to_json(engine.eval(line))))
                .collect();
            if let Some(here) = &here {
                got = got.replace(here.as_str(), " · ThreadId(1) · ");
            }
            let tidy = |s: &str| -> Vec<String> {
                s.trim_end()
                    .lines()
                    .map(|l| l.trim_end().to_string())
                    .collect()
            };
            // `BOOK_CELLS_PRINT=1 cargo test -p book-wasm --lib -- --nocapture` prints what
            // each cell answers instead of comparing: how a new chapter's outputs are read.
            if std::env::var_os("BOOK_CELLS_PRINT").is_some() {
                println!("=== {game:?}\n{command}\n--- got\n{got}=== end");
            } else {
                assert_eq!(
                    tidy(&got),
                    tidy(expected),
                    "{file}: the cell `{command}` answers differently from the chapter.\n--- got ---\n{got}"
                );
            }
            if let Some(game) = game.as_deref().filter(|g| boards.iter().any(|b| b == g)) {
                refresh(game);
            }
            transcript.push(got);
        }
        transcript
    }

    #[test]
    fn the_tic_tac_toe_cells_answer_as_the_chapter_says_in_page_order() {
        let transcript = run_chapter_in_page_order("tic-tac-toe-1.md", 5);

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

    /// Part II's cells, the same way — and the normalized-recompute lesson stated rather
    /// than only matched.
    #[test]
    fn the_second_tic_tac_toe_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("tic-tac-toe-2.md", 9);

        // Any cells, any length; a second spelling refused.
        assert_eq!(t[0], "---\n[computed]\n--\n[computed]\n");
        assert_eq!(
            t[1].matches("error: invalid argument `list`").count(),
            2,
            "{}",
            t[1]
        );
        // The row and its line are one cache entry: read one, the other is cached.
        assert_eq!(t[2], "---\n[computed]\ncached\n---\n[cached]\n");
        // The move through the corner un-caches row 0, column 0, diagonal 0 and the
        // board; the four lines that miss it stay cached.
        assert!(
            t[5].ends_with("not cached\nnot cached\nnot cached\nnot cached\n"),
            "{}",
            t[5]
        );
        assert_eq!(t[6], "cached\ncached\ncached\ncached\n");
        // The trace: row 0 recomputed down to the stored corner, its other two cells
        // and rows 1 and 2 served.
        let trace = &t[7];
        for (node, verdict) in [
            ("urn:iki:tutorial:ttt:board ", "computed"),
            ("urn:iki:tutorial:ttt:cells:0.0,1.0,2.0 ", "computed"),
            ("urn:iki:tutorial:ttt:cell:0:0 ", "computed"),
            ("urn:iki:tutorial:ttt:stored:0:0 ", "computed"),
            ("urn:iki:tutorial:ttt:cell:1:0 ", "cached"),
            ("urn:iki:tutorial:ttt:cell:2:0 ", "cached"),
            ("urn:iki:tutorial:ttt:cells:0.1,1.1,2.1 ", "cached"),
            ("urn:iki:tutorial:ttt:cells:0.2,1.2,2.2 ", "cached"),
        ] {
            let line = trace
                .lines()
                .find(|line| line.contains(node))
                .unwrap_or_else(|| panic!("{node} is not a node of:\n{trace}"));
            assert!(line.contains(&format!(" · {verdict} · ")), "{line}");
        }
        assert!(
            !trace.contains("urn:iki:tutorial:ttt:cell:0:1"),
            "row 1's cells were never asked:\n{trace}"
        );
        assert!(t[8].ends_with("X--\n-O-\n---\n[cached]\n"), "{}", t[8]);
    }

    /// Part III's cells, the same way — the rules as resources, the refusals at the edge,
    /// and what one move recomputes, stated rather than only matched.
    #[test]
    fn the_third_tic_tac_toe_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("tic-tac-toe-3.md", 13);

        // The CheckSet: names of other resources, computed once, then served.
        assert!(
            t[0].starts_with("urn:iki:tutorial:ttt:column:1\n"),
            "{}",
            t[0]
        );
        assert!(
            t[0].ends_with("urn:iki:tutorial:ttt:row:1\n[cached]\n"),
            "{}",
            t[0]
        );
        // Off the board: the empty list, still an answer.
        assert!(t[1].ends_with("[computed]\n[computed]\n"), "{}", t[1]);
        // A fresh board: nobody has won, X opens.
        assert_eq!(t[2], "-\n[computed]\nX\n[computed]\n");
        // Each refusal once, each an InvalidArgument naming what to fix.
        assert!(
            t[4].starts_with("error: invalid argument `content`:"),
            "{}",
            t[4]
        );
        assert_eq!(
            t[5].matches("error: invalid argument `x, y`:").count(),
            2,
            "{}",
            t[5]
        );
        assert!(t[11].starts_with("error: invalid argument `x, y`: the game is over"));
        // One move through the corner: its three lines, the winner and the turn are cut;
        // a line that misses it and the corner's CheckSet are not.
        assert!(
            t[8].ends_with(
                "not cached\nnot cached\nnot cached\nnot cached\nnot cached\ncached\ncached\n"
            ),
            "{}",
            t[8]
        );
        // The trace: three lines recomputed, five served, and one stored cell read.
        let trace = &t[9];
        let verdict = |node: &str| {
            trace
                .lines()
                .find(|line| line.contains(node))
                .unwrap_or_else(|| panic!("{node} is not a node of:\n{trace}"))
                .to_string()
        };
        for line in [
            "cells:0.0,1.0,2.0 ",
            "cells:0.0,0.1,0.2 ",
            "cells:0.0,1.1,2.2 ",
        ] {
            assert!(verdict(line).contains(" · computed · "), "{line}");
        }
        for line in [
            "cells:0.1,1.1,2.1 ",
            "cells:0.2,1.2,2.2 ",
            "cells:1.0,1.1,1.2 ",
            "cells:2.0,2.1,2.2 ",
            "cells:2.0,1.1,0.2 ",
        ] {
            assert!(verdict(line).contains(" · cached · "), "{line}");
        }
        assert_eq!(trace.matches("ttt-stored").count(), 1, "{trace}");
        // A win detected, and the turn closes.
        assert!(
            t[10].ends_with("X\n[computed]\n-\n[computed]\n"),
            "{}",
            t[10]
        );
        // Following the CheckSet's links reads the winning diagonal; the CheckSet itself
        // is still the entry computed in the first cell.
        assert!(t[12].contains("XXX\n"), "{}", t[12]);
        assert!(t[12].ends_with("cached\n"), "{}", t[12]);
    }

    /// Part IV's cells, the same way — several of them in games (`data-game`), each in its
    /// own chain on the one page kernel — and the lessons stated rather than only matched.
    #[test]
    fn the_fourth_tic_tac_toe_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("tic-tac-toe-4.md", 15);
        let fresh = "---\n---\n---\n[computed]\n";

        // Two games and the root: three boards under the same names. Game b's first move is
        // X's — its turn is read from its own board, not from game a's.
        assert!(t[0].ends_with("---\n-X-\n---\n[computed]\n"), "{}", t[0]);
        assert!(t[1].starts_with(fresh), "{}", t[1]);
        assert!(t[1].contains("X plays 0,0\n"), "{}", t[1]);
        assert_eq!(t[2], fresh);

        // A move stays in its game: a is won and refuses; b, whose corner is the same
        // square a just took, plays on.
        assert!(t[3].ends_with("X\n[computed]\n"), "{}", t[3]);
        assert!(t[4].contains("the game is over — X has won"), "{}", t[4]);
        assert!(t[5].starts_with("O plays 0,2\n"), "{}", t[5]);
        assert!(t[5].contains("-\n[computed]\n"), "{}", t[5]);

        // Two games, two cache partitions: the CheckSet computed in a is not cached in b or
        // in the root, and b computes it again.
        assert!(
            t[6].ends_with(
                "[computed]\nurn:iki:tutorial:ttt:column:2\nurn:iki:tutorial:ttt:row:1\n[cached]\n"
            ),
            "{}",
            t[6]
        );
        assert!(
            t[7].starts_with("not cached\n") && t[7].ends_with("[computed]\n"),
            "{}",
            t[7]
        );
        assert_eq!(t[8], "not cached\n");

        // A golden thread is a name: the ROOT's move on 0,0 un-caches game b's top row, and
        // b's corner is recomputed from b's own store — still b's X.
        // …and it had already happened: game a's moves on the top row un-cached game b's
        // row 0 before this section asked for it.
        assert_eq!(t[9], "X--\n[computed]\ncached\n");
        assert!(t[11].starts_with("not cached\n"), "{}", t[11]);
        let stored = t[11]
            .lines()
            .find(|line| line.contains("urn:iki:tutorial:ttt:stored:0:0 "))
            .unwrap_or_else(|| panic!("the stored corner is a node:\n{}", t[11]));
        assert!(stored.contains(" · computed · "), "{stored}");
        assert!(
            stored.contains("answered-by=urn:iki:tutorial:ttt:game:b"),
            "{stored}"
        );
        assert!(t[11].contains("→ 1b  X"), "{}", t[11]);

        // Three boards, one kernel.
        let boards: Vec<&String> = t[12..].iter().collect();
        assert_eq!(boards.len(), 3);
        assert!(boards[0] != boards[1] && boards[1] != boards[2] && boards[0] != boards[2]);
    }

    /// Part V's cells, the same way — with its two boards loaded first, as the page loads
    /// them, and each refreshed after a cell runs in its game.
    #[test]
    fn the_fifth_tic_tac_toe_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("tic-tac-toe-5.md", 8);

        // A template is served as written, slots and all.
        assert!(t[0].contains("aria-label=\"{{mark}} at {{x}},{{y}}\">{{mark}}</button>"));
        assert!(
            t[0].ends_with("{{mark}} to play.\n[computed]\n"),
            "{}",
            t[0]
        );
        // The boards loaded both views in game a before any cell ran.
        assert_eq!(t[1], "cached\nX to play.\n[cached]\n");
        // A play through the view: the move cut both views, the reply re-read the status.
        assert_eq!(
            t[2],
            "X plays 1,1. O to play.\n[uncacheable]\nnot cached\ncached\n"
        );
        // …and the board refresh after the cell cached the board again, with X in the center.
        assert!(
            t[3].contains("aria-label=\"X at 1,1\">X</button>"),
            "{}",
            t[3]
        );
        assert!(t[3].ends_with("[cached]\n"), "{}", t[3]);
        // A refused play is answered in the kernel's words, then the status.
        assert!(
            t[4].starts_with(
                "invalid argument `x, y`: 1,1 is taken — X played there. O to play.\n"
            ),
            "{}",
            t[4]
        );
        // The trace: the three lines through the corner recomputed, the other five served.
        let trace = &t[5];
        let verdict = |node: &str| {
            trace
                .lines()
                .find(|line| line.contains(node))
                .unwrap_or_else(|| panic!("{node} is not a node of:\n{trace}"))
                .to_string()
        };
        assert!(verdict("view:status ").contains(" · computed · "));
        for line in [
            "cells:0.0,1.0,2.0 ",
            "cells:0.0,0.1,0.2 ",
            "cells:0.0,1.1,2.2 ",
        ] {
            assert!(verdict(line).contains(" · computed · "), "{line}");
        }
        for line in [
            "cells:0.1,1.1,2.1 ",
            "cells:0.2,1.2,2.2 ",
            "cells:1.0,1.1,1.2 ",
            "cells:2.0,2.1,2.2 ",
            "cells:2.0,1.1,0.2 ",
            "template:status-turn ",
        ] {
            assert!(verdict(line).contains(" · cached · "), "{line}");
        }
        // Game b: untouched, but recomputed — a's move on 0,0 cut the name in every game.
        assert_eq!(t[6], "X to play.\n[computed]\n");
        assert_eq!(t[7], "The board is clear. X to play.\n[uncacheable]\n");
    }

    /// Part VI's cells: the questions its transcripts ask a Python- and a TypeScript-held
    /// store, asked of the page's own kernel, where the store is Rust.
    #[test]
    fn the_sixth_tic_tac_toe_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("tic-tac-toe-6.md", 1);
        // The same answers the peers give, and a trace one node shorter: no peer span.
        assert!(t[0].starts_with("-\n[computed]\nX plays 1,1\n[uncacheable]\n"));
        assert_eq!(t[0].matches("urn:iki:tutorial:ttt:stored:1:1 ").count(), 1);
        assert!(
            t[0].ends_with("The board is clear\n[uncacheable]\n"),
            "{}",
            t[0]
        );
    }

    /// "Time is a resource": its cells on a page whose kernel reads a clock the test holds,
    /// stopped where the chapter's outputs say it was, and the lessons stated rather than
    /// only matched.
    #[test]
    fn the_time_chapters_cells_answer_as_it_says_in_page_order() {
        let at = |instant: &str, seconds: u64| time_resource::millis_at(instant, seconds);
        let t = run_chapter_at(
            "time.md",
            7,
            &[
                (0, at("2026-09-29T14:05Z", 20)),
                (1, at("2026-09-29T14:06Z", 5)),
                (3, at("2026-09-29T14:06Z", 10)),
                (4, at("2026-09-29T14:07Z", 2)),
                (5, at("2026-09-29T14:07Z", 30)),
                (6, at("2026-09-29T14:07Z", 40)),
            ],
        );
        // Computed, then served; and computed again once the minute has turned.
        assert_eq!(t[0], "14:05\n[computed]\n14:05\n[cached]\n");
        assert_eq!(t[1], "not cached\n14:06\n[computed]\n");
        // The countdown: computed then served; gone at the minute while the event stays;
        // gone mid-minute when the event moves, while now stays.
        assert!(t[3].ends_with("90\n[computed]\n90\n[cached]\n"), "{}", t[3]);
        assert_eq!(t[4], "not cached\ncached\n89\n[computed]\n");
        assert!(
            t[5].ends_with("not cached\ncached\n30\n[computed]\n"),
            "{}",
            t[5]
        );
        // The view, composed, expiring with now.
        assert!(t[6].starts_with("<time datetime=\"14:07\">14:07</time>\n[computed]\n"));
    }

    /// Every other page's kernel has no clock, and the time chapter's names say so.
    #[test]
    fn a_page_without_data_clock_has_no_clock() {
        let engine = ikigai_engine::Engine::new(page_kernel());
        let reply: serde_json::Value = serde_json::from_str(&action_to_json(
            engine.eval("source urn:iki:tutorial:time:now"),
        ))
        .unwrap();
        assert_eq!(reply["kind"], "error");
        assert!(
            reply["text"].as_str().unwrap().contains("no clock"),
            "{reply}"
        );
    }

    /// The reply a view counts: `cached` or `computed`, by a probe taken before the request.
    #[test]
    fn an_issued_read_says_whether_the_cache_answered_it() {
        let clock = Arc::new(time_resource::ManualClock::at(time_resource::millis_at(
            "2026-09-29T14:05Z",
            0,
        )));
        let page = Page::with_clock(Some(Arc::clone(&clock) as Arc<dyn Clock>));
        let view = time_resource::VIEW_CLOCK;
        let ask = || {
            let was_cached = page.is_cached(None, Verb::Source, view);
            let reply = futures::executor::block_on(page.issue(None, Verb::Source, view));
            let json: serde_json::Value =
                serde_json::from_str(&reply_to_json(reply, was_cached)).unwrap();
            json["cache"].as_str().unwrap().to_string()
        };
        let mut verdicts = Vec::new();
        for _ in 0..61 {
            verdicts.push(ask());
            clock.advance(1000);
        }
        assert_eq!(verdicts[0], "computed");
        assert!(
            verdicts[1..60].iter().all(|v| v == "cached"),
            "{verdicts:?}"
        );
        assert_eq!(verdicts[60], "computed", "14:06:00");
    }

    /// A game's engine runs in that game; the root's does not; and a game id that is not
    /// one spelling of one segment is refused rather than made into a corridor.
    #[test]
    fn a_page_keeps_one_engine_per_game_over_one_kernel() {
        let page = Page::new();
        let a = page.engine(Some("a")).expect("a game");
        assert!(Rc::ptr_eq(
            &a,
            &page.engine(Some("a")).expect("the same game")
        ));
        assert!(!Rc::ptr_eq(&a, &page.engine(None).expect("the root")));
        assert!(page.engine(Some("a b")).is_err());
        assert!(page.engine(Some("")).is_err());

        let text = |engine: &Engine, line: &str| -> String {
            let reply: serde_json::Value =
                serde_json::from_str(&action_to_json(engine.eval(line))).unwrap();
            reply["text"].as_str().unwrap().to_string()
        };
        assert_eq!(
            text(&a, "sink urn:iki:tutorial:ttt:move:1:1"),
            "X plays 1,1"
        );
        let root = page.engine(None).unwrap();
        assert_eq!(text(&root, "source urn:iki:tutorial:ttt:cell:1:1"), "-");
        assert_eq!(text(&a, "source urn:iki:tutorial:ttt:cell:1:1"), "X");
    }

    /// What the chapter says `urn:kernel:topology` shows, without printing it: the game's
    /// space is an `ik:Alias` carrying eight rules over the four bound templates.
    #[test]
    fn the_topology_shows_the_alias_the_chapter_describes() {
        let engine = ikigai_engine::Engine::new(page_kernel());
        let reply: serde_json::Value =
            serde_json::from_str(&action_to_json(engine.eval("source urn:kernel:topology")))
                .unwrap();
        let turtle = reply["text"].as_str().unwrap();
        assert_eq!(turtle.matches(" a ik:Alias ").count(), 1, "{turtle}");
        assert_eq!(turtle.matches(" a ik:RewriteRule ").count(), 8, "{turtle}");
        for pattern in [
            tic_tac_toe::STORED,
            tic_tac_toe::CELL,
            tic_tac_toe::CELLS,
            tic_tac_toe::BOARD,
        ] {
            assert!(
                turtle.contains(&format!("ik:pattern \"{pattern}\"")),
                "{pattern}: {turtle}"
            );
        }
        assert!(
            turtle.contains("ik:logical \"urn:iki:tutorial:ttt:row:0\""),
            "{turtle}"
        );
    }

    #[test]
    fn part_one_and_the_game_share_the_page_without_overlapping() {
        // Each family resolves to its own endpoint: the Fallback's order decides nothing.
        let names = bound_names();
        for name in [
            "urn:iki:tutorial:ttt:stored:{x}:{y}",
            "urn:iki:tutorial:ttt:cell:{x}:{y}",
            "urn:iki:tutorial:ttt:cells:{list}",
            "urn:iki:tutorial:ttt:board",
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
