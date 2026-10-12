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
| | The ledger in a browser | transcripts (curl of the pages, the form as `/act`, the SPARQL 1.1 Protocol face of `/sparql` over the union of the caller's graphs and its refusal of a ledger outside the grant, the cross-site write, the foreign `Host` and a read another page LOADS (an image) each a `403` at the edge while a read it NAVIGATES to is answered (gonk PR 100, ledger #880), the anonymous SPARQL budget in the banner, `passkey invite --ledger --browse read`, the access log read back from a `tee`d file) · manual (the passkey ceremony, a browser) | **written** |
| An agent's ledger | Over the socket | transcripts: `ikigai --mount "urn:gk:=…/gonk.sock" -c …` (alias), why a mounted read is `[uncacheable]` at the client, the config-home `prefer` line (`XDG_CONFIG_HOME`: the cli has no config-home flag), `cap seal` to a `grants` list, the socket's access lines, and what the seal does and does not bound — a sealed ledger grant's raw `urn:iki:store:graph-update` to its graph refused over the socket since 77ac767 (gonk PR 98, ledger #899; ed43af3 admitted it) while root still writes raw, and a sealed session still filing an item whose `author` names a passkey (the socket runs no author rule); `ikigai-cli` pinned beside gonk | **written** |
| | Over MCP | transcripts: `client add agent --ledger default=write`, gonk serving the QUIC door on 1070, the bundle and a client-side `grants.json` ceiling (`show: ["ledger-*"]`) in `agent/`, `ikigai mcp --override urn:iki:ledger:=quic://… --cert-dir agent/gonk --grant agent` fed JSON-RPC from `file` blocks (stdin is null for a `run` command): the 31 tool names (eight new with `ikigai-ledger` 0.5.0: item state, the assertions, lifecycles, the doctor; two with 0.6.2: an item's `about`; one with 0.9.0: `count`), sorted with `LC_ALL=C`; a keyed append twice (`existing`), the item read by `key:`, its `author` the client's principal; a passkey-shaped `author`, a hidden `ledger-item__delete` and another ledger refused; the access log naming the client, and a ledger-less call logged as `default` (ledger #927, from ledger #782's verified path) | **written** |
| The repository browser | Browse roots | transcripts against a scratch git repository the page creates (`git init`, a commit, a `file`-block `config.toml` root); tree/file/hash/state over the socket, a commit picked up by the watcher without a restart, the signed-out `/browse` notice | **written** |
| | Explain, with a mounted model | Rust (`crates/browse-host`): browse over a scratch directory with a STUB model at `urn:llm:stub:{ask,model}` that counts its calls, so "derive once, then from the archive", the version tag, the content-keyed archive and the cost line are asserted; transcripts: the banner with `--mount` (a socket nothing listens on), `explain-status` answering and `explain` refused `unavailable` over the socket; manual for a real model | **written** |
| Review without a reviewer | One-off review, what a finding is | Rust (`crates/browse-host`, the same stub model): a pass minting two pending findings with stable ids, the archive (no second model call), decline with a reason from the closed set, a `should_panic` reproduction, publish with `reproduced=yes` → an annotation `derived_from` the finding, a decision refused without `revises=`, and the capability split (read / derive / annotate) | **written** |
| | The audit protocol | prose with numbers, cited: round 1 (`claude/research/review-exp-2026-10-03/README.md`: led 0.79 vs unled 1.68 serious per 100k tokens) and round 2 (`review-exp2-2026-10-05/README.md`: frontier per-commit review 0.72 against 1.49–1.99 for unled audits, two auditors per repo), the protocol from `claude/audits/README.md`, and a section on what the evidence is NOT. Both in the private `ikigai-devtools`; the chapter says so | **written** |
| A spec in the repository | OpenSpec in one chapter | prose + the committed tree `crates/openspec-ledger/openspec/` (two capabilities, `board` and `moves`; one change, `add-undo`, with a proposal, a delta spec and tasks), included verbatim; the book's one convention beyond OpenSpec (a task cites its requirement by name in a trailing parenthesis) is stated as the book's | **written** |
| | An openspec/ tree as a graph | Rust (`crates/openspec-ledger`): `ikigai-markdown`'s generic lift plus the mapping at `urn:markdown:mapping:openspec` (`config/markdown/mappings/openspec/mapping.ttl`, a config-home layout: one citation in the lift profile for RFC 2119 keywords, six CONSTRUCTs), lifted per document into the store, asked with `graph-select` over the set of document graphs | **written** |
| | Changes and tasks as ledger items | Rust: two queries over the spec graphs ∪ the ledger graph (`OPEN_TASKS`, `FINISHED`) and `sync_ledger` acting on them, filing each task with `key=<task IRI>` (the `existing` answer on a re-run shown; four concurrent syncs filing three items is a test, and so are the check-then-append's twelve and a hand-filed item without the key being filed again); items `about` the task and its requirement; check a box → close; archive (a real directory move) → close the rest. No cells: `urn:markdown:mapping:{name}` is not built for wasm, and the page would replay a frozen tree | **written** |
| Bridges | Filing roborev findings | transcripts: a fixture review (gonk's own test fixture, as a `file` block), `--dry-run`, for real, the hook's own invocation filing nothing, a fix-job's prose, a refused ledger; the `[[hooks]]` block | **written** |
| | Importing a kata ledger | transcripts: a fixture export (gonk's test fixture) into a ledger of its own, `--dry-run` (it reads the ledger), the import, `next` over the imported links, a re-run, a dry run of a later export naming the one new comment, that export converging, what is lost (ledger #774) | **written** |
| | Checkout | transcripts: `file://` scratch remotes, clone, `--write-config`, an update, `--all` with its coverage report, a refusal on a dirty checkout; the timer (manual) | **written** |
| | A fresh machine, end to end | transcripts: checkout → roots → start → kata import → the roborev hook → `next`, in one session | **written** |
| Several machines | The three doors | transcripts: `client add --port 1070` (the QUIC door opens only with a certificate enrolled; `--port` makes the printed `--connect` line right), gonk with `--port 1070` (which moves the QUIC door to UDP 1070 too, since gonk PR 93), one request through each door, the access log read back from a `tee`d file (`door=`, `principal=`: `anon`/`owner`/`-` on HTTP and the socket, and the client's `urn:iki:host:gonk:client:<fingerprint>` on every QUIC request), a QUIC write's author filled with that name and a write naming another principal refused, a refusal at the door's floor that writes no line | **written** |
| | A QUIC client | transcripts: `client add laptop --ledger default=write --browse read --root notes`, `grants.json`, the bundle copied to a second directory (`laptop/`), `ikigai --connect quic://… --cert-dir`, refusals outside the grant (delete at the floor, another ledger at the ledger), a stranger's certificate refused in the handshake (the client is told only "connection lost"), the server's log naming the client; a raw store write to the ledger graph refused at the door, then allowed after `--ledger-graph default --force` (the identity kept); `client list`, `--rotate` with no scope flags (the enrollment moves to the new certificate, same grant, since gonk PR 106) with the old bundle refused, `client remove`; a real second machine is manual | **written** |
| Scripts and sentences | Scripts and sentences in gonk | transcripts, the hub's live run of 2026-10-11 (`claude/research/nl-script-demo-2026-10-11.md`, private) as the checked sequence: a STUB model peer (`crates/model-stub`, `model-stub "$PWD/model.sock"`, a `serve` block) mounted by `gonk.mount`; `client add` with `--script 'team-*=publish'`, `--script 'team-*=run'`, `--worker default --script public=run` and `--nl draft`, and `passkey invite` with the same, the grants read back with `jq`; `open-by-label` published public over the socket and run by anonymous HTTP (CSV) and by the worker over QUIC, which claims and cannot close; the anonymous budget at 3 a minute (a cached answer not counted, a write cutting the cache, the refusal's `500`); anonymous drafting refused before any model call (the stub's log); what the drafter reads of the grounding (`parts=`: the actions counted and left out) unfocused and with `focus=` (ledger #1243; a scratch gonk with 48 roots measured in prose: the whole grounding 4.6 MB and refused, the drafter's parts 110 KB and an unfocused draft saved); the demo's draft with its empty preview, its warning, one repair the stub repeats, and the MARK on its first line (ledger #1242); `as=application/json` attempts (a repair) and `as=text/turtle` PROV-O; a draft absent to another holder of the namespace, the owner's catalog; running a draft refused as a draft, for its author and the owner alike (ledger #1261); a QUIC publish refused; the owner's check (the query piped: cli 0.1.52 fills a source's one required argument from a batch's stdin) and publish; a draft refused over a published name. The signed-in HTTP spelling is `manual` (a passkey ceremony) | **written** |
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
  scratch directory that is the working directory of every command, in page order. A `serve` block's one command is started and left running; its expected lines
  are its startup banner. A command naming 1062 or 1060 (ports a live service on this machine
  uses) is refused before it runs. pages.yml runs it in
  a `transcripts` job the deploy needs, with the gonk build cached by revision.
  **How a page names a scratch gonk** (gonk PR 94, ledger #799; in the pin since ed43af3):
  by flags, `ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" …`, and
  `--config-home` on `client` and `passkey invite`, and both on `checkout` (since gonk PR 106,
  ledger #918, in the pin since edb9a3d); never `env HOME=…`. `$PWD`, so a path means the same
  directory in every command (gonk makes a relative home absolute since PR 106, but leaves
  `--socket` as given). A browse root is written with `$PWD` too (`printf … "$PWD"`), because a
  `~` in `config.toml` is the home of whoever runs gonk. One exception, said on its page:
  `ikigai` has no config-home flag, so `socket.md`'s config-home section and `mcp.md` set
  `XDG_CONFIG_HOME` on the command (the checkout pages used to set `HOME` too, until checkout
  took the flags).
  **The check sees a missing flag** (ledger #917, part 2): `HOME` and `XDG_CONFIG_HOME` are NOT
  the page's directory but an empty TRAP beside it, and a page that leaves any file in the trap
  fails, naming the files — which is what a command that forgot `--config-home` or
  `--data-home` does, and what would have landed in a reader's real homes. An empty directory is
  not a write (`ikigai` makes `~/.ikigai/workspace` on every start). A page that truly needs
  `HOME` as its own directory declares it once, with its reason:
  `<!-- transcripts: home — why -->` (plural, a PAGE's declaration; none needs it at 2598192).
  Through 77ac767 the runner set `HOME` to the page's directory for every command, so a forgotten
  flag passed here and was reviewed by eye.
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
`ikigai-web` share the cli's version: 0.1.38 at 9b182ff, 60cf306 and ad0653b; 0.1.41 at ed43af3,
77ac767 and edb9a3d; 0.1.44 at d9c9bcf, 9704b54 and 9f2e89e; 0.1.52 at 2598192), so the client speaks the server's wire; moving gonk is a
reason to look at this file too. (0.1.42 and 0.1.43 were already on crates.io when the pin moved to
edb9a3d, 0.1.45 and 0.1.46 when it moved to d9c9bcf, 0.1.47 when it moved to 9704b54, and 0.1.50 when it
moved to 9f2e89e; the rule takes what gonk links. At 2598192 gonk links 0.1.52, the head when the
pin moved there; 0.1.53 to 0.1.55 changed no wire, so this pin stays with gonk's lock. The ikigai Book's REPL transcripts are a
separate check, `crates/book-urns/tests/book_transcripts.rs`, against a SEPARATE pin,
`books/ikigai/ikigai-cli.version`, which the same script installs into its own root and CI caches
under its own key: this pin follows gonk's lock, that one follows the release the ikigai Book's
pages were last replayed against. That one is 0.1.55 (2026-10-12, ledger #1353), the release the
book's crates lock every cli crate at, `ikigai-engine` (the in-page cells' grammar) among them, and
the replay runs it `--quiet`, which prints only the answers. Moving the ikigai Book's pin to 0.1.52
waited on the replay: from 0.1.48 a `-c` batch whose stdin is not a terminal fetches the contract of a bare
`source` (stdin might fill its one required argument) and caches it, and the replay drove a REPL
block as such a batch, so "Twenty minutes, nothing compiled" section 6's `urn:kernel:cache`
gained two rows an interactive REPL never shows. The replay now types a block's lines into the
REPL on its stdin, the way a reader does (ledger #1304).)

The book's crates link the PUBLISHED modules, at the versions that revision takes unless a gap is
recorded below: at 2598192 gonk links `ikigai-core` 0.1.96, `ikigai-ledger` 0.9.0, `ikigai-store`
0.2.15 (in-memory here, `persistent` off; 0.2.7 is a security floor, ledger #915), `ikigai-browse`
0.18.6 (0.18.2 is a security floor: it never serves `.git`, ledger #1126), `ikigai-script` 0.3.3,
`ikigai-nl` 0.3.0 and the cli crates at 0.1.52, with `oxigraph` 0.5 named directly because browse
takes an `oxigraph::store::Store`. A newer release that gonk does not link yet waits for the pin, so
a transcript and a listing are always answered by the same module, unless the book needs it first.
**There is a version gap NOW, OPEN since 2026-10-12.** The book's crates take each crate's published
head where gonk (2598192, and gonk's own main that day) does not: `ikigai-core` 0.1.98 (0.1.97 is
yanked), `ikigai-ledger` 0.11.0, `ikigai-store` 0.2.19 and the cli crates at 0.1.55 (ledger #1323,
#1353), and `ikigai-nl` 0.4.0 since tutorial PR 111 (ledger #1301). Of what those print, only core
0.1.97's family-scope refusal reaches this book: "Your own ledger", "Over the socket" and "A QUIC
client" show gonk refusing `urn:cap:ledger:delete:*` without the clause 0.1.97 adds (`; any grant
under …`), and each says so in prose. No cell here prints a family refusal, and nl 0.4.0's repeat
mark is shown only by the ikigai Book's stub. The gap closes when the pin moves to a gonk that links
these releases. There were three gaps before it, all CLOSED by moving the pin. From the
morning `ikigai-script` 0.3.3 and `ikigai-nl` 0.3.0 were published (2026-10-11, tutorial PR 108)
until the pin moved to 2598192 (ledger #1299), the book's crates took them while the pinned gonk
(9f2e89e) linked script 0.3.0 and nl 0.2.1, so "Scripts and sentences in gonk" printed the older
answers (`graph=`, an empty preview neither repaired nor marked, a draft's author refused for
`urn:cap:lisp`) and said in prose what the newer releases do. From ledger #1198
(2026-10-11) until the pin moved to 9f2e89e (ledger #1245) the book's crates took `ikigai-ledger`
0.9.0 while the pinned gonk (9704b54) linked 0.6.2, so part 5's walk asked `count` and no transcript
could; the crates' item-count tests then counted through `items`' footer (`3 item(s)`), and since
the move they ask `urn:iki:ledger:count`, the figure part 5 cites. Before that, from the morning
`ikigai-ledger` 0.4.0 was published until the pin moved to ad0653b (gonk PR #86, the keyed
bridges), the book's crates took 0.4.0 while the pinned gonk linked 0.3.0, so no transcript could
use `key=` against gonk. ⚠ When the book's crates must move ahead of gonk again, say so here, in the
introduction, and on the page whose transcripts cannot follow.

**`ikigai-markdown`** is taken at 0.1.0 from crates.io. Until it was published (2026-10-06) it was
pinned the way this book pins gonk, to one full git revision of its public repository, never a
path or a branch; that is the rule for any dependency the book needs before it is published.
**`ikigai-ledger` 0.4.0** (the keyed append) arrived the same day. Part 5 was first written on
0.3.0 and made its sync idempotent with a query (`FILTER NOT EXISTS { ?item ledger:about ?task }`),
which two concurrent syncs defeat; it now passes `key=<task IRI>` to `append`, branches on the
`existing` answer, and the chapter says what the key fixed. Its figure for the old way (four syncs,
twelve items for three tasks) is a test, `check_then_append_syncs_file_every_task_once_per_sync`,
with a barrier between each sync's question and its appends, so the number is pinned rather than
remembered from one run.

**No real model, anywhere checked.** A chapter that needs one binds a stub at `urn:llm:` (see
`crates/browse-host`): two names, `urn:llm:stub:ask` and `urn:llm:stub:model`, answering a fixed
explanation or a fixed review by the system prompt. Where the model must be gonk's own MOUNT
(drafting, in "Scripts and sentences in gonk"), the stub is a PROCESS: `crates/model-stub`'s
`model-stub <socket>` serves `urn:llm:ask` and `urn:llm:config` over `ikigai-ipc`, answering by the
ask the prompt carries and logging every ask, and `scripts/test-transcripts.sh` builds it and puts
it alone on the gonk pages' PATH (`target/model-stub/bin`). browse cannot tell a stub from a mount, so
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
