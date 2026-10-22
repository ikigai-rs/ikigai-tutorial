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
ikigai-core = "0.1.78"
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
- `one_of(..)` for enums, `default_value(..)` where a default exists. The kernel fills a
  declared default in before dispatch (since `ikigai-core` 0.1.105), so your code reads the
  argument and does not apply the default a second time. A default makes its input
  optional, and `Description::validate` refuses a description that marks a defaulted input
  required anyway.
- Single verb: author flat. More than one verb: one `ActionSpec` per verb, per
  [Multi-verb endpoints](building/multi-verb.md) — and the two forms are **exclusive per
  verb**: an explicit action's `requires` replaces the flat one for that verb rather than
  adding to it, so restate on each action what it still needs.
- `.requires("urn:cap:…")` on every action that needs authority, and no authority check
  anywhere else. Declared equals enforced; either half alone is a defect.
- `.cacheable()` only for a pure function of its declared inputs; when in doubt, do not.
- Names: nouns, kebab-case, under a prefix you own. `urn:iki:yours:word-count`, not
  `urn:fn:countWords` in somebody else's namespace.
- Skolemize: no blank nodes in any graph you emit. Stable IRIs make graphs diffable.
- Pipeline citizenship: a `Sink` or `Delete` that can receive a pipe declares an input
  named `content`, because that is where the engine puts the piped value; a `Source` is
  piped into through its one *required* unnamed argument, so a `Source` with no required
  by-value input cannot be piped into at all — making every intake optional silently
  removes it from every pipeline, and nothing warns.

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

## State how it fails

Since `ikigai-core` 0.1.108, every error has an `Error::class()`. Declare the classes
an endpoint can produce with `.fails(FailureClass::…)` on its `Description`, or on each
`ActionSpec` for a multi-verb endpoint (`ActionSpec.failures` holds the declaration):

```rust
# extern crate ikigai_core;
use ikigai_core::{ActionSpec, FailureClass, Verb};

let action = ActionSpec::new(Verb::Source)
    .fails(FailureClass::Transient)
    .fails(FailureClass::RetryAfter);
assert_eq!(action.admits_failure(FailureClass::Transient), Some(true));
assert_eq!(action.admits_failure(FailureClass::Unknown), Some(false));
```

`Transient` says the same request may succeed now; `RetryAfter` says to wait until a
stated time; `OverBudget` says the input exceeds a bound; `Denied` says the caller lacks
a capability; `Refused` says a remote authority refused the credential presented;
`Permanent` says the request or current state is wrong; `Unknown` says the endpoint
could not classify it. `Permanent` is always implied, and `Denied` is implied beside a
nonempty `requires`. Leaving the declaration unstated makes no promise; it does not
mean the action cannot fail. State what the implementation can return so a consumer
can decide whether to retry, wait, reduce the input or ask for authority.

## The tests

A test builds a kernel over `space()`, resolves a name, and asserts the answer — the
`text_of` helper in `crates/your-endpoints` is the four lines. Then add the one test every
module in the ecosystem runs: [`ikigai-conformance`](https://github.com/ikigai-rs/ikigai-conformance)
walks every endpoint of a kernel and reports every departure from the recipe on this
page at once — a missing class, an undeclared scope, a `Sink` without `content`. It is
what "done" means for a module, and it is a dev-dependency, not a rule you remember. Pin
it as a range you read in both directions: under Cargo's 0.x rules `"0.10.0"` means
`>=0.10.0, <0.11.0`, and most of the ecosystem's adopters sat one minor behind the current
definition of done for weeks without a test saying so.

Version 0.9 added `RECOMPUTE`: cut a never-expiring cached answer's golden threads
and recompute it repeatedly, checking that its bytes agree. A cache hit alone cannot prove that the
computation is repeatable. Version 0.10 added `FAILURES`, which compares observed
failures with the action's declared classes. In 0.10 it is opt-in with
`Suite::check_failures()`; it becomes the default from 0.11. A `"0.10.0"` pin is a
ceiling below that default, so adopting it takes a deliberate version move. An action
that states no classes is reported as unstated, never as a `FAILURES` finding.

`crates/your-endpoints/tests/conformance.rs` runs the suite with `check_failures()` over
`space()`, scoped to its own `word-count` endpoint; the endpoints inherited from
`hello-camel` and `ikigai-fn` are listed separately. It declares `word-count` pure
because its cacheable answer depends only on its input. The test also pins the inherited
list so a new endpoint cannot silently escape the walk, and uses `Suite::fails_under`
to check that non-UTF-8 input produces a `Permanent` failure.

Test the description too
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
