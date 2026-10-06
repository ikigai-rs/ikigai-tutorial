# The gonk Book — design note

Not published: mdbook builds `src/`, and this file is beside it. It records what the book is
for, how it is laid out, and how each kind of example is kept true, so the next arc starts
from decisions rather than re-deriving them.

## What it is

A second book in `ikigai-tutorial`, beside The ikigai Book, published at
`https://ikigai-rs.github.io/ikigai-tutorial/gonk/`. Same style (taught from the outside in,
every claim checked), same gates (`mdbook test`, the `book-a11y` contrast and structure gate,
axe over the built pages, the no-kernel check), so the two cannot drift apart. Brian, 2026-10-06:
"A Gonk tutorial like the overall ikigai tutorial … to capture the different ways it can be
used stand alone."

## The outline: one part per standalone use, each building on the last

| part | chapter | kind of example | status |
| --- | --- | --- | --- |
| A ledger of your own | A ledger of your own | cells (wasm) · Rust (`crates/ledger-host`) · transcripts | **written** |
| | The ledger in a browser | transcripts (curl of the pages, `passkey invite`) · manual (the passkey ceremony, a browser) | outline |
| An agent's ledger | Over the socket | transcripts: `ikigai --mount "urn:iki:ledger:=…/gonk.sock" -c …` against the scratch gonk; needs a pinned `ikigai-cli` beside gonk | outline |
| | Over MCP | transcripts once gonk projects MCP (ledger item 782); *coming* until then | coming |
| The repository browser | Browse roots | transcripts against a scratch git repository the page creates (`git init`, a file, `--browse-root`) | outline |
| | Explain, with a mounted model | Rust: browse in a kernel with a STUB `urn:llm:` space (a deterministic model), so the archive and the version tag are testable; transcript of the banner with `--mount`; manual for a real model | outline |
| Review without a reviewer | One-off review, what a finding is | Rust with the stub model: a review pass minting pending findings, a publish | outline |
| | The audit protocol | prose: why per-commit review is off (`ikigai-devtools/claude/research/review-exp2-2026-10-05/README.md`: frontier per-commit review 0.72 serious bugs per 100k tokens against 1.49–1.99 for unled audits, two auditors per repo) | outline |
| A spec in the repository | OpenSpec in one chapter | prose + an `openspec/` tree committed under `crates/` | outline |
| | An openspec/ tree as a graph | Rust: `ikigai-markdown`'s generic lift plus a MAPPING for OpenSpec (a SPARQL CONSTRUCT + lift profile, found at `urn:markdown:mapping:openspec`), over that tree | outline |
| | Changes and tasks as ledger items | Rust: tasks (`- [ ] 1.1 …`) become ledger items `about` their requirement; `next` over them; cells if the page kernel can carry the markdown lift | outline |
| Bridges | Filing roborev findings | transcripts: `ikigai-gonk roborev file --dry-run` and for real, from a fixture review | outline |
| | Importing a kata ledger | transcripts, as `kata import` lands | coming |
| | Checkout | transcripts, as `checkout` lands (a gonk satellite is building it, 2026-10-06) | coming |
| Several machines | The three doors | transcripts: what each door grants, refusals included | outline |
| | A QUIC client | transcripts: `client add`, then a second scratch process connecting with `--cert-dir`; two scratch homes on one machine | outline |
| Embedding | The ledger, store and browse in your own kernel | Rust: the whole composition gonk makes, minus the doors | outline |

Unwritten chapters are mdbook DRAFT entries (`- [Title]()`), listed in the sidebar without a
link, so the outline is visible on the published site and nothing 404s.

## How each kind is checked

- **Runnable in the page.** `crates/gonk-book-wasm` is the page's kernel:
  `ledger_host::kernel()` (the ledger over an in-memory store, with the story clock) under the
  CLI's engine. It compiles for `wasm32-unknown-unknown` (`ikigai-store` has a wasm face; the
  ledger needs nothing more). pages.yml binds it with `wasm-bindgen --out-name book_wasm` into
  `books/gonk/book/wasm/`, so the ONE `js/run.js` (a symlink to the ikigai book's) loads it
  unchanged. Its test runs every cell of a page natively, in page order, against one fresh
  kernel, and compares the output with the page's (with `…` for minted ids and times).
- **Rust.** `mdbook test` with the workspace's deps, as for the ikigai book. Listings are
  included by anchor from the crate that compiles them.
- **Shell transcripts.** `crates/book-transcripts` + `scripts/test-transcripts.sh` (ledger
  #165). A `console` fence of `$ ` lines is declared `run`, `serve` or `manual — why` in a
  comment above it; under `--strict` an undeclared one FAILS. One page is one session: a fresh
  scratch directory that is the working directory, `HOME` and the config home of every command,
  in page order. A `serve` block's one command is started and left running; its expected lines
  are its startup banner. A command naming 1060 is refused before it runs. pages.yml runs it in
  a `transcripts` job the deploy needs, with the gonk build cached by revision.
  To adopt it for the ikigai book: pin `ikigai-cli` the same way, rewrite its `ikigai> ` text
  blocks as `console` blocks of `$ ikigai -c …`, and add the book to the script. The matcher is
  a superset of the old replay's (whole-line `…` as before, plus `…` inside a line).

## Dependencies, and the temporary pin

gonk is NOT on crates.io. **For now it is pinned to a git revision** (Brian, 2026-10-06), held
in ONE file, `books/gonk/gonk.rev`, which the book prints its install line from
(`{{#include}}`), `scripts/test-transcripts.sh` installs from
(`cargo install --locked --git https://github.com/ikigai-rs/ikigai-gonk --rev <rev>`), and CI
keys its cache on. Moving gonk is a one-line change. ⚠ **This is temporary.** Publishing gonk
is Brian's call; what it would take: the crate is already publishable in shape (MIT OR
Apache-2.0, every dependency a published crate, `version = "0.1.0"`), so it is a `just release
ikigai-gonk` preflight, then a decision about whether a server binary with a launchd story
belongs on crates.io at all (`cargo install ikigai-gonk` would then replace the git line, and
the pin would become a version in the same file).

The book's crates link the PUBLISHED modules at the versions that revision takes:
`ikigai-ledger` 0.3.0, `ikigai-store` 0.2.6 (in-memory, `persistent` off), and `ikigai-browse`
0.18.0 when the browser part is written.

## Layout of the site

One Pages artifact. `scripts/assemble-site.sh` puts each built book where its `book.toml`'s
`site-url` says, relative to the ikigai book's (`/ikigai-tutorial/` → the root,
`/ikigai-tutorial/gonk/` → `gonk/`), so the layout is stated once per book. The books
cross-link relatively (`../index.html` from here, `gonk/` from the ikigai book's "Where to go
next"), and `a11y/links.mjs` checks every relative link in the ASSEMBLED site, so a cross-book
link that would 404 fails the build.

## Shared assets

`css/a11y.css`, `css/run.css`, `js/a11y.js` and `js/run.js` are symlinks into
`books/ikigai/`. One contrast repair and one cell script; `book-a11y` measures this book's own
cascade, so a repair that does not hold here fails here.
