# Contribute a module

A module is a crate with a `space()`. This page is the shape of a new one, from the first
commit, in the order the pieces matter. The reference is
[`ikigai-fn`](https://github.com/ikigai-rs/ikigai-fn): one file, eight endpoints, a
README that says what each is for, and a three-line CI — the smallest complete example,
and the crate this book chains onto.

## The crate

```toml
[package]
name = "ikigai-yours"
version = "0.1.0"
edition = "2021"
license = "MIT OR Apache-2.0"
description = "One line saying what resources this offers, as `urn:…` names."
# Until it is published this is the honest setting. Publishing is not yours to do —
# see the end of this page.
publish = false

[dependencies]
# The PUBLISHED kernel, at the version whose API you actually use. Never a path to a
# sibling checkout: a path dependency only resolves on the machine that has both.
ikigai-core = "0.1.66"
```

No OS or platform API unless the module is about one — read files through the kernel
(`urn:file:…`), not `std::fs`; take time from `inv.now()`, not `SystemTime`. A module that
keeps to that compiles to `wasm32-unknown-unknown` for free, and the same crate then
links into the native CLI and into a browser host.

## The endpoints, described from day one

Every endpoint is a function, a description, and a binding, exactly as [Hello,
resource](getting-started/hello-resource.md) built one. The description is not optional
paperwork: the engine routes named arguments by it, the catalog is built from it, and an
agent's tool is projected from it. So, from the first endpoint:

- `ArgSpec` for every argument — `name`, `summary`, `optional()` where it is, `class(..)`
  always: an XSD datatype IRI for a scalar, an `rdfs:Class` for an entity. The one people
  leave off, and the one [The graph face](building/graph-face.md) shows is the point.
- `one_of(..)` for enums, `default_value(..)` where a default exists — and remember that a
  declared default is *not* injected; your code applies it.
- Single verb: author flat. More than one verb: one `ActionSpec` per verb, per
  [Multi-verb endpoints](building/multi-verb.md).
- `.requires("urn:cap:…")` on every action that needs authority, and no authority check
  anywhere else. Declared equals enforced; either half alone is a defect.
- `.cacheable()` only for a pure function of its declared inputs; when in doubt, do not.
- Names: nouns, kebab-case, under a prefix you own. `urn:iki:yours:word-count`, not
  `urn:fn:countWords` in somebody else's namespace.
- Skolemize: no blank nodes in any graph you emit. Stable IRIs make graphs diffable.

<!-- urn-gate: illustration urn:iki:yours:word-count — a stand-in for a name under a prefix
     the contributor owns; nothing binds it, by design. -->
<!-- urn-gate: illustration urn:fn:countWords — the anti-example: a verb-phrase name in a
     library's prefix, shown so the reader does not write one. -->

And `space()`:

```rust,ignore
pub fn space() -> EndpointSpace {
    EndpointSpace::new()
        .bind(Exact::new("urn:iki:yours:word-count"), word_count())
}
```

A host chains onto it. Your crate does not decide where it is mounted.

## The tests

A test builds a kernel over `space()`, resolves a name, and asserts the answer — the
`text_of` helper in `crates/your-endpoints` is the four lines. Test the description too
(`describe().inputs`), test a `.requires` by resolving under an attenuated capability and
asserting `Denied`, and test anything with a clock under `FixedClock` ([Testing an
endpoint hermetically](building/hermetic.md)). If the crate reads a config home, take it
at construction and test against a scratch directory ([Configuration](getting-started/configuration.md)).

## The README is the pitch

Look at `ikigai-fn`'s: a table of endpoints — constructor, conventional IRI, what it
does — and a usage block showing a host chaining onto `space()`. A reader decides whether
to mount your crate from that table. Say what is true about its maturity; a README that
oversells is a bug report waiting to be filed against you.

## CI from the first commit

The organization shares one workflow, and a repository calls it. This is the whole
`.github/workflows/ci.yml` of `ikigai-fn`:

```yaml
name: ci
on:
  push: { branches: [main] }
  pull_request:
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: ${{ github.event_name == 'pull_request' }}   # never cancel main
jobs:
  ci:
    uses: ikigai-rs/.github/.github/workflows/rust.yml@main
    with:
      wasm-lib: true
```

That runs `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo
test` — and, with `wasm-lib: true`, clippy on the wasm target too, which is what catches
a dependency reaching for `std::time`. Run the same three locally *before* the first
push, so CI is never born red. Every change after the scaffold lands through a pull
request with that workflow green.

> ⚠ A gate that silently covers less than it looks like it does is worse than no gate.
> Two to know: `--all-targets` does not see a target behind `required-features`, and a
> committed `Cargo.lock` means CI tests the graph you froze, never the graph a stranger's
> fresh `cargo add` resolves.

## Publishing is not yours

Crates are published to crates.io by the maintainer, in a cascade that bumps the crates
depending on yours. Open the pull request, get it green, and say it is ready; do not
publish, and do not commit a path override to work around a crate that is not published
yet — stop and say so instead. The ecosystem's field guide calls this out because it has
cost real days: a version pin that states the true minimum API you use is what keeps a
stranger's build from resolving stale.

## Where to start

Copy the shape of [`crates/your-endpoints`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates/your-endpoints)
in this repository — it already has one worked endpoint, the test helper, and a host —
then read `ikigai-fn`'s `src/lib.rs` top to bottom. It is shorter than this page.
