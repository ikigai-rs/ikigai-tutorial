# Scope and alias

[Binding, and a host of your own](binding.md) put three names in a space and a kernel
around it, and everything so far has resolved a name against that one space, from the
outside. This chapter is about the two things that sit *between* a name and its binding,
which a function call has no word for: **where you are standing** when you ask — the
scope — and **what a name is short for** — the alias. Both are ikigai-core's own API
(0.1.72), both are small, and both change what the cache is allowed to remember.

First, though, the two combinators every host in this book already uses and no chapter
has named.

## Composing spaces: `Fallback` and `Mount`

A `Space` is anything that can answer "which endpoint, if any, is bound at this name".
`EndpointSpace` is the plain one — a row of bindings, first match wins — and two
combinators compose them:

- **`Fallback::new(vec![a, b, …])`** tries each space in order; the first hit wins. Order
  is precedence, and precedence is the semantics: whatever `a` binds, `b` never sees.
  `building_endpoints::space()` in [Transreption](../building/transreption.md) is one —
  this book's bindings first, then the published SPARQL face.
- **`Mount::new(prefix, inner)`** admits only names that start with `prefix` and hands
  those to `inner`, unchanged. Nothing is rewritten: the inner space's names are already
  full names. The prefix is a fence, and a name outside it is not there at all.

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Kernel, Mount, Request, Space, Verb};

// Part I's space, reachable only under this book's prefix.
let tutorial: Arc<dyn Space> = Arc::new(hello_camel::space());
let kernel = Kernel::new(Arc::new(Mount::new("urn:iki:tutorial:", tutorial)));
let root = Capability::root();
let upper = |name: &str| {
    Request::new(Verb::Source, Iri::parse(name).unwrap())
        .with_arg("in", ArgRef::Inline(b"a b".to_vec()))
};

// Inside the fence: bound.
let inside = block_on(kernel.issue(upper("urn:iki:tutorial:camel-case"), &root)).unwrap();
assert_eq!(String::from_utf8_lossy(&inside.bytes), "aB");

// The same space binds `urn:iki:fn:toUpper` — and through this mount it is not there.
let outside = block_on(kernel.issue(upper("urn:iki:fn:toUpper"), &root)).unwrap_err();
assert!(matches!(outside, Error::Unresolved(_)));
```

That `Unresolved` is worth a second look, because the rest of this chapter turns on it.
The endpoint exists, in a space the kernel holds, and the answer is not "you may not"
but "there is no such name here". A mount does not refuse; it makes a region of the name
space *absent*. Keep that distinction: an absence is a fact about the arrangement, and
nobody can misconfigure it per request.

## A name for a name

A **logical rewrite** is a stable name that resolves to a different backing resource
without the caller knowing. Two shapes ship in core: `Rewrite::new(inner, rule)`, a
closure `Fn(&Iri) -> Option<Iri>` in front of a space, and **`Alias`**, its table-driven
form — exact rules (`urn:iki:tutorial:heading` → one backing name) and prefix rules (a
whole namespace moves, and every name under it follows). A host installs the table with
`Kernel::with_aliases`, which wraps the root in an `Alias` *and* teaches the kernel the
table, so the kernel can refuse a bad table before dispatch and answer
`urn:kernel:aliases` with the rules and their counters.

Here is the one that matters — a write through one name invalidating the read through
the other:

<!-- urn-gate: illustration urn:iki:tutorial:heading — a second name for `title` that
     exists only in the alias table the block below installs; no book host binds it. -->

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{AliasTable, ArgRef, Capability, Iri, Kernel, Request, Verb};

// Part I's space, with `title`'s state in hand so the read counter can be watched —
// and one alias: `heading` is another name for `title`.
let state = Arc::new(hello_camel::TitleState::default());
let table = AliasTable::new().exact("urn:iki:tutorial:heading", hello_camel::TITLE);
let kernel = Kernel::new(Arc::new(hello_camel::space_over(Arc::clone(&state))))
    .with_aliases(Arc::new(table));
let root = Capability::root();
let read = |name: &str| Request::new(Verb::Source, Iri::parse(name).unwrap());

// Read through both names: the endpoint ran once, and there is ONE cache entry.
let via_alias = block_on(kernel.issue(read("urn:iki:tutorial:heading"), &root)).unwrap();
let via_backing = block_on(kernel.issue(read(hello_camel::TITLE), &root)).unwrap();
assert_eq!(via_alias.bytes, via_backing.bytes);
assert_eq!(state.reads.load(Ordering::SeqCst), 1);
assert_eq!(kernel.cache_len(), 1);

// Write through the alias. The read through the backing name went with it.
let write = Request::new(Verb::Sink, Iri::parse("urn:iki:tutorial:heading").unwrap())
    .with_arg("content", ArgRef::Inline(b"one resource, two names".to_vec()));
block_on(kernel.issue(write, &root)).unwrap();
assert!(!kernel.is_cached(&read(hello_camel::TITLE), &root));

let after = block_on(kernel.issue(read(hello_camel::TITLE), &root)).unwrap();
assert_eq!(String::from_utf8_lossy(&after.bytes), "one resource, two names");
assert_eq!(state.reads.load(Ordering::SeqCst), 2);
```

Nothing about `title` knows it has a second name. What makes that safe is a set of
decisions core made once, each of them the answer to a way a rewrite goes wrong:

1. **The rewrite happens before the capability check.** Authority is named by URN too,
   so the floor is evaluated against the *backing* resource — the thing that will
   actually be touched. The other order is an escalation device: check the public name,
   then rewrite to the secret one. The cost is real and deliberate: a grant that names
   the old spelling does not follow the name, so a denial on an aliased target names the
   hop in its message rather than failing silently.
2. **`Meta` reports the backing name.** Ask `urn:iki:tutorial:heading` what it is and the
   description says `title`. A description that reported the alias would make the
   catalog, selection and the agent's tool list disagree about what exists.
3. **Two names share one cache entry and one golden thread.** That is the block above.
   It holds however the `Alias` was composed — by hand, under another overlay — because
   the overlay *reports* the rewrite (`Resolved::canonical`) and the kernel adopts the
   reported name before it computes the key, fires the cut, or checks the floor. Whoever
   rewrites a name says so; the kernel keys on what it is told.
4. **The catalog lists the backing name, once.** Listing both would offer one resource
   twice. During a migration that property is the point: consumers that discover names
   by reading the catalog move forward on their own, and consumers holding the old name
   keep working.
5. **Chains resolve; cycles are refused, loudly.** `a → b → c` is followed, up to eight
   hops. `a → b, b → a` is refused with the trail in the error — never truncated to
   whichever name the loop stopped on, because a half-applied rewrite is a wrong answer
   that looks like a right one.

> ⚠ Two things in this book are called an alias, and they make opposite promises on the
> one field that matters. Core's `Alias` is the table above: the two names are **one
> resource in this kernel**, so it reports its canonical name and the kernel fuses them.
> The *alias mount* in [Preferential resolution](../beyond/preference.md) is
> `MountedRemote`: it rewrites a name on the way **out** to another kernel, where the
> rewritten spelling is a wire address — and it must report nothing, because this kernel
> may serve an unrelated resource under that spelling. Same word, two contracts. The
> question that tells them apart: *does the rewritten name resolve to this very resource
> here?* If yes, report it; if it crosses out of the kernel, do not.

`Rewrite` differs from `Alias` in one way a host feels: a closure cannot be listed, so a
`Rewrite` in front of a space leaves the catalog unable to say what it rewrites — the
kernel reports that it cannot enumerate rather than pretending the catalog is complete.
A table can be listed. Prefer the table.

## Where you are standing

Everything above resolved against the kernel's **root** — the space you handed it at
construction. Since 0.1.72 a request also resolves in a **chain**: corridors placed ahead
of the root, chosen per request, and whether the root is on the end of the chain at all.
That chain is `Scope`, and it has two faces.

### Injection: a corridor ahead of the root

`Kernel::issue_in(request, capability, scope)` resolves the request in `scope`'s
corridors first, innermost first, then the root. A corridor that binds a name the root
also binds **shadows** it — for this request and for every sub-request it gives rise to.
Here `camel-title`, which resolves `title` from inside its own invocation, is run once
in the root and once standing in a corridor that binds `title` to a different door:

<!-- urn-gate: illustration urn:iki:tutorial:ctx:six-pm — a corridor's name, not a
     resource: the identity the cache keys the corridor by. Nothing binds it. -->

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{
    Capability, EndpointSpace, Exact, FnEndpoint, Iri, ReprType, Representation, Request,
    Scope, Verb,
};

let kernel = hello_camel::kernel();
let root = Capability::root();
let camel_title =
    || Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:camel-title").unwrap());

// One door, recognizing the same name as the host's `title`.
let at_six = Arc::new(EndpointSpace::new().bind(
    Exact::new(hello_camel::TITLE),
    FnEndpoint::new("title-at-six", |_| {
        Ok(Representation::new(ReprType::new("text/plain"), b"as it stood at six".to_vec())
            .cacheable())
    }),
));
let six_pm = Scope::empty().with_named(Iri::parse("urn:iki:tutorial:ctx:six-pm").unwrap(), at_six);

// The request named `camel-title` both times. Its SUB-request for `title` is what the
// corridor answered — the chain travels with the request, all the way down.
let here = block_on(kernel.issue(camel_title(), &root)).unwrap();
let there = block_on(kernel.issue_in(camel_title(), &root, six_pm)).unwrap();
assert_eq!(String::from_utf8_lossy(&here.bytes), "resourceOrientedComputing");
assert_eq!(String::from_utf8_lossy(&there.bytes), "asItStoodAtSix");

// Two contexts, two resources — and FOUR cache entries: the composite and its `title`
// sub-request, each once per chain, because the chain is in the key.
assert_eq!(kernel.cache_len(), 4);
```

`issue_in` lives on the kernel, and only there. Whoever can push a corridor ahead of the
root can stand in for *any* door for every sub-request below — bind the contacts
resource in a corridor and every sub-request that asks for contacts gets the corridor's
answer. So injection is authority, and it belongs to whoever holds the kernel: the same
trust line as `Capability::root()`. An endpoint cannot reach it from its invocation.

The corridor's **name** is what the cache remembers it by. `with_named(name, space)` is
a claim — *any corridor named this holds the same doors as this space* — exactly the
claim a reported canonical makes. Two requests carrying corridors named alike share
cached answers, so name two different corridors alike and one request is served the
other's answer. `Scope::with(space)`, the unnamed form, mints a fresh identity per call:
sound, and useless for the case injection exists for, since a corridor rebuilt per
request never shares with the last one.

### Severing: a chain with no root

The other face goes the other way. **`Confine::new(name, space, inner)`** wraps an
endpoint so that every sub-request it issues resolves in *the corridors the host
injected, then `space`* — and no root. A name the chain does not bind is `Unresolved`,
never `Denied`, and the root endpoint it would have reached is never entered:

<!-- urn-gate: illustration urn:iki:tutorial:document — a door in the corridor the block
     below confines an endpoint to; bound only inside that block's kernel. -->
<!-- urn-gate: illustration urn:iki:tutorial:excerpt — the confined endpoint's own name,
     bound only inside that block's kernel. -->
<!-- urn-gate: illustration urn:iki:tutorial:ctx:document — the confining corridor's name,
     the identity the cache keys it by. Nothing binds it. -->

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{
    AsyncFnEndpoint, Capability, Confine, EndpointSpace, Error, Exact, FnEndpoint, Iri,
    Kernel, ReprType, Representation, Request, Verb,
};

// An endpoint that reads a document, then tries to read the host's `title` as well.
let excerpt = AsyncFnEndpoint::new("excerpt", |inv| {
    Box::pin(async move {
        let document = inv.source(&Iri::parse("urn:iki:tutorial:document").unwrap()).await?;
        assert_eq!(String::from_utf8_lossy(&document.bytes), "the document");
        inv.source(&Iri::parse(hello_camel::TITLE).unwrap()).await
    })
});
// The corridor it is confined to holds the document and nothing else.
let corridor = Arc::new(EndpointSpace::new().bind(
    Exact::new("urn:iki:tutorial:document"),
    FnEndpoint::new("document", |_| {
        Ok(Representation::new(ReprType::new("text/plain"), b"the document".to_vec()))
    }),
));

let state = Arc::new(hello_camel::TitleState::default());
let kernel = Kernel::new(Arc::new(hello_camel::space_over(Arc::clone(&state)).bind_arc(
    Exact::new("urn:iki:tutorial:excerpt"),
    Arc::new(Confine::new(
        Iri::parse("urn:iki:tutorial:ctx:document").unwrap(),
        corridor,
        Arc::new(excerpt),
    )),
)));

// Under ROOT authority. The document resolved; `title` had nowhere to go.
let err = block_on(kernel.issue(
    Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:excerpt").unwrap()),
    &Capability::root(),
))
.unwrap_err();
assert!(matches!(err, Error::Unresolved(ref name) if name.as_str() == hello_camel::TITLE));
assert_eq!(state.reads.load(Ordering::SeqCst), 0);
```

Read the last two lines together. The caller held root, so no capability refused
anything; `title` is bound in the kernel's root, so it was not missing. It was
*unreachable from where the endpoint stood*, and the `title` endpoint's read counter
says it was never entered. That is the difference between a denial and an absence: a
denial is a decision, and a decision can be misconfigured; an unresolvable name has
nowhere to go. When the goal is a bound that cannot be got wrong per request, this is the
construct — an extractor handed a document and a model, and structurally nothing else.

The same move is available from inside an invocation as `inv.confine(name, space)`, which
returns the same invocation with its chain narrowed. It is the **only** chain-changing
operation an endpoint gets, and its shape is the whole argument: it puts `space` where
the root was and cuts the root off, behind every corridor the host injected — so it can
never shadow one, and relative to the chain it started in nothing resolves differently
except what the root would have answered. Narrowing is an endpoint's to do; widening is
the host's. That asymmetry is the authority staircase from
[Resolution](resolution.md), applied to the chain instead of the capability.

## What the key holds now

[What resolution buys you](payoff.md) said the cache is keyed on the request and the
capability. It is keyed on three things: the content-addressed request (verb, canonical
name, every argument), a fingerprint of the capability, and a fingerprint of the chain.
The third slot is why the injection block above ended with four entries — two resources,
each with its sub-request, once per chain — and the alias block with one: a corridor
splits one name into two resources, a reported rewrite joins two names into one, and in
both cases the key is what changed. The chain slot holds the
*whole* chain rather than the corridors actually consulted — sound and over-partitioned:
two chains that differ only in a corridor neither request touched do not share.

Two things the chain does not touch. `urn:kernel:*` is intercepted ahead of it, so no
corridor can shadow the catalog and a severed chain still reaches it — which means a
confined endpoint can read a list of names it cannot resolve (names, never content). And
the declared-capability floor runs unchanged, against whichever endpoint the chain
resolved to: a corridor shadowing an open door with a gated one is gated.

## As of: the corridor that pins time

Everything this chapter has gestured at comes together in one corridor. "Now" is the
least cacheable resource there is — a live door, `Always`, and anything that reads it
inherits that — and a corridor that binds `now` to one fixed instant turns it into a pure
function of its context. Since 0.1.75 the chain can carry a **clock** as well as doors:
`Scope::with_named_at(name, space, clock)` injects the corridor and sets the chain's clock
in one call, the only way a chain acquires one, so the binding and the clock cannot
disagree. `inv.now()` then answers from the chain's clock before the kernel's. An endpoint
that reads time by *resolution* and one that reads it from the *clock* see the same
instant, and the result is `Never` under the corridor and `Always` under the root —
checked here with `Kernel::is_cached_in`, the cache probe for a chain, rather than by
counting entries:

<!-- urn-gate: illustration urn:iki:tutorial:now — the time door, bound only inside the
     block below: live in its root, pinned in its corridor. -->
<!-- urn-gate: illustration urn:iki:tutorial:stamp — the endpoint that reads time both
     ways; bound only inside the block below. -->

```rust
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{
    AsyncFnEndpoint, Capability, EndpointSpace, Exact, Expiry, FixedClock, FnEndpoint, Iri,
    Kernel, ReprType, Representation, Request, Scope, Verb,
};

fn text(s: String) -> Representation {
    Representation::new(ReprType::new("text/plain"), s.into_bytes())
}
fn millis(time: Option<ikigai_core::Time>) -> String {
    time.map(|t| t.as_millis().to_string()).unwrap_or_default()
}

// `now`, as the root binds it: the kernel's clock, read through the invocation. A live
// door — uncacheable, the default `Always`.
let live_now = FnEndpoint::new("now", |inv| Ok(text(millis(inv.now()))));
// `stamp` reads time both ways an endpoint can — by resolution and from the clock — and
// is a pure function of what it read.
let stamp = AsyncFnEndpoint::new("stamp", |inv| {
    Box::pin(async move {
        let resolved = inv.source(&Iri::parse("urn:iki:tutorial:now").unwrap()).await?;
        let resolved = String::from_utf8_lossy(&resolved.bytes).into_owned();
        Ok(text(format!("{resolved} {}", millis(inv.now()))).cacheable())
    })
});
let kernel = Kernel::new(Arc::new(
    EndpointSpace::new()
        .bind(Exact::new("urn:iki:tutorial:now"), live_now)
        .bind(Exact::new("urn:iki:tutorial:stamp"), stamp),
))
.with_clock(Arc::new(FixedClock::at(2_000)));
let root = Capability::root();
let stamp = || Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:stamp").unwrap());

// The corridor: `now` pinned to one instant AND the clock derived from it, in one call.
let six_pm = || {
    Scope::empty().with_named_at(
        Iri::parse("urn:iki:tutorial:ctx:six-pm").unwrap(),
        Arc::new(EndpointSpace::new().bind(
            Exact::new("urn:iki:tutorial:now"),
            FnEndpoint::new("now-at-six", |_| Ok(text("1000".to_string()).cacheable())),
        )),
        Arc::new(FixedClock::at(1_000)),
    )
};

// Under the root: live on both faces, and not kept.
let live = block_on(kernel.issue(stamp(), &root)).unwrap();
assert_eq!(String::from_utf8_lossy(&live.bytes), "2000 2000");
assert_eq!(live.expiry, Expiry::Always);
assert!(!kernel.is_cached(&stamp(), &root));

// Under the corridor: one instant, whichever seam the endpoint used — and immutable.
// Cached in that chain, and in that chain only.
let pinned = block_on(kernel.issue_in(stamp(), &root, six_pm())).unwrap();
assert_eq!(String::from_utf8_lossy(&pinned.bytes), "1000 1000");
assert_eq!(pinned.expiry, Expiry::Never);
assert!(kernel.is_cached_in(&stamp(), &root, &six_pm()));
assert!(!kernel.is_cached(&stamp(), &root));
```

That is as-of resolution: the same request, the same endpoint, and a *then* in place of
a *now* for the whole resolution and every cache entry under it. Two things keep it
honest. The corridor's name is still the whole of its identity — the fingerprint does not
include the clock, so two corridors named alike with different clocks would share
answers, and the pairing is the injector's claim exactly as the name is. And validity is
judged on the **kernel's** clock, never the chain's: a corridor pinned in the past cannot
un-expire an `At(deadline)` entry that has passed, and one pinned in the future cannot
expire a fresh one. The chain's clock is what an endpoint *sees*; the kernel's is what
decides whether to serve.

## Not yet

The chain landed in 0.1.72, and by 0.1.75 three of the four kernel faces that used to
answer for the empty chain answer for the chain instead: **selection**
(`select_transreptor_in`, `select_action_in` — a confined endpoint's manifold no longer
offers an action its chain cannot resolve, and `Meta … as=` plans its route inside the
chain), **the cache probe** (`is_cached_in`, and a scope column on `urn:kernel:cache`),
and **the pipe** (`issue_with_incoming_in`). What remains, stated so you do not discover
it at a keyboard (`docs/design/resolution-scope.md` in `ikigai-core` is the full account):

- **The REPL's pipeline runs in the plain root**, so nothing on this page has a Run
  button.
- **The chain does not cross the wire.** A mount inside a confining corridor forwards
  with no chain, and the remote resolves in its own root — confining to a corridor that
  contains a mount confines to whatever the mount reaches. The wire's default refuses a
  non-empty chain rather than dropping it, so a module endpoint inside a confinement
  fails loudly instead of quietly escaping. Carrying it is a protocol bump, its own
  decision.
- **The key holds the whole chain, not the corridors consulted.** Sound and
  over-partitioned, as above. The datum a consulted-corridors key would need —
  *which* space answered — is reported since 0.1.78 (`Resolved::answered_by`); keying
  on it is not built.

And one thing that was "not built at all" here at 0.1.72 and is now a construct:
**`Limit`**, the door that stops a family of names in place (0.1.76). `Limit::new(prefix)`
is a space that answers a *hit on nothing* for every name under `prefix` and a miss for
everything else, so `Fallback([Limit("urn:personal:"), root])` is the root minus that
family — by structure, with no decision left to misconfigure per request — and a `Limit`
injected with `with_named` is the same wall at a position in the chain. A limited name
is `Unresolved`, never `Denied`, and the catalog subtracts it too.

The three constructs — mount, alias, scope — are the arrangement, the naming, and the
context of resolution. [What resolution buys you](payoff.md) is what you get for having
them.
