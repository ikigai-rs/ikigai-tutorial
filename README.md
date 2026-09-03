# ikigai-tutorial

Onboarding material for [ikigai](https://github.com/ikigai-rs) — a resource-oriented
computing kernel in Rust. Books, worked examples, and the code they teach.

## The books

| book | covers |
|---|---|
| [`books/getting-started`](books/getting-started) | The resolution model, running a kernel, your first endpoint (`toCamel`), self-description, binding, configuration, the file workspace |
| [`books/loadable-modules`](books/loadable-modules) | Modules vs. linked-in spaces, the host callback, the wire session, dual-mode crates, and an honest account of what is actually finished |

Read one:

```bash
cargo install mdbook
mdbook serve books/getting-started --open
```

Run the worked example:

```bash
cargo run -p hello-camel -- "resource oriented computing"
cargo run -p loadable-module
```

```
in  resource oriented computing
out resourceOrientedComputing

host    resolves urn:greet:hello name=urn:host:name
module  asks the host for urn:host:name
out     Hello, Peter!
```

## The books are tested

There is no separate "keep the docs up to date" chore, because there is a gate:

```bash
./scripts/test-books.sh
```

`mdbook test` compiles and runs **every Rust block in the book** against the real crate,
and CI runs it on every push. The longer listings are not copied into the prose at all —
they are pulled in from the crate by anchor:

````markdown
```rust,ignore
{{#include ../../../crates/hello-camel/src/lib.rs:impl}}
```
````

So there is exactly one `toCamel` and it is the one that compiles.

This matters more than it sounds. A tutorial's examples rot silently: the API moves, the
snippet still *looks* right, and the first person to hit it assumes they are the problem.
Two claims in the first draft of book 1 were wrong about the API — `Description::id` is a
field rather than a method, and `config_home` lives in `ikigai_core::config` rather than
the crate root — and the tests caught both before anyone read them.

> ⚠ Use the script rather than calling `mdbook test` by hand. It needs `-L <deps>`, and
> rustc refuses (E0464) when that directory holds more than one candidate for a crate — a
> shared `target/` collects them, because `cargo clippy` leaves `.rmeta` beside `cargo
> build`'s `.rlib` and a restored CI cache can carry stale ones. The script builds into a
> target directory of its own, wiped first. The error rustc gives says nothing about
> books, so this is worth not rediscovering.

## Layout

```
books/          mdbook sources — prose
crates/         the code each book teaches — compiled, linted, tested
```

## Dependencies are published crates, deliberately

The code depends on `ikigai-core` and `ikigai-fn` from crates.io, **not** on path
references to sibling checkouts. Clone this repository on its own and it builds. A path
reference would silently require the reader to have the whole ecosystem laid out beside
it, which is exactly the friction onboarding material exists to remove.

## Adding a book

Add an mdbook under `books/`, and put the code it teaches in a crate under `crates/` so it
is compiled and linted like anything else. Include code into chapters by anchor rather
than pasting it — paraphrase is how a tutorial starts lying.

Planned:

- **Transports** — embedded, IPC, QUIC, and capability-on-the-wire.
- **Graphs** — RDF, SPARQL, transreption, and the vocabulary.

## License

MIT — see [LICENSE-MIT](LICENSE-MIT).
