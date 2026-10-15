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
| An agent's ledger | Over the socket | transcripts: `ikigai --mount "urn:gk:=…/gonk.sock" -c …` (alias), why a mounted read is `[uncacheable]` at the client, the config-home `prefer` line (`XDG_CONFIG_HOME`: the cli has no config-home flag), `cap seal` to a `grants` list, the socket's access lines, and what the seal does and does not bound — a sealed ledger grant's raw `urn:iki:store:graph-update` to its graph refused over the socket since 77ac767 (gonk PR 98, ledger #899; ed43af3 admitted it) while root still writes raw, and a sealed session still filing an item whose `author` names a passkey (the socket runs no author rule); `urn:sparql:select` under the alias (`urn:gk:sparql:select`), its default dataset the union of the graphs the caller may read (two at root, one under a sealed ledger read) and a `graph=` outside the seal refused, a deleted graph and a missing one in the same words (ledger #928); `ikigai-cli` pinned beside gonk | **written** |
| | Over MCP | transcripts: `client add agent --ledger default=write`, gonk serving the QUIC door on 1070, the bundle and a client-side `grants.json` ceiling (`show: ["ledger-*"]`) in `agent/`, `ikigai mcp --override urn:iki:ledger:=quic://… --cert-dir agent/gonk --grant agent` fed JSON-RPC from `file` blocks (stdin is null for a `run` command): the 31 tool names (eight new with `ikigai-ledger` 0.5.0: item state, the assertions, lifecycles, the doctor; two with 0.6.2: an item's `about`; one with 0.9.0: `count`), sorted with `LC_ALL=C`; a keyed append twice (`existing`), the item read by `key:`, its `author` the client's principal; a passkey-shaped `author`, a hidden `ledger-item__delete` and another ledger refused; the access log naming the client, and a ledger-less call logged as `default` (ledger #927, from ledger #782's verified path) | **written** |
| The repository browser | Browse roots | transcripts against a scratch git repository the page creates (`git init`, a commit, a `file`-block `config.toml` root); tree/file/hash/state over the socket, a commit picked up by the watcher without a restart, the signed-out `/browse` notice | **written** |
| | Explain, with a mounted model | Rust (`crates/browse-host`): browse over a scratch directory with a STUB model at `urn:llm:stub:{ask,model}` that counts its calls, so "derive once, then from the archive", the version tag, the content-keyed archive and the cost line are asserted; transcripts: the banner with `--mount` (a socket nothing listens on), `explain-status` answering and `explain` refused `unavailable` over the socket; manual for a real model | **written** |
| Review without a reviewer | One-off review, what a finding is | Rust (`crates/browse-host`, the same stub model): a pass minting two pending findings with stable ids, the archive (no second model call), decline with a reason from the closed set, a `should_panic` reproduction, publish with `reproduced=yes` → an annotation `derived_from` the finding, a decision refused without `revises=`, and the capability split (read / derive / annotate) | **written** |
| | The audit protocol | prose with numbers, cited: round 1 (`claude/research/review-exp-2026-10-03/README.md`: led 0.79 vs unled 1.68 serious per 100k tokens) and round 2 (`review-exp2-2026-10-05/README.md`: frontier per-commit review 0.72 against 1.49–1.99 for unled audits, two auditors per repo), the protocol from `claude/audits/README.md`, and a section on what the evidence is NOT. Both in the private `ikigai-devtools`; the chapter says so | **written** |
| A spec in the repository | OpenSpec in one chapter | prose + the committed tree `crates/openspec-ledger/openspec/` (two capabilities, `board` and `moves`; one change, `add-undo`, with a proposal, a delta spec and tasks), included verbatim; the book's one convention beyond OpenSpec (a task cites its requirement by name in a trailing parenthesis) is stated as the book's | **written** |
| | An openspec/ tree as a graph | Rust (`crates/openspec-ledger`): `ikigai-markdown`'s generic lift plus the mapping at `urn:markdown:mapping:openspec` (`config/markdown/mappings/openspec/mapping.ttl`, a config-home layout: one citation in the lift profile for RFC 2119 keywords, six CONSTRUCTs), lifted per document into the store, asked with `graph-select` over the set of document graphs | **written** |
| | Changes and tasks as ledger items | Rust: two queries over the spec graphs ∪ the ledger graph (`OPEN_TASKS`, `FINISHED`) and `sync_ledger` acting on them, filing each task with `key=<task IRI>` (the `existing` answer on a re-run shown, and that re-run leaving a cached `next` cached, asserted with `Kernel::is_cached` (ledger #929); four concurrent syncs filing three items is a test, and so are the check-then-append's twelve and a hand-filed item without the key being filed again); items `about` the task and its requirement; check a box → close; archive (a real directory move) → close the rest. No cells: `urn:markdown:mapping:{name}` is not built for wasm, and the page would replay a frozen tree | **written** |
| Bridges | Filing roborev findings | transcripts: a fixture review (gonk's own test fixture, as a `file` block), `--dry-run`, for real, the hook's own invocation filing nothing, a fix-job's prose, a refused ledger; the `[[hooks]]` block | **written** |
| | Importing a kata ledger | transcripts: a fixture export (gonk's test fixture) into a ledger of its own, `--dry-run` (it reads the ledger), the import, `next` over the imported links, a re-run, a dry run of a later export naming the one new comment, that export converging, what is lost (ledger #774) | **written** |
| | Checkout | transcripts: `file://` scratch remotes, clone, `--write-config`, an update, `--all` with its coverage report, a refusal on a dirty checkout; the timer (manual) | **written** |
| | A fresh machine, end to end | transcripts: checkout → roots → start → kata import → the roborev hook → `next`, in one session | **written** |
| Several machines | The three doors | transcripts: `client add --port 1070` (the QUIC door opens only with a certificate enrolled; `--port` makes the printed `--connect` line right), gonk with `--port 1070` (which moves the QUIC door to UDP 1070 too, since gonk PR 93), one request through each door, the access log read back from a `tee`d file (`door=`, `principal=`: `anon`/`owner`/`-` on HTTP and the socket, and the client's `urn:iki:host:gonk:client:<fingerprint>` on every QUIC request), a QUIC write's author filled with that name and a write naming another principal refused, a refusal at the door's floor that writes no line | **written** |
| | A QUIC client | transcripts: `client add laptop --ledger default=write --browse read --root notes`, `grants.json`, the bundle copied to a second directory (`laptop/`), `ikigai --connect quic://… --cert-dir`, refusals outside the grant (delete at the floor, another ledger at the ledger), a stranger's certificate refused in the handshake (the client is told only "connection lost"), the server's log naming the client; a raw store write to the ledger graph refused at the door, then allowed after `--ledger-graph default --force` (the identity kept); `client list`, `--rotate` with no scope flags (the enrollment moves to the new certificate, same grant, since gonk PR 106) with the old bundle refused, `client remove`; a real second machine is manual | **written** |
| Scripts and sentences | Scripts and sentences in gonk | transcripts, the hub's live run of 2026-10-11 (`claude/research/nl-script-demo-2026-10-11.md`, private) as the checked sequence: a STUB model peer (`crates/model-stub`, `model-stub "$PWD/model.sock"`, a `serve` block) mounted by `gonk.mount`; `client add` with `--script 'team-*=publish'`, `--script 'team-*=run'`, `--worker default --script public=run` and `--nl draft`, and `passkey invite` with the same, the grants read back with `jq`; `open-by-label` published public over the socket and run by anonymous HTTP (CSV) and by the worker over QUIC, which claims and cannot close; the anonymous budget at 3 a minute (a cached answer not counted, a write cutting the cache, the refusal's `500`); anonymous drafting refused before any model call (the stub's log); what the drafter reads of the grounding (`parts=`: the actions counted and left out) unfocused and with `focus=` (ledger #1243; a scratch gonk with 48 roots measured in prose: the whole grounding 4.6 MB and refused, the drafter's parts 110 KB and an unfocused draft saved); the demo's draft with its empty preview, its warning, one repair the stub repeats, and the MARK on its first line (ledger #1242); `as=application/json` attempts (a repair) and `as=text/turtle` PROV-O; a draft absent to another holder of the namespace, the owner's catalog; running a draft refused as a draft, for its author and the owner alike (ledger #1261); a QUIC publish refused; the owner's check (the query piped: cli 0.1.52 fills a source's one required argument from a batch's stdin) and publish; a draft refused over a published name. The signed-in HTTP spelling is `manual` (a passkey ceremony) | **written** |
| Several teams | Two teams, one gonk | transcripts (ledger #1603), the applied form of the ikigai Book's "A query's authority is its dataset" (linked, not repeated): the stub model and a scratch repository `shop` as a browse root; `client add` for `alpha-agent` (`--ledger alpha=write`), `beta-agent` and `manager` (`--ledger alpha=read --ledger beta=read --browse read`), all three with `--script 'report-*=run'`, the grants read back with `jq`; the owner files a `default` notice and two items per team over the socket, one per team `about` the same file; each agent's `urn:iki:ledger:{ledger}:next` and the other's refused; one SPARQL file piped to `urn:sparql:select` by each identity over QUIC and to anonymous `/sparql` (four answers, the union of each caller's graphs); the manager's cross-team join with `graph=` both ledgers, alpha's agent refused naming beta's token, and the same query with no `graph=` answered EMPTY at the pin (store 0.2.21; 0.2.22 refuses by name; the page says so and tells the reader to name the dataset); a review pass and a published finding over the socket, then a join of both ledgers with browse's annotations by the manager, alpha's agent refused naming the browse graph; `report-alpha-open` published `public=true` at the socket SEALED to the lead's grant (standing in for a passkey), run by alpha's agent and the manager and refused to beta's agent at the graph token (a run grant is not a read grant), `report-everyone` over both ledgers refused at publish (no elevation); the signed-out `/l/alpha` notice, anonymous `/sparql` seeing `default` alone, the public report refused at alpha's graph token (`public` is who may run, not what a run reads); a delete over the socket, `next` without it, quads per graph at root (the graveyard listed) and as the manager (not), `graph=…:deleted` refused, `grants alpha purge` holding the graveyard's read token; `urn:kernel:actions` per identity sorted into files and compared with `comm` (alpha and beta differ only by alpha's report, the ledger tools being `{ledger}` templates; the manager lacks the writer's tools and has 19 of the root's) | **written** |
| Ways of working | Solo on one machine | transcripts: the stub model (`model-stub`, which since this part also answers browse's explain, review and judge asks by their system prompts, at `urn:llm:ask` and `urn:llm:coder:ask`, and names itself at `urn:llm:coder:model`), a scratch repository, a gonk with `--no-quic`; over the socket: file with a body as a comment, `next`, `claim lease=30m`, `renew=true`, one explain derived once (the stub's log shows one ask for two reads), a branch and a commit, a `handback=true` comment with the evidence, the release `from=`, `next` excluding the item as awaiting its dispatcher, the close with a note, the doctor clean · manual (the forge: push, `gh pr create`, CI); **With the starter** (transcripts): the pinned starter cloned and checked out, `new.py --process solo`, `LC_ALL=C ls -A`, the `AGENTS.md` headings, `.gonk/process.toml`, `gonk.py init` over the page's socket (it publishes the sample's four saved queries as the owner) and `status`; the default `solo,workers,spec` and the one refused pair in prose | **written** |
| | Remote workers | transcripts: `client add worker-1 --worker default` and `worker-2`, the grant read back with `jq`, gonk with its QUIC door on 1070, bundles copied to `worker-1/` and `worker-2/`; the dispatcher files two items with briefs piped from `file` blocks (a `-c` batch's stdin fills a content-less sink); over QUIC: `next`, a claim, a second worker's claim refused as a `conflict` and its release of the first's claim `denied` (the claim's principal), a renewal, hand-backs with evidence and releases, a worker's close refused at the grant; over the socket: `next` empty (both awaiting), a close, a `rejects=true` that makes the other ready again, the worker's comment attributed to its principal, the refusals in the access log · manual (a worker machine's agent, worktree and forge); the starter's `tools/worker.py` named as the packaged form, not used; **With the starter** (transcripts): `new.py --process workers` (`tools/worker.py`, `worker/`), headings, `process.toml`, `gonk.py init` over QUIC with `worker-1`'s bundle (the identity and alias rows; the four script publishes `failed` at the worker grant; the enrollment lines elided with `…`) and `status` | **written** |
| | Review every commit | transcripts: the stub model as above, a scratch repository with a `post-commit` hook (a `file` block) running `ikigai-gonk review request`, `gonk.review.space`/`grant`/`arm` and the exact reviewer grant (a `file` block for `grants.json`), gonk ARMED (the banner's `review` line), a commit whose hook prints the request id, a wait on `urn:iki:gonk:review:depth`, the stub's log (one review, one judge), the findings listing, the judge's verdict read with `jq`, a decline with a reason and a publish with `reproduced=yes` over the socket, the annotation `derived_from` the finding, `ikigai-gonk findings file demo` filing the published finding (`--dry-run`, then twice: `filed`, then `already`; key `urn:iki:finding:{id}`) and the item it made (gonk 3edf830; the hand step with `key=finding:<id>` it replaces would have been filed twice, ledger #1472); roborev and kata named as tools that implement the process, linked to the Bridges chapters (hub, 2026-10-13) · manual (the signed-in Queue page); **With the starter** (transcripts): `new.py --process review-every-commit` (`hooks/git/post-commit` + `review-on-commit.py`, the `[review]` table), `core.hooksPath`, `gonk.py init` with the hook `ok` and `review` missing (the page's gonk serves `demo`, not the generated repository; prose says what the root line and a restart add) and `status` | **written** |
| | Hub and spoke | transcripts, gonk primitives only (the authors' private hub tooling is named only as that): a goal and two arcs filed with briefs piped from `file` blocks, `parent` and `blocks` links, the hub's claim on the goal (no lease), `next` offering one arc and saying why the others wait, a spoke's leased claim and a second spoke's `conflict`, a hand-back and release, the hub's close making the blocked arc ready, the second arc the same way, the hub's OWN item handed back to a person with "Merge?" and cleared by the person's comment, the closes that release the hub's claims, an arc's item showing its `parent`, the doctor clean · manual (a spoke's worktree, pull request, CI and self-merge); **With the starter** (transcripts): `new.py --process hub` (`tools/hub.py`, `.gonk/templates/brief.md`), headings, `process.toml`, `gonk.py init` and `status` | **written** |
| | Spec first | transcripts: a scratch repository with an OpenSpec change (`proposal.md` and the spec part's `tasks.md` as `file` blocks) committed as the proposal; the seeding written out as the requests a seeder sends (keyed by the task IRI, `about` the task and its requirement, labeled with the change; a re-seed answering `existing`), `blocks` in `tasks.md` order, `next` one task at a time, `count about=` a requirement, a box checked in the same commit as the work, a close by `item=key:…`, the change archived with `git mv`, the remaining closes, `next` empty. The lift is not in gonk, so the seeding is not gonk's; the chapter links the spec part's Rust walk for the host that sends the same requests; **With the starter** (transcripts): `new.py --process solo,spec` (the combining example: `spec` has no roles), headings, `process.toml` with both profiles, `gonk.py init`, `seed --dry-run` (eight tasks, six `blocks` links, asking gonk nothing) and `status` | **written** |
| | Periodic unled audits | transcripts: a two-file shell repository as the audited code (so a reproduction runs in the page with no compiler), the audit's brief as a keyed item claimed by the coordinator, two detached worktrees at the pinned commit, each auditor's reproduction run (a glob-expanded name; a doubled dash), two findings filed one item each (keyed, `parent` the audit, `about` the file, the reproduction in the body), the report's budget, unread scope and one suspicion as a comment on the audit, `next`, a finding closed with its evidence. What an auditor does with its budget is an agent's, and is described, not run; **With the starter** (transcripts): `new.py --process audits,review-every-commit` refused with its reason, then `--process audits` (`tools/audit.py`, `.gonk/templates/audit.md`), headings, `process.toml`, `gonk.py init`, `audit.py … --dry-run` (the item it would file) and `status` | **written** |
| | Harness engineering | prose: Böckeler's model in the book's words (guides and sensors, computational and inferential, stages, categories, the steering loop, templates), the article cited and one short phrase quoted; transcripts, all against the pinned starter (the chapter IS the starter, so it has no separate "With the starter" section): `new.py --process solo`, the harness files listed, one guide and the sensor that `statedBy` it shown with `awk -v RS=` (a paragraph each, not the whole graph), `harness.py check`, `gonk.py init` (its `harness` and `hook` rows), a rule made to FIRE (`report` without `--evidence`) and the firing-log segment read back with `cat` and `harness.py firings`, `gonk.py harness` over the page's socket (the page's gonk is the owner, which loading the graph needs) with its four answers, a second load adding nothing; the steering loop shown: one "rule with no sensor" (`stage-files-by-name`, its guide shown, a sensor for it described) and one "never fired" (`edit-without-live-claim`, silent because the hook is off; the claim released and the hook run by hand with a Claude Code tool call piped in, so it fires) and the three things "never fired" can mean; the namespace (`https://ikigai-rs.dev/ns/harness#` answered 404 when the page was written, so the starter's `vocabulary.ttl` is named as the definition; say it is served only once a GET answers 200); the authors' private tooling named as running a larger harness the same way, no detail | **written** |
| Embedding | The ledger, store and browse in your own kernel | Rust (`crates/embed-host`): one dataset handed to the store and to browse with a `SharerWrites` promise (`in_memory_shared_declaring`) and browse confined to that graph; the stub model; three callers' capabilities (agent, headless reviewer, person) each refused outside its lines; the ledger↔browse join through `graph-select` over both graphs; a cached ledger read asserted with `Kernel::is_cached` | **written** |

**Several teams sits after "Scripts and sentences"** (ledger #1603): its one chapter uses the
QUIC identities of "Several machines", the review of "Review without a reviewer" and the published
scripts of "Scripts and sentences", so it comes after all three, and before the processes.
**Ways of working sits after "Scripts and sentences" and before "Embedding"** (ledger #1435). Its
chapters are processes that COMPOSE the feature parts: remote workers need the QUIC identities of
"Several machines", review every commit needs the review part and a mounted model, spec first
needs "A spec in the repository", so it goes after every part it links to. Embedding stays last
because it is a different kind of use (no gonk at all), not a process around one. Each chapter has
the same sections in the same order (Who, What runs where, The loop step by step, What holds it,
What it costs and when to choose it, With the starter) so that two chapters can be read side by
side. **With the starter** (ledger #1436's profiles, ikigai-rs/ikigai-gonk-starter PR 26) closes
each one with the same process packaged: the starter's `tools/new.py --process <profile>` run
against the pin below, what the generated repository holds for that profile, and what
`tools/gonk.py init` and `status` say about it against the page's own scratch gonk. The loop above
it stays in gonk's primitives, so the section shows the packaging and teaches nothing the chapter
has not. The guide and
sensor words are Böckeler's, used lightly in those six; **Harness engineering** (ledger #1462) is
the seventh chapter and uses them fully, because its subject is the controls around any of the six
rather than a process of its own. It keeps the five questions, and its loop is the starter's
harness from the first step, so it ends without a "With the starter" section. A chapter on
collaborating across organizations comes later and is not in the outline yet.

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
  A command may carry `{{#include PATH}}` of a ONE-line file, which the runner expands as mdbook
  renders it (`book_transcripts::expand_includes`; a range, an anchor or a longer file is an
  error), so a page can name a pinned revision in a command it runs and the line checked is the
  line the reader sees: the "With the starter" sections' `git -C starter checkout --quiet
  {{#include ../../starter.rev}}`.
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

**The gonk starter is pinned the same way, in `books/gonk/starter.rev`** (ledger #1436's arc), one
full revision of `ikigai-rs/ikigai-gonk-starter`, never a path into a checkout. It was 38f539e (PR 26,
the process profiles) and moved to c7a501a (PR 30, after PR 29) for Harness engineering: PR 29 is
the harness template (`.gonk/harness/`, `tools/harness.py`, the firing log, `gonk.py harness`) and
PR 30 makes `vocabulary.ttl` the namespace's complete public definition, which the chapter names.
The move changed the other six pages' "With the starter" output in three ways and no other: seven
more generated files, `harness.py` in `tools/`, and a `note harness` row after `layout` in `init`. Each "Ways of working" page clones it the way a reader does, `git clone
https://github.com/ikigai-rs/ikigai-gonk-starter starter`, then checks out the pin through an
include (above). `scripts/test-transcripts.sh` clones it ONCE per revision into
`target/gonk-starter/<rev>/` and runs the pages with git's `url.<that clone>.insteadOf` set in the
environment (`GIT_CONFIG_COUNT`/`_KEY_0`/`_VALUE_0`, so no config file lands in any home): seven
pages make no network fetch, and each still checks out exactly the pinned commit, so the redirect
changes where the bytes come from and never which bytes. The clone's own `main` is put at the pin
on every run, so its HEAD is a branch as GitHub's is: a clone of a repository whose HEAD is
detached prints git's detached-HEAD advice whatever `--quiet` says, and once the starter's `main`
moved past the pin (2026-10-14) every "With the starter" page printed it. Run by hand without it, the same pages
clone from GitHub. No CI cache: the clone is a few megabytes and costs seconds. Moving the pin is
the one-line change plus the pages' output where the starter changed it; the introduction prints
the install line from the same file. What the pinned starter does that a reader may trip on, as
the pages show it: its `init` reports a gonk that binds no `urn:nl:sparql` (no model mounted) as
`unknown … could not ask gonk`, because the pinned cli's `describe` of an unbound name exits non-zero
(0.1.56, and 0.1.61 still),
which `gonk.py`'s `offered()` reads as no answer; and over QUIC with a worker grant `init` tries to
publish the sample's saved queries and prints four `failed` rows before `ready.`. Both are the
starter's to change, not this book's, and the pages say what each row means rather than hiding it.

**`ikigai-cli` is pinned the same way, in `books/gonk/ikigai-cli.version`** (arc 2), and since arc 3
it is installed **with its `quic` feature** (not a default; without it `--connect quic://…` is
refused), into `target/ikigai-cli/<version>+quic/`, under a CI cache key ending `-quic`. It IS on
crates.io, so the pin is an exact version (`cargo install --locked ikigai-cli --version =X`), and
the file is one line read by the introduction's install line, the script and CI's cache key. Choose
the release whose transports gonk's lock takes (`ikigai-ipc`, `ikigai-wire` and `ikigai-web` share
the cli's version: 0.1.38 at 9b182ff, 60cf306 and ad0653b; 0.1.41 at ed43af3, 77ac767 and edb9a3d;
0.1.44 at d9c9bcf, 9704b54 and 9f2e89e; 0.1.52 at 2598192; 0.1.55 at 71ca542; 0.1.61 at 3edf830), so
the client speaks the server's wire; moving gonk is a reason to look at this file too. At 3edf830
the pin is 0.1.61, gonk's own transports, and the exception below is closed (ledger #1472's arc,
2026-10-14). 0.1.62 was already on crates.io and the rule took what gonk links; both link
`ikigai-core` 0.1.99 as published, so a socket mount's family-scope refusal reads the same either
way. 0.1.63 (the evening of 2026-10-14) changed no transport crate's source, so the rule still
takes 0.1.61. ⚠ **At 71ca542 the pin was 0.1.56, one release PAST gonk's 0.1.55 transports** (ledger
#1365), the one exception so far. The
reason is what a page prints: a client that mounts gonk's socket checks a family-scope floor in ITS
OWN kernel, so "Over the socket"'s refusal is worded by the client's `ikigai-core`, and only 0.1.56
links core 0.1.98 or later (0.1.55 links 0.1.96: replayed with 0.1.55, that page printed the refusal
without core 0.1.97's clause while "A QUIC client"'s, refused at gonk's door, printed it with).
0.1.56 changed no wire. Its one behavior change is the QUIC keep-alive: a 0.1.56 client pings, and a
0.1.55 door does not answer pings while it resolves, so a QUIC call silent for more than 15 s can be
lost (cli's own changelog: upgrade doors before clients). No checked page has such a call (the stub
answers at once); the introduction says so for a reader with a real model. When gonk takes the
0.1.56 transports this pin is back to following gonk's lock. (0.1.42 and 0.1.43 were already on
crates.io when the pin moved to edb9a3d, 0.1.45 and 0.1.46 when it moved to d9c9bcf, 0.1.47 when it
moved to 9704b54, and 0.1.50 when it moved to 9f2e89e; the rule takes what gonk links. At 2598192
gonk linked 0.1.52, the head when the pin moved there; 0.1.53 to 0.1.55 changed no wire, so this pin
stayed with gonk's lock. The ikigai Book's REPL transcripts are a separate check,
`crates/book-urns/tests/book_transcripts.rs`, against a SEPARATE pin,
`books/ikigai/ikigai-cli.version`, which the same script installs into its own root and CI caches
under its own key: this pin follows gonk's lock (but for the exception above), that one follows the
release the ikigai Book's pages were last replayed against. That one is 0.1.63 (the evening of
2026-10-14, the second update wave, which links core 0.1.103 as the book's crates do; 0.1.62 that
morning; 0.1.56 before it, ledger #1365; 0.1.55 before that, ledger #1353), the release the
book's crates lock every cli crate at,
`ikigai-engine` (the in-page cells' grammar) among them, and the replay runs it `--quiet`, which
prints only the answers. Moving the ikigai Book's pin to 0.1.52 waited on the replay: from 0.1.48 a
`-c` batch whose stdin is not a terminal fetches the contract of a bare `source` (stdin might fill
its one required argument) and caches it, and the replay drove a REPL block as such a batch, so
"Twenty minutes, nothing compiled" section 6's `urn:kernel:cache` gained two rows an interactive
REPL never shows. The replay now types a block's lines into the REPL on its stdin, the way a reader
does (ledger #1304).)

The book's crates link the PUBLISHED modules, at the versions that revision takes unless a gap is
recorded below: at 3edf830 gonk links `ikigai-core` 0.1.99, `ikigai-ledger` 0.11.1, `ikigai-store`
0.2.21 (in-memory here, `persistent` off; 0.2.21 is a security floor, ledger #1426–#1429),
`ikigai-browse` 0.18.8 (0.18.2 is a security floor: it never serves `.git`, ledger #1126),
`ikigai-script` 0.3.3, `ikigai-nl` 0.4.0 and the cli crates at 0.1.61, with `oxigraph` 0.5 named
directly because browse takes an `oxigraph::store::Store`. A newer release that gonk does not link
yet waits for the pin, so a transcript and a listing are always answered by the same module, unless
the book needs it first.

**A version gap is OPEN, by decision** (2026-10-14, the update wave). The hub's wave moved every ikigai pin to
its crate's newest release, ahead of gonk's lock, and the evening's wave moved them again, so the
book's crates take `ikigai-core` 0.1.103, `ikigai-ledger` 0.12.0, `ikigai-store` 0.2.23,
`ikigai-browse` 0.18.9, `ikigai-script` 0.3.6 and the cli crates at 0.1.63, while the pinned gonk (3edf830) links core 0.1.99, ledger 0.11.1, store 0.2.21,
browse 0.18.8, script 0.3.3 and cli 0.1.61. No page prints differently because of it, checked by
replaying every transcript and running every cell and listing: no console reads a ledger item's
JSON face (where 0.12.0 adds `"children": []`), no query names a `GRAPH` outside its set (which
store 0.2.22 refuses rather than answering empty), and no page shows a typed selection or a trace
of a `urn:kernel:*` operation through gonk (core 0.1.102 and 0.1.103). The gap closes when the pin moves to a
gonk that links those releases; until then a page that comes to show one of those differences says
so where it shows it. There were four gaps before this one, all CLOSED by moving the pin; the last
was closed 2026-10-12 (ledger #1365) by moving the pin to 71ca542, which linked core 0.1.99,
ledger 0.11.0, store 0.2.19, browse 0.18.8, script 0.3.3 and nl 0.4.0, as the book's crates then
did.
From tutorial PR 111 and PR 112 (2026-10-12) the book's crates took `ikigai-core` 0.1.98,
`ikigai-ledger` 0.11.0, `ikigai-store` 0.2.19 and `ikigai-nl` 0.4.0 while the pinned gonk (2598192)
linked core 0.1.96, ledger 0.9.0, store 0.2.15 and nl 0.3.0, and the `ikigai` both books replayed
(cli 0.1.55) linked core 0.1.96: "Your own ledger", "Over the socket" and "A QUIC client" showed
gonk refusing `urn:cap:ledger:delete:*` without the clause core 0.1.97 adds (`; any grant under …`)
and said so in prose, and "Scripts and sentences in gonk" printed nl 0.3.0's "no attempt within the
bound of 3 did better" for a draft whose repair repeated attempt 1. Moving the pin also changed what
a mounted peer that cannot be reached says ("the last 1 dial(s) failed; the next is held off for …
more", the cli 0.1.55 transports gonk now links). Before that, from the morning `ikigai-script`
0.3.3 and `ikigai-nl` 0.3.0 were published (2026-10-11, tutorial PR 108) until the pin moved to
2598192 (ledger #1299), the book's crates took them while the pinned gonk (9f2e89e) linked script
0.3.0 and nl 0.2.1, so "Scripts and sentences in gonk" printed the older answers (`graph=`, an empty
preview neither repaired nor marked, a draft's author refused for `urn:cap:lisp`) and said in prose
what the newer releases do. From ledger #1198 (2026-10-11) until the pin moved to 9f2e89e (ledger
#1245) the book's crates took `ikigai-ledger` 0.9.0 while the pinned gonk (9704b54) linked 0.6.2, so
part 5's walk asked `count` and no transcript could; the crates' item-count tests then counted
through `items`' footer (`3 item(s)`), and since the move they ask `urn:iki:ledger:count`, the
figure part 5 cites. Before that, from the morning `ikigai-ledger` 0.4.0 was published until the pin
moved to ad0653b (gonk PR #86, the keyed bridges), the book's crates took 0.4.0 while the pinned
gonk linked 0.3.0, so no transcript could use `key=` against gonk. ⚠ When the book's crates must
move ahead of gonk again, say so here, in the
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
ask the prompt carries and logging every ask (and, since "Ways of working", `urn:llm:coder:ask` and
`urn:llm:coder:model` too, answering browse's explain, review and judge asks by their system
prompts, so a scratch gonk with a browse root explains, reviews on a commit and judges through a
real mount), and `scripts/test-transcripts.sh` builds it and puts
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
