# Why an endpoint describes itself

In [What resolution buys you](payoff.md#4-described), you read `camel-case`'s
description as a graph. [Golden threads in practice](golden-threads.md) explained how
cached answers stay valid. Now return to that description: it lets callers discover
what an endpoint accepts and what it can do, without knowing the endpoint in advance.

<a id="the-description-does-four-jobs"></a>

## Four jobs for a `Description`

1. **The engine routes named arguments by it.** `in="hello"` finds the right slot because
   the endpoint declared a slot called `in`.
2. **Selection matches on it.** Finding a transreptor, finding an endpoint for a task, and
   inferring what actions a set of things affords are all the same query against declared
   types.
3. **The catalog is built from it.** `urn:kernel:catalog` and `urn:kernel:actions` are
   assembled out of these descriptions, not maintained separately.
4. **The agent tool list *is* it.** Projected over the Model Context Protocol (MCP), an
   endpoint's description becomes a tool definition. Nobody writes that by hand.

The third job deserves one more sentence, because it looks circular and is not. The
catalog is the arrangement describing itself — the kernel's own bindings, rendered from
the same descriptions — and it is one resource among the resources, reached by a name
like any other. A reader who can resolve `urn:kernel:catalog` has everything needed to
learn what else can be resolved, including the catalog; nothing about that requires
knowing in advance.

An endpoint with a thin description still runs. It is simply invisible to everything above
— like a library function with no signature.

Here is `camel-case`'s, included from the crate:

```rust,ignore
{{#include ../../../../crates/hello-camel/src/lib.rs:endpoint}}
```

## ArgSpecs, from day one

An `ArgSpec` describes one input: its name, whether it is required, and the type of
value it accepts. Here is the `in` argument from `camel-case`'s description:

```rust
# extern crate ikigai_core;
use ikigai_core::ArgSpec;

let spec = ArgSpec::new("in")
    .summary("the text to camel-case")
    .class("http://www.w3.org/2001/XMLSchema#string");
# let _ = spec;
```

- `optional()` — an argument is **required by default**, and this marks the exception. The
  default is the strict one on purpose: forgetting to say "required" should not quietly
  widen what the endpoint accepts.
- `class(…)` — an **XSD datatype IRI** for a scalar, or an **`rdfs:Class`** for an entity.
  This is the one people leave off, and it is the one that matters most.
- `one_of(…)` for enums, `default_value(…)` where there is one.

## Why `class` is the interesting field

Because it turns "what can I do with these things?" into a query.

If endpoints declare the *entity types* they consume — this one takes three
`schema:Person`, a `schema:Place` and a `schema:Date` — then given a set of things, the
actions they afford are the endpoints whose required input types are a subset of the types
present. Set containment over the catalog. A SPARQL query.

That is the same mechanism as finding a transreptor from one media type to another, one
level up. It is why the description is not paperwork.

<a id="the-invariant-that-keeps-it-safe"></a>

## Capabilities gate execution

An **affordance** is an action offered for the things available. Choosing actions by
type sounds alarming: whoever asserts types influences what gets offered. A hostile
assertion could surface an action that should not be there.

The defense is already in the model. **Type intersection only *offers* an action;
executing it still requires the actor's capability.** Affordance is type-driven,
authorization is capability-driven, and they are separate gates. Adversarial data can make
the menu wrong. It cannot make the kitchen cook.

<a id="the-one-that-bites"></a>

## Declare the authority you enforce

> ⚠ **Declare the authority your endpoint needs.** The kernel checks declared
> requirements before dispatch, even if the endpoint does not repeat the check. If your
> endpoint checks additional authority it never declared, the catalog over-offers — it
> advertises something that will fail. Parameterized authority (network hosts, filesystem
> paths) declares the wildcard form: `urn:cap:net:*` means "holds some grant under this
> prefix". The endpoint must still check the grant for the specific host or path it uses.

You now have `camel-case`'s description and the reasons to keep it complete.
[Binding, and a host of your own](binding.md) connects that endpoint to a name and
adds the renderer that turns its description into the graph you read.
