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
| | The ledger in a browser | transcripts (curl of the pages, the form as `/act`, the cross-site refusal, `passkey invite --ledger --browse read`, the access log read back from a `tee`d file) · manual (the passkey ceremony, a browser) | **written** |
| An agent's ledger | Over the socket | transcripts: `ikigai --mount "urn:gk:=…/gonk.sock" -c …` (alias), the config-home `prefer` line, `cap seal` to a `grants` list, the socket's access lines; `ikigai-cli` pinned beside gonk | **written** |
| | Over MCP | transcripts once gonk projects MCP (ledger item 782); *coming* until then | coming |
| The repository browser | Browse roots | transcripts against a scratch git repository the page creates (`git init`, a commit, a `file`-block `config.toml` root); tree/file/hash/state over the socket, a commit picked up by the watcher without a restart, the signed-out `/browse` notice | **written** |
| | Explain, with a mounted model | Rust (`crates/browse-host`): browse over a scratch directory with a STUB model at `urn:llm:stub:{ask,model}` that counts its calls, so "derive once, then from the archive", the version tag, the content-keyed archive and the cost line are asserted; transcripts: the banner with `--mount` (a socket nothing listens on), `explain-status` answering and `explain` refused `unavailable` over the socket; manual for a real model | **written** |
| Review without a reviewer | One-off review, what a finding is | Rust (`crates/browse-host`, the same stub model): a pass minting two pending findings with stable ids, the archive (no second model call), decline with a reason from the closed set, a `should_panic` reproduction, publish with `reproduced=yes` → an annotation `derived_from` the finding, a decision refused without `revises=`, and the capability split (read / derive / annotate) | **written** |
| | The audit protocol | prose with numbers, cited: round 1 (`claude/research/review-exp-2026-10-03/README.md`: led 0.79 vs unled 1.68 serious per 100k tokens) and round 2 (`review-exp2-2026-10-05/README.md`: frontier per-commit review 0.72 against 1.49–1.99 for unled audits, two auditors per repo), the protocol from `claude/audits/README.md`, and a section on what the evidence is NOT. Both in the private `ikigai-devtools`; the chapter says so | **written** |
| A spec in the repository | OpenSpec in one chapter | prose + the committed tree `crates/openspec-ledger/openspec/` (two capabilities, `board` and `moves`; one change, `add-undo`, with a proposal, a delta spec and tasks), included verbatim; the book's one convention beyond OpenSpec (a task cites its requirement by name in a trailing parenthesis) is stated as the book's | **written** |
| | An openspec/ tree as a graph | Rust (`crates/openspec-ledger`): `ikigai-markdown`'s generic lift plus the mapping at `urn:markdown:mapping:openspec` (`config/markdown/mappings/openspec/mapping.ttl`, a config-home layout: one citation in the lift profile for RFC 2119 keywords, six CONSTRUCTs), lifted per document into the store, asked with `graph-select` over the set of document graphs | **written** |
| | Changes and tasks as ledger items | Rust: two queries over the spec graphs ∪ the ledger graph (`UNFILED`, `FINISHED`) and `sync_ledger` acting on them; items `about` the task and its requirement; check a box → close; archive (a real directory move) → close the rest. No cells: `urn:markdown:mapping:{name}` is not built for wasm, and the page would replay a frozen tree | **written** |
| Bridges | Filing roborev findings | transcripts: a fixture review (gonk's own test fixture, as a `file` block), `--dry-run`, for real, the hook's own invocation filing nothing, a fix-job's prose, a refused ledger; the `[[hooks]]` block | **written** |
| | Importing a kata ledger | transcripts: a fixture export (gonk's test fixture) into a ledger of its own, `--dry-run`, the import, `next` over the imported links, a re-run, a later export converging, what is lost (ledger #774) | **written** |
| | Checkout | transcripts: `file://` scratch remotes, clone, `--write-config`, an update, `--all` with its coverage report, a refusal on a dirty checkout; the timer (manual) | **written** |
| | A fresh machine, end to end | transcripts: checkout → roots → start → kata import → the roborev hook → `next`, in one session | **written** |
| Several machines | The three doors | transcripts: `client add` (the QUIC door opens only with a certificate enrolled), gonk with `--quic-bind 127.0.0.1:1070` (the default QUIC bind is UDP 1060 on every interface, a real gonk's), one request through each door, the access log read back from a `tee`d file (`door=`, `principal=`: `anon`/`owner`/`-`, and `-` on QUIC even for a write), a refusal at the door's floor that writes no line | **written** |
| | A QUIC client | transcripts: `client add laptop --ledger default=write --browse read --root notes`, `grants.json`, the bundle copied to a second home (`laptop/`), `ikigai --connect quic://… --cert-dir`, refusals outside the grant (delete at the floor, another ledger at the ledger), a stranger's certificate refused in the handshake (the client is told only "connection lost"), the server's log; a real second machine is manual | **written** |
| Embedding | The ledger, store and browse in your own kernel | Rust (`crates/embed-host`): one dataset handed to the store and to browse with a `SharerWrites` promise (`in_memory_shared_declaring`) and browse confined to that graph; the stub model; three callers' capabilities (agent, headless reviewer, person) each refused outside its lines; the ledger↔browse join through `graph-select` over both graphs; a cached ledger read asserted with `Kernel::is_cached` | **written** |

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
  Since arc 2 two more things: a fence of ANY language declared `file NAME` is written to
  `NAME` in the scratch directory at that point in the page (how a page hands its commands a
  config file or a fixture, since one `sh -c` line cannot hold a heredoc), and a `run`
  command's standard error goes to the same pipe as its standard output, so lines come back in
  the order a terminal shows them. A `serve` command runs in its own process group, which is
  what is stopped at the end of the page, so a serve line may be a pipeline
  (`ikigai-gonk … 2>&1 | tee gonk.log`, which is how a page reads the access log back).
  To adopt it for the ikigai book: rewrite its `ikigai> ` text
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

⚠ Each revision installs from its OWN target directory (`target/gonk/<rev>/build`). Moving
the pin from 5050879 to 9b182ff with one shared build directory, `cargo install --git --rev`
called the new revision `Fresh` and installed the previous binary, so the check would have gone
on testing the old gonk under the new pin's name.

**`ikigai-cli` is pinned the same way, in `books/gonk/ikigai-cli.version`** (arc 2), and since
arc 3 it is installed **with its `quic` feature** (not a default; without it `--connect quic://…`
is refused), into `target/ikigai-cli/<version>+quic/`, under a CI cache key ending `-quic`. It IS on
crates.io, so the pin is an exact version (`cargo install --locked ikigai-cli --version =X`),
and the file is one line read by the introduction's install line, the script and CI's cache
key. Choose the release whose transports gonk's lock takes (`ikigai-ipc`, `ikigai-wire` and
`ikigai-web` share the cli's version: 0.1.38 at both 9b182ff and 60cf306), so the client speaks the server's
wire; moving gonk is a reason to look at this file too.

The book's crates link the PUBLISHED modules at the versions that revision takes:
`ikigai-ledger` 0.3.0, `ikigai-store` 0.2.6 (in-memory, `persistent` off), and `ikigai-browse`
0.18.0, with `oxigraph` 0.5 named directly because browse takes an `oxigraph::store::Store`.

**`ikigai-markdown` is not on crates.io** (2026-10-06), so the OpenSpec part takes it the way this
book takes gonk: from its public repository at ONE full git revision, in the workspace
`Cargo.toml` (`git = …, rev = …`), never a path or a branch. When it is published that line
becomes a version. **`ikigai-ledger` 0.4.0** (the keyed append) was not published when part 5 was
written either, so part 5 runs on 0.3.0 and makes the sync idempotent with a query
(`FILTER NOT EXISTS { ?item ledger:about ?task }`) instead of a key; the chapter says what that
costs (two concurrent syncs could file a task twice). Moving to 0.4.0 is: bump the workspace pin,
pass `key=<task IRI>` in `sync_ledger`, and rewrite that paragraph.

**No real model, anywhere checked.** A chapter that needs one binds a stub at `urn:llm:` (see
`crates/browse-host`): two names, `urn:llm:stub:ask` and `urn:llm:stub:model`, answering a fixed
explanation or a fixed review by the system prompt. browse cannot tell a stub from a mount, so
everything about the archive and the findings is shown for real; what a real model would SAY is
the one thing not shown, and the chapters mark it `manual`.

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
