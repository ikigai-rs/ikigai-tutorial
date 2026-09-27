# The shape of resolution

[Resolution](resolution.md) named the terms and said where each one is *done*. Before you
run any of it, five things are true of every system that resolves names rather than calling
functions — true of ikigai, true of NetKernel, true of the Web — and they are easier to
carry as shapes than as rules. Each gets one picture and, where it earns one, one line of
math. Nothing here needs a proof, and nothing here is specific to Rust.

The pictures are a hotel. Peter Rodgers, who co-designed NetKernel, wrote resource-oriented
computing down as set theory in *Peter's Hotel: A Set-Theoretic Formalism*, with a parable
beside it in which guests carry codes and corridors hold doors. The doors and the corridors
below are his. The keys, the vault, and the clock are what this book adds, because they are
the parts of ikigai the parable does not need and the code cannot do without.

## 1. A name and a context

A keypad door does not hold one code. It recognizes a *family* of codes — `Exact` is a
family of one; a template with a variable in it is a family of many — and a corridor is a
row of such doors, tried in order, the first that recognizes the code opening. A corridor
can end in another corridor. It can have a whole wing imported under a prefix. It can have
a door that, instead of opening, rewrites the code and tries the corridor behind it.

So what a code opens is not a property of the code. It is a property of the code *and the
corridor you are standing in*:

```text
  code ──▶ [door][door][door] ──miss──▶ [door][door] ──miss──▶ nowhere
            corridor A                    corridor B
            ▲
            the one you are standing in decides
```

Written once, and then never again: a resource is `ρ(name, context)`. The same name on two
machines is two resources. The same name in two corridors is two resources. They may happen
to answer alike, and nothing about the name says so.

| in the hotel | in ikigai | where you build one |
|---|---|---|
| a keypad door | a grammar bound to an endpoint | [Hello, resource](hello-resource.md) |
| a corridor | `EndpointSpace` — first match wins | [Binding, and a host of your own](binding.md) |
| a corridor that ends in another | `Fallback` — one more space, tried in order | [Two ways to reach a kernel](../modules/two-ways.md) |
| a wing imported under a prefix | `Mount` — a prefix in front of a space | [Scope and alias](scope-and-alias.md); over a wire, [Preferential resolution](../beyond/preference.md) |
| a door that rewrites the code | `Rewrite`; `Alias` is its table-driven form | below, and [Scope and alias](scope-and-alias.md) |
| the corridor you are standing in | `Scope`; `Confine` for cutting the root off | below, and [Scope and alias](scope-and-alias.md) |

The last row is the one a function call has no word for. A kernel has a root corridor — the
space you hand it when you build it — and a request can carry a chain of corridors to try
*ahead* of the root, each with a name, chosen per request. Here the tutorial host's own
`title` is resolved twice: once in the root, and once standing in a corridor that binds the
same name to a different door.

<!-- urn-gate: illustration urn:iki:tutorial:ctx:six-pm — a corridor's name, not a resource: it is the identity the cache keys the corridor by, and nothing binds it. -->

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
let title = || Request::new(Verb::Source, Iri::parse(hello_camel::TITLE).unwrap());

// A corridor with one door in it, recognizing the SAME name as the host's `title`.
let corridor = Arc::new(EndpointSpace::new().bind(
    Exact::new(hello_camel::TITLE),
    FnEndpoint::new("title-at-six", |_| {
        Ok(Representation::new(ReprType::new("text/plain"), b"as it stood at 18:00".to_vec())
            .cacheable())
    }),
));
let at_six = Scope::empty().with_named(Iri::parse("urn:iki:tutorial:ctx:six-pm").unwrap(), corridor);

// One name. Two contexts. Two resources.
let here = block_on(kernel.issue(title(), &root)).unwrap();
let there = block_on(kernel.issue_in(title(), &root, at_six)).unwrap();
assert_eq!(String::from_utf8_lossy(&here.bytes), "resource oriented computing");
assert_eq!(String::from_utf8_lossy(&there.bytes), "as it stood at 18:00");

// And two cache entries, because the corridor is part of the key — the next idea.
assert_eq!(kernel.cache_len(), 2);
```

The corridor's *name* is what the kernel remembers it by. Two requests carrying corridors
named alike share cached answers, which is a claim the injector is making — that the two
corridors bind the same doors — and an unnamed corridor shares with nothing, which is sound
and useless. An endpoint can narrow the chain for its own sub-requests — `Confine` puts a
corridor where the root was and cuts the root off, so a name the corridor does not bind has
nowhere to go — but it cannot widen the chain, and it cannot get ahead of a corridor the
host injected. That asymmetry is idea 3 in a different coat, and it is why the chain is
safe to expose at all.

### The arrangement is a resource, so a question about it is a query

One more kind of door, and then the corridor can answer a question about itself. A
**limiter** admits a whole family of codes and opens on a wall. Because a corridor is
first-match, every door in that family *behind* the wall is unreachable from where you
stand, and a guest who tries one gets exactly the answer a missing door gives — not a
refusal, nothing. A wall that said "there is something back here" would already be a leak.

So a host that serves one corridor to the public can keep its private wing out of reach by
*structure*: put the wall ahead of the wing, and there is no decision left to misconfigure.
The question the operator then wants answered is the paper's Theorem 4(b), gatekeeper
completeness: **is any door of the private family reachable from the public entrance
without passing a wall over that family?** In the paper it is decided by a pushdown
analysis over the graph of imports. In ikigai the arrangement is a tree with no way back
up it — a wing that misses hands the *original* code back to its parent, which just tries
the next door — so nothing is ever pushed to be popped later, and the question collapses to
a path query. That needs the arrangement to be something a query can read, and it is:
`urn:kernel:topology` renders the corridor you are standing in as Turtle, every space an
IRI, every corridor an ordered list of what it tries.

Here is the arrangement, in hotel terms and as the kernel renders it. The public entrance
is a corridor of two: first a wall over `urn:personal:`, then the host's root, which imports
a personal wing and a public one.

```text
  entrance ──▶ [ wall: urn:personal:* ][ root ]
                                          └──▶ [ wing "urn:personal:" ][ wing "urn:public:" ]
                                                       └─ door urn:personal:calendar
```

<!-- urn-gate: illustration urn:ikigai:chain:root — the entry node the kernel names an empty chain by. -->
<!-- urn-gate: illustration urn:ikigai:chain:root:layer:1 — a list cell of the rendered chain. -->
<!-- urn-gate: illustration urn:ikigai:space:_:1 — an unnamed space, skolemized by the renderer. -->
<!-- urn-gate: illustration urn:ikigai:space:_:1:layer:1 — a list cell. -->
<!-- urn-gate: illustration urn:ikigai:space:_:1:layer:2 — a list cell. -->
<!-- urn-gate: illustration urn:ikigai:space:_:2 — an unnamed space, skolemized by the renderer. -->
<!-- urn-gate: illustration urn:example:space:gatekeeper — the wall's name in the example arrangement. -->
<!-- urn-gate: illustration urn:example:space:root — the example root's name. -->
<!-- urn-gate: illustration urn:example:space:root:layer:1 — a list cell. -->
<!-- urn-gate: illustration urn:example:space:root:layer:2 — a list cell. -->
<!-- urn-gate: illustration urn:example:space:personal — the private wing's name in the example. -->
<!-- urn-gate: illustration urn:personal:calendar — the one door in the private wing; a stand-in, not a name a reader can resolve. -->
<!-- urn-gate: illustration urn:public: — the public wing's family in the example. -->
<!-- urn-gate: illustration urn:personal:calendar: — the narrower family in the last caveat, a wall over one door. -->
```turtle
@prefix ik:  <https://ikigai-rs.dev/ns#> .
@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .

<urn:ikigai:chain:root> a ik:Chain ; ik:severed false ;
    ik:layers <urn:ikigai:chain:root:layer:1> .
<urn:ikigai:chain:root:layer:1> rdf:first <urn:ikigai:space:_:1> ; rdf:rest rdf:nil .

<urn:ikigai:space:_:1> a ik:Fallback ; ik:layers <urn:ikigai:space:_:1:layer:1> .
<urn:ikigai:space:_:1:layer:1> rdf:first <urn:example:space:gatekeeper> ;
    rdf:rest <urn:ikigai:space:_:1:layer:2> .
<urn:ikigai:space:_:1:layer:2> rdf:first <urn:example:space:root> ; rdf:rest rdf:nil .

<urn:example:space:gatekeeper> a ik:Limit ; ik:family "urn:personal:" .

<urn:example:space:root> a ik:Fallback ; ik:layers <urn:example:space:root:layer:1> .
<urn:example:space:root:layer:1> rdf:first <urn:ikigai:space:_:2> ;
    rdf:rest <urn:example:space:root:layer:2> .
<urn:ikigai:space:_:2> a ik:Mount ; ik:prefix "urn:personal:" ;
    ik:space <urn:example:space:personal> .
<urn:example:space:personal> a ik:EndpointSpace ; ik:pattern "urn:personal:calendar" .
# … the public wing, the same shape, under layer 2.
```

Two things to notice before the query. The lists are spelled out as `rdf:first` / `rdf:rest`
cells with their own IRIs rather than with the `( … )` shorthand, because that shorthand
makes blank nodes and a blank node cannot be named in a second graph or a diff. And a space
that has no name of its own is given one — `urn:ikigai:space:_:1` — so that the graph has no
anonymous nodes at all. The order of a list is the order the corridor tries doors in, and it
is the whole reason the query below can say "ahead of".

```sparql
PREFIX ik:  <https://ikigai-rs.dev/ns#>
PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>
ASK {
  BIND("urn:personal:" AS ?family)
  <urn:ikigai:chain:root> (ik:layers/rdf:rest*/rdf:first | ik:space)* ?leaf .
  ?leaf a ik:EndpointSpace ; ik:pattern ?door .
  FILTER(STRSTARTS(?door, ?family))
  FILTER NOT EXISTS {
    <urn:ikigai:chain:root> (ik:layers/rdf:rest*/rdf:first | ik:space)* ?list_owner .
    ?list_owner ik:layers ?head .
    ?head rdf:rest* ?cell . ?cell rdf:first ?limiter .
    ?limiter a ik:Limit ; ik:family ?limited .
    FILTER(STRSTARTS(?family, ?limited))
    ?cell rdf:rest+ ?later . ?later rdf:first ?branch .
    ?branch (ik:layers/rdf:rest*/rdf:first | ik:space)* ?leaf .
  }
}
```

Read it one clause at a time:

- **`BIND("urn:personal:" AS ?family)`** names the family you are asking about. Change this
  line and nothing else to ask about a different wing.
- **`(ik:layers/rdf:rest*/rdf:first | ik:space)*`** is the whole of "walk down the
  arrangement." From any node you may step into *any* cell of its list — `ik:layers` to the
  head, `rdf:rest*` along it as far as you like, `rdf:first` into the space that cell holds —
  *or* step through a wing (`ik:space`) into what it imports. The outer `*` says: any number
  of such steps, including none. Everything reachable from the entrance by resolution is
  reachable from `<urn:ikigai:chain:root>` by this path, and nothing else is.
- **`?leaf a ik:EndpointSpace ; ik:pattern ?door`** with **`STRSTARTS(?door, ?family)`** finds
  a corridor of real doors, reachable from the entrance, holding a door whose code is in the
  family. That is a candidate path into the private wing.
- **`FILTER NOT EXISTS { … }`** throws the candidate out if a wall stands in its way. Inside
  it: find some list on the way down (`?list_owner`, itself reached by the same walk), a
  cell of that list holding a limiter (`?cell rdf:first ?limiter`) whose family covers ours
  (`STRSTARTS(?family, ?limited)` — the wall is over `urn:personal:` or something broader),
  and a **later** cell (`?cell rdf:rest+ ?later`, one or more steps further along the same
  list) whose space leads on down to the very leaf we found. First-match is what makes
  "later" mean "unreachable": the wall answers first, and the branch behind it is never
  consulted.
- **`ASK`** answers `true` if any candidate survives — a door of the family is reachable
  with no covering wall ahead of it — and `false` if none does.

So the answer is inverted from the way the theorem is phrased: **`false` is the safe
answer**. For the arrangement above it is `false`; remove `<urn:example:space:gatekeeper>`
from the first list and it is `true`. A wall injected as a corridor for one request is a
cell of the `ik:Chain`'s own list, ahead of the root, and the same query finds it there.

What the query answers, and what it does not. It answers *exactly* for arrangements built
from a chain, `Fallback`, `Mount`, `Limit` and `EndpointSpace` with prefix-shaped patterns —
walls and doors over families of names, which is the arrangement above. It does not answer
for three kinds of node on a path *below* a wall: an opaque space (a remote peer, or an
overlay written before the kernel could ask), a rewriting door, and a template door or
family whose codes are not a prefix. A path through one of those is *unknown*, never *no*,
and a second query — which the kernel's formal document will carry from ikigai-core
0.1.79 — reports exactly those nodes so that "unknown" is a list rather than a shrug. The
rewriting door is the one to take seriously: a code from *outside* the family walks past
the wall, is rewritten *into* the family by an alias behind it, and opens the private
door, because the wall is over the name asked for and not over the door. The first query
alone says guarded; the arrangement leaks. And, as before, a wall over a *narrower* family
than the one you asked about does not count: `STRSTARTS("urn:personal:",
"urn:personal:calendar:")` is false, and a wall over one door is not a wall over the wing.

You can run this against any store that speaks SPARQL — the ikigai host's own `urn:sparql:ask`
over the Turtle that `urn:kernel:topology` returns, for one — and ikigai-core's test suite
walks the same triples without a store and pins the answers: no with the wall, yes without,
no again with the wall injected as a corridor, still yes under a narrower wall, and unknown
when an opaque space is on the path.

## 2. What the key must hold

A cache is a vault of answers, each behind a key, and the key must hold **everything the
answer depended on**:

```text
  key = ( name , arguments , who asked , which corridors )
          └──────┴───────────┴───────────┴───── one slot per fact the answer turned on
```

Leave a slot out and the vault hands an answer to someone it was not computed for — another
caller's arguments, another caller's authority, another corridor's door. The failure is not
a miss. It is a *hit*, and hits do not log.

ikigai's key has three slots, and they cover the four facts: the content-addressed identity
of the request (verb, name, and every argument, `as=` included), a fingerprint of the
capability it ran under, and a fingerprint of the chain it was resolved in. The chain slot
holds the *whole* chain rather than the corridors actually consulted, which is sound and
over-partitioned: two chains differing only in a corridor neither request touched do not
share. You feel the capability slot in [What resolution buys you](payoff.md), where
`.cacheable()` is safe to say on an endpoint whose answer depends on who is asking.

The rule that follows is the one that bites: **a cached answer is only as cacheable as the
least cacheable thing it depended on**, because a slot has to hold that dependency too. The
cautionary tale is three sentences long. One source was joined into a cached resource on a
live server, a correctness no-op; every read went from about 20 µs to about 1.0 s; and 68
tests stayed green, because the types are identical either way. Nothing catches this but
measuring the read.

The converse mistake is a slot too many. Two names for one resource make two entries to fill
and two threads to cut, and a write through one name invalidates only what that name
declared. So whoever rewrites a name *reports* it, and the kernel keys, cuts, and gates on
the name reported — the one the resource is actually reached by:

<!-- urn-gate: illustration urn:iki:tutorial:camel — a second name for camel-case that exists only inside the kernel this block builds; no book host binds it. -->

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request, Rewrite, Verb};

// One rewriting door in front of the book's space: the short name is `camel-case`.
let space = Rewrite::new(Arc::new(hello_camel::space()), |iri: &Iri| {
    (iri.as_str() == "urn:iki:tutorial:camel")
        .then(|| Iri::parse("urn:iki:tutorial:camel-case").unwrap())
});
let kernel = Kernel::new(Arc::new(space));
let root = Capability::root();
let camel = |name: &str| {
    Request::new(Verb::Source, Iri::parse(name).unwrap())
        .with_arg("in", ArgRef::Inline(b"two names one entry".to_vec()))
};

let short = block_on(kernel.issue(camel("urn:iki:tutorial:camel"), &root)).unwrap();
let long = block_on(kernel.issue(camel("urn:iki:tutorial:camel-case"), &root)).unwrap();
assert_eq!(short.bytes, long.bytes);

// Two names, ONE entry: the rewrite reported the name it reached.
assert_eq!(kernel.cache_len(), 1);
```

Put the two blocks side by side and the shape is the whole idea: a corridor splits one name
into two resources, and a reported rewrite joins two names into one — and in both cases the
key is what changed.

## 3. Authority only shrinks

A capability is a set of scopes — a ring of keys. When an endpoint resolves a sub-request,
the sub-request runs under the caller's ring or a subset of it: passing authority on can
*intersect* it, never widen it. So along any chain of calls, authority is a staircase that
only goes down:

```text
  caller           holds  { read, write, net }
  └─ endpoint      runs under  { read, write }      ⊆
     └─ sub-request      runs under  { read }       ⊆
        └─ …                                        ⊆    never a step up
```

The one line of math is `t ∩ s ⊆ t`, and it is the reason the staircase has no up-step: an
intersection cannot contain anything either side lacks. The place a step *up* would go — a
module acting under authority its caller does not hold — has a name, the confused deputy,
and that is why it is not built. In ikigai there is no operation on the capability type
that widens one; `attenuate` is the intersection, and when a capability arrives over a
wire it is clamped to what the channel itself authenticated, which is the same intersection
from the other side.

Part I runs everything as root, on purpose. [The file workspace](file-workspace.md) is
where a scoped capability first refuses something, and [Who is asking](../beyond/identity.md)
is where the staircase crosses a machine boundary and is still a staircase.

## 4. Keeping information, and losing it

A resource does not have *a* format. The same image as PNG and as WebP is one resource
twice, and the thumbnail beside them is something else:

```text
  ┌──────────┐    ┌──────────┐        ┌────┐
  │  image   │ ⇄  │  image   │        │ ▪  │
  │  (png)   │    │  (webp)  │        └────┘
  └──────────┘    └──────────┘       thumbnail:
     the same resource, twice      a different resource
```

The one line: `H(f(X)) ≤ H(X)`, with equality exactly when `f` is injective. A conversion
that loses nothing can be undone on what it represents, so a kernel is free to perform it
on your behalf — that is a **transreption**, and asking for `as=text/turtle` is a routing
question. A summary, an embedding, a classification, a thumbnail is not injective; it
cannot be undone, so it is not a format of the resource. It is a **projection**: a
different resource, with its own name, that your code resolves on purpose.

The test to apply is the inequality, not the file size. A transreptor declares what it
converts between, and the kernel selects on the declaration — [Transreption](../building/transreption.md)
is where you write one and watch the kernel find it.

## 5. The clock that bounds the cost

Two clocks tick at once. Reality changes at some rate; readers ask at some other rate. In a
system where derivation is recorded — golden threads — a reader never invalidates anything.
Only a change does. So a cached answer is recomputed at most once per change *and* at most
once per read:

```text
  changes    ──┬──────────┬────────────────────┬──┬───────────▶   rate λ_c
  reads      ──┼─┬─┬─┬─┬──┼─┬─┬─┬─┬─┬─┬─┬─┬─┬──┼──┼─┬─┬─┬─────▶   rate λ_o
  recomputes   ●          ●                    ●  ●                ≤ min(λ_c, λ_o)
```

The one line, stated once: if both clocks are memoryless, the recompute rate is
`λ_o λ_c / (λ_o + λ_c)`. In words — a recompute needs a change and then the next read
after it, so the expected gap between recomputes is the expected wait for a change plus
the expected wait for a read, and the rate is the reciprocal of that sum. It is smaller
than either rate on its own. When readers are fast relative to change, it approaches the
change rate; when change is fast relative to readers, it approaches the read rate. Either
way, **the cost of serving an immutable answer is bounded by how fast reality changes, not
by how often it is asked for.** Ten thousand readers of a value that changes once an hour
cost one recompute an hour.

You cut a thread and watch exactly one derived answer recompute in [What resolution buys
you](payoff.md).

## What ikigai builds for each

| idea | the mechanism | status |
|---|---|---|
| 1. name and context | `Scope`: a named chain of corridors ahead of the root, per request; `Confine` for narrowing it from inside | built (ikigai-core 0.1.72) |
| 1, again: the arrangement | `Limit`, a door onto a wall; `urn:kernel:topology`, the corridor you stand in as Turtle | built (ikigai-core 0.1.76, 0.1.78); the gatekeeper check is the query above |
| 2. the key | request id + capability fingerprint + chain fingerprint | built; whole chain, not consulted corridors |
| 3. authority shrinks | `attenuate` is the intersection; `clamp` is it on the wire | built; enforced everywhere a capability is declared |
| 4. lossless vs lossy | a transreptor declares the pair it converts between, and `.lossy()` says which are projections | declared, and the planner refuses a lossy hop without consent (ikigai-core 0.1.77); nothing checks that a declared transreption is injective |
| 5. the clock | golden threads: a write cuts, a read never does | built |

One sentence ties them. A pinned context — a named corridor that answers `now` with the
same *then* every time — turns an uncacheable question into an immutable one, and every
row above is what makes that safe: the corridor is in the key, the authority it ran under is
in the key, and only a change to the pinned answer could ever cut it. That is what the first
block on this page did with a title; the same move with a clock is as-of resolution.

Now run one. [Running it](running-it.md) is next.
