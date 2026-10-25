# Where to go next

Three reference pages close the book: a [glossary](glossary.md) of the nine terms
[Resolution](getting-started/resolution.md) introduced and three later chapters added, a [map of every
repository](repositories.md) in the organization, and [how to contribute a
module](contribute.md) of your own. The short list below is the handful this book leaned
on.

## The gonk Book

gonk is the work-ledger and repository-browser host built from the pieces this book teaches:
a durable store, the work ledger, the repository browser and ikigai's own transports, composed
into one server. Its book walks through each way to use it on its own, starting with a ledger
of your own that runs in the page: [The gonk Book](gonk/index.html).

## Building ikigai UIs

A view is a resource too. The third book builds a kernel inspector, the cache, the golden
threads and the arrangement this book printed as text, turned into pages that follow the
kernel's cuts while you watch: [Building ikigai UIs](ui/index.html).

## The ikigai Architecture

How the pieces fit: one ledger item carried through the capability spine, the door, the overlay
chain, the economics of golden threads and caching, and the audit axis, with the patterns that
confine shown to be the patterns that serve. Read it after this book's chapters on dynamic spaces,
which the overlay chain is built from: [The ikigai Architecture](architecture/index.html).

[The library](library/index.html) lists all four books, with who each is for, what to read first
and where to start.

## The repositories

- [`ikigai-core`](https://github.com/ikigai-rs/ikigai-core) — the kernel,
  `Description`/`ArgSpec`, capabilities, golden threads
- [`ikigai-fn`](https://github.com/ikigai-rs/ikigai-fn) — the smallest complete example of
  a space, and the one this book chains onto
- [`ikigai-cli`](https://github.com/ikigai-rs/ikigai-cli) — a real host: transports, the
  engine grammar, MCP projection
- [`ikigai-xslt`](https://github.com/ikigai-rs/ikigai-xslt) — one crate showing both the
  linked and the loadable shape side by side
- [`ikigai-gonk`](https://github.com/ikigai-rs/ikigai-gonk) — the linkage-gated server from
  Part III, whose `Cargo.toml` is its manifest (it took over from the retired
  `ikigai-dev-server`)
- [`ikigai-emacs`](https://github.com/ikigai-rs/ikigai-emacs) — the client whose command
  surface is generated from the manifold
- [`ikigai-llm`](https://github.com/ikigai-rs/ikigai-llm) — one front grammar over
  pluggable backends, each still addressable on its own

## A closing note on style

You will notice the comments in this codebase are unusually long, and that they argue
rather than describe. That is deliberate: the code says what it does, so a comment that
repeats it earns nothing. What a comment is *for* is the constraint the code cannot state
— why this is not cached, why this lock is dropped before that call, what broke the last
time somebody did the obvious thing.

Where a comment states the *shape* of something that leaves the process, it wants a test
in the same edit — because a test is the only comment the compiler reads. That rule is why
every example in this book is compiled.
