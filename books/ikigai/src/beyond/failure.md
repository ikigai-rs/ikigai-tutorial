# Failure has a class

[Preferential resolution](preference.md) let one question decide a whole mount: could another
target fix this failure? That was the only use it made of a failure's type, and the type says
more. [A kernel behind a socket](socket.md) showed a failure arriving typed. This chapter is what
the type is for: **who may repeat a failure, when, and on whose authority.**

Nothing here starts a server that outlives its listing, so a socket or a REPL still open from
earlier in the book can stay open. Every listing below runs when the book is tested.

## Seven classes

Every error has exactly one class, and `Error::class()` is a pure function of the variant, so
nothing can disagree with itself. Each class is a retry rule:

| Class | What it says | Who may repeat it |
|---|---|---|
| `transient` | the same request may succeed now | an overlay holding retry budget, for an idempotent verb |
| `retry-after` | it may succeed, not before a stated time | an overlay holding budget and time; otherwise the caller |
| `over-budget` | this input exceeds a bound, and under that bound it fails the same | nobody in the chain; a holder of a larger bound, with a new request |
| `denied` | the caller's capability does not cover it | nobody; a different authority is a different request |
| `refused` | a remote authority refused the credential this endpoint presented | the holder of the credential, once, after renewing it |
| `permanent` | the request or the current state is wrong | nobody; a change of request or state clears it |
| `unknown` | the endpoint could not say | nobody automatically |

A client reads that table as a `match`. `FailureClass` is `#[non_exhaustive]`, so the compiler
insists on a wildcard arm, and the arm is the rule: a class you have never heard of repeats
nothing.

```rust
# extern crate ikigai_core;
# use ikigai_core::{Error, FailureClass};
{{#include ../../../../crates/two-hosts/src/failure.rs:who_may_repeat}}
# fn main() {
let refused = Error::Refused("api.example.com answered 401".into());
assert_eq!(refused.class(), FailureClass::Refused);
assert_eq!(
    who_may_repeat(refused.class()),
    "the holder of the credential, once, after renewing it"
);

let opaque = Error::Endpoint("boom".into());
assert_eq!(opaque.class().word(), "unknown");
assert_eq!(who_may_repeat(opaque.class()), "nobody automatically");
# }
```

Note `Endpoint`, the catch-all. It is `unknown`, which is why a prefer-mount will not fall through
on it. An endpoint that wants its failure retried has to say what kind of failure it was.

## `is_transient` is the older question

`Error::is_transient` predates the classes and still answers: `true` for `Timeout` and
`Unavailable`, `false` for everything else. Read beside `class`, one row stands out:

```rust
# extern crate ikigai_core;
use std::time::Duration;

use ikigai_core::{Error, FailureClass};

let limited = Error::RetryAfter {
    delay: Some(Duration::from_millis(1_500)),
    detail: "rate-limited: 5 requests per minute".into(),
};
assert_eq!(limited.class(), FailureClass::RetryAfter);
assert!(!limited.is_transient(), "may succeed later, and is not transient");

let down = Error::Unavailable("connection refused".into());
assert_eq!(down.class(), FailureClass::Transient);
assert!(down.is_transient());
```

That `false` is deliberate. An overlay written before `RetryAfter` existed gates on `is_transient`,
and it reads a rate limit as permanent, which is the safe mistake: re-issuing at once goes straight
back into the limit. New code reads `class`, because `is_transient` cannot say "yes, but not yet".

## Four variants the classes needed

`ikigai-core` 0.1.108 added four variants alongside the classes:

- **`RetryAfter { delay, detail }`** is the same request, later. `delay` is `None` when the producer
  knows only "not now". It is named for what the receiver does, so a rate limit says "rate-limited"
  in its `detail`.
- **`OverBudget { detail }`** is an input past a bound someone holds: a query past its time budget, a
  decompression past its size bound.
- **`Refused(detail)`** is an origin's 401 or 403. It is not `Denied`. `Denied` is the capability
  floor saying *you may not ask*. `Refused` is someone else's server saying no to this endpoint's
  credential, and the caller can do nothing about it.
- **`Unsupported { verb, detail }`** is a target that does not do this at all. `verb` says which kind:
  `Some(verb)` is "this target does not answer that verb", `None` is a facility the host did not
  provide, such as a clock. Its class is `permanent`, because a retry does not change a configuration.

The line worth remembering is the one between `Timeout` and `OverBudget`. `Timeout` is *a wait on
something else ran out*, and is worth repeating. `OverBudget` is *this computation ran past a bound
you hold*, and costs the same and fails the same on a repeat. Be careful with the store here: the
budget in [A query's authority is its dataset](datasets.md) still answers `Timeout` (checked at
`ikigai-store` 0.3.3), because core left the budgeted modules to move in their own releases. Until
it moves, a `Retry` overlay above a store will spend its budget re-running a query that ran out of
time.

## The classes at the edge

An HTTP door translates a failure into a status (`ikigai-web` 0.2.9). The status keeps the part of
the class a web client can act on, and is finer than the class in places:

| Variant | Class | Status |
|---|---|---|
| `Timeout` | `transient` | 504 |
| `Unavailable` | `transient` | 503 |
| `RetryAfter` | `retry-after` | 429, with `Retry-After` |
| `OverBudget` | `over-budget` | 422 |
| `Denied` | `denied` | 403 |
| `Refused` | `refused` | 502 |
| `Unsupported`, a verb | `permanent` | 405 |
| `Unsupported`, a facility | `permanent` | 501 |
| `NotFound`, `Unresolved` | `permanent` | 404 |
| `Conflict` | `permanent` | 409 |
| `MissingArgument`, `InvalidArgument` | `permanent` | 400 |
| `DepthExceeded` | `permanent` | 500 |
| `Endpoint` | `unknown` | 500 |

`Refused` is a 502 and not a 401 because the caller's credentials were never at issue: an upstream
turned this server away, and a status that asked the caller to sign in again would be wrong.

```rust
# extern crate ikigai_core;
# extern crate ikigai_web;
use std::time::Duration;

use ikigai_core::{Error, Verb};
use ikigai_web::{error_json, error_status, retry_after_seconds};

let limited = Error::RetryAfter {
    delay: Some(Duration::from_millis(1_500)),
    detail: "5 per minute".into(),
};
assert_eq!(error_status(&limited), 429);
// In seconds, rounded up: a delay rounded down tells the client to come back early.
assert_eq!(retry_after_seconds(&limited), Some(2));

assert_eq!(error_status(&Error::Timeout("slow".into())), 504);
assert_eq!(error_status(&Error::OverBudget { detail: "past 5000 ms".into() }), 422);
assert_eq!(error_status(&Error::Refused("401".into())), 502);

let read_only = Error::Unsupported { verb: Some(Verb::Sink), detail: "append-only".into() };
assert_eq!(error_status(&read_only), 405);
let no_clock = Error::Unsupported { verb: None, detail: "no clock".into() };
assert_eq!(error_status(&no_clock), 501);

// A door that asks for JSON bodies hands its client the class itself.
assert_eq!(
    error_json(&limited),
    r#"{"kind":"retry-after","class":"retry-after","message":"retry after 1500 ms: 5 per minute","retry_after":2}"#
);
```

## May the verb be repeated?

A class says whether a repeat could help. It does not say whether a repeat is safe. That belongs to
the verb, and [A kernel behind a socket](socket.md) ended by saying so:

```rust
# extern crate ikigai_core;
use ikigai_core::Verb;

let idempotent: Vec<Verb> = [Verb::Source, Verb::Sink, Verb::Exists, Verb::Delete, Verb::Meta]
    .into_iter()
    .filter(|verb| verb.is_idempotent())
    .collect();
assert_eq!(idempotent, [Verb::Source, Verb::Exists, Verb::Delete, Verb::Meta]);

// Idempotent is not the same as read-only.
assert!(Verb::Delete.is_idempotent() && Verb::Delete.is_mutating());
```

`Sink` is the one verb out, which is the line `a_prefer_mount_never_replays_a_sink` tests. `Retry` and
`Failover` ask both questions, and neither re-sends a `Sink`, whatever the class says.

## Saying how you can fail

An action may state the classes it fails with. Here is a feed whose origin can limit it or be
down:

```rust
# extern crate ikigai_core;
# use std::sync::atomic::{AtomicU32, Ordering};
# use std::sync::Arc;
# use ikigai_core::{Description, Error, FailureClass, FnEndpoint, Invocation, ReprType, Representation, Verb};
{{#include ../../../../crates/two-hosts/src/failure.rs:declared_feed}}
# fn main() {
#     let (endpoint, reads) = feed(0, || Error::Unavailable("down".into()));
#     let _ = (endpoint, reads.load(Ordering::SeqCst));
# }
```

`fails(..)` on a `Description` applies to every verb. For a multi-verb endpoint, each `ActionSpec`
has its own, and the declaration lives in `ActionSpec.failures`. Two classes come without being
stated: `permanent` for every action, since the kernel's own refusals can reach any of them, and
`denied` for one that `requires` a scope. Stating nothing is `Failures::Unstated`, which promises
nothing. It does not mean the action cannot fail.

```rust
# extern crate ikigai_core;
use ikigai_core::{ActionSpec, FailureClass, Failures, Verb};

let fetch = ActionSpec::new(Verb::Source)
    .requires("urn:cap:net:*")
    .fails(FailureClass::RetryAfter);
assert_eq!(fetch.failures.classes(), Some(&[FailureClass::RetryAfter][..]));
assert_eq!(fetch.admits_failure(FailureClass::RetryAfter), Some(true));
assert_eq!(fetch.admits_failure(FailureClass::Permanent), Some(true)); // implied
assert_eq!(fetch.admits_failure(FailureClass::Denied), Some(true)); // it requires a scope
assert_eq!(fetch.admits_failure(FailureClass::Unknown), Some(false));

// An action that states nothing has nothing to compare against.
let quiet = ActionSpec::new(Verb::Source);
assert_eq!(quiet.failures, Failures::Unstated);
assert_eq!(quiet.admits_failure(FailureClass::Unknown), None);
```

The declaration is **for planning, not for run time**. It appears in the contract as `ik:mayFail`,
so an agent reading the manifold can plan around a rate limit. No overlay reads it: an overlay
decides from the error it received. And the kernel never rewrites an error to fit a declaration,
because that would destroy what the caller needs to act on. When an action fails outside what it
stated, the kernel only notes it, and only while the resolution is traced:

```rust
# extern crate ikigai_core;
# extern crate two_hosts;
# extern crate futures;
use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use ikigai_core::{
    Capability, Error, Iri, Kernel, Request, TraceEvent, Tracer, Verb, FAILED_NOTE,
    UNDECLARED_FAILURE_NOTE,
};
use two_hosts::failure::{feed_space, FEED};

struct Collect(Mutex<Vec<TraceEvent>>);
impl Tracer for Collect {
    fn record(&self, event: TraceEvent) {
        self.0.lock().unwrap().push(event);
    }
}

// The feed from above declares `retry-after` and `transient`, and this time fails opaquely.
let (space, _reads) = feed_space(1, || Error::Endpoint("boom".into()));
let kernel = Kernel::new(Arc::new(space));
let tracer = Arc::new(Collect(Mutex::new(Vec::new())));
let read = Request::new(Verb::Source, Iri::parse(FEED).unwrap());

let error = block_on(kernel.issue_traced(read, &Capability::root(), tracer.clone())).unwrap_err();

// The error is the endpoint's, untouched…
assert_eq!(error, Error::Endpoint("boom".into()));
// …and the trace says the declaration did not admit it.
let events = tracer.0.lock().unwrap();
assert!(events[0].notes.contains(&(FAILED_NOTE.to_string(), "endpoint".to_string())));
assert!(events[0].notes.contains(&(UNDECLARED_FAILURE_NOTE.to_string(), "unknown".to_string())));
```

The note reads `undeclared-failure` with the class word as its value. The kernel is a sensor
here, not an enforcer.

## A retry budget is authority

Retry layers multiply. A `Retry` of three attempts inside another `Retry` of three attempts is nine
calls against a dependency that is already failing, and neither layer can see the other. So the
budget does not belong to a layer. It belongs to whoever issued the request, and one pool serves
the whole resolution tree:

```rust
# extern crate ikigai_core;
# extern crate ikigai_throttle;
# extern crate two_hosts;
# extern crate futures;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{Capability, Error, Iri, Kernel, Request, Verb};
use ikigai_throttle::Retry;
use two_hosts::failure::{feed_space, FEED};

let read = || Request::new(Verb::Source, Iri::parse(FEED).unwrap());
let down = || Error::Unavailable("the origin is down".into());

// Nothing configured: the layers multiply. This is every kernel that sets no budget.
let (space, reads) = feed_space(99, down);
let kernel = Kernel::new(Arc::new(Retry::new(Retry::new(space, 3), 3)));
assert!(block_on(kernel.issue(read(), &Capability::root())).is_err());
assert_eq!(reads.load(Ordering::SeqCst), 9);

// A base of 0 and a ceiling of 2: one first try and two re-issues, wherever they happen.
let (space, reads) = feed_space(99, down);
let kernel = Kernel::new(Arc::new(Retry::new(Retry::new(space, 3), 3))).with_retry_budget(0, 2);
assert!(block_on(kernel.issue(read(), &Capability::root())).is_err());
assert_eq!(reads.load(Ordering::SeqCst), 3);
```

How much of the pool a caller gets is set the way [A query's authority is its
dataset](datasets.md) set a store's time budget: through the capability. The grant is
`urn:cap:retry:<n>`, and the rule is the same as for `urn:cap:store:budget:<milliseconds>`. Root
gets the ceiling. A scoped capability gets the largest grant it holds, never below the base and
never above the ceiling.

```rust
# extern crate ikigai_core;
use ikigai_core::{cap_retry, Capability, RetryLimits};

let host = RetryLimits::new(0, 3);
assert_eq!(cap_retry(2), "urn:cap:retry:2");

assert_eq!(host.for_capability(&Capability::root()), 3);
assert_eq!(host.for_capability(&Capability::scoped(["urn:cap:read"])), 0);

// A door that mints its batch client two retries; a grant past the ceiling gets the ceiling.
let batch = Capability::scoped([cap_retry(2), "urn:cap:read".to_string()]);
assert_eq!(host.for_capability(&batch), 2);
assert_eq!(host.for_capability(&Capability::scoped([cap_retry(9)])), 3);

// Attenuating can only remove the grant, and that lowers the budget to the base.
assert_eq!(host.for_capability(&batch.attenuate(["urn:cap:read"])), 0);

// A smaller number is not an attenuation of a larger one: it is a scope the holder never held.
assert_eq!(host.for_capability(&batch.attenuate([cap_retry(1)])), 0);
```

So a caller cannot raise its own budget by attenuating, and the last line is the one to watch: to
hand a sub-request *one* of your two retries, narrow the run with `Invocation::with_retries(1)` (or
`IssueContext::with_retry_limit(1)` at the door). A narrowed view is a child pool, and a spend in the
child is a spend in every pool above it.

A failover's advance is a re-issue too. That turns "may this fall through?" into a question about
the caller, which [Preferential resolution](preference.md) did not ask. Here is the pair a prefer-mount is made
of, under two capabilities:

```rust
# extern crate ikigai_core;
# extern crate ikigai_throttle;
# extern crate two_hosts;
# extern crate futures;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{cap_retry, Capability, Error, FailureClass, Iri, Kernel, Request, Space, Verb};
use ikigai_throttle::Failover;
use two_hosts::failure::{feed_space, FEED};

let ask = |capability: &Capability| {
    let down = || Error::Unavailable("the peer is down".into());
    let (peer, _) = feed_space(99, down);
    let (local, _) = feed_space(0, down);
    let pair = Failover::new(vec![Arc::new(peer) as Arc<dyn Space>, Arc::new(local)]);
    let kernel = Kernel::new(Arc::new(pair)).with_retry_budget(0, 3);
    let read = Request::new(Verb::Source, Iri::parse(FEED).unwrap());
    block_on(kernel.issue(read, capability))
};

let holder = ask(&Capability::scoped([cap_retry(1)])).unwrap();
assert_eq!(holder.bytes, b"fresh".to_vec());

let error = ask(&Capability::scoped(["urn:cap:read"])).unwrap_err();
assert_eq!(error.class(), FailureClass::Transient);
```

A host that wants its callers to own every retry sets a base of 0 and grants nothing. A kernel with
no budget configured has no pool at all, `Invocation::retry_budget` answers `None`, and each overlay
falls back to its own `attempts`, exactly as before the budget existed.

## A run has a deadline

Beside the count, a run can carry a time. A door mints an `IssueContext`, gives it a `not_after` on
the kernel clock, and issues under it. Every sub-request inherits the deadline, and any of them may
shorten it. None may extend it.

```rust
# extern crate ikigai_core;
# extern crate two_hosts;
# extern crate futures;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    Capability, EndpointSpace, Exact, FixedClock, Iri, IssueContext, Kernel, Request, Time, Verb,
};
use two_hosts::failure::{parse_run_report, run_report, RUN_REPORT};

// A later deadline does not extend an earlier one.
let run = IssueContext::new()
    .with_not_after(Time::from_millis(1_250))
    .with_not_after(Time::from_millis(9_000));
assert_eq!(run.not_after(), Some(Time::from_millis(1_250)));

// An endpoint reads what is left with `Invocation::time_left`.
let space = EndpointSpace::new().bind(Exact::new(RUN_REPORT), run_report());
let kernel = Kernel::new(Arc::new(space)).with_clock(Arc::new(FixedClock::at(1_000)));
let read = Request::new(Verb::Source, Iri::parse(RUN_REPORT).unwrap());
let answer = block_on(kernel.issue_with_context(read, &Capability::root(), run)).unwrap();
assert_eq!(parse_run_report(&answer.bytes), (None, Some(250)));
```

A retry overlay is what reads it first. Given a `RetryAfter` with a delay, `Retry` waits and
re-issues only when the delay fits what the run has left. One that does not fit comes straight
back, so the caller, who owns the time, decides. With no deadline and no `with_max_wait` of its
own, the overlay has no time to spend, and returns every stated delay unchanged.

```rust
# extern crate ikigai_core;
# extern crate ikigai_throttle;
# extern crate two_hosts;
# extern crate futures;
# extern crate async_trait;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use futures::executor::block_on;
use ikigai_core::{Capability, Error, FixedClock, Iri, IssueContext, Kernel, Request, Time, Verb};
use ikigai_throttle::{Retry, Timer};
use two_hosts::failure::{feed_space, FEED};

/// Records what it was asked to wait, and returns at once.
#[derive(Default)]
struct Recording(Mutex<Vec<Duration>>);

#[async_trait::async_trait]
impl Timer for Recording {
    async fn sleep(&self, delay: Duration) {
        self.0.lock().unwrap().push(delay);
    }
}

// The feed is limited for 100 ms, once; the clock says 1000, the run ends at `not_after`.
let attempt = |not_after: u64| {
    let (space, _) = feed_space(1, || Error::RetryAfter {
        delay: Some(Duration::from_millis(100)),
        detail: "limited".into(),
    });
    let timer = Arc::new(Recording::default());
    let kernel = Kernel::new(Arc::new(Retry::new(space, 2).with_timer(timer.clone())))
        .with_clock(Arc::new(FixedClock::at(1_000)));
    let run = IssueContext::new().with_not_after(Time::from_millis(not_after));
    let read = Request::new(Verb::Source, Iri::parse(FEED).unwrap());
    let answer = block_on(kernel.issue_with_context(read, &Capability::root(), run));
    let waited = timer.0.lock().unwrap().clone();
    (answer, waited)
};

// 250 ms left: the wait fits.
let (answer, waited) = attempt(1_250);
assert_eq!(answer.unwrap().bytes, b"fresh".to_vec());
assert_eq!(waited, [Duration::from_millis(100)]);

// 50 ms left: it does not, nothing sleeps, and the delay reaches the caller.
let (answer, waited) = attempt(1_050);
assert_eq!(answer.unwrap_err().retry_after_millis(), Some(100));
assert!(waited.is_empty());
```

The kernel does not enforce a deadline. It tells the run how long it has, and an endpoint or an
overlay bounds its own work by that. A computation that ignores it keeps running, which is the
limit [Where this actually stands](../modules/status.md) describes as "abandoned, never stopped".

## Crossing a mount

None of this would mean much if it stopped at the first socket. Since wire v9 (`ikigai-cli` 0.1.66;
this book's `ikigai-ipc` 0.2.1 speaks v10, which adds streaming and leaves this alone), a mount
forwards what is left of its run. Here is the far side:

```rust
# extern crate ikigai_core;
# extern crate two_hosts;
# use std::sync::Arc;
# use ikigai_core::{EndpointSpace, Exact, FnEndpoint, Invocation, Kernel, ReprType, Representation, SystemClock};
# use two_hosts::failure::RUN_REPORT;
{{#include ../../../../crates/two-hosts/src/failure.rs:run_report}}
# fn main() { let _ = run_report_kernel(); }
```

And a door on the near side, issuing through a mount over a real socket:

```rust
# extern crate ikigai_core;
# extern crate ikigai_ipc;
# extern crate ikigai_resolve;
# extern crate two_hosts;
# extern crate futures;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{Capability, Clock, Iri, IssueContext, Kernel, Request, SystemClock, Verb};
use ikigai_resolve::{MountedRemote, Resolver};
use two_hosts::failure::{dial_when_up, parse_run_report, run_report_kernel, RUN_REPORT};

let path = std::env::temp_dir().join(format!("ikbook-run-{}.sock", std::process::id()));
let _ = std::fs::remove_file(&path);
let server = path.clone();
std::thread::spawn(move || ikigai_ipc::serve(run_report_kernel(), &server));

let peer: Arc<dyn Resolver> = Arc::new(dial_when_up(&path));
let mount = MountedRemote::overriding(peer, RUN_REPORT, "unix:peer");
let here = Kernel::new(Arc::new(mount))
    .with_retry_budget(0, 9)
    .with_clock(Arc::new(SystemClock));
let read = || Request::new(Verb::Source, Iri::parse(RUN_REPORT).unwrap());

// Thirty seconds and eight retries here. The peer's ceiling is three.
let run = IssueContext::new()
    .with_not_after(SystemClock.now().plus_millis(30_000))
    .with_retry_limit(8);
let answer = block_on(here.issue_with_context(read(), &Capability::root(), run)).unwrap();
let (retries, left) = parse_run_report(&answer.bytes);
assert_eq!(retries, Some(3));
assert!((20_000..=30_000).contains(&left.unwrap()));

// A run narrower than the peer's ceiling stays narrow there, and no deadline stays none.
let run = IssueContext::new().with_retry_limit(1);
let answer = block_on(here.issue_with_context(read(), &Capability::root(), run)).unwrap();
assert_eq!(parse_run_report(&answer.bytes), (Some(1), None));
# let _ = std::fs::remove_file(&path);
```

Two choices in the wire format are worth reading twice.

**The deadline travels as time left, never as an instant.** Two machines' clocks differ, and an
instant would be early on one and late on the other, whereas thirty seconds is the same length on
both. The far side adds what it received to its own `now`.

**The context narrows and cannot widen.** The server rebuilds the run from what the call carried,
then narrows it to what the caller's clamped capability is entitled to under its own policy. That is
the `3` above. A peer can spend less than you offered. A request cannot talk a peer into more, which
is why this rides beside the request instead of in it: a `Request` is half the cache key, and it
crosses a wire as data a client can forge.

A peer older than v9 never sees the context, and gets today's plain call. The new variants reach it
downgraded to the `Endpoint` it has always received, and that is `unknown`: it repeats nothing.

Between two v9 peers the classes themselves cross typed, delay included, so the rule from
[Preferential resolution](preference.md) holds over a real socket: the tests
`a_prefer_mount_decides_by_a_class_that_crossed_a_socket` and `a_stated_delay_survives_the_wire` in
`crates/two-hosts` serve a peer that answers `RetryAfter` or `Refused`. The first falls through, the
second comes back as itself, and a `RetryAfter` read directly from the socket still has its delay.

## What this does not do

**It does not make anything retry.** Classes, budgets and deadlines are what an overlay reads. A
kernel with no `Retry` or `Failover` in its arrangement repeats nothing, whatever the class says.

**A declaration is not a guarantee.** `fails(..)` says what an action expects. A trace note is
all that happens when it is wrong. Decide from the error you received.

**A budget bounds re-issues, not work.** It counts how many failed requests may be sent again. It
says nothing about how long each one takes, which is what the deadline is for, and the deadline in
turn only tells the run. Nothing here is a stop button.
