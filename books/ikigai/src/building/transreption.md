# Transreption

[Resolution](../getting-started/resolution.md) said a resource does not have *a* format:
it has whatever formats the kernel can reach from what it has. This chapter is where you
find out what "reach" means, by using a transreptor and then writing one.

The code is `crates/building-endpoints`, this part's crate. Its host chains onto Part I's
space, so everything you built there is bound here too, plus four endpoints: `banner`, which
[Configuration](../getting-started/configuration.md) borrowed from this part, and the three these
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
one endpoint's description or a whole catalog, and it knows nothing about which. It is
also, deliberately, not a transreption in the strict sense: it keeps the title, the
summary and the inputs and drops the rest, so the Turtle cannot be recovered from it. That
makes it a *rendering* — a projection — and the description says so:

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
*as* Markdown — with consent to the projection, the next section — and the kernel renders
Turtle, finds the route, and runs it:

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

## Declared, and the planner honors it

A transreptor claims to change form without changing what is represented: the function
is injective, and the caller can treat the output as the same resource in another
syntax. A summary, a thumbnail, an embedding, this chapter's Markdown are not injective.
They are **projections** — different resources that have lost information — and a
projection registered as a transreptor would become a silent hop in a route, handing the
caller less than it asked for under exactly the type it asked for, which no reader of
the type can detect. So the declaration carries the distinction, since core 0.1.77:
`.transreptor(from, to)` claims lossless by default, and `.lossy()` says this edge is
not. It is a declaration, not a checked property — core cannot decide injectivity — but
the planner reads it. Selection plans through lossless edges only, unless the caller
consents; consent *widens* the search without reordering it, so a lossless route still
wins wherever one exists, and every step of a plan reports its own declaration.

What that looks like at the `Meta` face. Ask for Markdown and the kernel finds the route,
sees that its one hop is a projection, and refuses — typed, naming the step and the
consent, never a silent projection and never a substitution that would hide that a route
exists. Ask again with `lossy=allow` and the same route runs:

```rust
# extern crate building_endpoints;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Request, Verb};

let kernel = building_endpoints::kernel_in(None);
let root = Capability::root();
let meta_as_markdown = || {
    Request::new(Verb::Meta, Iri::parse("urn:iki:tutorial:camel-case").unwrap())
        .with_arg("as", ArgRef::Inline(b"text/markdown".to_vec()))
};

// Without consent: refused, and the refusal says which step is lossy and what consents.
let refused = block_on(kernel.issue(meta_as_markdown(), &root)).unwrap_err();
assert!(matches!(refused, Error::Endpoint(_)));
assert_eq!(
    refused.to_string(),
    "endpoint error: meta face `text/markdown` is reachable only through a lossy \
     transreptor (`urn:iki:tutorial:markdown`): a projection loses information and \
     would not describe the endpoint; pass `lossy=allow` to consent"
);

// With consent: the route runs, and what comes back is Markdown.
let consented = meta_as_markdown().with_arg("lossy", ArgRef::Inline(b"allow".to_vec()));
let repr = block_on(kernel.issue(consented, &root)).unwrap();
assert_eq!(repr.repr_type.media_type, "text/markdown");
assert!(String::from_utf8_lossy(&repr.bytes).starts_with("## Camel-case (`camel-case`)\n"));
```

The error is an `Endpoint` error on purpose. The face is not absent, so it is not
`NotFound`; no capability is at issue, so it is not `Denied` — consent is per request,
not per holder; and the arguments are well-formed, so it is not `InvalidArgument`. A
mistyped consent (`lossy=alow`) *is* an `InvalidArgument`, rather than being read as the
default and blamed on the face. At the REPL the same refusal is what
`describe urn:iki:tutorial:camel-case text/markdown` prints.

> ⚠ One thing the declaration still does not do. When **no route at all** reaches the
> type a `Meta` request asked for, the kernel **substitutes** the canonical Turtle rather
> than failing: a description in a form you did not ask for, over no description. The
> type on what comes back says `text/turtle`, so a caller that reads it knows. The
> refusal above is only for a route that exists and is lossy; a route that does not
> exist is a substitution, and that is unchanged.

## A format that is not RDF

Everything above went into Turtle or came out of it. Turtle is the pivot of a two-hop route,
but it is not the only thing a transreptor converts. Any representation to any
representation is the same mechanism: a declaration, and a plan the kernel makes from it.

`ikigai-json` (0.1) is a module with three resources:

- `urn:json:query` runs a jq program over any resource;
- `urn:json:from-csv` reads CSV as JSON;
- `urn:json:to-csv` goes the other way.

`from-csv` declares `.transreptor(["text/csv"], ["application/json"])`, so it is also the
kernel's route from CSV to JSON. Nothing about it involves Turtle.

The data is a small shop's stock, a CSV table the way a spreadsheet exports it. It lives in
the crate `crates/shop`, which a later chapter, [Short names in a space](../applied/prefixes.md),
builds on. The table is an atom: `Source` answers it as `text/csv`, and `Sink` replaces it.

```text
{{#include ../../../../crates/shop/stock.csv}}
```

`urn:json:query` reads its `src` through the kernel, under the caller's capability. When the
answer is not JSON, it asks the kernel for a transreptor chain from what was served to
`application/json` (`Invocation::select_transreptor`, the kernel's planner, by the rules above), runs it, and gives the program the result. jq never sees CSV. Each record becomes an
object keyed by the header row, and every value is a string, because CSV has no types and
the converter does not guess.

The page's kernel links `ikigai-json` and the shop, so these cells run here. Elements from
the collection, one record, and a value from that record:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:shop:stock
source urn:json:query src=urn:iki:tutorial:shop:stock q="[.[].item]"
source urn:json:query src=urn:iki:tutorial:shop:stock q=".[2]"
source urn:json:query src=urn:iki:tutorial:shop:stock q=".[2].count" as=text/plain'>
<pre class="ikigai-run-expected">item,shelf,count
flour,A,12
sugar,A,0
rice,B,30
lentils,B,7
[computed]
[&quot;flour&quot;,&quot;sugar&quot;,&quot;rice&quot;,&quot;lentils&quot;]
[computed]
{&quot;item&quot;:&quot;rice&quot;,&quot;shelf&quot;:&quot;B&quot;,&quot;count&quot;:&quot;30&quot;}
[computed]
30
[computed]</pre>
</div>

A program that yields one result answers that value, and `as=text/plain` answers a single
scalar as bare text, with no quotes. That is the form a template splices, which
[Tic-tac-toe, V](../applied/tic-tac-toe-5.md#the-template-language) comes back to.

> ⚠ A `Source` of the stock with `as=application/json` still answers CSV. As the note above
> says, `as=` on a `Source` is the endpoint's own argument, and the stock has one face.
> The route through the transreptor is planned by whoever asks for JSON. Here that is
> `urn:json:query`, and in the REPL it is a pipe: `source urn:iki:tutorial:shop:stock |
> urn:json:from-csv`.

### Cached, and cut by a write

A query's answer is cached exactly when its read of `src` is, and it hangs from the stock's
thread. The conversion is a step inside it, so a second query over the same table finds the
answer cached. A query that uses a number converts the string itself (`tonumber`):

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:iki:tutorial:shop:stock q="[.[] | select(.count | tonumber > 10) | .item]"
source urn:json:query src=urn:iki:tutorial:shop:stock q="[.[] | select(.count | tonumber > 10) | .item]"'>
<pre class="ikigai-run-expected">[&quot;flour&quot;,&quot;rice&quot;]
[computed]
[&quot;flour&quot;,&quot;rice&quot;]
[cached]</pre>
</div>

Now change the table. The line below reads the stock as JSON and keeps only what is on a
shelf. Then it writes that back as CSV through `urn:json:to-csv` and sinks it to the stock.
The round trip is three resources and a pipe. The `Sink` cuts the stock's thread, as every
`Sink` does ([Golden threads in practice](../getting-started/golden-threads.md)), and both
cached selections above go with it. The first recomputes to the same answer, since flour and
rice are still on their shelves. The list of items, read in the first cell, recomputes without
sugar:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:iki:tutorial:shop:stock q="map(select(.count | tonumber > 0))" | urn:json:to-csv | sink urn:iki:tutorial:shop:stock
source urn:iki:tutorial:shop:stock
source urn:json:query src=urn:iki:tutorial:shop:stock q="[.[] | select(.count | tonumber > 10) | .item]"
source urn:json:query src=urn:iki:tutorial:shop:stock q="[.[].item]"'>
<pre class="ikigai-run-expected">ok
[2 computed · 1 uncacheable]
item,shelf,count
flour,A,12
rice,B,30
lentils,B,7
[computed]
[&quot;flour&quot;,&quot;rice&quot;]
[computed]
[&quot;flour&quot;,&quot;rice&quot;,&quot;lentils&quot;]
[computed]</pre>
</div>

The pipe line counts what it ran: the query and `to-csv` computed, and the `Sink`, which is
never cached. The table came back with CRLF line ends, which is what RFC 4180 writes, and
`from-csv` reads it the same as before. `to-csv` is declared lossy: `36` and `"36"` write the same. The
kernel would not plan it into a route without consent, and it did not have to. The pipe
called it by name.

The same listing in Rust, against the shop's own host:

```rust
# extern crate shop;
use shop::{source, Host, STOCK};

let mut host = Host::start("shop");
let shelved = || {
    source(
        "urn:json:query",
        &[("src", STOCK), ("q", "[.[] | select(.count | tonumber > 10) | .item]")],
    )
};
assert_eq!(host.read(shelved()).unwrap(), r#"["flour","rice"]"#);
assert!(host.is_cached(&shelved()));

// A write to the CSV cuts its thread, and the selection recomputes from the new table.
host.write(STOCK, "item,shelf,count\nflour,A,9\nrice,B,30\n");
assert!(!host.is_cached(&shelved()));
assert_eq!(host.read(shelved()).unwrap(), r#"["rice"]"#);
```

jq here is a sandbox: `ikigai-json` rewrites the program so every step is metered, and bounds
its size, its input, its work, its depth and its output. On a native host a bound reached is
a typed refusal. In this page it is an abort, because WebAssembly cannot unwind, and it stops
the page's kernel until you reload. Edit these cells freely, but keep programs small: a
recursion or a string built a million times over is the way to find that out.

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
