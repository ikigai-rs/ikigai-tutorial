# Twenty minutes, nothing compiled

Two front doors, and this is the second. The first is already open: every **Run** button
in this book resolves against a kernel compiled into the page — [Running
it](../getting-started/running-it.md) says how — and it binds only Part I's names. This
chapter is the other door: the full host, `ikigai`, on your machine, with everything it
links. Eight commands, no Rust, and by the end you have seen the whole model — a catalog,
a manifold, a pipe, a format change, a cache hit, a cut thread, and a trace.

## 0. Install

```bash
cargo install ikigai-cli --locked
ikigai --version
```

> ⚠ The crate is `ikigai-cli`; only the **binary** is called `ikigai`. `cargo install
> ikigai` does not fail — it fetches an unrelated crate by another author. `--locked`
> builds the dependency versions the release was tested with rather than whatever the
> registry resolves today.

Every command below is one-shot: `-c` runs a line and exits, `--plain` drops the
decoration. Run `ikigai` with no arguments for the REPL, which is the same grammar with
a memory — and the sixth command needs that memory, so it says so.

## 1. Everything resolvable

```bash
ikigai --plain -c 'source urn:kernel:catalog'
```

Every endpoint the host binds, describing itself, as one Turtle graph — several hundred
lines on a full install. Nobody wrote this document; it is assembled from each endpoint's
own description, which is why it cannot be out of date.

## 2. Everything *you* may do

```bash
ikigai --plain -c 'source urn:kernel:actions'
```

One IRI per line: the endpoints the capability you hold may invoke. As root that is
nearly everything. It is the same list an agent would be handed as its tools, computed
from the same descriptions, and it narrows as authority narrows — the difference between
this and the catalog is the whole capability model in one diff.

## 3. A pipe

```bash
ikigai --plain -c 'source urn:iki:fn:toUpper in="a b" | urn:iki:fn:reverseList'
```

```text
A B
[2 computed]
```

The first resolution's output became the second's unnamed argument. `[2 computed]` is the
kernel counting: two resolutions, neither served from cache. There is no function call in
that line — two names, resolved in order.

## 4. Another format

```bash
ikigai --plain -c 'describe urn:iki:fn:toUpper text/turtle'
```

`describe` is the `Meta` verb, and the trailing type is `as=`: the same endpoint's
description as a graph rather than as text. Try `text/plain` for the human face. A
resource does not have *a* format; it has whatever the kernel can reach.

## 5. A cache hit

This one needs the REPL, because the cache lives in the process and a `-c` run exits.
Start `ikigai`, then:

```text
ikigai> source urn:iki:fn:toUpper in="a b"
A B
[computed]
ikigai> source urn:iki:fn:toUpper in="a b"
A B
[cached]
ikigai> cache urn:iki:fn:toUpper in="a b"
cached
```

The second answer came from the cache; the endpoint did not run. `cache` asks without
resolving — a probe, not a call.

## 6. Cut a thread

Still in the REPL:

```text
ikigai> sink urn:kernel:cut urn:iki:fn:toUpper
cut urn:iki:fn:toUpper
ikigai> cache urn:iki:fn:toUpper in="a b"
not cached
```

A **golden thread** is what a cached answer hangs from. Writing to a resource cuts its own
thread; `urn:kernel:cut` is the resource for cutting somebody else's, and everything
derived from `toUpper` — directly or through composition — stopped being valid at that
instant, without anyone naming it.

## 7. Which threads have been cut

```text
ikigai> source urn:kernel:threads
threads (cut generations)
  urn:iki:fn:toUpper  gen 1
```

Every thread ever cut in this kernel, with its generation. A cached entry remembers the
generation it was made at; a higher number now means it is stale.

## 8. A trace

```text
ikigai> trace urn:iki:fn:toUpper in="a b"
trace  urn:iki:fn:toUpper
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process

urn:iki:fn:toUpper   toUpper · computed · main · 0ms   → 3b  A B
```

One real resolution, recorded: who asked, under what authority, over what transport, and
then the tree — each node saying whether it was computed or served, on which thread, how
long it took. A composite shows its sub-resolutions as children. This is the same
`issue_traced` the book's tests call, rendered.

## What you have seen

A system that can list what exists and what you may do, resolve names into other names'
arguments, change a representation's format on request, serve an answer without
recomputing it, invalidate precisely rather than by timeout, and show you its own
execution. None of it needed a line of Rust. All of it is what [Resolution](../getting-started/resolution.md)
describes, and Part I is where you build an endpoint that gets every one of these
properties for free.

Every name printed on this page is checked, on every commit, against a written-down
claim about the CLI (`books/ikigai/cli-vocabulary.txt`) — and that file is checked against
a real binary by a test that has to be run by hand, because CI has no `ikigai`. If a
command here fails on a newer CLI, that is where to look first.
