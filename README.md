# ikigai-tutorial

Onboarding material for [ikigai](https://github.com/ikigai-rs) — a resource-oriented
computing kernel in Rust. Books, worked examples, and the code they teach.

## The books

| book | covers |
|---|---|
| [`books/getting-started`](books/getting-started) | The resolution model, running a kernel, your first endpoint (`toCamel`), self-description, binding, configuration, the file workspace |

Read one:

```bash
cargo install mdbook
mdbook serve books/getting-started --open
```

Run the worked example:

```bash
cargo run -p hello-camel -- "resource oriented computing"
```

```
in  resource oriented computing
out resourceOrientedComputing
```

## The books are tested

There is no separate "keep the docs up to date" chore, because there is a gate:

```bash
cargo build && mdbook test books/getting-started -L target/debug/deps
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

> ⚠ `mdbook test` needs `-L target/debug/deps`, and rustc refuses (E0464) if that
> directory holds two candidates for a crate. `cargo clippy` leaves `.rmeta` beside
> `cargo build`'s `.rlib`, so run the book's test after a plain `cargo build` — which is
> why CI gives the book its own job.

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

- **Loadable modules** — the module ABI, and why a module calls *back* into the host
  mid-invocation where a remote kernel does not. (`ikigai-module` is currently
  "Phase 1: in-process proof", so this book should say what is proven and what is not.)
- **Transports** — embedded, IPC, QUIC, and capability-on-the-wire.
- **Graphs** — RDF, SPARQL, transreption, and the vocabulary.

## License

MIT — see [LICENSE-MIT](LICENSE-MIT).
