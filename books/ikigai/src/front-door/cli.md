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
Start `ikigai`, then write a file into the workspace and read it back twice:

```text
ikigai> sink urn:file:notes.txt remember the milk
wrote 17 bytes to notes.txt
[uncacheable]
ikigai> source urn:file:notes.txt
remember the milk
[computed]
ikigai> source urn:file:notes.txt
remember the milk
[cached]
ikigai> cache urn:file:notes.txt
cached
```

The second read came from the cache; the file was not opened. `cache` asks without
resolving — a probe, not a call. A write is never served from a cache, so the `sink`
says so.

## 6. Cut a thread

Still in the REPL. A **golden thread** is what a cached answer hangs from: the file
endpoint declared that its answer depends on a thread named after the file, and
`urn:kernel:cut` is the resource for cutting a thread by hand:

<!-- transcript: continues -->

```text
ikigai> sink urn:kernel:cut urn:file:notes.txt
cut urn:file:notes.txt
[uncacheable]
ikigai> cache urn:file:notes.txt
not cached
```

The cached read stopped being valid at that instant, and so would anything derived from
it — a composite that had read the file inherits its thread. Writing to the file does the
same cut without your help; this is the resource for doing it on somebody else's behalf,
which is what a filesystem watcher does when a file changes out from under the kernel.

Now the part the first draft of this page got wrong. Try the same cut on a pure function:

<!-- transcript: continues -->

```text
ikigai> source urn:iki:fn:toUpper in="a b"
A B
[computed]
ikigai> sink urn:kernel:cut urn:iki:fn:toUpper
cut urn:iki:fn:toUpper
[uncacheable]
ikigai> cache urn:iki:fn:toUpper in="a b"
cached
```

Still cached. A cut invalidates the entries whose **declared threads** include the one
you cut, and `toUpper` — a pure function of its arguments — declared none: same input,
same answer, forever, and no thread to hang that on. The entry is valid until the kernel
forgets it. Nothing was named wrongly here; there was simply nothing to invalidate, and
`urn:kernel:cache` shows exactly that:

<!-- transcript: continues -->

```text
ikigai> source urn:kernel:cache
cache
  entries  3
  urn:file:notes.txt  text/plain                     17 B  1 thread
  urn:iki:fn:toUpper  application/json              268 B  0 threads
  urn:iki:fn:toUpper  text/plain                      3 B  0 threads
```

Two entries for `toUpper` — its description, fetched once to route the arguments, and the
answer — each with **0 threads**. The file's entry has **1 thread**, and it is still
listed even though section 6 cut it and the probe said `not cached`: a cut bumps a
generation and nothing more, and a stale entry is evicted lazily, the next time
something *resolves* the name. `cache` only looks. Read the file again and the entry is
replaced.

## 7. Which threads have been cut

<!-- transcript: continues -->

```text
ikigai> source urn:kernel:threads
threads (cut generations)
  urn:file:notes.txt  gen 2
  urn:iki:fn:toUpper  gen 1
```

Every thread ever cut in this kernel, with its generation — the file's twice (once by the
`sink` that wrote it, once by hand), `toUpper`'s once. A generation bumps whether or not
anything depended on the thread: cutting is cheap and blind, and validity is decided
lazily, when an entry is next looked up, by comparing the generation it was made at with
the one now.

## 8. A trace

<!-- transcript: continues -->

```text
ikigai> trace urn:iki:fn:toUpper in="a b"
trace  urn:iki:fn:toUpper
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process

urn:iki:fn:toUpper   toUpper · cached · main · 0ms   → 3b  A B
```

One real resolution, recorded: who asked, under what authority, over what transport, and
then the tree — each node saying whether it was computed or served (served, here: the
cut in section 6 did not touch it), on which thread, how long it took. A composite shows its sub-resolutions as children. This is the same
`issue_traced` the book's tests call, rendered.

## What you have seen

A system that can list what exists and what you may do, resolve names into other names'
arguments, change a representation's format on request, serve an answer without
recomputing it, invalidate precisely — what declared the thread, nothing else, and never by
timeout — and show you its own execution. None of it needed a line of Rust. All of it is what [Resolution](../getting-started/resolution.md)
describes, and Part I is where you build an endpoint that gets every one of these
properties for free.

Every name printed on this page is checked, on every commit, against a written-down
claim about the CLI (`books/ikigai/cli-vocabulary.txt`) — and that file is checked against
a real binary by a test that has to be run by hand, because CI has no `ikigai`. Every
transcript on this page is replayed against a real binary by another
(`cargo test -p book-urns --test book_transcripts -- --ignored`), which is how the first
draft's section 6 was caught claiming a cut had invalidated something it had not. If a
command here fails on a newer CLI, those two are where to look first.
