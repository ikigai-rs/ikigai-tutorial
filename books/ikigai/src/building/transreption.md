# Transreption

[Resolution](../getting-started/resolution.md) said a resource does not have *a* format:
it has whatever formats the kernel can reach from what it has. This chapter is where you
find out what "reach" means, by using a transreptor and then writing one.

The code is `crates/building-endpoints`, this part's crate. Its host chains onto Part I's
space, so everything you built there is bound here too, plus the four endpoints these
chapters add:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/lib.rs:space}}
```

## Using one

The `ikigai` CLI links a transreptor between RDF syntaxes, `urn:rdf:transrept`. It
declares what it converts — `describe` it and look for the `transrepts:` line — and it is
driven the way every transreptor is driven: content in through the pipe, the target type
in `as`:

```bash
ikigai --plain -c 'source urn:kernel:catalog | urn:rdf:transrept as=application/ld+json'
```

The catalog came out as Turtle and went in as Turtle; what came back is the same graph as
JSON-LD. Nothing about `urn:kernel:catalog` knows JSON-LD exists. That is the shape:
**the format is a property of the route, not of the resource.**

## What a transreptor declares

An ordinary endpoint's description says what it does. A transreptor's says, in addition,
what it *converts between* — `Description::transreptor(from, to)` — and that declaration
is what the kernel selects on. Two rules follow from how selection works, and they are
worth knowing before you write one:

- A transreptor is **auto-invocable** when its only required inputs are `content` and
  `as`. Those are exactly the two the kernel supplies when it drives a step. Declare a
  third required input — a stylesheet, say — and it is still a transreptor, still
  callable by hand, and never chosen automatically.
- Routing is **one hop, or two through Turtle**. If no transreptor goes directly from
  what you have to what you want, the kernel looks for one that reaches `text/turtle` and
  one that leaves it. Turtle is the pivot, which is why the vocabulary renders to Turtle
  first and everything else second.

## Writing one

The endpoint below turns the ikigai description vocabulary — the Turtle every `Meta`
answers with — into Markdown. It parses real triples rather than lines, so it works on
one endpoint's description or a whole catalog, and it knows nothing about which:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/transreption.rs:impl}}
```

And its description, which is the half that makes it reachable:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/transreption.rs:endpoint}}
```

## The kernel finds it

Bound at `urn:iki:tutorial:markdown`, the transreptor is found by what it declared, not by
its name:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/transreption.rs:selected}}
```

And then the payoff. `camel-case` has never heard of Markdown. Ask for its description
*as* Markdown and the kernel renders Turtle, finds the route, and runs it:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/transreption.rs:meta_as}}
```

```bash
cargo run -p building-endpoints -- --markdown
```

```text
## Camel-case (`camel-case`)

Camel-cases the UTF-8 text supplied in the `in` argument.

- `in` (required) — the text to camel-case
```

Look at what was not written: no `as=text/markdown` branch in `camel-case`, no registry of
formats, no call from the requester to the transreptor. A new format became reachable
from every endpoint in the host by binding one endpoint that declared what it converts.

> ⚠ What the kernel routes today is `Meta`. A `Source` with `as=` reaches the endpoint
> with `as` as an ordinary argument — an endpoint that offers faces (text and Turtle, say)
> reads it and answers, and one that does not ignores it. Routing a `Source` through a
> transreptor chain is what the pipe does explicitly: `source X | urn:rdf:transrept as=…`.
> If you expected `source urn:iki:tutorial:camel-case as=text/markdown` to produce
> Markdown, that is the chapter's one disappointment, and it is stated here so it is not
> discovered at a keyboard.

## Try it

<!-- urn-gate: illustration urn:x — the subject of a hand-written Turtle document fed to
     the transreptor by hand; a graph node in test data, not a resource anything binds. -->

```rust
# extern crate building_endpoints;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

let kernel = building_endpoints::kernel_in(None);

// The transreptor, driven by hand, exactly as the kernel drives it: content plus as.
let turtle = "@prefix ik: <https://ikigai-rs.dev/ns#> .
<urn:x> a ik:Endpoint ; ik:id \"x\" ; ik:title \"Example\" ; ik:summary \"Made up.\" .";
let request = Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:markdown").unwrap())
    .with_arg("content", ArgRef::Inline(turtle.as_bytes().to_vec()))
    .with_arg("as", ArgRef::Inline(b"text/markdown".to_vec()));

let repr = block_on(kernel.issue(request, &Capability::root())).unwrap();
assert_eq!(String::from_utf8_lossy(&repr.bytes), "## Example (`x`)\n\nMade up.\n\n");
```
