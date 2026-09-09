# Where to go next

Three reference pages close the book: a [glossary](glossary.md) of the nine terms
[Resolution](getting-started/resolution.md) introduced, a [map of every
repository](repositories.md) in the organization, and [how to contribute a
module](contribute.md) of your own. The short list below is the handful this book leaned
on.

## The repositories

- [`ikigai-core`](https://github.com/ikigai-rs/ikigai-core) — the kernel,
  `Description`/`ArgSpec`, capabilities, golden threads
- [`ikigai-fn`](https://github.com/ikigai-rs/ikigai-fn) — the smallest complete example of
  a space, and the one this book chains onto
- [`ikigai-cli`](https://github.com/ikigai-rs/ikigai-cli) — a real host: transports, the
  engine grammar, MCP projection
- [`ikigai-xslt`](https://github.com/ikigai-rs/ikigai-xslt) — one crate showing both the
  linked and the loadable shape side by side
- [`ikigai-dev-server`](https://github.com/ikigai-rs/ikigai-dev-server) — the linkage-gated
  server from Part III, whose `Cargo.toml` is its manifest
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
