# Golden threads in practice

[What resolution buys you](payoff.md) cut one thread and watched one derived answer
recompute. This chapter is the mechanism whole — declare, derive, cut, recompute — and
then the two things around it that the first chapter could not say yet: what *bounds*
the cache, and the two rules that decide whether an answer is safe to keep at all.

## Declare, derive, cut

A **golden thread** is a name. An endpoint declares that its answer hangs from one with
`.depends_on(thread)`; by convention the thread is named after the resource whose state
it tracks, which is why `title`'s thread is `urn:iki:tutorial:title`. Three things then
happen without the endpoint's help:

- **Derive.** A composite — anything that resolves another resource from inside its own
  invocation, `inv.source(..)` — inherits the threads of everything it resolved. Its
  cache entry hangs from the union.
- **Cut.** After a successful `Sink` or `Delete`, the kernel cuts the thread named after
  the write's canonical target. A cut is cheap and blind: it bumps the thread's
  generation and nothing else. `Kernel::cut` and the REPL's `sink urn:kernel:cut` do the
  same by hand, which is what a filesystem watcher does when a file changes out from
  under the kernel.
- **Recompute, lazily.** Nothing is evicted at the cut. An entry records the generation
  each of its threads held when it was stored; the next lookup compares, and an entry
  whose thread has moved is thrown away then. Readers never invalidate anything; only a
  change does.

The one-liner that falls out, and the reason the mechanism is cheap: **rebuilding a
composite costs one lookup per dependency**, not one recompute per dependency. The
dependency graph *is* the decomposition of a resource into the resources it needs, and
recomputation stops at every part whose thread is still whole. Here a composite hangs
from a thread of its own *and* from `title`'s; cutting its own thread rebuilds it and
finds `title` still in the cache:

<!-- urn-gate: illustration urn:iki:tutorial:framed-title — a composite bound only inside
     the block below. -->
<!-- urn-gate: illustration urn:iki:tutorial:frame — a golden thread's name, not a
     resource: nothing is bound at it, and the block cuts it by hand. -->

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{
    AsyncFnEndpoint, Capability, Exact, Iri, Kernel, ReprType, Representation, Request,
    Verb,
};

// `framed-title` reads `title` and hangs from one more thread, its "frame".
let framed = AsyncFnEndpoint::new("framed-title", |inv| {
    Box::pin(async move {
        let title = inv.source(&Iri::parse(hello_camel::TITLE).unwrap()).await?;
        let text = format!("[{}]", String::from_utf8_lossy(&title.bytes));
        Ok(Representation::new(ReprType::new("text/plain"), text.into_bytes())
            .cacheable()
            .depends_on("urn:iki:tutorial:frame"))
    })
});
let state = Arc::new(hello_camel::TitleState::default());
let kernel = Kernel::new(Arc::new(
    hello_camel::space_over(Arc::clone(&state))
        .bind(Exact::new("urn:iki:tutorial:framed-title"), framed),
));
let root = Capability::root();
let framed = || Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:framed-title").unwrap());

// Derive: one read of `title`, and the composite hangs from both threads.
let first = block_on(kernel.issue(framed(), &root)).unwrap();
assert_eq!(String::from_utf8_lossy(&first.bytes), "[resource oriented computing]");
assert_eq!(state.reads.load(Ordering::SeqCst), 1);

// Cut the frame by hand — the watcher's move. The composite is gone…
kernel.cut("urn:iki:tutorial:frame");
assert!(!kernel.is_cached(&framed(), &root));

// …and rebuilding it cost one LOOKUP of `title`, not a second read.
block_on(kernel.issue(framed(), &root)).unwrap();
assert_eq!(state.reads.load(Ordering::SeqCst), 1);

// Cut `title`'s thread instead and the recompute goes all the way down.
kernel.cut(hello_camel::TITLE);
block_on(kernel.issue(framed(), &root)).unwrap();
assert_eq!(state.reads.load(Ordering::SeqCst), 2);
```

Ten thousand readers of a value that changes once an hour cost one recompute an hour,
and a composite over ten such values costs ten lookups when any one of them moves.

## What bounds the cache

The cache is a resource with a **policy**, installed at construction beside the clock
and the Meta renderer, because the right trade is the host's: an edge process facing
strangers, a batch job that wants everything it computed and a browser tab want
different ceilings. A `CachePolicy` is consulted in exactly two places — when the kernel
is about to store an entry (*admit*), and when the cache is over its bound and something
must go (*victim*). Absent, it is **LRU at 4096 entries and 64 MiB**, which is a change
from the unbounded cache before core 0.1.70: a long-lived process now evicts instead of
growing. `Fifo` and `CostAware` (keep what was expensive) ship beside it, and a policy
that wants an unbounded cache says so by declining to name a victim — visibly, not by
omission.

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{ArgRef, CacheBound, Capability, Iri, Kernel, Lru, Request, Verb};

// A cache with room for two entries.
let kernel = Kernel::new(Arc::new(hello_camel::space()))
    .with_cache_policy(Arc::new(Lru::with_bound(CacheBound::new(2, 8 << 20))));
let root = Capability::root();
let upper = |text: &str| {
    Request::new(Verb::Source, Iri::parse("urn:iki:fn:toUpper").unwrap())
        .with_arg("in", ArgRef::Inline(text.as_bytes().to_vec()))
};

for text in ["one", "two", "three"] {
    block_on(kernel.issue(upper(text), &root)).unwrap();
}

// Three cacheable answers, two kept: the least recently used went.
assert_eq!(kernel.cache_len(), 2);
assert!(!kernel.is_cached(&upper("one"), &root));
assert!(kernel.is_cached(&upper("three"), &root));
```

The line to carry away: **a policy decides what is worth keeping, never what is correct
to serve.** Golden threads and expiry are evaluated on the serving path, where no policy
is consulted, so no policy — including a hostile one — can hand back an entry whose
thread was cut or whose deadline has passed. Eviction bounds what is *kept*; validity is
somebody else's job. The REPL's `source urn:kernel:cache` shows both halves at once —
the bound (`entries 3 / 4096`, `size … / 64.0 MB`) and, per entry, how many threads it
hangs from.

One more thing the cache does, stated because it closed a race: every cut takes the next
number from one counter, and a request takes a snapshot of that counter *before it
resolves anything*. When the result comes back to be stored, the store declines if any
thread the result depends on was cut after the snapshot. So a write that lands while a
read is in flight cannot be swallowed by the read's stale answer — the answer is
returned and not kept, and the next read recomputes. Declining to cache is never wrong.

## Two rules you can state as rules

**1. The key holds everything the answer depended on.** A cache entry is filed under the
content-addressed request (verb, canonical name, every argument), a fingerprint of the
capability it ran under, and a fingerprint of the chain it was resolved in — [Scope and
alias](scope-and-alias.md) shows the third slot splitting and joining entries. Leave a
slot out and the cache hands an answer to a caller it was not computed for; the failure
is a *hit*, and hits do not log. What the key does not hold, the threads and the expiry
cover: the state of what the answer read, and time.

**2. Cacheability is only as good as the least cacheable dependency.** A representation
has one of three expiries — `Always` (the default: never cached; an endpoint opts *in*),
`At(deadline)` (until an instant, judged against the kernel's clock; a kernel with no
clock declines to store it), `Never` (`.cacheable()`: a pure function of its inputs) —
and a composite's is the *meet* of its own and every dependency's, with `Always` at the
bottom. Adding one uncacheable source to a cached resource makes it uncacheable. That is
a correctness no-op, and it is a performance change with no type signal. The cautionary
tale is three sentences long: one uncacheable overlay was joined into a cached graph on a
live server (2026-08-13, in the reading room's own repository); every read went from
about 20 µs to about 1.0 s; and 68 tests stayed green, because the types are identical
either way. Nothing catches it but measuring the read — so when you add a source to a
cached resource, measure the read before and after. That is the only signal there is.

## Not yet

Two holes in the mechanism, on the record so you build around them rather than into
them. Both are the kernel's to close (its design notes track them as one arc, ledger item
512); neither is closed at 0.1.72.

**A cacheable read does not automatically hang from its own name.** The kernel stores the
threads an endpoint *declared* plus those of its sub-requests; it does not pair a
cacheable `Source` with a thread named after its own canonical target. So the `Sink`'s
auto-cut invalidates a cached read of the same name only if that read said
`.depends_on(<its own name>)` — which `title` does, by hand, and which `ikigai-fs` does.
Every other cacheable endpoint fronting mutable state is one forgotten line from serving
stale bytes after a write through the same name. The block below **pins today's
behaviour**: it will fail the day the kernel closes the hole, and that failure is the
signal to rewrite this paragraph.

<!-- urn-gate: illustration urn:iki:tutorial:note — a mutable resource bound only inside
     the block below, written so its cacheable read declares no thread. -->

```rust
# extern crate ikigai_core;
# extern crate futures;
use std::sync::{Arc, Mutex};
use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, EndpointSpace, Exact, FnEndpoint, Iri, Kernel, ReprType,
    Representation, Request, Verb,
};

// Like `title`, minus the one line: the Source is cacheable and declares NO thread.
let text = Arc::new(Mutex::new(String::from("first")));
let note = {
    let text = Arc::clone(&text);
    FnEndpoint::new("note", move |inv| {
        let mut current = text.lock().unwrap();
        Ok(match inv.request.verb {
            Verb::Sink => {
                *current = inv.inline_str("content")?.to_string();
                Representation::new(ReprType::new("text/plain"), b"ok".to_vec())
            }
            _ => Representation::new(ReprType::new("text/plain"), current.clone().into_bytes())
                .cacheable(),
        })
    })
};
let kernel = Kernel::new(Arc::new(
    EndpointSpace::new().bind(Exact::new("urn:iki:tutorial:note"), note),
));
let root = Capability::root();
let name = || Iri::parse("urn:iki:tutorial:note").unwrap();

block_on(kernel.issue(Request::new(Verb::Source, name()), &root)).unwrap();
let write = Request::new(Verb::Sink, name())
    .with_arg("content", ArgRef::Inline(b"second".to_vec()));
block_on(kernel.issue(write, &root)).unwrap();

// The write cut `urn:iki:tutorial:note` — and the cached read hangs from nothing.
let stale = block_on(kernel.issue(Request::new(Verb::Source, name()), &root)).unwrap();
assert_eq!(String::from_utf8_lossy(&stale.bytes), "first"); // today: stale, served
```

Until that closes, the rule for your own endpoints is the one `title` follows: a
cacheable read of mutable state declares the thread named after itself.

**A failed sub-request records no dependency.** An endpoint that catches an `Unresolved`
and returns a cacheable fallback is cached with no thread on the missing name, so a
`Sink` that later creates it cuts nothing the entry hangs from; and a result built on a
*denial* is cached the same way, though a change of grant has no thread to cut. The fix
is to record the failure as a dependency too — a thread on the missing name; no caching
at all for a result built on a refusal — and it is not built. When in doubt, do not
cache a fallback.

[Why an endpoint describes itself](self-description.md) is next: the other half of what
made the fourth demonstration in the previous chapter possible.
