# ikigai-tutorial

Onboarding material for [ikigai](https://github.com/ikigai-rs) — a resource-oriented
computing kernel in Rust. Books, worked examples, and the code they teach.

## The book

**[The ikigai Book](books/ikigai)** — one book, in parts.

| part | covers |
|---|---|
| **The front door** | Twenty minutes with the CLI, nothing compiled — catalog, manifold, pipe, `as=`, cache hit, cut thread, trace — then the REPL grammar with a Run button wherever the in-page kernel can answer |
| **Getting started** | The resolution model, running a kernel, your first endpoint (`camel-case`), what resolution buys you (a cache hit, a golden thread cut, a trace, a catalog — each one run), self-description, binding, configuration (a config file the host reads, layered), the file workspace |
| **Building endpoints** | A transreptor of your own and the kernel selecting it, a three-verb endpoint whose declared capability is enforced per verb, SPARQL over the catalog, and a clock-reading endpoint tested under a fixed clock |
| **Loadable modules** | Modules vs. linked-in spaces, the host callback, the wire session, dual-mode crates, and an honest account of what is actually finished |
| **Beyond one host** | A kernel behind a socket, the authority a certificate mints, the three things a mount can mean, an editor and a machine as clients — and three exercises against all of it |
| *end matter* | A glossary, a map of every repository in the organization (derived from the org's list, not from memory), and how to contribute a module |

Read it:

**<https://ikigai-rs.github.io/ikigai-tutorial/>** — published from `main` on every merge,
by the same run that tests it (below), so what is at that URL is a book that passed.

The published book runs **its own kernel in the page**: `crates/book-wasm` is a
wasm-bindgen face over `hello_camel::kernel()` and the CLI's engine, built by `pages.yml`
into `wasm/` beside the book, and `books/ikigai/js/run.js` turns a
`<div class="ikigai-run" data-cmd='…'>` in a chapter into a Run button. The four runs in
"What resolution buys you" resolve in your browser, against the same space the chapter's
tests run against; with the wasm absent a cell shows the listing's output and says so.

Or clone it and serve it locally:

```bash
cargo install mdbook
mdbook serve books/ikigai --open
```

Each part ends in exercises, and each exercise names the file to open, the command to run,
what "right" looks like, and the section to re-read — with a worked hint behind a
disclosure triangle. Hints rather than solutions, deliberately: a full answer becomes the
thing people read *instead of* the exercise, and an exercise whose hint cannot be written
without giving the answer away is an exercise that needed rewriting.

The file every exercise opens is in **`crates/your-endpoints`** — the reader's crate.
Nothing in the book includes a listing from it (a test enforces that), so it can be
broken freely; the crates the chapters quote stay as the book describes them.

Parts rather than separate books, deliberately: one navigation tree, one search index, and
cross-references between them that actually resolve. Two mdbook projects cannot link to
each other without hand-built relative paths into each other's output directories — which
is exactly as brittle as it sounds, and was already broken here before this was merged.

Run the worked example:

```bash
cargo run -p hello-camel -- "resource oriented computing"
cargo run -p hello-camel -- --catalog
cargo run -p loadable-module
```

```
in  resource oriented computing
out resourceOrientedComputing

@prefix ik: <https://ikigai-rs.dev/ns#> .
<urn:ikigai:endpoint:toUpper> a ik:Endpoint ;
    …

host    resolves urn:greet:hello name=urn:host:name
module  asks the host for urn:host:name
out     Hello, Peter!
```

The tutorial host is built with `ikigai-vocab`'s `TurtleRenderer`, which is what lets it
print that catalog — and answer `Meta` — rather than `no Meta renderer configured`. The
four things Part I promises (a cache hit, a golden thread cut, a traced sub-resolution, a
description that is a graph) are tests in `crates/hello-camel/tests/payoff.rs`, included
into the chapter that makes the promise.

## The book is tested

There is no separate "keep the docs up to date" chore, because there is a gate:

```bash
./scripts/test-books.sh
```

`mdbook test` compiles and runs **every Rust block in the book** against the real crates,
and CI runs it on every push — the deploy to the URL above is downstream of that same job
(`.github/workflows/pages.yml`), so a red book never publishes. The longer listings are not
copied into the prose at all — they are pulled in from the crate by anchor:

````markdown
```rust,ignore
{{#include ../../../crates/hello-camel/src/lib.rs:impl}}
```
````

So there is exactly one `camel-case` and it is the one that compiles.

This matters more than it sounds. A tutorial's examples rot silently: the API moves, the
snippet still *looks* right, and the first person to hit it assumes they are the problem.
Two claims in the first draft were wrong about the API — `Description::id` is a
field rather than a method, and `config_home` lives in `ikigai_core::config` rather than
the crate root — and the tests caught both before anyone read them.

> ⚠ Use the script rather than calling `mdbook test` by hand. It needs `-L <deps>`, and
> rustc refuses (E0464) when that directory holds more than one candidate for a crate — a
> shared `target/` collects them, because `cargo clippy` leaves `.rmeta` beside `cargo
> build`'s `.rlib` and a restored CI cache can carry stale ones. The script builds into a
> target directory of its own, wiped first. The error rustc gives says nothing about
> books, so this is worth not rediscovering.

### The names in the prose are tested too

`mdbook test` compiles Rust blocks, which covers less of a tutorial than it sounds. Of the
resource names this book prints, exactly **one** sits in a block it compiles. The rest are
prose, `ignore` blocks, program output, and — the first command a new reader types — a
shell line in a fenced `bash` block, which nothing runs. The book's correctness gate was
thinnest exactly where a beginner's experience is most fragile.

So `cargo test` also runs a URN gate (`crates/book-urns`). It pulls every `urn:` literal
out of the markdown and checks it against the authority that can actually answer for it:

| where the name appears | who confirms it |
|---|---|
| a Rust example | `hello_camel::space()` / `building_endpoints::space()` / `loadable_module::host_space()` / `your_endpoints::module::host_space()` — hosts built right here — or, for `urn:kernel:*`, the kernel built over the first of them, since those names are intercepted rather than bound |
| a runnable cell's `data-cmd` | `hello_camel::space()` alone — the kernel the page runs — plus `urn:kernel:*` |
| an `ikigai …` shell example | `books/ikigai/cli-vocabulary.txt` — the book's written-down claim about the CLI, which is another crate on another release schedule |
| prose | either |

A name that is *supposed* not to resolve declares itself next to the claim it qualifies:

```markdown
<!-- urn-gate: unbound urn:fn:toCamel — the name this endpoint used to be bound at. -->
```

`unbound` is checked in the other direction, so a paragraph saying "this used to be called
X" goes red if somebody binds X again.

> ⚠ What it cannot do is confirm that `cli-vocabulary.txt` is still true of a newer CLI —
> CI has no `ikigai` binary. The test that can is in the same file and marked `#[ignore]`,
> so it announces itself in the test output rather than skipping quietly:
>
> ```bash
> cargo install ikigai-cli --locked
> cargo test -p book-urns -- --ignored --nocapture
> ```

### And the cross-references

A resource name is not the only thing prose asserts about a world outside itself.
`[chapter 6](status.md)` asserts *where to go* and *what it is called there*, and only the
first half is checked by anything. Merging two books into one renumbered every chapter in
the second part; five cross-references went on naming the old numbers while pointing at
exactly the right files, so every link resolved and no link checker had a word to say.

The same test file now numbers `SUMMARY.md` the way mdbook does and checks any surviving
`[chapter N]` against it — and refuses a *bare* `chapter N` outright, because with no
target there is nothing to check it against. Both are fixed the same way, which is why
neither is a nuisance: name the chapter. `[Where this actually
stands](../modules/status.md)` says both halves and a reorder cannot falsify either.

### And the colours

`mdbook` is sound by default in the ways a generator can be — `<html lang>`, a `<main>`, a
labelled `<nav>`, labelled theme buttons, no image without `alt`. Colour is the exception,
because a theme is a palette somebody chose by eye and **nothing measures one**.

So `./scripts/test-books.sh` also runs `crates/book-a11y`, which reads the palette mdbook
just generated plus this book's `additional-css` — in `book.toml`'s order, because that is
the cascade — and measures every pair a reader actually meets with
[`ikigai-a11y`](https://crates.io/crates/ikigai-a11y), the ecosystem's own WCAG crate.
Body text against 4.5:1 (SC 1.4.3), a non-text UI component against 3:1 (SC 1.4.11).

It found **22 pairs below the floor across the five themes**, including the ones that hurt
most in a book like this: on `rust`, the default, a link was 3.67:1 on the page and 2.69:1
inside a blockquote, and inline code — which this book uses in nearly every sentence — was
4.07:1 and 2.99:1. The repairs are in
[`books/ikigai/css/a11y.css`](books/ikigai/css/a11y.css), each with the ratio it started
from in a comment, and the gate is what says they worked. The tool prints the nearest
passing colour along with the fault, so a repair starts from a number rather than a guess:

```bash
cargo run -p book-a11y -- books/ikigai --survey   # every pair, pass or fail
```

That crate reads generated CSS rather than holding a table of hex values on purpose. A
table would be a gate that goes stale the first time mdbook changes a theme, and stays
green while it does.

The other half is a **skip link**, added by `additional-js` rather than by forking the
theme: with a full-book sidebar, a keyboard or screen-reader user otherwise traverses every
chapter link on every page before reaching the words (SC 2.4.1). The same command checks
that the element it aims at is still in the built HTML — mdbook renamed that id once
already (`#content` became `#mdbook-content`), and a skip link pointing at nothing looks
exactly like one that works.

> ⚠ **Left alone, deliberately: two `<h1>`s per page** — the book title in the menu bar and
> the chapter title — which is mdbook's own template, not ours. It is a best practice
> rather than an AA failure, and the two ways to fix it both cost more than it: overriding
> `theme/index.hbs` pins the whole page template, so future mdbook chrome changes stop
> reaching this book silently (which is exactly how the id above moved); or `a11y.js` could
> `aria-hidden` the menu title, which is three lines and no fork but hides a visible
> heading from assistive technology. A decision, not an oversight.
>
> The audit also flagged an `<h2>` ("Keyboard shortcuts") before the first `<h1>`. Measured
> against the accessibility *tree* rather than the HTML, it is not there: mdbook's help
> popup is `display: none` until you press `?`, so it is out of the tree entirely — worth
> recording, because a static scan of the markup will keep reporting it.

## Layout

```
books/                 mdbook sources — prose
crates/                the code each part teaches — compiled, linted, tested
                       (hello-camel, building-endpoints, loadable-module, two-hosts)
crates/your-endpoints  the reader's crate: where the exercises are done, never quoted
crates/book-urns       not a lesson: the prose gates above, and "never quoted" itself
crates/book-a11y       likewise: the contrast gate and the skip link's target
```

## Dependencies are published crates, deliberately

The code depends on `ikigai-core` and `ikigai-fn` from crates.io, **not** on path
references to sibling checkouts. Clone this repository on its own and it builds. A path
reference would silently require the reader to have the whole ecosystem laid out beside
it, which is exactly the friction onboarding material exists to remove.

## Adding to it

A new part is a heading in
[`books/ikigai/src/SUMMARY.md`](books/ikigai/src/SUMMARY.md) and a directory of chapters
beside it. Put the code it teaches in a crate under `crates/` so it is compiled and linted
like anything else, and include that code into chapters by anchor rather than pasting it —
paraphrase is how a tutorial starts lying.

Start a genuinely separate book under `books/` only when it shares no cross-references
with this one; the script and CI already build every book they find.

Planned:

- **Transports** — embedded, IPC, QUIC, and capability-on-the-wire.
- **Graphs** — RDF, SPARQL, transreption, and the vocabulary.

## License

MIT — see [LICENSE-MIT](LICENSE-MIT).
