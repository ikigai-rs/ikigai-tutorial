# One wire, three languages

Everything before this part was Rust, and the model never depended on it. A resolution is
a request and a representation crossing a boundary, and the boundary is a wire protocol
with a version number: `ikigai-ipc` speaks it over a Unix socket in Rust, and so do two
other implementations of the same codec — [`ikigai-python`](https://github.com/ikigai-rs/ikigai-python)
(pure standard library, no dependencies) and [`ikigai-deno`](https://github.com/ikigai-rs/ikigai-deno)
(zero-dependency TypeScript). Each is both halves at once: a **client** that drives a
running kernel, and a **servable peer** that a Rust host mounts as a space.

That pairing is called **L0** of the polyglot ladder, and the name is honest about what it
is. A Python or TypeScript process can resolve any name a kernel binds, be described,
be cached and traced by that kernel, and be refused by its capabilities. What it cannot do
is what a Rust endpoint does with `inv.source(..)` — reach back into the kernel from
inside its own invocation. An L0 peer is a leaf: it answers, it does not compose. There is
no back-channel yet, and the two tracks below say so where it bites rather than pretending
the ladder is taller than it is.

## The two tracks

Both tracks mirror Part I chapter for chapter, in a language with no kernel in it:

| Part I | in Python and TypeScript |
|---|---|
| [Resolution](../getting-started/resolution.md) | `connect()` and `source()`: a name resolved over the wire, a representation back |
| [Hello, resource](../getting-started/hello-resource.md) | an endpoint as a decorated function, served on a socket |
| [Why an endpoint describes itself](../getting-started/self-description.md) | the signature is the contract: ArgSpecs derived from annotations, `describe()` as data |
| [Binding, and a host of your own](../getting-started/binding.md) | `ikigai --mount urn:py:=<socket>` — a mount is a binding, and the host owns the name |
| [What resolution buys you](../getting-started/payoff.md) | `is_cached`, `source_traced`: the kernel's cache and trace, seen from outside |
| capabilities | a scoped `connect()`, and a typed `DeniedError` |

Then a [notebook](notebook.md): the catalog as a graph, in rdflib, with one SPARQL query.

## What you need

A running kernel to talk to — `ikigai serve <socket>`, from the CLI the [front
door](../front-door/cli.md) installed — and one of the two client packages. Neither is
published to PyPI or JSR yet; both install from a checkout (`pip install <path>`, or a Deno
import by path), and the listings in this part say so where it matters. The listings
themselves live in this repository under `examples/`, modeled on the sibling repositories'
own examples (read-only for this book) so that the book builds from a clone of this
repository alone; every one of them was run against `ikigai-cli` 0.1.18 as it was written.

> ⚠ Names under `urn:py:` and `urn:ts:` in this part are served by *your* peer process and
> resolve only while it runs and is mounted. They are not in the CLI, and the book's URN
> gate declares them as such rather than claiming otherwise.

<!-- urn-gate: illustration urn:py: — the family the Python peer in examples/python
     serves; it exists only while that process runs and a host mounts it. -->
<!-- urn-gate: illustration urn:ts: — the family the Deno peer in examples/deno serves;
     likewise. -->
