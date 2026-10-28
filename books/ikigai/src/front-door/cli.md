# Twenty minutes with the CLI

You can try ikigai in two places. The book's **Run** buttons use a kernel running in
your browser, compiled from Rust to WebAssembly. Here, a *kernel* is the part of ikigai
that handles requests for named resources. [Running it](../getting-started/running-it.md)
explains the browser version.

This chapter uses the `ikigai` command on your machine. It has its own kernel and cache,
separate from the page's. You will list resources, pass results between them, request a
different format, reuse a cached answer, invalidate it, and inspect a request's execution.
You do not need to write Rust.

## 0. Install

You need Rust and Cargo, Rust's package manager. If `cargo --version` reports
`command not found`, install the toolchain with [rustup](https://rustup.rs/), then open
a new terminal. You do not need to know Rust to follow this walkthrough.

Cargo compiles the CLI from source, which can take several minutes on the first
installation. Allow about twenty minutes for the walkthrough after installation.

```bash
cargo install ikigai-cli --locked
ikigai --version
```

> ⚠ The crate is `ikigai-cli`; only the **binary** is called `ikigai`. `cargo install
> ikigai` does not fail — it fetches an unrelated crate by another author. `--locked`
> builds the dependency versions the release was tested with rather than whatever the
> registry resolves today.
>
> The other side of `--locked`: it also holds you to that release's flaws. `ikigai-cli`
> 0.1.68 is the first release whose lockfile fixes the `rustls-webpki` advisory
> (RUSTSEC-2026-0382), using 0.103.16 instead of 0.103.15. Install 0.1.68 or later.

The default build supports the commands below. Remote QUIC connections need the optional
`quic` feature. The HTTP server, `serve --http`, needs `web`. The layers HTTPS server,
`--layers-https`, needs both. To include both features, install with:

```bash
cargo install ikigai-cli --locked --features quic,web
```

Feature flags select code at build time. Reinstalling with these flags rebuilds the binary;
it does not add a plug-in to the existing executable. Current releases list which features
were compiled in at the bottom of `ikigai --help`.

The shell examples use `-c` to run a command and exit. For interactive use, `ikigai`
opens the full-screen REPL. `ikigai --plain` selects the line-by-line REPL.
A REPL keeps the process running between commands, so it keeps the cache too.
[5. A cache hit](#5-a-cache-hit) needs that shared session.

In a transcript, `ikigai>` is the prompt the program displays. Type only the command
after it. The other lines are output, not more commands. Inside the REPL, omit the
shell wrapper `ikigai --plain -c '…'` as well.

## 1. Everything resolvable

```bash
ikigai --plain -c 'source urn:kernel:catalog'
```

This prints the catalog: descriptions of the endpoints available in this host, assembled
into one graph. An endpoint is code that handles requests for a resource. Each endpoint
supplies its own description. Turtle is a text format for RDF graphs, whose statements
have a subject, a predicate, and an object.

## 2. Everything *you* may do

```bash
ikigai --plain -c 'source urn:kernel:actions'
```

This prints one resource name per line, limited to endpoints your current permissions
allow you to invoke. The book calls this list the **manifold**. The session starts with
root authority, which allows nearly everything. Narrow the session's capability and the
list gets shorter. An agent can use the same list to discover its available tools.

The names are **IRIs**, Internationalized Resource Identifiers. A URL is one kind of
identifier; the `urn:…` names here use the URN scheme. They identify resources without
requiring a web address. Ikigai routes each name to the endpoint that handles it.

## 3. A pipe

```bash
ikigai --plain -c 'source urn:demo:split "a,b" | urn:iki:fn:toUpper | urn:iki:fn:reverseList'
```

```text
B
A
[3 computed]
```

`split` turns `a,b` into two lines. The pipe passes that whole result to `toUpper`,
producing `A` and `B` on separate lines. The next pipe passes those lines to `reverseList`,
which reverses their order. It reverses newline-separated items, so a single line such as
`a b` would stay in the same order.

`[3 computed]` reports three pipeline stages whose answers were computed. The `source` verb means
"read this resource"; after each pipe it is implied. `demo` is part of the name of a
bundled example endpoint. It is not a special word in the command grammar.

## 4. Another format

```bash
ikigai --plain -c 'describe urn:iki:fn:toUpper text/turtle'
```

`describe` uses the `Meta` verb to request the endpoint's description, including its
arguments and supported operations. `text/turtle` asks for that description as an RDF
graph written in Turtle. Try `text/plain` for a human-readable summary. Both are text;
they organize the description differently. The trailing media type is shorthand for
`as=text/turtle`. The available formats depend on what the endpoint provides and what
the kernel's format converters, called *transreptors*, can produce.

## 5. A cache hit

This one needs the REPL, because the cache lives in the process and a `-c` run exits.
Start `ikigai --plain`, then enter these commands, one line at a time:

```text
sink urn:file:notes.txt remember the milk
source urn:file:notes.txt
source urn:file:notes.txt
cache urn:file:notes.txt
```

Here is the session with prompts and expected output:

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

`sink` writes a real file named `notes.txt` in the CLI's workspace, by default
`~/.ikigai/workspace/`. `[uncacheable]` describes the write operation: it must run each
time you ask. It does not prevent the file's contents from being cached when read.

The first `source` reads the file. The second reuses the cached answer without opening
it. `cache` only checks whether a valid cached answer is available; it neither reads the
file nor adds an answer to the cache.

## 6. Cut a thread

Still in the REPL. A **golden thread** records a dependency that the kernel can invalidate.
The cached file contents depend on the thread named `urn:file:notes.txt`. "Cutting" that
thread tells the kernel that answers depending on it are out of date. Use `sink` to send
the thread's name to the kernel's cut endpoint:

<!-- transcript: continues -->

```text
ikigai> sink urn:kernel:cut urn:file:notes.txt
cut urn:file:notes.txt
[uncacheable]
ikigai> cache urn:file:notes.txt
not cached
```

The file still exists, but its cached answer is now invalid. Any cached result computed
from that answer becomes invalid too. Reading the file again computes and caches a fresh
answer. Cutting does not switch caching off.

Writing through the file endpoint cuts the same thread automatically. The CLI also
watches for filesystem changes, so another program's edit can invalidate the cache.
A host you build yourself must arrange that notification for external changes.

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

This answer is invalidated too. `toUpper` depends only on its arguments, so it declares
no changing external dependency. Even so, the kernel automatically gives every cacheable
`source` result a dependency on a thread named after its target. Cutting `urn:iki:fn:toUpper`
invalidates all cached reads through that name, including ones with different arguments.
You can cut a name even if nothing currently depends on it.

[Golden threads in practice](../getting-started/golden-threads.md#two-holes-closed)
explains this rule's history. To see the entries the cuts left behind, read the cache:

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

The first two lines show the cache's limits: 4096 entries and 64 MiB by default. The
default policy removes the least recently used entry when either limit is exceeded.
[Golden threads in practice](../getting-started/golden-threads.md) shows how a host
configures the cache and how streamed answers count toward its size limit. Since core
0.2.0, a stream that can only be read once, a *one-shot body*, is never cached. An answer
stored on disk counts its full length toward the limit. A cache hit shares the stored
bytes rather than copying them.

Each row lists an entry's resource name, state, format, size, dependency count, and
resolution chain. `root` means the host's own space.

The file contents and `toUpper`'s answer say `cut`. They remain in memory but are no
longer usable, which is why the earlier probes said `not cached`. A cut increments a
version counter, called a generation. It does not immediately remove every affected
entry. The next read detects the old generation and replaces the stale answer. A cache
probe only reports that state.

The `live` row is `toUpper`'s description, which the engine read to route its arguments.
It depends on the host's bindings, because changing a binding can change the endpoint's
description. Cutting the thread for `toUpper`'s results leaves this description valid.

## 7. Which threads have been cut

The relevant rows from this session are shown below. Other file threads may appear,
including temporary files the CLI used while writing. `…` marks omitted rows.

<!-- transcript: continues -->

```text
ikigai> source urn:kernel:threads
threads (cut generations)
…
  urn:file:notes.txt  gen 2
  urn:iki:fn:toUpper  gen 1
```

This lists the threads cut during this kernel's lifetime and their generation counters.
We cut `toUpper`'s once by hand. The file's thread was cut at least twice, once by the
write and once by hand.

The CLI's filesystem watcher can add more cuts, including for the CLI's own writes.
One edit can produce several operating-system events, with different timing on different
platforms. Your file's generation may therefore be higher than `2`. It counts cuts,
not file edits.

A cut increments the counter even when nothing depends on the thread. When the kernel
next looks up a cached entry, it compares the generations recorded with the entry to
the current ones. A mismatch means the answer is stale.

## 8. A trace

<!-- transcript: continues -->

```text
ikigai> trace urn:iki:fn:toUpper in="a b"
trace  urn:iki:fn:toUpper
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process

urn:iki:fn:toUpper   toUpper · computed · main · 0ms   → 3b  A B
```

`trace` executes the request and records how it ran. The header identifies the client,
its permissions, and the connection used. Here, `embedded` means the kernel runs in the
CLI's own process.

The result was `computed` because section 6 invalidated the cached answer. `main` is the
execution thread, not a golden thread. `0ms` is the duration displayed in whole
milliseconds, and `3b` is the size of `A B` in bytes. When an endpoint requests other
resources, the trace shows those requests underneath it.

## What you have seen

You have listed available resources and permitted actions, passed results between
endpoints, requested another format, reused a cached result, invalidated dependent
results, and traced a request. The invalidation shown here reacts to a dependency
changing; it does not need to wait for a timeout. [Resolution](../getting-started/resolution.md)
explains how these operations fit together. Part I shows how to build an endpoint that
uses them.

CI checks this page's resource names against `books/ikigai/cli-vocabulary.txt` and probes
that vocabulary against a real CLI. It also replays the REPL transcripts against
`ikigai-cli` {{#include ../../ikigai-cli.version}}, pinned in
`books/ikigai/ikigai-cli.version`. Contributors can run these checks with
`./scripts/test-transcripts.sh`. That pin identifies the replayed version; a newer CLI
may print different output.

This replay covers the blocks with `ikigai>` prompts. It does not execute shell command
blocks or prompt-free REPL examples.
