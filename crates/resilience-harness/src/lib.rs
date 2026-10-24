//! **The resilience harness** — the driver the architecture book's "Spaces that wrap spaces"
//! runs in the page.
//!
//! A stub upstream the reader controls, behind the REAL `ikigai-throttle` overlays, on a clock
//! the reader advances, drawn by `ikigai-diagram` with the governors' state over it. A reader
//! sets the upstream failing and watches the circuit breaker trip, advances the clock and
//! watches it probe and close, and sees the retry spend its budget and the timeout fire,
//! against the governor code every ikigai host runs. Nothing is mocked but the upstream.
//!
//! ```text
//! sink   urn:iki:tutorial:harness:mode fail       ok | slow | fail | limited
//! source urn:iki:tutorial:harness:call            the governed call
//! source urn:iki:tutorial:harness:upstream        the same stub, ungoverned
//! sink   urn:iki:tutorial:harness:clock 5s        move the harness clock
//! source urn:throttle:state                       what the breaker holds, in words
//! source urn:diagram:state                        the arrangement, with that state drawn on it
//! ```
//!
//! ## The resource map, as built
//!
//! - **Atoms that hold state.** [`MODE`], the one thing the reader writes (a Sink of anything
//!   but the four modes is an invalid argument), and [`CLOCK`], the harness's time
//!   ([`time::HarnessClock`]), which the reader advances and a governor's wait moves.
//! - **Code, because its whole job is to produce typed failures.** [`UPSTREAM`], the stub: it
//!   reads [`MODE`] through the kernel and answers, waits past the timeout's budget on the
//!   harness timer, fails transiently, or answers `RetryAfter`. No composition can manufacture
//!   a typed failure. It numbers its calls, so an answer says whether a call reached it.
//! - **Code, because the arrangement vocabulary cannot declare an overlay.** [`CALL`], the
//!   stub behind `Retry(CircuitBreaker(Timeout(…)))`: the stack is built in Rust
//!   ([`root`]), because a declared arrangement has no form for a throttle governor today.
//! - **Code, because it is the platform seam.** [`time::VirtualTimer`], what `Timeout` races
//!   and `Retry` waits on: virtual time on the harness clock, not the browser's.
//! - **Composed, not written.** The breaker's state, `urn:throttle:state` and
//!   `urn:throttle:state:breaker` (throttle's `StateBoard`, mounted beside the stack); the
//!   picture, `urn:diagram:state` (ikigai-diagram over this host's own topology).
//!
//! Resource ratio: three endpoints written (mode, clock, upstream; the call is the upstream
//! endpoint bound at a second name, behind three published overlays built in code), and three
//! resources composed and used (`urn:throttle:state`, `urn:throttle:state:{governor}` and
//! `urn:diagram:state`; the diagram's other three doors are bound and unused).

pub mod time;

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use ikigai_core::{
    space_iri, ActionSpec, ArgRef, ArgSpec, AsyncFnEndpoint, Capability, Description, Endpoint,
    EndpointSpace, Error, Exact, FailureClass, Fallback, FnEndpoint, Invocation, InvokeFuture, Iri,
    Kernel, ReprType, Representation, Request, Result, Space, Verb,
};
use ikigai_throttle::{CircuitBreaker, Retry, StateBoard, Timeout};

use time::{span, HarnessClock, VirtualTimer};

// ANCHOR: names
/// The stub's mode: `ok`, `slow`, `fail` or `limited`. The only state the reader writes.
pub const MODE: &str = "urn:iki:tutorial:harness:mode";
/// The stub upstream, ungoverned.
pub const UPSTREAM: &str = "urn:iki:tutorial:harness:upstream";
/// The governed call: the stub behind `Retry(CircuitBreaker(Timeout(…)))`.
pub const CALL: &str = "urn:iki:tutorial:harness:call";
/// The harness clock: read it, or Sink a duration (`5s`, `500ms`) to move it forward.
pub const CLOCK: &str = "urn:iki:tutorial:harness:clock";
/// The name the circuit breaker is observed under: `urn:throttle:state:breaker`.
pub const GOVERNOR: &str = "breaker";
// ANCHOR_END: names

// ANCHOR: settings
/// How long `Timeout` lets a call run.
pub const TIMEOUT: Duration = Duration::from_secs(2);
/// How long the upstream takes in `slow` mode: past the timeout's budget.
pub const SLOW: Duration = Duration::from_secs(10);
/// Consecutive transient failures that trip the breaker open.
pub const THRESHOLD: u32 = 3;
/// How long an open circuit refuses before it lets one call through as a probe.
pub const COOLDOWN: Duration = Duration::from_secs(5);
/// The most invocations `Retry` makes of one call: more than the budget allows, so the
/// budget is what stops it.
pub const ATTEMPTS: u32 = 5;
/// The longest `RetryAfter` delay `Retry` waits out; a longer one is returned unchanged.
pub const MAX_WAIT: Duration = Duration::from_secs(2);
/// The delay the upstream asks for in `limited` mode.
pub const LIMITED_DELAY: Duration = Duration::from_secs(1);
/// Re-issues one run may spend in all, across every overlay: the run's retry budget.
pub const RETRY_BUDGET: u32 = 2;
// ANCHOR_END: settings

/// The four things the stub can be told to be.
pub const MODES: [&str; 4] = ["ok", "slow", "fail", "limited"];

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const TEXT_PLAIN: &str = "text/plain;charset=utf-8";

fn text(body: impl Into<String>) -> Representation {
    Representation::new(
        ReprType::new("text/plain").with_param("charset", "utf-8"),
        body.into().into_bytes(),
    )
}

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a constant IRI")
}

/// What the harness holds: the mode the reader set, and how many calls reached the stub.
#[derive(Debug)]
pub struct HarnessState {
    mode: std::sync::Mutex<&'static str>,
    calls: AtomicU64,
}

impl Default for HarnessState {
    fn default() -> Self {
        HarnessState {
            mode: std::sync::Mutex::new("ok"),
            calls: AtomicU64::new(0),
        }
    }
}

impl HarnessState {
    /// How many calls have reached the stub, governed or not.
    pub fn calls(&self) -> u64 {
        self.calls.load(Ordering::SeqCst)
    }
}

// ANCHOR: mode
/// `harness-mode`: the stub's mode. Source reads it; Sink sets it to one of [`MODES`] and
/// refuses anything else as an invalid argument. The read is cacheable and hangs from its
/// own name, which the kernel cuts after every write.
pub fn mode(state: Arc<HarnessState>) -> FnEndpoint {
    FnEndpoint::new("harness-mode", move |inv: &Invocation<'_>| {
        let mut mode = state.mode.lock().expect("mode lock");
        match inv.request.verb {
            Verb::Sink => {
                let asked = inv.inline_str("content")?.trim().to_string();
                let Some(word) = MODES.iter().find(|m| **m == asked) else {
                    return Err(Error::InvalidArgument {
                        name: "content".into(),
                        detail: format!("`{asked}` is not a mode: one of ok, slow, fail, limited"),
                    });
                };
                *mode = word;
                Ok(text(format!("the upstream is now {word}")))
            }
            _ => Ok(text(*mode).cacheable().depends_on(MODE)),
        }
    })
    .with_description(
        Description::new("harness-mode")
            .title("Harness upstream mode")
            .summary(
                "What the harness's stub upstream does when called: answer (ok), wait past the \
                 timeout (slow), fail transiently (fail), or ask to be retried after a delay \
                 (limited).",
            )
            .verb(Verb::Meta)
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("the upstream's current mode")
                    .output(TEXT_PLAIN),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("set the upstream's mode")
                    .fails(FailureClass::Permanent)
                    .input(
                        ArgSpec::new("content")
                            .summary("the mode")
                            .class(XSD_STRING)
                            .one_of(MODES),
                    )
                    .output(TEXT_PLAIN),
            ),
    )
}
// ANCHOR_END: mode

/// Read a duration as the clock's Sink takes it: `5s` or `500ms`.
fn duration(written: &str) -> Option<Duration> {
    let written = written.trim();
    if let Some(ms) = written.strip_suffix("ms") {
        return ms.parse().ok().map(Duration::from_millis);
    }
    written
        .strip_suffix('s')
        .and_then(|s| s.parse().ok())
        .map(Duration::from_secs)
}

// ANCHOR: clock
/// `harness-clock`: the harness's time, as how far it has moved since the harness started.
/// Sink a duration (`5s`, `500ms`) to move it forward; it never moves back. Never cached:
/// a governor's wait moves it too, and that is no write.
pub fn clock(clock: Arc<HarnessClock>) -> FnEndpoint {
    FnEndpoint::new("harness-clock", move |inv: &Invocation<'_>| {
        match inv.request.verb {
            Verb::Sink => {
                let written = inv.inline_str("content")?;
                let Some(by) = duration(written) else {
                    return Err(Error::InvalidArgument {
                        name: "content".into(),
                        detail: format!(
                            "`{}` is not a duration: write whole seconds (5s) or milliseconds (500ms)",
                            written.trim()
                        ),
                    });
                };
                clock.advance(by);
                Ok(text(format!(
                    "the harness clock moved {}: it reads {}",
                    span(by),
                    span(clock.elapsed())
                )))
            }
            _ => Ok(text(format!("the harness clock reads {}", span(clock.elapsed())))),
        }
    })
    .with_description(
        Description::new("harness-clock")
            .title("Harness clock")
            .summary(
                "The harness's own time, which moves only when the reader moves it or a \
                 governor waits: how far it has moved since the harness started.",
            )
            .verb(Verb::Meta)
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("how far the harness clock has moved")
                    .output(TEXT_PLAIN),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("move the harness clock forward")
                    .fails(FailureClass::Permanent)
                    .input(
                        ArgSpec::new("content")
                            .summary("how far: whole seconds (5s) or milliseconds (500ms)")
                            .class(XSD_STRING),
                    )
                    .output(TEXT_PLAIN),
            ),
    )
}
// ANCHOR_END: clock

// ANCHOR: upstream
/// `upstream`: the stub. It reads [`MODE`] through the kernel, numbers the call, and then
/// answers (`ok`), waits [`SLOW`] on the harness timer (`slow`), fails transiently
/// (`fail`, an `Unavailable`), or asks to be called again after [`LIMITED_DELAY`]
/// (`limited`, a `RetryAfter`). Never cached: every call is a call.
pub fn upstream(state: Arc<HarnessState>, timer: Arc<VirtualTimer>) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "upstream",
        move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            let state = Arc::clone(&state);
            let timer = Arc::clone(&timer);
            Box::pin(async move {
                let mode = inv.source(&iri(MODE)).await?;
                let n = state.calls.fetch_add(1, Ordering::SeqCst) + 1;
                match mode.bytes.as_slice() {
                    b"slow" => {
                        timer.wait(SLOW).await;
                        Ok(text(format!("ok, slowly (call {n}, after {})", span(SLOW))))
                    }
                    b"fail" => Err(Error::Unavailable(format!("upstream failed (call {n})"))),
                    b"limited" => Err(Error::RetryAfter {
                        delay: Some(LIMITED_DELAY),
                        detail: format!("upstream limited (call {n})"),
                    }),
                    _ => Ok(text(format!("ok (call {n})"))),
                }
            })
        },
    )
    .with_description(
        Description::new("upstream")
            .title("Harness stub upstream")
            .summary(
                "A stub dependency that does what its mode says: answers, waits past the \
                 timeout, fails transiently, or asks to be retried after a delay. Each answer \
                 numbers the call, so a refusal that never reached it is visible.",
            )
            .fails(FailureClass::Transient)
            .fails(FailureClass::RetryAfter)
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN),
    )
}
// ANCHOR_END: upstream

/// The harness: its kernel, its clock, its board, and what it holds.
pub struct Harness {
    kernel: Arc<Kernel>,
    clock: Arc<HarnessClock>,
    state: Arc<HarnessState>,
    board: StateBoard,
}

// ANCHOR: root
/// The harness's root: the governed call, the stub's own space, the board, and the
/// diagram, in that order.
///
/// The stack is the one the chapter draws, innermost first: `Timeout` bounds each invocation
/// of the upstream, `CircuitBreaker` counts the transient failures that come out of it
/// (a timeout among them), and `Retry` re-issues what is worth re-issuing from the run's
/// budget. Only the breaker has state worth showing (`Retry` and `Timeout` hold nothing past
/// one call), so it is the one observed on the board.
pub fn root(
    state: &Arc<HarnessState>,
    clock: &Arc<HarnessClock>,
    board: &StateBoard,
) -> Arc<dyn Space> {
    let timer = Arc::new(VirtualTimer::new(Arc::clone(clock)));
    let stub: Arc<dyn Endpoint> = Arc::new(upstream(Arc::clone(state), Arc::clone(&timer)));

    let governed = EndpointSpace::new()
        .bind_arc(Exact::new(CALL), Arc::clone(&stub))
        .named(space_iri("tutorial:harness:governed"));
    let timed = Timeout::new(governed, TIMEOUT).with_timer(timer.clone());
    let breaker = CircuitBreaker::new(timed, THRESHOLD, COOLDOWN)
        .with_clock(Arc::clone(clock) as _)
        .observed(board, GOVERNOR)
        .expect("one governor, named once");
    let retry = Retry::new(breaker, ATTEMPTS)
        .with_timer(timer)
        .with_max_wait(MAX_WAIT);

    let own = EndpointSpace::new()
        .bind(Exact::new(MODE), mode(Arc::clone(state)))
        .bind(Exact::new(CLOCK), self::clock(Arc::clone(clock)))
        .bind_arc(Exact::new(UPSTREAM), stub)
        .named(space_iri("tutorial:harness:upstream"));

    Arc::new(Fallback::new(vec![
        Arc::new(retry) as Arc<dyn Space>,
        Arc::new(own),
        Arc::new(board.space().named(space_iri("tutorial:harness:board"))),
        Arc::new(ikigai_diagram::space()),
    ]))
}
// ANCHOR_END: root

impl Harness {
    /// A harness as it starts: mode `ok`, the clock at zero, every circuit closed.
    pub fn new() -> Self {
        let state = Arc::new(HarnessState::default());
        let clock = Arc::new(HarnessClock::default());
        let board = StateBoard::new();
        // ANCHOR: kernel
        let kernel = Arc::new(
            Kernel::with_meta_renderer(
                root(&state, &clock, &board),
                Arc::new(ikigai_vocab::TurtleRenderer),
            )
            // The governors' clock and the kernel's are one, so a board answer cached
            // until the cooldown runs out expires when the harness clock passes it.
            .with_clock(Arc::clone(&clock) as _)
            .with_retry_budget(0, RETRY_BUDGET),
        );
        board.attach(&kernel);
        // ANCHOR_END: kernel
        Harness {
            kernel,
            clock,
            state,
            board,
        }
    }

    /// The harness's kernel.
    pub fn kernel(&self) -> &Arc<Kernel> {
        &self.kernel
    }

    /// The harness clock.
    pub fn clock(&self) -> &Arc<HarnessClock> {
        &self.clock
    }

    /// What the harness holds.
    pub fn state(&self) -> &Arc<HarnessState> {
        &self.state
    }

    /// The board the breaker is observed on.
    pub fn board(&self) -> &StateBoard {
        &self.board
    }

    /// Issue one request under root and read the answer as text.
    pub async fn ask(&self, verb: Verb, name: &str, content: Option<&str>) -> Result<String> {
        let mut request = Request::new(verb, iri(name));
        if let Some(content) = content {
            request = request.with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()));
        }
        let answer = self.kernel.issue(request, &Capability::root()).await?;
        Ok(String::from_utf8_lossy(&answer.bytes).into_owned())
    }

    /// What the board says: `urn:throttle:state`, in words.
    pub async fn words(&self) -> Result<String> {
        self.ask(Verb::Source, ikigai_throttle::STATE, None).await
    }

    /// The picture: `urn:diagram:state`, the harness's arrangement with the breaker's state
    /// drawn over it, as SVG.
    pub async fn picture(&self) -> Result<String> {
        self.ask(Verb::Source, ikigai_diagram::STATE, None).await
    }
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

/// The picture's text alternative: what it draws, in words. The state is the board's own
/// words, which a page says in its live region rather than in this `alt`, so the two are
/// never out of step.
pub const PICTURE_ALT: &str = "The resilience harness, drawn: the governed call \
    urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with \
    urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and \
    urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's \
    space. The circuit breaker's state is drawn in the governed space's box and said in words \
    in the status line below the picture.";
