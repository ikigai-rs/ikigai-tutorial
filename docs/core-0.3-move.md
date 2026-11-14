# Moving the workspace to ikigai-core 0.3: the scope

Ledger #3671, the scoping pass. This page does not change code. It records what the move needs and
what it will change. The numbers come from crates.io's index and from one trial of the move on
main at bbfaaeb (2026-11-13). The trial was not committed.

## Why the move cannot be partial

The workspace pins `ikigai-core = "0.2.12"` and `ikigai-store = "0.3.1"`. `ikigai-ledger` 0.19.0
is the first release on core 0.3, and from 0.21.2 on (0.22.0 included) it requires
`ikigai-core ^0.3.4` and `ikigai-store ^0.5.0`. The crates pass core types to each other, so the
graph must hold one core line, and the ledger pin (#3668) can only move when every crate below does.

## Every ikigai crate the workspace pins, and its core-0.3 release

Every crate has a release on core 0.3, and none is missing. The "move to" column is the newest
release that keeps the graph on ONE copy of each crate. It is not always the newest release
overall. `ikigai-fn` stays on 0.10 because `ikigai-toolbox` 0.4, `ikigai-ui` 0.5, `ikigai-web` 0.5
and `ikigai-browse` 0.22 take `^0.10`. `ikigai-http` stays on 0.6 because `ikigai-llm` 0.20 and
`ikigai-web` 0.5 take `^0.6`.

Direct pins are in the root `Cargo.toml` `[workspace.dependencies]`. The crate-local pins are in
the named crates' own `Cargo.toml`.

| crate | pinned where | locked now (core 0.2) | first on core 0.3 | newest | move to |
|---|---|---|---|---|---|
| ikigai-core | workspace | 0.2.12 | 0.3.0 | 0.3.5 | 0.3.5 |
| ikigai-store | workspace | 0.3.3 | 0.4.0 | 0.5.0 | 0.5.0 |
| ikigai-ledger | workspace | 0.18.0 | 0.19.0 | 0.22.0 | 0.22.0 (needs core ^0.3.4) |
| ikigai-vocab | workspace | 0.2.2 | 0.3.0 | 0.3.5 | 0.3.5 |
| ikigai-fn | workspace, inspector | 0.9.0 | 0.10.0 | 0.11.0 | 0.10.0 |
| ikigai-json | workspace, inspector | 0.3.1 | 0.4.0 | 0.4.0 | 0.4.0 |
| ikigai-log | workspace | 0.9.2 | 0.10.0 | 0.11.0 | 0.11.0 |
| ikigai-compress | workspace | 0.3.0 | 0.5.0 | 0.5.0 | 0.5.0 |
| ikigai-sexpr | workspace | 0.3.0 | 0.4.0 | 0.4.0 | 0.4.0 |
| ikigai-diagram | workspace | 0.3.2 | 0.4.0 | 0.4.0 | 0.4.0 |
| ikigai-module | workspace | 0.5.1 | 0.6.0 | 0.6.0 | 0.6.0 |
| ikigai-sparql | workspace | 0.3.2 | 0.5.0 | 0.5.0 | 0.5.0 |
| ikigai-rdf | workspace, inspector | 0.3.2 | 0.5.0 | 0.5.0 | 0.5.0 |
| ikigai-browse | workspace | 0.20.6 | 0.21.0 | 0.22.0 | 0.22.0 |
| ikigai-markdown | workspace | 0.2.0 | 0.4.0 | 0.5.0 | 0.5.0 |
| ikigai-script | workspace | 0.4.0 | 0.5.0 | 0.5.0 | 0.5.0 |
| ikigai-nl | workspace | 0.5.0 | 0.6.0 | 0.6.0 | 0.6.0 |
| ikigai-llm | workspace | 0.19.0 | 0.20.0 | 0.20.0 | 0.20.0 |
| ikigai-http | workspace | 0.5.1 | 0.6.0 | 0.7.0 | 0.6.0 |
| ikigai-toolbox | workspace | 0.3.0 | 0.4.0 | 0.4.0 | 0.4.0 |
| ikigai-shacl | workspace | 0.5.0 | 0.6.0 | 0.6.1 | 0.6.1 |
| ikigai-engine | architecture-book-wasm, book-wasm, gonk-book-wasm, ui-book-wasm, ttt-host | 0.2.1 | 0.3.0 | 0.5.0 | 0.5.0 |
| ikigai-ui | book-view, inspector | 0.4.1 | 0.5.0 | 0.5.0 | 0.5.0 |
| ikigai-fs | book-urns, inspector-htmx, inspector | 0.2.2 | 0.4.0 | 0.4.2 | 0.4.2 |
| ikigai-xslt | book-wasm, inspector | 0.4.0 | 0.7.0 | 0.8.1 | 0.8.1 |
| ikigai-text | book-wasm | 0.4.0 | 0.5.0 | 0.5.0 | 0.5.0 |
| ikigai-resolve | book-wasm, two-hosts, ttt-host, your-endpoints | 0.2.1 | 0.3.0 | 0.5.0 | 0.5.0 |
| ikigai-web | inspector-htmx, two-hosts, ttt-host | 0.2.9 | 0.4.0 | 0.5.2 | 0.5.2 (needs core ^0.3.5) |
| ikigai-ipc | model-stub, log-host, two-hosts, ttt-host, your-endpoints | 0.2.1 | 0.4.0 | 0.5.2 | 0.5.2 (needs core ^0.3.5) |
| ikigai-quic | two-hosts | 0.2.1 | 0.4.0 | 0.5.2 | 0.5.2 (needs core ^0.3.5) |
| ikigai-throttle | resilience-harness, two-hosts | 0.3.0 | 0.4.0 | 0.5.1 | 0.5.1 (needs core ^0.3.5) |
| ikigai-conformance | your-endpoints | 0.11.0 | 0.14.0 | 0.14.0 | 0.14.0 |
| ikigai-a11y | book-a11y | 0.4.0 | 0.6.0 | 0.6.0 | 0.6.0 |
| ikigai-sign | (transitive, via log) | 0.4.2 | 0.5.0 | 0.5.0 | 0.5.0 |
| ikigai-wire | (transitive, via ipc, quic) | 0.2.1 | 0.4.0 | 0.5.2 | 0.5.2 |
| ikigai-module-abi | (transitive, via module) | 0.1.0 | no core dependency | 0.1.0 | 0.1.0 |
| ikigai-xrust | (transitive, via xslt 0.8) | (none: xrust 2.3.0) | n/a | 2.3.0-ikigai.1 | 2.3.0-ikigai.1 |

The repository has no justfile, so the item's `just lock-diff` does not exist here. The
lock diff below compares the registry `ikigai-*` packages in `Cargo.lock`, before and after the
trial's `cargo update -w`:

```
ikigai-a11y: 0.4.0 -> 0.6.0          ikigai-module: 0.5.1 -> 0.6.0
ikigai-browse: 0.20.6 -> 0.22.0      ikigai-nl: 0.5.0 -> 0.6.0
ikigai-compress: 0.3.0 -> 0.5.0      ikigai-quic: 0.2.1 -> 0.5.2
ikigai-conformance: 0.11.0 -> 0.14.0 ikigai-rdf: 0.3.2 -> 0.5.0
ikigai-core: 0.2.12 -> 0.3.5         ikigai-resolve: 0.2.1 -> 0.5.0
ikigai-diagram: 0.3.2 -> 0.4.0       ikigai-script: 0.4.0 -> 0.5.0
ikigai-engine: 0.2.1 -> 0.5.0        ikigai-sexpr: 0.3.0 -> 0.4.0
ikigai-fn: 0.9.0 -> 0.10.0           ikigai-shacl: 0.5.0 -> 0.6.1
ikigai-fs: 0.2.2 -> 0.4.2            ikigai-sign: 0.4.2 -> 0.5.0
ikigai-http: 0.5.1 -> 0.6.0          ikigai-sparql: 0.3.2 -> 0.5.0
ikigai-ipc: 0.2.1 -> 0.5.2           ikigai-store: 0.3.3 -> 0.5.0
ikigai-json: 0.3.1 -> 0.4.0          ikigai-text: 0.4.0 -> 0.5.0
ikigai-ledger: 0.18.0 -> 0.22.0      ikigai-throttle: 0.3.0 -> 0.5.1
ikigai-llm: 0.19.0 -> 0.20.0         ikigai-toolbox: 0.3.0 -> 0.4.0
ikigai-log: 0.9.2 -> 0.11.0          ikigai-ui: 0.4.1 -> 0.5.0
ikigai-markdown: 0.2.0 -> 0.5.0      ikigai-vocab: 0.2.2 -> 0.3.5
                                     ikigai-web: 0.2.9 -> 0.5.2
                                     ikigai-wire: 0.2.1 -> 0.5.2
                                     ikigai-xslt: 0.4.0 -> 0.8.1
                                     ikigai-xrust: (new) 2.3.0-ikigai.1 (xrust 2.3.0 removed)
```

With those versions, `cargo tree -d` lists one copy of each ikigai crate.

## What the trial found

**It compiles almost unchanged.** `cargo check --workspace --all-targets` failed in one place:
`ikigai-quic` 0.5's `PeerIdentity` gained `source_address: Option<IpAddr>`, and
`crates/two-hosts/src/identity.rs`'s test helper `peer()` builds one. The fix is
`source_address: None`. The book includes only the `minter` anchor of that file
(Beyond: identity), and `peer()` is outside it, so no printed listing changes.

**Behavior and printed output change.** `cargo test --workspace --no-fail-fast` failed 11
targets. Each one is below, with the page that has to change or the regression to look into first.

| target | book: page | what changed |
|---|---|---|
| book-urns `book_claims` | ikigai: modules/status.md | the status chapter must name `ikigai-core` **0.3.5** and `ikigai-module` **0.6.0** |
| book-wasm lib | ikigai: streams/bytes.md | the gzip round trip prints `[1 cached · 1 computed]`, not `[2 cached · 1 computed]` |
| book-wasm lib | ikigai: explore/lab.md | `urn:kernel:dependents` counts 8 live cached entries, not 10 |
| book-transcripts (datasets) | ikigai: beyond/datasets.md | the SPARQL refusals were reworded: the advice comes first ("Add LIMIT…", "Split the query…"), and the "(ledger #970)" citation is gone |
| book-transcripts (scripts) | ikigai: code/scripts.md | `urn:script:open-items:result`'s transcript differs (cached/computed annotations; trial output not yet diffed line by line) |
| book-transcripts (draft) | ikigai: code/sentence.md | `urn:nl:grounding`'s summary differs (counts of actions/vocabulary) |
| two-hosts lib | ikigai: beyond/failure.md | `with_no_budget_configured_the_layers_multiply`: nested `Retry` layers now make 3 reads, not 9. Core 0.3 seems to apply a retry budget by default, and the chapter's claim that "every kernel that configures nothing still" multiplies must be rechecked |
| ttt-host `host` | ikigai: applied/tic-tac-toe-5.md / -6.md | **regression to investigate**: a peer store refuses Meta ("no Meta renderer configured"), so `run_lines_answer_with_the_game_kernels_verdicts_over_a_peer_store` fails. The host probably has to inject a renderer under core 0.3 |
| openspec-ledger lib | gonk: spec/openspec.md, spec/ledger.md | **regression to investigate**: `ikigai-markdown` 0.5 no longer finds the `openspec` mapping ("nothing serves urn:markdown:file:mappings/openspec/mapping.ttl"). The crate must bind a file module for named mappings |
| gonk-book-wasm lib | gonk: ledger/your-own.md | `urn:iki:ledger:claim` prints `#3 claimed by you at epoch 1`. That is the ledger 0.19–0.22 change, and #3668 already plans to teach it |
| architecture-book-wasm `pages` | architecture: spine/manifold.md, overlays/overlays.md and the harness stages | the action listing differs, a retry failure now reads `gave up after 3 attempts over 1 ms: …`, and the stage blocks change. In one run the harness clock cell printed `5216ms` where the page says `5s`, so check whether it depends on wall-clock time |
| inspector `views` | ui: the inspector's catalog snapshot (`crates/inspector/tests/snapshots/catalog.html`) | the catalog view's rows differ (62 endpoints in both) |

The trial did not run `./scripts/test-books.sh` (`mdbook test`) or the wasm builds. The failures
above already block the move, so those runs belong to the move PRs.

## The proposed series

1. **Pins and lock.** Make the version changes in the table, add the `two-hosts` `source_address`
   fix, and fix the two regressions first (ttt-host's Meta renderer, openspec-ledger's mapping
   module). Neither change should touch printed output.
2. **The ikigai Book.** Replay status, streams/bytes, explore/lab, datasets, scripts, sentence
   and failure. Add one `changes.toml` entry.
3. **The architecture Book and UI Book.** Replay manifold, overlays, the harness stages and the
   inspector snapshot. Add their `changes.toml` entries.
4. **The gonk Book.** The ledger 0.22 changes, which is #3668's remaining work, unblocked by step 1.

Steps 1–3 must land together or behind one another on a branch, because CI requires every book
to be green. Step 1 alone leaves the books' tests red. The likely shape is one PR made of these
commits.
