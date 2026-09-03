# ikigai-tutorial

Onboarding material for [ikigai](https://github.com/ikigai-rs) — a resource-oriented
computing kernel in Rust. Books, worked examples, and the code they teach.

## The books

| book | covers |
|---|---|
| [`books/getting-started`](books/getting-started) | The resolution model, running a kernel, your first endpoint (`toCamel`), self-description, binding, configuration, the file workspace |

Read one:

```bash
cargo doc -p ikigai-book-getting-started --open
```

Run the worked example:

```bash
cargo run -p ikigai-book-getting-started -- "resource oriented computing"
```

```
in  resource oriented computing
out resourceOrientedComputing
```

## A book is its rustdoc

There is no separate documentation build. **The prose lives in `//!` module docs, each
chapter is a module, and every example is a doctest** — so `cargo test` compiles and runs
the entire book.

That is a deliberate choice, not a convenience. A tutorial's examples rot silently: the
API moves, the snippet still *looks* right, and the first person to hit it assumes they
are the problem. Here a stale example fails the build. Two claims in the first draft of
book 1 were wrong about the API, and the doctests caught both before anyone read them.

It also matches the house rule the rest of the ecosystem follows — *a doctest is the only
comment the compiler reads.*

## Dependencies are published crates, deliberately

The books depend on `ikigai-core` and `ikigai-fn` from crates.io, **not** on path
references to sibling checkouts. Clone this repository on its own and it builds. A path
reference would silently require the reader to have the whole ecosystem laid out beside
it, which is exactly the friction onboarding material exists to remove.

## Adding a book

Add a member crate under `books/`, and give it a `lib.rs` whose crate docs are the
introduction and whose modules are the chapters. Keep the code the chapter discusses *in*
that chapter — chapters should not paraphrase code that lives somewhere else, because
paraphrase is how a tutorial starts lying.

Planned:

- **Loadable modules** — the module ABI, and why a module calls *back* into the host
  mid-invocation where a remote kernel does not. (`ikigai-module` is currently
  "Phase 1: in-process proof", so this book should say what is proven and what is not.)
- **Transports** — embedded, IPC, QUIC, and capability-on-the-wire.
- **Graphs** — RDF, SPARQL, transreption, and the vocabulary.

## License

MIT — see [LICENSE-MIT](LICENSE-MIT).
