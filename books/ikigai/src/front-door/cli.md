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

Now the same cut on a pure function, which declares no thread at all:

<!-- transcript: continues -->

```text
ikigai> source urn:iki:fn:toUpper in="a b"
A B
[computed]
ikigai> sink urn:kernel:cut urn:iki:fn:toUpper
cut urn:iki:fn:toUpper
[uncacheable]
ikigai> cache urn:iki:fn:toUpper in="a b"
not cached
```

Invalidated too. `toUpper` is a pure function of its arguments and declares no golden
thread, but it does not have to: the kernel hangs every cacheable read from a thread named
after the read's own target, so a cut of `urn:iki:fn:toUpper` reaches every answer resolved
through that name, whatever its arguments. (Before core 0.1.73 it did not, and this block
said `cached`: a cut reached only the threads an endpoint declared. [Golden threads in
practice](../getting-started/golden-threads.md#two-holes-closed) tells that story, and why
the change closed a hole.) `urn:kernel:cache` shows what the two cuts left behind:

<!-- transcript: continues -->

```text
ikigai> source urn:kernel:cache
cache
  entries  3 / 4096 (2 stale)
  size     338 B / 64.0 MB
  urn:file:notes.txt  cut      text/plain                     17 B  1 thread    root
  urn:iki:fn:toUpper  live     application/json              318 B  1 thread    root
  urn:iki:fn:toUpper  cut      text/plain                      3 B  1 thread    root
```

The first two lines are the cache's bound — 4096 entries and 64 MiB by default, the
least-recently-used entry going when either is exceeded; [Golden threads in
practice](../getting-started/golden-threads.md) is where a host picks a different one. Then
one row per entry: the name it was resolved from, its **state**, its type and size, how
many threads it hangs from, and the resolution chain it was computed in (`root` here, the
host's own space).

Two rows say `cut`: the file and `toUpper`'s answer, each hanging from its one thread, the
one named after it. They are still listed, and counted as `(2 stale)`, even though the probes
said `not cached`: a cut bumps a generation and nothing more, and a stale entry is evicted
lazily, the next time something *resolves* the name. `cache` only looks, so it marks a stale
entry rather than evicting it. Read the file again and its entry is replaced. The `live` row
is `toUpper`'s description, fetched once to route the arguments; it hangs from the thread
of the host's bindings, not from the name, because a description changes when what is bound
changes, so the cut left it alone.

## 7. Which threads have been cut

<!-- transcript: continues -->

```text
ikigai> source urn:kernel:threads
threads (cut generations)
  urn:file:notes.txt  gen 2
  urn:iki:fn:toUpper  gen 1
```

Every thread ever cut in this kernel, with its generation: `toUpper`'s once, by hand, and
the file's at least twice, once by the `sink` that wrote it and once by hand. The CLI also
watches its workspace and cuts a file's thread for every change event the operating system
reports, its own writes included, and how many events one write produces, and whether they
arrive before your next command, depends on the platform. So the file's count varies: a Mac
printed the `gen 2` above, and a Linux machine running the same lines printed `gen 5`.

A generation bumps whether or not anything depended on the thread: cutting is cheap and
blind, and validity is decided lazily, when an entry is next looked up, by comparing the
generation it was made at with the one now.

## 8. A trace

<!-- transcript: continues -->

```text
ikigai> trace urn:iki:fn:toUpper in="a b"
trace  urn:iki:fn:toUpper
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process

urn:iki:fn:toUpper   toUpper · computed · main · 0ms   → 3b  A B
```

One real resolution, recorded: who asked, under what authority, over what transport, and
then the tree — each node saying whether it was computed or served (computed, here: the
cut in section 6 invalidated the cached answer, so this read made a new one), on which
thread, how long it took. A composite shows its sub-resolutions as children. This is the same
`issue_traced` the book's tests call, rendered.

## What you have seen

A system that can list what exists and what you may do, resolve names into other names'
arguments, change a representation's format on request, serve an answer without
recomputing it, invalidate precisely — what hangs from the cut thread, nothing else, and never
by timeout — and show you its own execution. None of it needed a line of Rust. All of it is what [Resolution](../getting-started/resolution.md)
describes, and Part I is where you build an endpoint that gets every one of these
properties for free.

Every name printed on this page is checked, on every commit, against a written-down
claim about the CLI (`books/ikigai/cli-vocabulary.txt`) — and that file is checked, on
every commit too, against the pinned binary named below. Every transcript on this page is
replayed, on every commit, against a real binary: `ikigai-cli`
{{#include ../../ikigai-cli.version}}, the version pinned in `books/ikigai/ikigai-cli.version`
(`./scripts/test-transcripts.sh` runs it locally). That replay did not always run in CI, and
while it did not, section 6 went on saying a cut left a pure function's answer `cached` long
after core 0.1.73 changed it. If a command here fails on a newer CLI than the pinned one,
the replay is where to look first.
