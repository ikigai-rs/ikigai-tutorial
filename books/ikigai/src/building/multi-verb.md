# Multi-verb endpoints

`camel-case` answers one verb, so its flat description *is* its contract — the 93% case,
and the reason the flat form exists. `title`, in [What resolution buys
you](../getting-started/payoff.md), answered two and quietly declared two `ActionSpec`s.
This chapter is that move made deliberate: one endpoint, three verbs, three contracts —
and a capability on your own endpoint that the kernel enforces, per verb, before your
code runs.

## Three verbs, three actions

`counter` is a number the host holds. `Source` reads it, `Sink` sets it, `Delete` resets
it. Reading needs no authority; writing and resetting do.

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/multi_verb.rs:endpoint}}
```

The implementation is one closure branching on `inv.request.verb`. The description is
three `ActionSpec`s, and that is not repetition: each verb has a different input list, a
different output, and a different `requires`. Flattening them would make the catalog say
`counter` takes `content` — on a read, which it does not — and requires the write scope
— on a read, which it must not.

The per-verb view is what everything above the endpoint consumes:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/multi_verb.rs:actions}}
```

That is the catalog's `ik:Action` nodes, the manifold's rows, and the agent's tool list —
one per verb, each with its own contract. An agent holding a read-only capability is
offered `Source` on `counter` and not `Sink`, because the offer is computed from exactly
this.

## Declared, therefore enforced

The rule [Resolution](../getting-started/resolution.md) stated three times is now
something you can feel. `.requires(WRITE)` on the `Sink` and `Delete` actions is checked
by the **kernel**, before dispatch: a caller whose capability does not satisfy it never
reaches the closure.

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/multi_verb.rs:enforced}}
```

Four things in that test, in order. A narrow capability *reads*, because `Source`
declared nothing. The same capability's *write* is refused, with a message naming both
the scope it lacked and the resource that declared it. A capability that holds the scope
writes — and the narrow reader then sees `7`, because the `Sink` cut `counter`'s golden
thread and the reader's cached `0` went with it. And `Delete` follows the same rule as
`Sink`, because it declared the same requirement.

Now look at the closure again. There is no capability check in it. There is deliberately
no capability check in it, because a check the catalog does not know about is the
mirror-image lie: an action that *enforces* authority it never *declared* fails callers
the catalog told were fine. The description is the only place authority is stated, and
the kernel is the only place it is checked — which is what keeps the offer, the
pre-flight and the enforcement from ever disagreeing.

> ⚠ Delete `.requires(WRITE)` from the `Sink` action and run the test. The write goes
> through for the narrow reader, and no test in the crate but this one notices — the
> code is unchanged and correct. **Declared = enforced** means the declaration is the
> enforcement, and a description is code you have to read with that in mind.

## What this is not

It is not a module boundary. [Where this actually stands](../modules/status.md) says
plainly that an endpoint reached *through a module* can declare `requires` and not be
gated by the host today. Everything on this page is about a linked-in endpoint, where the
kernel resolving the name is the kernel enforcing the floor. That is the 95% case, and it
is the case this book stands you in.

## Try it

```rust
# extern crate building_endpoints;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Request, Verb};

let kernel = building_endpoints::kernel_in(None);
let counter = Iri::parse("urn:iki:tutorial:counter").unwrap();
let reader = Capability::root().attenuate(["urn:cap:kernel:inspect"]);

// Reads under a narrow capability.
let repr = block_on(kernel.issue(Request::new(Verb::Source, counter.clone()), &reader)).unwrap();
assert_eq!(String::from_utf8_lossy(&repr.bytes), "0");

// Writes do not: the kernel refuses before the endpoint runs.
let write = Request::new(Verb::Sink, counter)
    .with_arg("content", ArgRef::Inline(b"3".to_vec()));
let err = block_on(kernel.issue(write, &reader)).unwrap_err();
assert!(matches!(err, Error::Denied(_)));
assert!(err.to_string().contains("urn:cap:tutorial:counter:write"));
```
