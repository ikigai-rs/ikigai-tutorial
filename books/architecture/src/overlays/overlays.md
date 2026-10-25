# Spaces that wrap spaces

The capability spine decided *who may* reach a resource. This chapter is about what happens to a
request *on the way* — the governors that make a resource resilient and bounded: rate limits,
retries, circuit breakers, failover, timeouts, and concurrency caps. In ikigai these are not
features baked into each endpoint. They are **overlays**: a space wraps another space and changes
how requests pass through it, and because a space can wrap a space, they **stack**.

This builds directly on dynamic spaces from The ikigai Book. An overlay is a space whose job is
not to answer but to *govern* the space it wraps, so a wrapped resource answers exactly as before
until a governor has something to say — at which point what it says is a refusal.

```text
   request
     │
     ▼
  ┌──────────────────────────────────────────────────┐
  │ Retry (idempotent verbs, spends a budget)        │
  │  ┌─────────────────────────────────────────────┐ │
  │  │ CircuitBreaker (open / closed / half-open)  │ │
  │  │  ┌────────────────────────────────────────┐ │ │
  │  │  │ Timeout (a deadline on the call)       │ │ │
  │  │  │  ┌───────────────────────────────────┐ │ │ │
  │  │  │  │  the space being guarded          │ │ │ │
  │  │  │  │  (a door, a mount, an endpoint)   │ │ │ │
  │  │  │  └───────────────────────────────────┘ │ │ │
  │  │  └────────────────────────────────────────┘ │ │
  │  └─────────────────────────────────────────────┘ │
  └──────────────────────────────────────────────────┘
```

## The three facts every overlay reads

The overlays share a vocabulary. Each one makes its decision from the same three facts about a
request, and no more:

- **Is the verb idempotent?** A Source or an Exists can be safely re-issued; a Sink that is not
  idempotent cannot. Retry and Failover refuse to re-run what is unsafe to re-run.
- **What class is the failure?** A failure is *transient* (worth another try or a different target)
  or *permanent* (do not bother). A `429`-style "try again after" carries its own delay.
- **What is left of the retry budget?** A single counter shared across the whole resolution tree.

That third fact is the one security hook in the resilience family, and it is worth dwelling on.
Nested retries *multiply*: a Retry wrapping a space that itself wraps a Retry could turn one request
into many. A shared **retry budget** caps the *total* number of re-issues across the entire tree,
so no arrangement of nested governors can amplify one request into a storm. It is a blast-radius
bound, expressed as authority — you hold a budget, and when it is spent, re-issues stop.

## The two faces, overlay by overlay

Here is the split that makes this chapter part of the book rather than an appendix. Every governor
has a security reading and a UX reading, and the verdict is which one it is *primarily* for:

| overlay | security face | UX face | primarily |
|---|---|---|---|
| **RateLimit** | abuse / denial-of-service control; politeness to a published rate | a "try again after" lets a client back off instead of hammering | **both** |
| **Retry** | only the shared budget, capping amplification | hides a transient blip; fewer visible failures | **UX** |
| **CircuitBreaker** | blast-radius containment: stop feeding a dead dependency; recover deliberately | fail **fast** instead of hanging the person on a dead backend | **both** |
| **Failover** | — (availability, not a security control) | continuity: move to a backup | **UX** |
| **Timeout** | a resource-exhaustion bound: a worker is never held forever | never hang; drive the loading / "did not answer" state | **both** |
| **Throttle / Bulkhead** | isolation: one tenant or hot prefix cannot starve the pool | backpressure — park the excess, never an error | **both** |

The pattern in the table is the pattern of the whole book. A refusal that protects *the dependency
or the host* (RateLimit, CircuitBreaker, Timeout, Throttle) has a security face; a re-issue that
protects *the caller's experience* (Retry, Failover) is UX-facing, with the budget its only security
hook. Same mechanism, read by what the refusal is *for*.

## Transparency: a governor you can see through

One property makes all of this drawable, and it is worth naming because it is unusual. The overlays
are **transparent to structure**: a governor reports the topology, the identity, and the cache
identity of the space it wraps. So `Retry(CircuitBreaker(Timeout(space)))` still reports the doors
underneath it; the system's own introspection sees *through* the stack to what is guarded. What a
governor *adds* is only ever a refusal — and the kernel never caches a refusal — so the arrangement
can be harvested from the topology while the governor's **state** (a breaker open or closed, a
budget's remaining count) is a separate thing drawn on top.

## The governor as a constraint made explicit

Step back and the whole chain has one organizing idea, borrowed from the theory of constraints
(Goldratt): a system's throughput is set by its bottleneck, and the useful moves are to **exploit**
the bottleneck, **subordinate** everything else to it, and make the constraint **explicit and
enforceable**. The overlays are the governance half of that: a rate, a timeout, a concurrency cap
is a constraint written down as a resource and enforced by the kernel, so "do not exceed this"
stops being a hope in a code review and becomes a refusal at resolution. Non-conforming behavior
simply fails to resolve. (The *throughput* half of the same idea — cache and golden thread as ways
to exploit the constraint — is the next chapter.)

## Drive the overlays yourself

The box diagram above is a picture. Below it runs: a **resilience harness** in this page, the
real `ikigai-throttle` governors wrapped around a stub dependency you control, so you can make
the dependency fail, stall or push back and watch what each governor does about it. Nothing is
mocked except the dependency. The breaker that trips here is the `CircuitBreaker` every ikigai
host runs, and the timeout is its `Timeout`.

The harness is a kernel of its own (`crates/resilience-harness`), started the first time you
press Run in a cell below that says "in the resilience harness", or a button beside the
picture. It binds a handful of names:

- `urn:iki:tutorial:harness:mode`, the one thing you write: what the stub does when called.
  `ok` answers; `slow` takes 10 seconds, past the timeout's budget; `fail` fails transiently
  (`unavailable`); `limited` asks to be called again after a second (`retry after`). Anything
  else is refused as an invalid argument.
- `urn:iki:tutorial:harness:upstream`, the stub itself, ungoverned. It numbers every call it
  receives, so an answer tells you whether a call reached it.
- `urn:iki:tutorial:harness:call`, the same stub behind the stack this chapter draws:
  `Retry(CircuitBreaker(Timeout(upstream)))`.
- `urn:iki:tutorial:harness:clock`, the harness's time. It moves only when you move it (Sink
  a duration such as `5s`) or when a governor waits.
- `urn:throttle:state`, the governors' state in words, and `urn:diagram:state`, the
  harness's arrangement with that state drawn over it: the picture below.

The stack is built like this, innermost first:

```rust,ignore
{{#include ../../../../crates/resilience-harness/src/lib.rs:root}}
```

with these settings: a call may run 2 seconds; the breaker trips after 3 consecutive transient
failures and stays open for 5 seconds; `Retry` waits out a `retry after` of at most 2 seconds;
and the kernel gives every run a retry budget of 2 re-issues in all, whatever `Retry`'s own
attempts would allow:

```rust,ignore
{{#include ../../../../crates/resilience-harness/src/lib.rs:settings}}
```

**Why the harness has its own clock.** A breaker's cooldown, a timeout's budget and a retry's
wait are all time, and on the wall clock this page would answer differently for every reader
and for every run of the test that replays it. So the harness clock moves only when you say
("wait 5 seconds") or when a governor waits, and a governor's wait is *virtual*: the earliest
wait pending finishes first and moves the clock to its deadline, in no real time. A timeout
racing a 10-second call against its 2-second budget fires at 2 seconds, exactly, on every page,
and the call it abandoned never finishes. That makes every answer below the same for you as for
the chapter.

**What the picture draws, and what it cannot.** Only the circuit breaker appears in the picture
as a governor. That is the transparency property above at work: `Retry` and `Timeout` hold
nothing past a single call (an attempt count, a deadline), so they have no state to draw, and
they report the topology of the space they wrap, so the picture sees through them to the door
they govern. The breaker's state is drawn in that space's box, the first one under the root,
and said in words in the box below the picture. The rest of the picture is the harness's other
doors, and they do not change.

### The walkthrough, in stages

The walkthrough below is a sequence of **stages**. A stage is the harness after a step that
changed its picture, and each one is a numbered tab. Read a tab from the top: the step that makes
the stage, as cells you run, then what that step did: which step made it, what changed since the
stage before (the board's words, compared line by line with the stage before's: the words that
differ are marked, and the line they replaced is said beside them), the facts at that stage, the
picture as it was, and the board's words in full. A step that changes nothing the picture shows
makes no stage, and the tab it sits in says "no change" instead. Six stages, from every circuit
closed to a circuit half-open again. (The tabs are the books' stages viewer, `js/stages.js`, which
knows nothing about governors: the page names the line that draws the picture and the line that
says it in words, and the viewer does the rest.)

**Watch the circuit, not the upstream.** The picture draws the governors, not the stub they
govern. Setting the stub's mode changes nothing in the picture: the breaker learns of a mode
only when a call goes through it and fails. The line above the tabs keeps the three facts that
change current, each read from the name that holds it after every step: the upstream's mode
(`urn:iki:tutorial:harness:mode`), the circuit (the board, `urn:throttle:state`) and the harness
time (`urn:iki:tutorial:harness:clock`).

**The buttons are yours.** They drive the same harness the cells do. A press that changes the picture
adds a stage after the chapter's, numbered on, and opens it, so you can flip
between any two states the harness has been in.

**Why flipping back costs nothing.** A stage's picture is `urn:diagram:state`, a pure composition
of two resources: the harness's topology and the governors' board. It is cacheable, and it hangs
from what it read: the board hangs from the breaker's golden thread,
`urn:throttle:governor:breaker`, which the breaker cuts on every change its board shows (a
failure counted, a circuit tripped or closed). A cooldown running out cuts nothing; the board's
answer is cached only until it would run out, on the harness clock. So a stage's picture stays
true until the state moves, and a tab is drawn once, when its stage happens, and then only
shown: flipping back asks the kernel nothing. A step that changes nothing cuts nothing, so the
page's read of the picture after it is a cache hit, and the page says so; stage 4 shows one in a
trace. The chapter's six pictures are files the same harness drew natively, held to it byte for
byte by a test, so they show before the page's kernel loads, without it, and in print; a stage
you make is drawn by the kernel in this page when it happens. Every picture is kept in both the
light and the dark colors (`scheme=light`, `scheme=dark`), and the page shows the one the book's
theme needs.

<div class="ikigai-stages" data-chain='harness:overlays'
     data-picture='source urn:diagram:state scheme={scheme}'
     data-words='source urn:throttle:state'
     data-tabs-label='Stages of the harness' data-controls-label='Drive the harness'
     data-alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture.">
<ul class="ikigai-stages-controls" hidden>
<li data-cmd='sink urn:iki:tutorial:harness:mode ok'>Set the upstream to ok</li>
<li data-cmd='sink urn:iki:tutorial:harness:mode slow'>Set the upstream to slow</li>
<li data-cmd='sink urn:iki:tutorial:harness:mode fail'>Set the upstream to fail</li>
<li data-cmd='sink urn:iki:tutorial:harness:mode limited'>Set the upstream to limited</li>
<li data-cmd='source urn:iki:tutorial:harness:call'>Call once</li>
<li data-cmd='source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call
source urn:iki:tutorial:harness:call'>Call ten times</li>
<li data-cmd='sink urn:iki:tutorial:harness:clock 5s'>Wait 5 seconds</li>
</ul>
<ul class="ikigai-stages-facts">
<li data-cmd='source urn:iki:tutorial:harness:mode' data-label='upstream'>upstream: ok</li>
<li data-cmd='source urn:throttle:state' data-line='3' data-none='no circuit has failed yet'>no circuit has failed yet</li>
<li data-cmd='source urn:iki:tutorial:harness:clock'>the harness clock reads 0s</li>
</ul>
<section class="ikigai-stage" data-label="closed">

#### Stage 1: the start

The harness as it starts. No call has been made, so no circuit has failed, and the breaker holds
nothing but its settings.

The cells in the stages assume you run them in order on a fresh page. If you have pressed the
buttons first, the cells answer from where you left the harness and say they differ from the
chapter, and a cell that then changes something adds a stage of yours. Reloading the page starts
the harness over.

<div class="ikigai-stage-result">
<p class="ikigai-stage-facts">upstream: ok · no circuit has failed yet · the harness clock reads 0s</p>
<p class="ikigai-stage-picture"><a class="ikigai-scheme-light" href="harness-stage-1-light.svg"><img src="harness-stage-1-light.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s."></a><a class="ikigai-scheme-dark" href="harness-stage-1-dark.svg"><img src="harness-stage-1-dark.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s."></a></p>
<pre class="ikigai-stage-words">throttle state: 1 governor visible
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s</pre>
</div>

</section>
<section class="ikigai-stage" data-label="open">

#### Stage 2: the breaker trips

Set the stub failing. This changes nothing in the picture: the mode is the stub's.

<div class="ikigai-run" data-harness='overlays' data-cmd='sink urn:iki:tutorial:harness:mode fail'>
<pre class="ikigai-run-expected">the upstream is now fail
[uncacheable]</pre>
</div>

Make one governed call. It fails, but look at the call number:

<div class="ikigai-run" data-harness='overlays' data-cmd='source urn:iki:tutorial:harness:call'>
<pre class="ikigai-run-expected">error: unavailable: upstream failed (call 3)</pre>
</div>

One call from you was three calls to the stub. `unavailable` is transient, so `Retry` re-issued
it, twice, until the run's budget of 2 re-issues was spent; and the breaker below it counted
every one of the three failures. Three is its threshold, so the circuit is now open:

<div class="ikigai-stage-result">
<p class="ikigai-stage-facts">upstream: fail · circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures · the harness clock reads 0s</p>
<p class="ikigai-stage-picture"><a class="ikigai-scheme-light" href="harness-stage-2-light.svg"><img src="harness-stage-2-light.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures."></a><a class="ikigai-scheme-dark" href="harness-stage-2-dark.svg"><img src="harness-stage-2-dark.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures."></a></p>
<pre class="ikigai-stage-words">throttle state: 1 governor visible
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures</pre>
</div>

The board says so, in words:

<div class="ikigai-run" data-harness='overlays' data-cmd='source urn:throttle:state:breaker'>
<pre class="ikigai-run-expected">breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures
[computed]</pre>
</div>

Call again:

<div class="ikigai-run" data-harness='overlays' data-cmd='source urn:iki:tutorial:harness:call'>
<pre class="ikigai-run-expected">error: retry after 5000 ms: circuit open for `urn:iki:tutorial:harness:call`</pre>
</div>

No call number: this one never reached the stub. An open circuit fails fast, and it says when to
come back (the cooldown left) as a `retry after`, the same class a `429` is. `Retry` saw that
class too, and returned it unchanged, because 5 seconds is longer than it is allowed to wait.
**No change**: a refusal is not a failure the breaker counts, so nothing it shows moved, and this
step makes no stage.

</section>
<section class="ikigai-stage" data-label="half-open">

#### Stage 3: the probe

Wait out the cooldown. The circuit is now half-open: the next call is let through as a probe.

<div class="ikigai-run" data-harness='overlays' data-cmd='sink urn:iki:tutorial:harness:clock 5s
source urn:throttle:state:breaker'>
<pre class="ikigai-run-expected">the harness clock moved 5s: it reads 5s
[uncacheable]
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures
[computed]</pre>
</div>

Nothing wrote the board between the last two reads of it, and the second was still computed
afresh: the board's answer was cached only until the cooldown would run out, measured on the
same clock, so moving the clock past it was enough. Time alone made this stage:

<div class="ikigai-stage-result">
<p class="ikigai-stage-facts">upstream: fail · circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures · the harness clock reads 5s</p>
<p class="ikigai-stage-picture"><a class="ikigai-scheme-light" href="harness-stage-3-light.svg"><img src="harness-stage-3-light.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures."></a><a class="ikigai-scheme-dark" href="harness-stage-3-dark.svg"><img src="harness-stage-3-dark.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures."></a></p>
<pre class="ikigai-stage-words">throttle state: 1 governor visible
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures</pre>
</div>

</section>
<section class="ikigai-stage" data-label="closed">

#### Stage 4: the probe closes the circuit

Mend the stub and make the probe call:

<div class="ikigai-run" data-harness='overlays' data-cmd='sink urn:iki:tutorial:harness:mode ok
source urn:iki:tutorial:harness:call
source urn:throttle:state:breaker'>
<pre class="ikigai-run-expected">the upstream is now ok
[uncacheable]
ok (call 4)
[uncacheable]
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: closed, 0 consecutive transient failures
[computed]</pre>
</div>

Call 4: the probe reached the stub, it answered, and the circuit closed. Had the stub still been
failing, the probe's failure would have opened the circuit again for another cooldown.

<div class="ikigai-stage-result">
<p class="ikigai-stage-facts">upstream: ok · circuit `urn:iki:tutorial:harness:call`: closed, 0 consecutive transient failures · the harness clock reads 5s</p>
<p class="ikigai-stage-picture"><a class="ikigai-scheme-light" href="harness-stage-4-light.svg"><img src="harness-stage-4-light.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: closed, 0 consecutive transient failures."></a><a class="ikigai-scheme-dark" href="harness-stage-4-dark.svg"><img src="harness-stage-4-dark.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: closed, 0 consecutive transient failures."></a></p>
<pre class="ikigai-stage-words">throttle state: 1 governor visible
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: closed, 0 consecutive transient failures</pre>
</div>

##### No change: the retry spends its budget

Now make the stub push back: `limited` answers `retry after 1000 ms`. This time trace the call,
so you can see inside it:

<div class="ikigai-run" data-harness='overlays' data-cmd='sink urn:iki:tutorial:harness:mode limited
trace urn:iki:tutorial:harness:call'>
<pre class="ikigai-run-expected">the upstream is now limited
[uncacheable]
trace  urn:iki:tutorial:harness:call
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:harness:call   upstream · failed · … · 2000ms · failed=retry-after answered-by=urn:iki:space:tutorial:harness:governed
├─ urn:iki:tutorial:harness:mode   harness-mode · computed · … · 0ms · answered-by=urn:iki:space:tutorial:harness:upstream
├─ urn:iki:tutorial:harness:mode   harness-mode · cached · … · 0ms · answered-by=urn:iki:space:tutorial:harness:upstream
└─ urn:iki:tutorial:harness:mode   harness-mode · cached · … · 0ms · answered-by=urn:iki:space:tutorial:harness:upstream
urn:iki:tutorial:harness:call   error: retry after 1000 ms: upstream limited (call 7)</pre>
</div>

Read it from the inside out. The stub reads its mode once per call, and there are three reads
under the one call you made: three attempts, calls 5, 6 and 7. (The first read is computed
because setting the mode a line earlier cut the mode's thread; the other two are served from the
cache.) A one-second delay fits inside `Retry`'s two-second bound, so it waited each one out
before re-issuing, and the call took 2000 ms of harness time: two waits, two re-issues, and then
the budget of 2 was spent and the third `retry after` came back to you. `Retry` would have made
5 attempts; the run's budget is what stopped it. And the breaker, which a `retry after` says
nothing to, has not moved:

<div class="ikigai-run" data-harness='overlays' data-cmd='source urn:iki:tutorial:harness:clock
source urn:throttle:state:breaker
trace urn:diagram:state scheme=light'>
<pre class="ikigai-run-expected">the harness clock reads 7s
[uncacheable]
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: closed, 0 consecutive transient failures
[cached]
trace  urn:diagram:state
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:diagram:state   diagram-state · cached · … · 0ms · answered-by=urn:iki:space:diagram   → …</pre>
</div>

The clock moved the two seconds the retry waited. The board is the same answer as before, served
from the cache: nothing a governor shows changed, so nothing cut its thread. **No change**, then,
and the trace of the picture says the same thing from the other side: `urn:diagram:state` was
served from the cache, because the picture this stage shows was drawn when the stage happened,
and nothing since has cut a thread it hangs from.

</section>
<section class="ikigai-stage" data-label="open">

#### Stage 5: the timeout

Make the stub slow: it now takes 10 seconds, and the call's budget is 2.

<div class="ikigai-run" data-harness='overlays' data-cmd='sink urn:iki:tutorial:harness:mode slow
source urn:iki:tutorial:harness:call'>
<pre class="ikigai-run-expected">the upstream is now slow
[uncacheable]
error: timeout: `urn:iki:tutorial:harness:call` exceeded 2000ms</pre>
</div>

The timeout names the target it bounded, and its class is transient, because the overlay cannot
tell a slow dependency (which a repeat may beat) from expensive work (which it will not). So the
two governors above it treated it like any other transient failure, and the circuit opened:

<div class="ikigai-stage-result">
<p class="ikigai-stage-facts">upstream: slow · circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures · the harness clock reads 13s</p>
<p class="ikigai-stage-picture"><a class="ikigai-scheme-light" href="harness-stage-5-light.svg"><img src="harness-stage-5-light.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures."></a><a class="ikigai-scheme-dark" href="harness-stage-5-dark.svg"><img src="harness-stage-5-dark.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures."></a></p>
<pre class="ikigai-stage-words">throttle state: 1 governor visible
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures</pre>
</div>

The picture is the same as stage 2's, because the state is: the breaker cannot tell a stalled
dependency from a failing one, and does not need to. The clock and the board say how it got
there:

<div class="ikigai-run" data-harness='overlays' data-cmd='source urn:iki:tutorial:harness:clock
source urn:throttle:state:breaker'>
<pre class="ikigai-run-expected">the harness clock reads 13s
[uncacheable]
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: open, 3 consecutive transient failures
[computed]</pre>
</div>

Six seconds: `Retry` re-issued twice from the budget, each attempt was cut off at its 2 seconds,
and the breaker counted three transient failures and opened. A stalled dependency trips the
breaker exactly as a failing one does, which is the point of putting the timeout *inside* it.

</section>
<section class="ikigai-stage" data-label="half-open">

#### Stage 6: the same stub, ungoverned

For contrast, call the same stub with no governor in the way:

<div class="ikigai-run" data-harness='overlays' data-cmd='source urn:iki:tutorial:harness:upstream'>
<pre class="ikigai-run-expected">ok, slowly (call 11, after 10s)
[uncacheable]</pre>
</div>

It answers, after the full 10 seconds. Without the stack, a caller waits as long as the
dependency takes; with it, the caller hears in 2. And those 10 seconds were harness time: the
clock moved past the open circuit's 5-second cooldown while this call waited, so the circuit went
half-open with nobody calling through it, the same way it did in stage 3:

<div class="ikigai-stage-result">
<p class="ikigai-stage-facts">upstream: slow · circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures · the harness clock reads 23s</p>
<p class="ikigai-stage-picture"><a class="ikigai-scheme-light" href="harness-stage-6-light.svg"><img src="harness-stage-6-light.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures."></a><a class="ikigai-scheme-dark" href="harness-stage-6-dark.svg"><img src="harness-stage-6-dark.svg" alt="The resilience harness, drawn: the governed call urn:iki:tutorial:harness:call inside its space, the stub's own space beside it with urn:iki:tutorial:harness:mode, urn:iki:tutorial:harness:clock and urn:iki:tutorial:harness:upstream, then the governor state board and the diagram's space. The circuit breaker's state is drawn in the governed space's box and said in words below the picture. At this stage: throttle state: 1 governor visible; breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s; circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures."></a></p>
<pre class="ikigai-stage-words">throttle state: 1 governor visible
breaker: circuit-breaker, governs urn:iki:space:tutorial:harness:governed, trips after 3, cooldown 5s
  circuit `urn:iki:tutorial:harness:call`: half-open, 3 consecutive transient failures</pre>
</div>

</section>
</div>

Failover and the bulkhead (`Throttle`) are not in the harness yet. They are the next pieces of
this chapter to run, and until then the table above is their description.
