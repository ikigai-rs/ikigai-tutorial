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
  ┌──────────────────────────────────────────────┐
  │ Retry (idempotent verbs, spends a budget)      │
  │  ┌──────────────────────────────────────────┐ │
  │  │ CircuitBreaker (open / closed / half-open) │ │
  │  │  ┌──────────────────────────────────────┐ │ │
  │  │  │ Timeout (a deadline on the call)       │ │ │
  │  │  │  ┌──────────────────────────────────┐ │ │ │
  │  │  │  │   the space being guarded        │ │ │ │
  │  │  │  │   (a door, a mount, an endpoint) │ │ │ │
  │  │  │  └──────────────────────────────────┘ │ │ │
  │  │  └──────────────────────────────────────┘ │ │
  │  └──────────────────────────────────────────┘ │
  └──────────────────────────────────────────────┘
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

## What is drawn here, and what will run

The diagrams above are **static** in this revision of the book. Making them *run* — letting you
poke a dependency from the page so it fails transiently or stalls, and watching the breaker trip,
the budget drain, the failover move — needs two things that are not built yet: a diagram that can
draw **state** on top of the arrangement it already harvests (a breaker's open/closed, a budget's
count, a parked-work queue), and a small **driver** you can control from the page to make a stubbed
dependency misbehave. Both are planned, and the transparency property above is exactly what will let
a live diagram read the arrangement and the state separately. Until then these are pictures, and the
book says so rather than faking a run.
