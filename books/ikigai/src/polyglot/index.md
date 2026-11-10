# One wire, three languages

Almost everything before this part was Rust ([Tic-tac-toe, VI](../applied/tic-tac-toe-6.md)
borrowed these faces ahead of time), and the model never depended on it. A resolution is
a request and a representation crossing a boundary, and the boundary is a wire protocol
with a version number: `ikigai-ipc` speaks it over a Unix socket in Rust, and so do two
other implementations of the same codec — [`ikigai-python`](https://github.com/ikigai-rs/ikigai-python)
(standard library, and one dependency: `blake3`, for the digests in a served door's contract IRIs) and [`ikigai-deno`](https://github.com/ikigai-rs/ikigai-deno)
(TypeScript, with one dependency, `@std/crypto`, for the same digests, and `zod` only for its optional `./zod` entry point). Each is both halves at once: a **client** that drives a
running kernel, and a **servable peer** that a Rust host mounts as a space.

That pairing is called **L0**, the first level of the polyglot ladder. A client can resolve
names the kernel binds, use its cache and trace, and be refused by its capabilities. A
servable peer supplies descriptions and answers requests the host forwards to it. What a
peer cannot do is what a Rust endpoint does with `inv.source(..)` — reach back into the kernel from
inside its own invocation. An L0 peer is a leaf: it answers, it does not compose. There is
no back-channel yet; the tracks below show where that limits what a peer can do.

## The two tracks

Both tracks revisit the endpoints, bindings, cache, and trace you built in
[What resolution buys you](../getting-started/payoff.md), then cover capabilities and
endpoint families. Each uses a language with no kernel in it:

| Rust chapter or topic | Python and TypeScript equivalent |
|---|---|
| [Resolution](../getting-started/resolution.md) | `connect()` and `source()`: a name resolved over the wire, a representation back |
| [Hello, resource](../getting-started/hello-resource.md) | an endpoint as a decorated function, served on a socket |
| [Why an endpoint describes itself](../getting-started/self-description.md) | the signature is the contract: ArgSpecs derived from annotations, `describe()` as data |
| [Binding, and a host of your own](../getting-started/binding.md) | `ikigai --mount urn:py:=<socket>` — a mount is a binding, and the host owns the name |
| [What resolution buys you](../getting-started/payoff.md) | `is_cached`, `source_traced`: the kernel's cache and trace, seen from outside |
| Capabilities | a scoped `connect()`, and a typed `DeniedError` |
| [Multi-verb endpoints](../building/multi-verb.md) | a **family**: one URI template, a contract per verb (Source, Sink, Delete, Exists), and which door answers a name |

Then a [notebook](notebook.md): the catalog as a graph, in rdflib, with one SPARQL query.
And for a whole application served this way, you have already seen
[the same game in Python and TypeScript](../applied/tic-tac-toe-6.md): a store in each
language under a Rust host, and an app in each language that serves the host's board.

## What you need

A running kernel to talk to — `ikigai serve <socket>`, from the CLI the [front
door](../front-door/cli.md) installed — and one of the two client packages. Neither is
published to PyPI or JSR yet; both install from a checkout (`pip install <path>`, or a Deno
import by path), and the listings in this part say so where it matters. The listings
themselves live in this repository under `examples/`, modeled on the sibling repositories'
own examples (read-only for this book) so that the book builds from a clone of this
repository alone. The client listings were run against `ikigai-cli` 0.1.18 as they were written
(one refusal line shown in the Python track carries a later kernel's wording, and says so).
The steps that start a peer and mount it are replayed against the book's pinned `ikigai-cli`
on every change to this repository, by `scripts/polyglot-tracks.sh`, and each track says which
those are.

> ⚠ Names under `urn:py:` and `urn:ts:` in this part are served by *your* peer process and
> resolve only while it runs and is mounted. They are not in the CLI, and the book's URN
> gate declares them as such rather than claiming otherwise.

<!-- urn-gate: illustration urn:py: — the family the Python peer in examples/python
     serves; it exists only while that process runs and a host mounts it. -->
<!-- urn-gate: illustration urn:ts: — the family the Deno peer in examples/deno serves;
     likewise. -->
