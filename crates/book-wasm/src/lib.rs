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
//! A cell may also name a **game** (`data-game='a'`, Part IV of the applied chapter) or a
//! person's **scenario** (`data-scenario='alice'`, the spreadsheet's part VI). It then runs in
//! that chain — [`tic_tac_toe::game`] or [`spreadsheet::scenario`], one corridor over a store
//! of its own — on the SAME kernel, through an engine whose resolver ([`InChain`]) issues
//! every request in the chain. Choosing the chain is the host's
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
    ArgRef, Capability, Clock, CutBatch, Error, Expiry, Fallback, FixedClock, Iri, Kernel,
    Provenance, Representation, Request, Scope, Space, SpaceEntry, Tracer, Verb,
};
use ikigai_engine::Engine;
use ikigai_resolve::{CacheStatus, Resolver};
use push::{Drawn, Listening};
use spreadsheet::InputStore;
use time_resource::ManualClock;
use wasm_bindgen::prelude::*;

thread_local! {
    // The clock the page's kernel is built with, if the page asked for one before the
    // kernel was built (`useClock`, `useStoryClock`), and whether it has been built yet.
    static CLOCK: RefCell<Option<Arc<dyn Clock>>> = const { RefCell::new(None) };
    // The story clock, when the page asked for one: the same clock as above, kept by its own
    // type so a cell can set it (`setStoryClock`).
    static STORY: RefCell<Option<Arc<ManualClock>>> = const { RefCell::new(None) };
    static BUILT: Cell<bool> = const { Cell::new(false) };
    // `Rc` so an async eval can own a handle across `.await` points — a thread-local
    // borrow cannot span an await.
    static PAGE: Rc<Page> = {
        BUILT.with(|built| built.set(true));
        Rc::new(Page::with_clock(CLOCK.with(|clock| clock.borrow().clone())))
    };
}

/// Every name a cell may resolve: Part I's space first, then the applied chapter's game,
/// then the time chapter's clock, then the spreadsheet, then push (the spreadsheet's part VII),
/// then the game's declarations and the build that reads them (tic-tac-toe's part VII).
///
/// One space for the whole book rather than one per chapter, because the families are
/// disjoint (`urn:iki:tutorial:{camel-case,title,camel-title}` and `urn:iki:fn:*` against
/// `urn:iki:tutorial:ttt:*`, `urn:iki:tutorial:time:*` and `urn:iki:tutorial:sheet:*`), so the order decides
/// nothing and every cell Part I ships answers exactly as it did. What a cell *changes* is
/// still per page: each page load instantiates the wasm afresh, so a game played on one
/// chapter's page is not on the next one's. The URN gate asks this same space which names
/// a cell may use.
pub fn page_space() -> Fallback {
    page_space_over(Arc::default(), Arc::default())
}

/// [`page_space`], with the sheet's inputs in a store the caller keeps (the page keeps it,
/// because its temporal corridors read the same store's history), and push's listeners in a
/// registry the caller keeps (the page keeps it, to await a listener and to attach the kernel).
pub fn page_space_over(inputs: Arc<InputStore>, listening: Arc<Listening>) -> Fallback {
    Fallback::new(vec![
        Arc::new(hello_camel::space()) as Arc<dyn Space>,
        Arc::new(tic_tac_toe::space()),
        Arc::new(time_resource::space()),
        Arc::new(spreadsheet::space_over(inputs)),
        Arc::new(push::space(listening)),
        // Tic-tac-toe's part VII: the game's declarations as resources, the build that reads
        // one through this kernel, and ikigai-sexpr's arrangement transreptors, which read an
        // `.arrangement` as Turtle. `urn:iki:tutorial:ttt:arrangement:*`,
        // `urn:iki:tutorial:ttt:build` and `urn:sexpr:arrangement-*` overlap nothing above.
        Arc::new(tic_tac_toe::declared::declaration_space()),
    ])
}

// ANCHOR: as_of_doors
/// The doors a temporal corridor binds on this page, for every line that says `as-of=`: the
/// sheet's inputs as of the instant, from the page's own store, and the time chapter's `now`
/// and `today`, pinned. Which names a corridor rebinds is a fact about the host, so the host
/// hands them to the engine; the engine builds the corridor, named for its instant.
pub fn as_of_doors(inputs: Arc<InputStore>) -> Arc<dyn Space> {
    Arc::new(Fallback::new(vec![
        Arc::new(spreadsheet::as_of_doors(inputs)) as Arc<dyn Space>,
        Arc::new(time_resource::as_of_doors()),
    ]))
}
// ANCHOR_END: as_of_doors

/// The page's kernel: [`page_space`] with the Meta renderer `hello_camel::kernel()` uses,
/// so `describe` and `urn:kernel:catalog` answer in the page. No clock.
pub fn page_kernel() -> Kernel {
    page_kernel_with_clock(None)
}

// ANCHOR: page_kernel_with_clock
/// [`page_kernel`], with `clock` if there is one: the browser's on a page that asked for
/// it, a clock a test holds in a test.
pub fn page_kernel_with_clock(clock: Option<Arc<dyn Clock>>) -> Kernel {
    page_kernel_over(Arc::default(), Arc::default(), clock)
}
// ANCHOR_END: page_kernel_with_clock

/// [`page_kernel_with_clock`], over the sheet's inputs in `inputs` and push's registry in
/// `listening`. The caller attaches the kernel to `listening` once it is in an `Arc`.
fn page_kernel_over(
    inputs: Arc<InputStore>,
    listening: Arc<Listening>,
    clock: Option<Arc<dyn Clock>>,
) -> Kernel {
    let kernel = Kernel::with_meta_renderer(
        Arc::new(page_space_over(inputs, listening)),
        Arc::new(ikigai_vocab::TurtleRenderer),
    );
    match clock {
        Some(clock) => kernel.with_clock(clock),
        None => kernel,
    }
}

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
    before_the_kernel()?;
    CLOCK.with(|clock| *clock.borrow_mut() = Some(host_clock()));
    Ok(())
}

// ANCHOR: story_clock
/// Build this page's kernel with a STORY clock, stopped at `instant` (`YYYY-MM-DDTHH:MMZ`),
/// which the page moves when a cell says when it runs (`data-at`, [`set_story_clock`]).
///
/// For a chapter about time as context, where a cell reads the sheet as of last Tuesday: with
/// the browser's clock, "last Tuesday" would be a date before anything on the page was
/// typed, and every such cell would answer an empty sheet. With the story's, every reader's
/// page runs at the instants the chapter says, and answers what the chapter shows.
#[wasm_bindgen(js_name = useStoryClock)]
pub fn use_story_clock(instant: String) -> Result<(), JsValue> {
    before_the_kernel()?;
    let clock = Arc::new(ManualClock::at(story_millis(&instant)?));
    STORY.with(|story| *story.borrow_mut() = Some(Arc::clone(&clock)));
    CLOCK.with(|slot| *slot.borrow_mut() = Some(clock as Arc<dyn Clock>));
    Ok(())
}

/// Move the story clock to `instant`: what `js/run.js` does before a cell that carries
/// `data-at` runs. Refused on a page with no story clock.
#[wasm_bindgen(js_name = setStoryClock)]
pub fn set_story_clock(instant: String) -> Result<(), JsValue> {
    let millis = story_millis(&instant)?;
    STORY.with(|story| match story.borrow().as_ref() {
        Some(clock) => {
            clock.set(millis);
            Ok(())
        }
        None => Err(JsValue::from_str(
            "this page has no story clock: its markup does not carry data-clock='<instant>'",
        )),
    })
}
// ANCHOR_END: story_clock

fn story_millis(instant: &str) -> Result<u64, JsValue> {
    time_resource::parse_instant(instant)
        .and_then(|minute| u64::try_from(minute).ok())
        .map(|minute| minute * 60_000)
        .ok_or_else(|| {
            JsValue::from_str(&format!(
                "`{instant}` is not an instant spelled YYYY-MM-DDTHH:MMZ, after 1970"
            ))
        })
}

fn before_the_kernel() -> Result<(), JsValue> {
    if BUILT.with(Cell::get) {
        return Err(JsValue::from_str(
            "the page's kernel is already built, without a clock",
        ));
    }
    Ok(())
}

/// The page: one kernel, the engine a cell with no chain runs in, and one engine per chain a
/// cell (or a board, or a sheet) has named — every engine over the SAME kernel, so one cache
/// and one set of golden threads serve them all, partitioned by the chain each request
/// carries.
pub struct Page {
    kernel: Arc<Kernel>,
    doors: Arc<dyn Space>,
    root: Rc<Engine>,
    chains: RefCell<BTreeMap<String, Chain>>,
    /// The kernel's clock, if the page gave it one: what "now" is when the page asks which of
    /// the views it drew have expired.
    clock: Option<Arc<dyn Clock>>,
    /// Push's listeners, by name (part VII).
    listening: Arc<Listening>,
    /// The views the page has drawn for a sheet that listens, and what each answer carried.
    drawn: RefCell<Drawn>,
}

/// One chain on the page: the engine its cells run in, and the chain itself — the same
/// chain, cloned, so a cell and a click in one game (or one scenario) are one to the cache.
struct Chain {
    engine: Rc<Engine>,
    scope: Scope,
}

impl Default for Page {
    fn default() -> Self {
        Self::new()
    }
}

impl Page {
    /// A fresh page: [`page_kernel`], and no chain used yet.
    pub fn new() -> Self {
        Self::with_clock(None)
    }

    /// A fresh page whose kernel has `clock`, if there is one.
    pub fn with_clock(clock: Option<Arc<dyn Clock>>) -> Self {
        let inputs = Arc::new(InputStore::default());
        let listening = Arc::new(Listening::default());
        let kernel = Arc::new(page_kernel_over(
            Arc::clone(&inputs),
            Arc::clone(&listening),
            clock.clone(),
        ));
        listening.attach(&kernel);
        let doors = as_of_doors(inputs);
        Self {
            root: Rc::new(Engine::new(Arc::clone(&kernel)).with_as_of_doors(Arc::clone(&doors))),
            kernel,
            doors,
            chains: RefCell::default(),
            clock,
            listening,
            drawn: RefCell::default(),
        }
    }

    /// The engine for a cell: the root's when it names no chain, else that chain's — created
    /// on first use and kept for the life of the page, since a corridor is built ONCE (its
    /// name is the cache's claim that it is one game, one scenario).
    pub fn engine(&self, chain: Option<&str>) -> Result<Rc<Engine>, String> {
        match chain {
            None => Ok(Rc::clone(&self.root)),
            Some(spec) => {
                self.declared(spec)?;
                self.chain(spec, |chain| Rc::clone(&chain.engine))
            }
        }
    }

    /// Make chain `spec` if it names a game built from `tic-tac-toe.arrangement`
    /// (`declared:a`, tic-tac-toe's part VII) and the page does not have it yet: one corridor
    /// holding the whole declared game, ahead of the root's coded one. Any other spec is left
    /// to [`chain`](Self::chain).
    fn declared(&self, spec: &str) -> Result<(), String> {
        let Some(id) = spec.strip_prefix("declared:") else {
            return Ok(());
        };
        if self.chains.borrow().contains_key(spec) {
            return Ok(());
        }
        let scope = tic_tac_toe::declared::game(id).map_err(|e| e.to_string())?;
        let chain = Chain {
            engine: Rc::new(
                Engine::new(InChain::new(Arc::clone(&self.kernel), scope.clone()))
                    .with_as_of_doors(Arc::clone(&self.doors)),
            ),
            scope,
        };
        self.chains.borrow_mut().insert(spec.to_string(), chain);
        Ok(())
    }

    // ANCHOR: chains
    /// Chain `spec`, made on first use, handed to `with`. A spec is what the page's markup
    /// says: `scenario:alice` is alice's scenario over the shared sheet;
    /// `scenario:alice@2026-09-22T18:00Z` is her scenario inside the sheet as of that instant,
    /// so her overrides win over the past; anything else is a tic-tac-toe game's id.
    fn chain<T>(&self, spec: &str, with: impl FnOnce(&Chain) -> T) -> Result<T, String> {
        self.build(spec)?;
        Ok(with(self.chains.borrow().get(spec).expect("built above")))
    }

    /// Make chain `spec`, unless the page has it already.
    fn build(&self, spec: &str) -> Result<(), String> {
        if self.chains.borrow().contains_key(spec) {
            return Ok(());
        }
        let scope = match spec.strip_prefix("scenario:") {
            // One person, one corridor: the scenario over an instant stacks the SAME scope
            // (the same store, under the same name) as the scenario alone, since a name is a
            // claim that its doors are one set.
            Some(rest) => match rest.split_once('@') {
                Some((who, at)) => {
                    let alone = format!("scenario:{who}");
                    self.build(&alone)?;
                    let alone = self.chains.borrow()[&alone].scope.clone();
                    self.as_of(at)?.stack(&alone)
                }
                None => spreadsheet::scenario(rest, Arc::default()).map_err(|e| e.to_string())?,
            },
            None => {
                let store = Arc::new(tic_tac_toe::stored_space(Arc::default()));
                tic_tac_toe::game(spec, store).map_err(|e| e.to_string())?
            }
        };
        let chain = Chain {
            engine: Rc::new(
                Engine::new(InChain::new(Arc::clone(&self.kernel), scope.clone()))
                    .with_as_of_doors(Arc::clone(&self.doors)),
            ),
            scope,
        };
        self.chains.borrow_mut().insert(spec.to_string(), chain);
        Ok(())
    }

    /// The sheet as of `instant` (`YYYY-MM-DDTHH:MMZ`): the corridor a line's `as-of=` builds,
    /// named the way the engine names it, over the same doors.
    fn as_of(&self, instant: &str) -> Result<Scope, String> {
        let millis =
            story_millis(instant).map_err(|e| e.as_string().unwrap_or_else(|| format!("{e:?}")))?;
        let canonical = format!("{}:00Z", instant.trim_end_matches('Z'));
        let name = Iri::parse(format!("urn:ctx:time:{canonical}")).map_err(|e| e.to_string())?;
        Ok(Scope::empty().with_named_at(
            name,
            Arc::clone(&self.doors),
            Arc::new(FixedClock::at(millis)),
        ))
    }
    // ANCHOR_END: chains

    /// The scope of chain `spec`, or the empty chain for none.
    fn scope_of(&self, chain: Option<&str>) -> Result<Scope, String> {
        match chain {
            None => Ok(Scope::empty()),
            Some(spec) => {
                self.declared(spec)?;
                self.chain(spec, |chain| chain.scope.clone())
            }
        }
    }

    // ANCHOR: page_issue
    /// One request, straight to the kernel — not a line of the REPL — in chain `chain`, or
    /// the root's when it names none. What the page's htmx shims send for a click: `GET` a
    /// view is a `Source`, `POST` a play or an edit is a `Sink`, in the chain the markup
    /// names, with a form's fields (`args`) as named arguments, the way a server reads a form
    /// body. Resolves to the representation, or the kernel's refusal as its message.
    pub fn issue_with_args(
        &self,
        chain: Option<&str>,
        verb: Verb,
        target: &str,
        args: Vec<(String, String)>,
    ) -> impl std::future::Future<Output = Result<Representation, String>> + 'static {
        let kernel = Arc::clone(&self.kernel);
        let scope = self.scope_of(chain);
        let target = Iri::parse(target).map_err(|e| format!("not a resource name: {e}"));
        async move {
            let request = args
                .into_iter()
                .fold(Request::new(verb, target?), |request, (name, value)| {
                    request.with_arg(name, ArgRef::Inline(value.into_bytes()))
                });
            kernel
                .issue_in(request, &Capability::root(), scope?)
                .await
                .map_err(|e| e.to_string())
        }
    }
    // ANCHOR_END: page_issue

    /// [`issue_with_args`](Self::issue_with_args) with no arguments: every read, and a play,
    /// whose square is in its name.
    pub fn issue(
        &self,
        chain: Option<&str>,
        verb: Verb,
        target: &str,
    ) -> impl std::future::Future<Output = Result<Representation, String>> + 'static {
        self.issue_with_args(chain, verb, target, Vec::new())
    }

    // ANCHOR: page_push
    /// The page drew `view` from `answer`, for a sheet that listens: remember what the answer
    /// carried, so a cut can be mapped to the views it makes stale.
    pub fn drew(&self, view: &str, answer: &Representation) {
        self.drawn.borrow_mut().drew(view, answer);
    }

    /// The views of the page's that a batch of cuts makes stale.
    pub fn stale(&self, batch: &CutBatch) -> Vec<String> {
        self.drawn.borrow().stale(batch)
    }

    /// The earliest deadline among the views the page drew, in milliseconds since 1970.
    pub fn deadline(&self) -> Option<u64> {
        self.drawn.borrow().deadline()
    }

    /// The views the page drew whose deadline has passed, by the kernel's clock. None on a
    /// page with no clock, where nothing the kernel answers has a deadline it can honor.
    pub fn expired(&self) -> Vec<String> {
        match &self.clock {
            Some(clock) => self.drawn.borrow().expired(clock.now().as_millis()),
            None => Vec::new(),
        }
    }
    // ANCHOR_END: page_push

    /// Would [`issue`](Self::issue) be answered from the cache right now? A probe, which
    /// resolves nothing and runs nothing: what a page asks before a request so it can say
    /// afterwards whether the kernel did any work. A name that does not parse is not cached.
    pub fn is_cached(&self, chain: Option<&str>, verb: Verb, target: &str) -> bool {
        let Ok(scope) = self.scope_of(chain) else {
            return false;
        };
        Iri::parse(target).is_ok_and(|target| {
            self.kernel
                .is_cached_in(&Request::new(verb, target), &Capability::root(), &scope)
        })
    }
}

// ANCHOR: in_chain
/// A [`Resolver`] that issues every request in ONE resolution chain — a game's, a person's
/// scenario — on a shared kernel. The engine takes any resolver, so an engine over `InChain`
/// is the whole REPL grammar (`source`, `sink`, `cache`, `trace`, pipes, maps) in one chain:
/// every stage, every contract fetch and every cache probe carries it, and the kernel hands it
/// on to every sub-request.
///
/// A line may bring a chain of its own: `as-of=<instant>` resolves the line in a temporal
/// corridor. The engine hands that chain to [`issue_as_async_in`](Resolver::issue_as_async_in),
/// and `InChain` stacks it INSIDE its own (`Scope::stack`, core 0.1.82), so the line's corridor
/// is the innermost, and the one that answers first.
pub struct InChain {
    kernel: Arc<Kernel>,
    scope: Scope,
}

impl InChain {
    /// Resolve in `scope` on `kernel`.
    pub fn new(kernel: Arc<Kernel>, scope: Scope) -> Self {
        Self { kernel, scope }
    }

    // ANCHOR: with_line
    /// This chain with the line's inside it: the line's corridor innermost, answering first.
    fn with_line(&self, line: &Scope) -> Scope {
        self.scope.clone().stack(line)
    }
    // ANCHOR_END: with_line

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
impl Resolver for InChain {
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
        self.issue_as_async_in(request, capability, None, Scope::empty())
            .await
    }

    async fn issue_as_async_with_incoming(
        &self,
        request: Request,
        capability: &Capability,
        incoming: Provenance,
    ) -> Result<(Representation, CacheStatus), Error> {
        self.issue_as_async_in(request, capability, Some(incoming), Scope::empty())
            .await
    }

    async fn issue_as_async_in(
        &self,
        request: Request,
        capability: &Capability,
        incoming: Option<Provenance>,
        line: Scope,
    ) -> Result<(Representation, CacheStatus), Error> {
        let scope = self.with_line(&line);
        let was_cached = self.kernel.is_cached_in(&request, capability, &scope);
        let representation = match incoming {
            Some(incoming) => {
                self.kernel
                    .issue_with_incoming_in(request, capability, incoming, scope)
                    .await?
            }
            None => self.kernel.issue_in(request, capability, scope).await?,
        };
        let status = Self::status(was_cached, &representation);
        Ok((representation, status))
    }

    fn is_cached(&self, request: &Request, capability: &Capability) -> bool {
        self.kernel.is_cached_in(request, capability, &self.scope)
    }

    fn is_cached_in(&self, request: &Request, capability: &Capability, line: &Scope) -> bool {
        self.kernel
            .is_cached_in(request, capability, &self.with_line(line))
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

    /// What `trace` prints as the transport — so a traced line says which chain it ran in.
    fn transport(&self) -> String {
        format!("embedded · in-process · chain {}", self.scope)
    }
}
// ANCHOR_END: in_chain

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

/// [`eval_line_async`] in chain `chain`, as [`Page::engine`] reads a chain's spec: what a cell
/// carrying `data-scenario='…'` (and perhaps `data-as-of='…'`) sends.
#[wasm_bindgen(js_name = evalLineInChainAsync)]
pub fn eval_line_in_chain_async(chain: String, line: String) -> js_sys::Promise {
    eval_in(Some(chain), line)
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
/// `verb` is `source`, `sink`, `exists` or `delete`; `game` is the chain the markup names (a
/// board's game, a sheet's scenario), or empty for the root's. Resolves to the JSON of [`reply_to_json`], with the
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

/// [`issue_async`] with a form's fields as named arguments: what `js/sheet.js` sends for
/// the formula bar. `args` is JSON, an array of `[name, value]` pairs in form order.
#[wasm_bindgen(js_name = issueWithArgsAsync)]
pub fn issue_with_args_async(
    game: String,
    verb: String,
    target: String,
    args: String,
) -> js_sys::Promise {
    let reply = verb_named(&verb).and_then(|verb| {
        let args: Vec<(String, String)> = serde_json::from_str(&args)
            .map_err(|e| format!("the form's fields are not [name, value] pairs: {e}"))?;
        let game = (!game.is_empty()).then_some(game);
        Ok(PAGE.with(|page| {
            let was_cached = page.is_cached(game.as_deref(), verb, &target);
            (
                was_cached,
                page.issue_with_args(game.as_deref(), verb, &target, args),
            )
        }))
    });
    wasm_bindgen_futures::future_to_promise(async move {
        let json = match reply {
            Ok((was_cached, issued)) => reply_to_json(issued.await, was_cached),
            Err(refusal) => reply_to_json(Err(refusal), false),
        };
        Ok(JsValue::from_str(&json))
    })
}

// ANCHOR: push_exports
/// A read of `view` in the root's chain, as [`issue_async`] makes it, for a sheet that LISTENS
/// (`data-push`, part VII): the page also remembers what the answer carried ([`Page::drew`]),
/// so that a cut can be mapped to the views it makes stale, and an expiry can be timed.
#[wasm_bindgen(js_name = drawAsync)]
pub fn draw_async(view: String) -> js_sys::Promise {
    let page = PAGE.with(Rc::clone);
    let was_cached = page.is_cached(None, Verb::Source, &view);
    let issued = page.issue(None, Verb::Source, &view);
    wasm_bindgen_futures::future_to_promise(async move {
        let answer = issued.await;
        if let Ok(representation) = &answer {
            page.drew(&view, representation);
        }
        Ok(JsValue::from_str(&reply_to_json(answer, was_cached)))
    })
}

/// Wait for the next cuts the listener `name` hears (registered by a `Sink` to
/// `urn:iki:tutorial:push:listener:{name}`), and resolve to what they mean for the page:
/// `{ "cuts": [thread…], "dropped": n, "stale": [view…] }`. `stale` is every view the page drew
/// whose answer carried a thread that was cut, or every view it drew when cuts were dropped.
/// Resolves to `{ "error": … }` when nothing is listening under that name.
///
/// The kernel never calls the page: a cut appends to the listener's queue and wakes this
/// future, and the page decides what to do, on its own turn of the event loop.
#[wasm_bindgen(js_name = pushWaitAsync)]
pub fn push_wait_async(name: String) -> js_sys::Promise {
    let page = PAGE.with(Rc::clone);
    let listener = page.listening.listener(&name);
    wasm_bindgen_futures::future_to_promise(async move {
        let Some(listener) = listener else {
            let error = format!("nothing is listening as `{name}`");
            return Ok(JsValue::from_str(
                &serde_json::json!({ "error": error }).to_string(),
            ));
        };
        let batch = listener.wait().await;
        Ok(JsValue::from_str(&batch_to_json(
            &batch,
            &page.stale(&batch),
        )))
    })
}

/// The earliest deadline among the views the page drew, in milliseconds since 1970, or `-1`
/// when none has one: when to set the page's one timer.
#[wasm_bindgen(js_name = pushDeadline)]
pub fn push_deadline() -> f64 {
    PAGE.with(|page| page.deadline().map_or(-1.0, |at| at as f64))
}

/// The views the page drew whose deadline has passed, as a JSON array: what the timer draws
/// again when it fires.
#[wasm_bindgen(js_name = pushExpired)]
pub fn push_expired() -> String {
    PAGE.with(|page| serde_json::json!(page.expired()).to_string())
}
// ANCHOR_END: push_exports

/// A batch of cuts as JSON for the page, with the views it makes stale.
pub fn batch_to_json(batch: &CutBatch, stale: &[String]) -> String {
    let cuts: Vec<&str> = batch.events.iter().map(|e| e.thread.as_str()).collect();
    serde_json::json!({ "cuts": cuts, "dropped": batch.dropped, "stale": stale }).to_string()
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
            "cache": match InChain::status(was_cached, &representation) {
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

    /// One runnable cell of a chapter: the chain it runs in (`data-game`, or `data-scenario`
    /// with any `data-as-of`, as `js/run.js` spells it for the page), the instant it runs at
    /// on a page with a story clock (`data-at`), its command, and the text of its expected
    /// output.
    #[derive(Debug)]
    struct RunCell {
        chain: Option<String>,
        at: Option<String>,
        command: String,
        expected: String,
    }

    /// The value of `attribute='…'` in `tag`, if it is there.
    fn attribute(tag: &str, attribute: &str) -> Option<String> {
        let open = format!("{attribute}='");
        tag.find(&open).map(|at| {
            let value = &tag[at + open.len()..];
            value[..value.find('\'').expect("a closed attribute")].to_string()
        })
    }

    /// The chain a tag names, spelled as the page spells it for [`Page::engine`].
    fn chain_of(tag: &str) -> Option<String> {
        attribute(tag, "data-game")
            .or_else(|| attribute(tag, "data-declared").map(|id| format!("declared:{id}")))
            .or_else(|| {
                attribute(tag, "data-scenario").map(|who| match attribute(tag, "data-as-of") {
                    Some(at) => format!("scenario:{who}@{at}"),
                    None => format!("scenario:{who}"),
                })
            })
    }

    /// The runnable cells of a chapter, in page order, with the HTML escapes the markup needs
    /// undone in each expected output.
    fn cells(chapter: &str) -> Vec<RunCell> {
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
            // The cell's other attributes sit in the same tag, before `data-cmd`.
            let tag = &rest[rest[..at].rfind("<div").expect("a cell is a <div>")..at];
            let chain = chain_of(tag);
            let after = &rest[at + "data-cmd='".len()..];
            let end = after.find('\'').expect("a closed data-cmd");
            let command = after[..end].to_string();
            let open = "<pre class=\"ikigai-run-expected\">";
            let pre = after.find(open).expect("an expected <pre> after the cell") + open.len();
            let close = after[pre..].find("</pre>").expect("a closed <pre>") + pre;
            found.push(RunCell {
                chain,
                at: attribute(tag, "data-at"),
                command,
                expected: unescape(&after[pre..close]),
            });
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

    /// A sheet a chapter places: its shell, whether it shows the cache viewer (`data-cache`),
    /// and whether it listens (`data-push`, part VII).
    struct Sheet {
        shell: String,
        viewer: bool,
        listens: bool,
    }

    /// The sheets a chapter places, in page order.
    fn sheets(chapter: &str) -> Vec<Sheet> {
        chapter
            .split("<div class=\"sheet-play\"")
            .skip(1)
            .map(|rest| {
                let tag = &rest[..rest.find('>').expect("a closed tag")];
                Sheet {
                    shell: attribute(tag, "data-shell").expect("a sheet names its shell"),
                    viewer: tag.contains("data-cache"),
                    listens: tag.contains("data-push"),
                }
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
        // A story clock (`data-clock='<instant>'`) starts where the page says, and each cell
        // that says when it runs (`data-at`) moves it there first, as `js/run.js` does.
        let story = chapter
            .split("data-clock='")
            .nth(1)
            .map(|rest| rest[..rest.find('\'').expect("a closed data-clock")].to_string());
        if let Some(instant) = &story {
            clock.set(time_resource::millis_at(instant, 0));
        }
        let at = |i: usize| {
            if let Some((_, millis)) = times.iter().find(|(cell, _)| *cell == i) {
                clock.set(*millis);
            }
            if let Some(instant) = cells[i].at.as_deref() {
                clock.set(time_resource::millis_at(instant, 0));
            }
        };
        at(0);

        // A trace names the thread each node ran on. The page has one, which prints as
        // `ThreadId(1)`; a test runs on a thread named after the test (or `main`). Spell
        // this one the way the page does, so the chapter can show what a reader sees.
        let here = std::thread::current()
            .name()
            .map(|name| format!(" · {name} · "));

        // One page: one kernel, and an engine per chain a cell names — as in the browser.
        let page = Page::with_clock(clocked.then(|| Arc::clone(&clock) as Arc<dyn Clock>));
        // A view host loads its shell when the kernel loads, before any cell can run; its
        // polling starts paused, so the shell is the only request it makes.
        for shell in view_shells(&chapter) {
            futures::executor::block_on(page.issue(None, Verb::Source, &shell))
                .unwrap_or_else(|e| panic!("the view shell {shell}: {e}"));
        }
        // A playable board (`<div class="ttt-play" data-game='…'>`, Part V) loads when the
        // kernel does, before any cell can run: `js/ttt.js` fetches the page shell
        // (`view:game:{game}`), and the shell's markup asks for the status and then the
        // board. And after a cell runs in a board's game, the shim refreshes that board the
        // same way. Both are reads that fill the cache, so the chapter's cells see them —
        // replay them here, in the same order.
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
                &tic_tac_toe::view_game_name(game),
            ))
            .expect("the page shell");
            refresh(game);
        }
        // A sheet you can type into (`<div class="sheet-play" data-shell='…'>`, the
        // spreadsheet arc) loads the same way: `js/sheet.js` fetches its shell, and the shell's
        // markup asks for the grid. After a cell runs in the root's chain, the shim redraws
        // every sheet on the page — first, where the sheet shows the cache viewer
        // (`data-cache`), reading the cost view (whose probes resolve no value, but whose
        // templates are read and cached) — so replay those reads too. A cell in any other
        // chain (a game, a scenario) changes nothing a sheet shows, so it redraws none.
        let sheets = sheets(&chapter);
        // A sheet that listens (part VII) never redraws itself after a cell: while push is off,
        // which is how the page starts, nothing tells it, and that is the point.
        let redraw = || {
            for Sheet { viewer, .. } in sheets.iter().filter(|sheet| !sheet.listens) {
                if *viewer {
                    futures::executor::block_on(page.issue(
                        None,
                        Verb::Source,
                        spreadsheet::VIEW_COST,
                    ))
                    .expect("the sheet's cost view");
                }
                futures::executor::block_on(page.issue(None, Verb::Source, spreadsheet::VIEW_GRID))
                    .unwrap_or_else(|e| panic!("the sheet's grid: {e}"));
            }
        };
        // On load, a sheet reads its shell and then its grid, and nothing else: the viewer
        // says nothing until something has changed. A sheet that listens draws each cell's view
        // instead, in the shell's order (row by row), and the page remembers what each carried.
        for sheet in &sheets {
            futures::executor::block_on(page.issue(None, Verb::Source, &sheet.shell))
                .unwrap_or_else(|e| panic!("the sheet's {}: {e}", sheet.shell));
            if !sheet.listens {
                futures::executor::block_on(page.issue(None, Verb::Source, spreadsheet::VIEW_GRID))
                    .unwrap_or_else(|e| panic!("the sheet's grid: {e}"));
                continue;
            }
            for row in 1..=6 {
                for column in ["A", "B", "C", "D"] {
                    let view = format!("urn:iki:tutorial:sheet:view:cell:{column}{row}");
                    let answer = futures::executor::block_on(page.issue(None, Verb::Source, &view))
                        .unwrap_or_else(|e| panic!("{view}: {e}"));
                    page.drew(&view, &answer);
                }
            }
        }
        let mut transcript = Vec::new();
        for (i, cell) in cells.iter().enumerate() {
            let (chain, command, expected) = (&cell.chain, &cell.command, &cell.expected);
            at(i);
            let engine = page.engine(chain.as_deref()).expect("a valid chain");
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
                println!("=== {chain:?}\n{command}\n--- got\n{got}=== end");
            } else {
                assert_eq!(
                    tidy(&got),
                    tidy(expected),
                    "{file}: the cell `{command}` answers differently from the chapter.\n--- got ---\n{got}"
                );
            }
            if let Some(game) = chain.as_deref().filter(|g| boards.iter().any(|b| b == g)) {
                refresh(game);
            }
            if chain.is_none() {
                redraw();
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
        let t = run_chapter_in_page_order("tic-tac-toe-5.md", 9);

        // A template is served as written, markers and all.
        assert!(t[0]
            .contains("aria-label=\"$h{urn:iki:tutorial:ttt:cell:{x}:{y}} at $h{{x}},$h{{y}}\">"));
        assert!(
            t[0].ends_with("$h{urn:iki:tutorial:ttt:turn} to play.\n[computed]\n"),
            "{}",
            t[0]
        );
        // A square's view is the square template filled for its name, cached by the board.
        assert!(
            t[1].contains("aria-label=\"2,0: empty, play here\"></button>\n[cached]"),
            "{}",
            t[1]
        );
        let t = &t[1..];
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
        // The status template's conditional asked the winner, which the move changed.
        assert!(verdict("urn:iki:fn:conditional ").contains(" · computed · "));
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
            "template:status ",
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

    /// Part VII: the game built from its file answers every cell exactly as the coded game
    /// does, in a corridor of its own, and a declaration that names an endpoint nobody
    /// registered is refused at build.
    #[test]
    fn the_seventh_tic_tac_toe_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("tic-tac-toe-7.md", 5);
        // The build answers the file's own arrangement, printed without its comments, and
        // the second read is served: it hangs from the file's thread.
        let tree = tic_tac_toe::declared::topology().expect("the file reads");
        let printed = ikigai_sexpr::topology_to_arrangement(&tree).expect("it prints");
        let printed = printed.trim_end();
        assert_eq!(
            t[0],
            format!("{printed}\n[computed]\n{printed}\n[cached]\n")
        );
        // The coded game and the declared one answer the same moves identically.
        assert_eq!(t[1], t[2]);
        assert!(t[1].contains("1,1 is taken"), "{}", t[1]);
        // Every node of the trace was answered by the declared game's corridor.
        let nodes: Vec<&str> = t[3]
            .lines()
            .filter(|l| l.contains("answered-by="))
            .collect();
        assert_eq!(nodes.len(), 2, "the cell and its stored cell: {}", t[3]);
        assert!(
            nodes
                .iter()
                .all(|n| n.contains("answered-by=urn:iki:tutorial:ttt:declared:a")),
            "{}",
            t[3]
        );
        // And a declaration that names an endpoint nobody registered is refused at build.
        assert!(
            t[4].contains("binds `ttt-undo`, which the host did not register"),
            "{}",
            t[4]
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

    /// The spreadsheet's first part: its cells in page order, on a page whose sheet loaded
    /// the grid before any cell ran and draws it again after each one.
    #[test]
    fn the_first_spreadsheet_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("spreadsheet-1.md", 7);
        // One spelling, on the sheet.
        assert!(
            t[0].starts_with("error: invalid argument `ref`: `a1` is not a cell"),
            "{}",
            t[0]
        );
        assert!(t[0].contains("E1 is off the sheet"), "{}", t[0]);
        // The atom: written, computed, served; an untouched input is NotFound.
        assert_eq!(
            t[1],
            "42\n[uncacheable]\n42\n[computed]\n42\n[cached]\n\
             error: not found: nothing has been typed into B1\n"
        );
        assert_eq!(
            t[2],
            "Rent\n[uncacheable]\nRent\n[computed]\nRent\n[cached]\n"
        );
        // The grid read every value when the page loaded, the empty C5 included.
        assert_eq!(t[3], "[cached]\n");
        // The first write to C5 cut the value built on its emptiness.
        assert_eq!(
            t[4],
            "cached\nhello\n[uncacheable]\nnot cached\nhello\n[computed]\n"
        );
        assert_eq!(t[5], "ok\n[uncacheable]\n[computed]\n");
        // `$h` escapes a value and never expands a marker in it.
        assert!(
            t[6].ends_with("&lt;b&gt;not bold&lt;/b&gt; $a{urn:kernel:cache}\n[computed]\n"),
            "{}",
            t[6]
        );
    }

    /// The spreadsheet's second part: formulas compiled, evaluated, and wrong five ways.
    #[test]
    fn the_second_spreadsheet_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("spreadsheet-2.md", 9);
        // The compiler says what it converts, and that it is lossy.
        assert!(
            t[0].contains("ik:transreptsFrom \"text/x-formula\""),
            "{}",
            t[0]
        );
        assert!(t[0].contains("ik:lossless false"), "{}", t[0]);
        // Compiled once, then served.
        assert!(
            t[1].ends_with("(* A1 2)\n[computed]\n(* A1 2)\n[cached]\n"),
            "{}",
            t[1]
        );
        // A2 was read (and compiled) by the grid after the last cell; A3 is new.
        assert!(t[2].ends_with("10\n[cached]\n11\n[computed]\n"), "{}", t[2]);
        // A named cell's edit leaves the formula compiled; its own input's edit does not, and
        // the other formula that names it stays compiled.
        assert_eq!(
            t[3],
            "7\n[uncacheable]\ncached\n=A1*3\n[uncacheable]\nnot cached\ncached\n"
        );
        assert!(
            t[4].contains("(* (sum (range A1 A3)) 2)\n[computed]\n"),
            "{}",
            t[4]
        );
        assert!(
            t[4].ends_with("A3:A1 is spelled top-left first: A1:A3\n"),
            "{}",
            t[4]
        );
        assert!(
            t[5].ends_with(
                "#DIV/0\n[computed]\n#DIV/0\n[computed]\n#REF\n[computed]\n#VALUE\n[computed]\n"
            ),
            "{}",
            t[5]
        );
        // A syntax error is a value, and it is cached.
        assert!(
            t[6].ends_with("#SYNTAX\n[computed]\n#SYNTAX\n[cached]\n"),
            "{}",
            t[6]
        );
        assert!(t[7].contains("#CYCLE\n[computed]\n"), "{}", t[7]);
        assert!(
            t[8].ends_with("urn:iki:tutorial:sheet:cell:D2\n[computed]\n5\n[computed]\n"),
            "{}",
            t[8]
        );
    }

    /// The spreadsheet's third part: an edit, and exactly its dependents computed again.
    #[test]
    fn the_third_spreadsheet_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("spreadsheet-3.md", 6);
        assert!(
            t[0].ends_with("26\n[computed]\n25\n[computed]\n"),
            "{}",
            t[0]
        );
        // Exactly the edited cell and its dependents are cut.
        assert_eq!(
            t[1],
            "7\n[uncacheable]\nnot cached\nnot cached\nnot cached\ncached\nnot cached\ncached\n"
        );
        // The trace: the chain of values computed, the compiled formulas served.
        let trace = &t[2];
        let verdict = |node: &str| {
            trace
                .lines()
                .find(|line| line.contains(node))
                .unwrap_or_else(|| panic!("{node} is not a node of:\n{trace}"))
                .to_string()
        };
        for node in ["cell:A3 ", "cell:A2 ", "cell:A1 ", "refs:A1 "] {
            assert!(verdict(node).contains(" · computed · "), "{node}: {trace}");
        }
        for node in ["formula:A3 ", "formula:A2 ", "refs:A2 "] {
            assert!(verdict(node).contains(" · cached · "), "{node}: {trace}");
        }
        assert!(
            t[3].contains("urn:iki:tutorial:sheet:input:A1  gen 3\n"),
            "{}",
            t[3]
        );
        // A new formula re-wires C1 with nothing told: B1 no longer cuts it, A1 now does.
        assert_eq!(
            t[4],
            "=A3/4\n[uncacheable]\n4.25\n[computed]\n200\n[uncacheable]\ncached\n\
             9\n[uncacheable]\nnot cached\n"
        );
        // No early cutoff: the same input cuts everything that read it.
        assert_eq!(t[5], "9\n[uncacheable]\nnot cached\n46\n[computed]\n");
    }

    /// The spreadsheet's fourth part: cells that read the clock, on a page whose kernel reads a
    /// clock the test holds, and the lessons stated rather than only matched.
    #[test]
    fn the_fourth_spreadsheet_chapters_cells_answer_as_it_says_in_page_order() {
        let at = |instant: &str, seconds: u64| time_resource::millis_at(instant, seconds);
        let t = run_chapter_at(
            "spreadsheet-4.md",
            5,
            &[
                (0, at("2026-09-29T14:05Z", 20)),
                (1, at("2026-09-29T14:05Z", 40)),
                (2, at("2026-09-29T14:06Z", 5)),
                (3, at("2026-09-29T14:07Z", 2)),
                (4, at("2026-09-30T00:00Z", 10)),
            ],
        );
        // The time, compiled once and read through a cell that never names it.
        assert!(
            t[0].ends_with("(now)\n[computed]\n2026-09-29T14:05Z\n[computed]\n"),
            "{}",
            t[0]
        );
        assert_eq!(t[1], "cached\ncached\n2026-09-29T14:05Z\n[cached]\n");
        // The minute turns: the time and what read it go, the date, the arithmetic and the
        // compiled formula stay, and the grid goes with the time.
        assert_eq!(
            t[2],
            "not cached\nnot cached\ncached\ncached\ncached\nnot cached\n\
             2026-09-29T14:06Z\n[computed]\n"
        );
        // The trace: the chain down to `now` computed, `today` served on both sides of it.
        let trace = &t[3];
        let verdict = |node: &str| {
            trace
                .lines()
                .find(|line| line.contains(node))
                .unwrap_or_else(|| panic!("{node} is not a node of:\n{trace}"))
                .to_string()
        };
        for node in [
            "sheet:cell:B1 ",
            "sheet:cell:A1 ",
            "time:instant ",
            "time:now ",
        ] {
            assert!(verdict(node).contains(" · computed · "), "{node}: {trace}");
        }
        for node in ["formula:A1 ", "precedents:B1 ", "input:A1 "] {
            assert!(verdict(node).contains(" · cached · "), "{node}: {trace}");
        }
        assert_eq!(
            trace.matches("time:today   time-today · cached").count(),
            2,
            "{trace}"
        );
        // Midnight: the date goes; the arithmetic still does not.
        assert_eq!(t[4], "not cached\ncached\n2026-09-30\n[computed]\n");
    }

    /// The spreadsheet's fifth part: a feed written from outside the sheet, and exactly its
    /// readers computed again.
    #[test]
    fn the_fifth_spreadsheet_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_in_page_order("spreadsheet-5.md", 5);
        assert!(t[0].ends_with("1000\n[computed]\n"), "{}", t[0]);
        // A write from outside: the feed's readers go, the rest stay.
        assert_eq!(
            t[1],
            "101.5\n[uncacheable]\nnot cached\ncached\nnot cached\ncached\n1015\n[computed]\n"
        );
        // The market's write, traced: the feed and its readers computed, A2 served.
        let trace = &t[2];
        let verdict = |node: &str| {
            trace
                .lines()
                .find(|line| line.contains(node))
                .unwrap_or_else(|| panic!("{node} is not a node of:\n{trace}"))
                .to_string()
        };
        for node in ["cell:A3 ", "cell:A1 ", "feed:acme "] {
            assert!(verdict(node).contains(" · computed · "), "{node}: {trace}");
        }
        for node in ["cell:A2 ", "formula:A1 ", "formula:A3 "] {
            assert!(verdict(node).contains(" · cached · "), "{node}: {trace}");
        }
        // An unwritten feed: #N/A, cached, and cut by the first write.
        assert_eq!(
            t[3],
            "=FEED(globex)\n[uncacheable]\n#N/A\n[computed]\n#N/A\n[cached]\n\
             55\n[uncacheable]\nnot cached\n55\n[computed]\n"
        );
        // Bad data is a value, and cached; the market's refusal is a Conflict.
        assert!(
            t[4].starts_with("halted\n[uncacheable]\n#VALUE\n[computed]\n#VALUE\n[cached]\n"),
            "{}",
            t[4]
        );
        assert!(t[4].contains("error: conflict: "), "{}", t[4]);
    }

    /// The spreadsheet's sixth part: a person's scenario, the sheet as of an instant, and the
    /// two stacked in both orders, on a page whose story clock the cells move.
    #[test]
    fn the_sixth_spreadsheet_chapters_cells_answer_as_it_says_in_page_order() {
        let t = run_chapter_at("spreadsheet-6.md", 13, &[]);
        // The week: 400 on Tuesday, 500 after the price rose.
        assert!(t[0].ends_with("400\n[computed]\n"), "{}", t[0]);
        assert_eq!(t[1], "110\n[uncacheable]\n500\n[computed]\n");
        // As of Tuesday evening: the sheet as it was, cached; before noon, empty.
        assert_eq!(t[2], "400\n[computed]\n400\n[cached]\n[computed]\n");
        // One now inside the corridor, and a then that caches.
        assert!(
            t[3].starts_with("2026-09-22T18:00Z\n[computed]\n2026-09-22T18:00Z\n[cached]\n"),
            "{}",
            t[3]
        );
        // Alice's what-if, and the shared sheet unmoved but recomputed (ledger item 581).
        assert_eq!(t[4], "12\n[uncacheable]\n720\n[computed]\n");
        assert_eq!(t[5], "500\n[computed]\n10\n[cached]\n");
        // A shared edit cuts both spaces' entries; a scenario edit cuts the shared one too.
        assert!(t[6].ends_with("not cached\n"), "{}", t[6]);
        assert_eq!(t[7], "not cached\n670\n[computed]\n");
        assert_eq!(t[9], "not cached\n450\n[computed]\n");
        // Both orders of one stack: Tuesday whole, and Alice's overrides over Tuesday.
        assert_eq!(t[10], "400\n[computed]\n");
        assert!(t[11].starts_with("900\n[computed]\n"), "{}", t[11]);
        assert!(t[11].ends_with("825\n[computed]\n"), "{}", t[11]);
        // Her cells need her capability; the shared cells need nothing.
        assert!(t[12].contains("error: denied: "), "{}", t[12]);
        assert!(t[12].contains("650\n[computed]\n"), "{}", t[12]);
    }

    /// The spreadsheet's seventh part: a listener, what a read carries, time that does not cut,
    /// and a scenario's edit heard by everybody, on a page whose sheet listens.
    #[test]
    fn the_seventh_spreadsheet_chapters_cells_answer_as_it_says_in_page_order() {
        let at = |instant: &str, seconds: u64| time_resource::millis_at(instant, seconds);
        let t = run_chapter_at(
            "spreadsheet-7.md",
            9,
            &[
                (0, at("2026-09-29T14:05Z", 0)),
                (4, at("2026-09-29T14:05Z", 20)),
                (5, at("2026-09-29T14:06Z", 5)),
            ],
        );
        // Listening is refused without the capability, and granted with it.
        assert!(
            t[0].contains("error: denied: capability does not grant `urn:cap:kernel:listen`"),
            "{}",
            t[0]
        );
        assert!(t[0].ends_with("heard nothing\n[uncacheable]\n"), "{}", t[0]);
        // One write, one cut, and the two values that read the feed.
        assert_eq!(
            t[2],
            "101.5\n[uncacheable]\nheard 1 cut\ncut urn:iki:tutorial:sheet:feed:acme\n  \
             invalidated urn:iki:tutorial:sheet:cell:A1\n  \
             invalidated urn:iki:tutorial:sheet:cell:A3\n  \
             invalidated urn:iki:tutorial:sheet:feed:acme\n[uncacheable]\n"
        );
        // A3's view carries the feed's thread; C1's does not.
        let (a3, c1) = t[3].split_at(t[3].find("urn:iki:tutorial:sheet:view:cell:C1\n").unwrap());
        assert!(a3.contains("  urn:iki:tutorial:sheet:feed:acme\n"), "{a3}");
        assert!(!c1.contains("feed:acme"), "{c1}");
        // NOW() has a deadline; the minute turns and nobody hears anything.
        assert!(t[4].contains("expires at 2026-09-29T14:06Z\n"), "{}", t[4]);
        assert_eq!(t[5], "heard nothing\n[uncacheable]\nnot cached\n");
        // Alice's write cuts the shared view of A3 (ledger item 581).
        assert!(
            t[7].contains("  invalidated urn:iki:tutorial:sheet:view:cell:A3\n"),
            "{}",
            t[7]
        );
        // A narrow listener hears the price its read rests on, and not B6.
        assert!(t[8].contains("heard 1 cut\ncut urn:iki:tutorial:sheet:feed:acme\n"));
        assert!(
            !t[8].contains("cut urn:iki:tutorial:sheet:input:B6"),
            "{}",
            t[8]
        );
    }

    /// The sheet's shim: a form's fields reach the kernel as named arguments, and the cost
    /// view the viewer shows names exactly what an edit cut.
    #[test]
    fn an_edit_with_args_writes_and_the_cost_view_names_what_it_cut() {
        let page = Page::new();
        let issue = |verb, target: &str, args: Vec<(&str, &str)>| {
            let args = args
                .into_iter()
                .map(|(n, v)| (n.to_string(), v.to_string()))
                .collect();
            futures::executor::block_on(page.issue_with_args(None, verb, target, args))
                .map(|r| String::from_utf8_lossy(&r.bytes).into_owned())
        };
        let edit = |cell: &str, typed: &str| {
            issue(
                Verb::Sink,
                spreadsheet::VIEW_EDIT,
                vec![("ref", cell), ("content", typed)],
            )
            .expect("an edit is answered")
        };
        let computed = || {
            let cost = issue(Verb::Source, spreadsheet::VIEW_COST, vec![]).expect("the cost");
            spreadsheet::CellRef::all()
                .filter(|cell| {
                    issue(
                        Verb::Source,
                        &format!("urn:iki:tutorial:sheet:view:cost:{cell}"),
                        vec![],
                    )
                    .expect("a cell's cost")
                    .contains("computed")
                })
                .map(|cell| cell.to_string())
                .collect::<Vec<_>>()
                .join(" ")
                + &format!(" ({})", cost.matches("computed").count())
        };
        assert_eq!(edit("a1", "5"), "A1 is now 5.");
        assert_eq!(edit("A2", "=A1*2"), "A2 is now =A1*2.");
        issue(Verb::Source, spreadsheet::VIEW_GRID, vec![]).expect("the grid");
        assert_eq!(computed(), " (0)");
        edit("A1", "6");
        assert_eq!(computed(), "A1 A2 (2)");
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
