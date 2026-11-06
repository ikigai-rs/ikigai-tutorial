# Multi-verb endpoints

In [Transreption](transreption.md), you used `camel-case`'s description to reach a new
format. That endpoint answers one verb, so its flat description is its contract.
`title`, in [What resolution buys you](../getting-started/payoff.md), answers two verbs
and declares an `ActionSpec` for each. Here you will write an endpoint with three
verbs, give each verb its own contract, and declare the authority each needs. The
kernel checks that authority before your code runs.

## Three verbs, three actions

`counter` is a number the host holds. `Source` reads it, `Sink` sets it, `Delete` resets
it. Reading needs no authority; writing and resetting do.

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/multi_verb.rs:endpoint}}
```

The implementation is one closure branching on `inv.request.verb`. The description
uses three `ActionSpec`s because the inputs, output and authority requirements depend
on the verb. Flattening them would make the catalog say `counter` takes `content` — on a read, which it does not — and requires the write scope
— on a read, which it must not.

The per-verb view is what everything above the endpoint consumes:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/multi_verb.rs:actions}}
```

That is the catalog's `ik:Action` nodes, the manifold's rows, and the agent's tool list —
one per verb, each with its own contract. The **manifold** is the list of actions the
current caller may invoke, filtered by its capability; later in the book,
[The machine client](../beyond/agent.md) shows how clients use it. An agent holding a
read-only capability is offered `Source` on `counter` and not `Sink`, because the offer
is computed from exactly this.

## Declared, therefore enforced

Here is the authority rule from [Resolution](../getting-started/resolution.md) in code.
`.requires(WRITE)` on the `Sink` and `Delete` actions is checked by the **kernel**,
before dispatch: a caller whose capability does not satisfy it never reaches the closure.

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/multi_verb.rs:enforced}}
```

Four things in that test, in order. A narrow capability *reads*, because `Source`
declared nothing. The same capability's *write* is refused, with a message naming both
the scope it lacked and the resource that declared it. A capability that holds the scope
writes — and the narrow reader then sees `7`, because the `Sink` cut `counter`'s golden
thread and the reader's cached `0` went with it. And `Delete` follows the same rule as
`Sink`, because it declared the same requirement.

The closure has no capability check. A check there that the description does not
declare would refuse callers the catalog said could proceed. Stating the requirement
in the description lets the kernel use the same rule when it lists available actions,
checks a request before execution, and dispatches it.

> ⚠ The two authoring forms are **exclusive per verb**. Had `counter` also carried a
> flat `.requires(..)` on its description, that scope would be dead for every verb with an
> explicit `ActionSpec` — an explicit action's `requires` *replaces* the flat one, it does
> not add to it. Restate on each action what it needs; [Who is
> asking](../beyond/identity.md) shows the normalized view that decides.

> ⚠ Delete `.requires(WRITE)` from the `Sink` action and run the test. The write goes
> through for the narrow reader, and no test in the crate but this one notices — the
> code is unchanged and correct. **Declared = enforced** means the declaration is the
> enforcement, and a description is code you have to read with that in mind.

## What this is not

It is not a module boundary — though that distinction has narrowed. [Where this actually
stands](../modules/status.md) records that an endpoint reached *through a module* has had
its declared `requires` enforced by the host since `ikigai-module` 0.2.0, and that since
0.3.0 the module's own descriptions cross at connect time, so the host gates against them
exactly as it gates a linked-in one. Everything on this page is about the linked-in case,
where the kernel resolving the name is the kernel enforcing the floor with nothing in
between. This is the host you have been building in these chapters.

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
