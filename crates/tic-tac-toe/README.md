# tic-tac-toe

The code taught by the book's applied arc: tic-tac-toe designed as resources
before it is code, after *Resource Oriented Analysis and Design*, Parts 1–9, by Peter
Rodgers (NetKernel News 3.34–3.42, August–October 2012). The prose is
`books/ikigai/src/applied/tic-tac-toe-N.md`, one page per increment; the listings are
included from this crate by anchor.

The six chapters are **public**: the book's "Applied: tic-tac-toe" part in `SUMMARY.md`
(linked with increment 6; until then they were drafts in `books/ikigai/drafts.txt`). A merge
to main deploys them. Read them with runnable cells and live boards through
`./scripts/serve-with-drafts.sh`, which builds the in-page kernel.

## The arc

Each increment was its own satellite, its own PR, and its own page, reviewed on the drafts
server before the next was dispatched.

| Increment | Chapter content | Series parallel | Status |
|---|---|---|---|
| **1** | Name the sets on paper (the resource model); the atoms: a stored cell and the platonic empty cell; wire the game into the page's kernel; runnable cells | Parts 1–3 | done — `tic-tac-toe-1.md` |
| **2** | Composites: a line of cells by name, rows / columns / diagonals as zero-code `Alias`es, the board; one move recomputes the lines through it and the board (trace shows it) | Part 4 | done — `tic-tac-toe-2.md` |
| **3** | Rules as resources: CheckSet (a representation that is a list of IRIs), winner, whose turn; constraint at the edge, model kept loose | Parts 6–7 | done — `tic-tac-toe-3.md` |
| **4** | Many games: the game as context (a `Scope` corridor) versus the game in the name — argue it; the store as a swappable space | Part 8 | done — `tic-tac-toe-4.md` |
| **5** | The views and templates as resources; a playable board in the page over the same kernel (htmx, answered by the wasm kernel); `ttt-host`; the wrap-up | Parts 5, 9 | done — `tic-tac-toe-5.md`, `crates/ttt-host` |
| **6** | The same game in Python and TypeScript: the atom held by a peer (the store contract as the whole interface), and an app in each language that fills the templates from the host; the limits across the boundary | — | done — `tic-tac-toe-6.md`, `scripts/ttt-polyglot-demo.sh` |

## What increment 1 left for the next ones

- **Names.** `urn:iki:tutorial:ttt:stored:{x}:{y}` (endpoint `ttt-stored`: Source /
  Sink / Delete) and `urn:iki:tutorial:ttt:cell:{x}:{y}` (endpoint `ttt-cell`: Source).
  The chapter's table also *plans* `row:{y}`, `column:{x}`, `diagonal:{n}`, `board`,
  `checkset:{x}:{y}`, `turn` and `winner` under `urn:iki:tutorial:ttt:`. Each carries an
  `urn-gate: unbound` directive in `tic-tac-toe-1.md`; binding one makes that directive
  fail, which is the reminder to delete it. Renaming one means editing the table.
- **Coordinates are any `i64`, in one spelling.** `01`, `+1`, `-0` are refused
  (`InvalidArgument`), because two names over one cell would be two golden threads over
  one piece of state. A composite that builds cell names should use `cell_name` /
  `stored_name`, which spell them the accepted way.
- **Writes go to `stored`, reads to `cell`.** The platonic cell is Source-only on
  purpose — see the chapter. A later increment that wants one name for both should argue
  it, because a forwarded Sink would cut the platonic cell's own thread and hide the
  failed-sub-request dependency the first chapter is about.
- **The page kernel is `book_wasm::page_space()`**: `Fallback([hello_camel::space(),
  tic_tac_toe::space()])`. New endpoints bound in `space_over` appear in the page and in
  the URN gate with no further wiring.
- **Cells are pinned by reading the chapter.** `book-wasm`'s
  `the_tic_tac_toe_cells_answer_as_the_chapter_says_in_page_order` parses every
  `data-cmd` and expected `<pre>` out of `tic-tac-toe-1.md`, runs them in page order
  against one kernel, and compares. A new page wants the same test pointed at it.
- ⚠ **A trace does not show a failed sub-request** (core 0.1.78). `trace
  urn:iki:tutorial:ttt:cell:2:2` on an unplayed cell prints one node — the stored cell's
  `NotFound` read, although it is a cache dependency, is not a span. Played, the same
  trace shows the child. Increment 2's "the trace shows it" will want played cells, or
  a sentence saying why an empty cell's trace is one line.

## What increment 2 left for the next ones

- **Names.** `urn:iki:tutorial:ttt:cells:{list}` (endpoint `ttt-cells`: Source), where
  `{list}` is an ORDERED sequence of `x.y` members joined by `,` —
  `cells:0.0,1.0,2.0`. Build one with `cells_name(&[(x, y), …])`, never by hand: each
  coordinate in its one spelling, no spaces, no empty member, no trailing comma, anything
  else `InvalidArgument`. `urn:iki:tutorial:ttt:board` (endpoint `ttt-board`: Source)
  answers the three rows, top to bottom, joined by `\n` with no trailing newline.
- **`y` runs DOWN.** Row 0 is the top row; `(2,0)` is the top-right corner. Diagonal 0 is
  `0.0,1.1,2.2`, diagonal 1 is `2.0,1.1,0.2` (top-right to bottom-left).
- **The rows, columns and diagonals are the text table `LINES`** — eight `exact` rules,
  parsed by `AliasTable::parse`, wrapped round the endpoints by `Alias::new` in
  `space_over`. So `space()` and `space_over()` now return an `Alias`, not an
  `EndpointSpace`; everything that wraps them (the page kernel, the URN gate) got the rows
  with no wiring. The 3×3 lives in that table and nowhere else.
- ⚠ **Probe the cache by the LINE's name, not the alias's.** `Kernel::is_cached` (the
  engine's `cache`) does not resolve, and a rewrite made inside a space is only known by
  resolving — so `cache …row:1` answers `not cached` while the entry is there (core
  documents this limit in `is_cached_in`). Serving is right; only the probe under-reports.
  Tests and cells use `cells:…` names for every probe. A `Kernel::with_aliases` table
  would fix the probe, at the cost of a second place the table lives.
- ⚠ **The URN gate cannot see an alias as a TEMPLATE.** `Alias::entries` is transparent
  (core decision 4), so `row:{y}` is not a pattern anywhere, and an `unbound` directive on
  it would NOT have failed once the rows were bound — only `board`'s did. The chapters
  therefore name concrete rows (`…:row:0`), which resolve through the alias and are
  checked. Part III's `checkset:{x}:{y}` is a real template if it is bound by an
  endpoint; if it is an alias table instead, rewrite part I's row and delete its
  directive by hand.
- **The gate now reads `,` as part of a name** (`book-urns`'s `is_urn_char`); before, it
  checked `…:cells:0.0` and passed on a name the book never printed.
- **Trace in a chapter.** `run_chapter_in_page_order` in `book-wasm` rewrites the test
  thread's name to `ThreadId(1)`, the page's one thread, so a trace can be pinned
  verbatim. A played cell's trace shows its stored child; an unplayed recomputed cell is
  one node (the failed read is a dependency, not a span).

### What increment 3 (rules as resources) inherits

- **CheckSet** (`checkset:{x}:{y}`): the lines through a cell, as a representation that
  is a list of IRIs. The honest source is the `LINES` table itself — the aliases whose
  target contains `x.y` — so a CheckSet computed from the table stays right if the table
  changes (Connect Four). Answer the ALIAS names (`row:0`) or the line names? The line
  names are the cache keys; the alias names read better. Argue it in the chapter; the
  trace note `alias=… -> …` shows both.
- **Winner**: a line whose marks are three of one kind — a pure function of line
  representations, so cacheable and normalized for free: a move recomputes the winner
  only through the lines it touched. The center is on four lines, a corner on three, an
  edge on two (part II ends on that count).
- **Whose turn**: counts marks on the board, which is already a resource; the constraint
  ("X plays when counts are equal") belongs at the edge, in a Sink-side check, with the
  stored cell still accepting any mark. Refusing an out-of-turn move is the first thing
  that makes `stored`'s Sink conditional — keep the store loose and put the rule in a
  new resource in front of it, or the atoms stop being reusable.

## What increment 3 left for the next ones

- **Names.** `urn:iki:tutorial:ttt:checkset:{x}:{y}` (endpoint `ttt-checkset`: Source),
  `urn:iki:tutorial:ttt:winner` (`ttt-winner`: Source), `urn:iki:tutorial:ttt:turn`
  (`ttt-turn`: Source) and `urn:iki:tutorial:ttt:move:{x}:{y}` (`ttt-move`: **Sink only**;
  any other verb is an `Endpoint` error). Name helpers: `checkset_name`, `move_name`.
  Part I's table now lists the move too; its three `urn-gate: unbound` directives are gone
  (re-adding one fails the gate — checked).
- **The CheckSet answers ALIAS names, sorted** (`column:1`, `diagonal:0`, `diagonal:1`,
  `row:1`), newline-joined with no trailing newline, and the empty string off the board.
  Sorted because `AliasTable::rules()` is most-specific-first (longest `from`, then
  alphabetical), NOT the order `LINES` is written in — a CheckSet in "table order" would
  have read diagonals, columns, rows. It is computed from the shared `Arc<AliasTable>` that
  `space_over` now builds once and hands to both the `Alias` and the rules: a line passes
  through `x.y` when its target's member list holds exactly that member (a string match,
  exact only because of the one-spelling rule). `.cacheable()`, no sub-requests: nothing
  but `urn:kernel:cut` can ever un-cache it.
- **Winner** answers `X`, `O`, `-` (`EMPTY`) or `draw` (`DRAW`). It reads ALL eight lines
  through their aliases in the table's own order (so a trace lists diagonals, columns, rows)
  and decides after: first full line in that order, else `draw` if no line holds `-`, else
  `-`. Reading all eight rather than stopping at the first win keeps "a pure function of the
  eight lines" literally true; an early return would also be correct and lazier.
- **Turn** answers `X` when X's count ≤ O's (X opens), `O` otherwise, and `-` whenever the
  winner is not `-`. It reads the winner and the board.
- **The move** checks, in order: on the board (its CheckSet is non-empty), game not over
  (winner), square free (the platonic cell), `content` (optional; empty or absent plays the
  turn; anything else must equal the turn). Every refusal is `InvalidArgument`: the square
  ones are named `` `x, y` `` (there is no single argument to name), the mark one
  `content`. Messages, verbatim: `3,1 is off the board — no line passes through it`,
  `the game is over — X has won` / `the game is over — a draw`, `1,1 is taken — X played
  there`, `it is O's turn, not X's`. On success it issues `Sink stored:{x}:{y}` through the
  invocation and answers `X plays 1,1`. `content` declares `one_of(["X","O"])` and
  `.optional()`; nothing enforces `one_of` at issue time (only `urn:kernel:validate` reads it).
- **A Sink issued inside an endpoint cuts exactly like a top-level one** — pinned by
  `tests/rules.rs::a_sink_issued_inside_the_move_cuts_the_stored_cells_thread`.
- **`..` CAN follow the CheckSet's links**, through `urn:iki:fn:compose`: the map feeds each
  name as compose's one declared argument, `src`, and compose sources it (a line has no
  `$a{}` markers, so compose is exactly "dereference"). The engine's map has no deref of
  its own — the stage's target is fixed and the item is its INPUT — so without a
  "source the IRI you are given" endpoint in the page it could not have been done.
- **The page's `cells` parser does not unescape `&#39;`** (only `&#32; &lt; &gt; &quot;
  &amp;`). A raw `'` inside an expected `<pre>` is fine; inside `data-cmd='…'` it is not.

### What increment 4 (many games) inherits

- ⚠ **Exact aliases do not parameterize.** `LINES` names `row:0` … `diagonal:1` for ONE
  board. Putting the game in the name (`…:game:{g}:row:0`) would need either eight rules per
  game or `prefix` rules, and a prefix rule rewrites a prefix, not an infix: the game id
  would have to be at the END of the name, or every composite would have to build
  game-qualified names itself. That argues for the game as a `Scope` corridor: the names
  stay as they are and the stored cell's state is looked up per scope.
- **Only `stored` holds state.** Everything above it (cell, line, board, CheckSet, winner,
  turn, move) is a pure function of names it sources, so a corridor that changes what
  `stored:{x}:{y}` resolves to changes the whole game with no code above it moving. The
  CheckSet is the exception worth stating: it depends on nothing, so ONE entry could serve
  every game — check whether the cache key under a scope keeps that true, or whether each
  game recomputes its own copy (harmless, but it would un-teach "computed once").
- **The cache and the probe.** Every cell in parts II–III probes by LINE name because
  `cache` does not resolve (ledger #561). Under a corridor, a probe will also need the
  scope, and `Kernel::is_cached_in` exists for that (core 0.1.75) — the engine's `cache`
  command would need a way to name it.
- **The move reads four resources before it writes.** Under a scope, each of those
  sub-requests must resolve in the SAME game's scope as the Sink; that is the property to
  pin first (a move in game A refused because game B is over would be the bug).

## What increment 4 left for the next ones

- **A game is a corridor.** `game(id, store)` answers `Scope::empty().with_named(
  urn:iki:tutorial:ttt:game:{id}, store)`, where `store` is any space binding `STORED`. The
  corridor name is NOT a resource (the chapter carries `urn-gate: illustration` directives
  for `…:game:a` and `…:game:b`). Ids are `[A-Za-z0-9-]+`, else `InvalidArgument` named
  `game`; a store that names itself (`Space::id`) differently is refused as an error rather
  than `with_named`'s panic. Build a game's corridor ONCE and keep it: the name is the cache's
  claim that it is one game.
- **The seam.** `space_with_store(store: Arc<dyn Space>) -> Alias` builds every composite
  and puts `store` after them in a `Fallback`, wrapped in `LINES`. `space()` /
  `space_over(CellStore)` are unchanged in behavior and are now `space_with_store(
  stored_space(store))`; `stored_space(Arc<CellStore>)` is the in-memory store alone. The
  topology changed shape (Alias → Fallback → [composites, store]); part II's prose was
  adjusted to say so. The root keeps its own store: that is the game a request plays when it
  names none.
- **Nothing above the atom changed.** Same names, same endpoints; `tests/games.rs` pins two
  games and the root on one kernel as three boards.
- **A move stays in its game** — pinned both ways (A over, B plays; ROOT over, B plays) and by
  counting reads on each game's `CellStore`. It holds because every sub-request inherits the
  invocation's chain; the move chooses nothing.
- **The CheckSet recomputes per game** (whole-chain key): root + three games × nine squares =
  nine distinct answers filed 36 times (`every_game_computes_its_own_checkset`). Fix = key on
  the corridor that answered (ledger item 563 — `Resolved::answered_by` exists since core
  0.1.78). Not built.
- ⚠ **A golden thread is a NAME, not a name in a chain.** `Kernel::cut` is by IRI, so a move
  in ANY game cuts `stored:{x}:{y}` in EVERY game: game B's cached lines through that square
  recompute (same answer — sound, never stale; a cost that grows with the number of games).
  Pinned by `a_move_in_one_game_uncaches_the_same_square_in_every_game` and visible in the
  chapter, where game A's moves had already un-cached game B's top row before the section
  about it. Same missing piece as the CheckSet's, from the other side.
- **The page.** A cell with `data-game='a'` runs in game `a`: `run.js` calls
  `evalLineInGameAsync(game, line)`, and `book_wasm::Page` keeps one `Engine` per game over
  the ONE page kernel, each engine's resolver an `InGame { kernel, scope }` that issues every
  request with `Kernel::issue_in` / `issue_with_incoming_in` and probes `is_cached_in`. No
  engine API was needed: the engine takes any `Resolver`, so `cache`, `trace`, pipes and maps
  all work per game. The label reads *Command, in game a*; the game is markup, not editable.
  `trace` in a game prints the chain as its transport and `answered-by=…:game:b` on the
  stored node. `pages.yml` smoke-checks the new export.
- ⚠ **Forward note for engine 0.1.29.** The book still resolves engine 0.1.28. On 0.1.29 the
  engine calls `issue_as_async_in(…, line_scope)`; `InGame` does not override it, so the
  trait default delegates an EMPTY line scope to `InGame`'s overrides (correct) and REFUSES
  a non-empty one — so `as-of=` inside a game cell would be an error, not a silently
  unpinned read. Composing a line's as-of corridor with a game corridor needs either a
  `Scope` concatenation in core (there is none: `spaces()` exposes the corridors but not
  their identities) or `InGame` rebuilding the chain itself.

## The store contract

What any space must do to be a game's store, in any language. `check_store(space)` runs it
(it WRITES: plays and clears `stored:7:-3`, so point it at a scratch store).

1. **Binds** `urn:iki:tutorial:ttt:stored:{x}:{y}` for any integer `x`, `y` in its ONE plain
   spelling (what `i64::to_string` prints). `01`, `+1`, `-0`, `-03` are `InvalidArgument`
   naming `x` or `y` — on every verb.
2. **Source** answers the mark: media type `text/plain` (`;charset=utf-8`), bytes EXACTLY the
   mark — no trailing newline — and **cacheable**. A square nothing was played at is
   **`NotFound`** — not an empty answer (a line would read `X` instead of `X--`) and not
   `Unresolved` (that means nothing is bound).
3. **Sink** takes the mark as the argument `content`, **trimmed**, and keeps ANY non-empty
   mark (the rules live in `move`, not the store). An empty or all-whitespace mark is
   `InvalidArgument` naming `content`. The answer's bytes are not read by anything.
4. **Exists** answers `true` where a mark is played and `false` where none is — **never
   `NotFound`** — as `text/plain`, **cacheable** (the same write cuts it). Added in increment
   5 (ledger #580): the Rust store's Exists used to fall into its Source arm (the mark, or
   NotFound) while the Python and Deno faces answered true/false; the faces' answer is the
   ecosystem's convention (core's own `an_exists_answer_hangs_from_its_target_too` has the
   same shape), so the Rust store changed. Both faces' DEFAULT Exists ("would Source
   succeed") already passes. Bad spellings are refused on Exists too.
5. **Delete** clears the square and succeeds whether or not anything was there.
6. **Meta** (optional, recommended): describe `Source`/`Sink`/`Exists`/`Delete` as four
   actions, each naming `x`, `y` as bindings, `Sink` naming `content` (see `stored_cell`).

Two consequences a peer store must know, neither checkable by `check_store`:

- **Cacheability propagates.** An uncacheable mark makes every cell, line, board, winner
  and turn above it uncacheable.
- **Writes must go through the host kernel.** The host cuts `stored:{x}:{y}` when ITS kernel
  issues the `Sink`/`Delete`. A write made to a peer store directly (another client of the
  Python process) is invisible to the host's cache, which then serves the old board until
  something else cuts it. A peer that can be written behind the host's back needs a way to
  tell the host — a golden thread over the wire — and there is none today.

## What increment 5 inherits

- **The view resources.** HTMX fragments as `text/html` resources over the same kernel —
  board, turn, winner — each cell of the board a button with `hx-post` to
  `urn:iki:tutorial:ttt:move:{x}:{y}` (a `Sink`), the response re-rendering the board. They
  are composites like the rest: pure functions of names they source, cacheable, and so they
  work in any game with no change (the corridor carries). Put the markup in a template
  resource, not in Rust strings, if the Python/TypeScript apps are to share it.
- **An htmx request shim in the page** routing `hx-post`/`hx-get` to the wasm kernel instead
  of the network (htmx's `htmx:beforeRequest` / a custom extension), with the game coming from
  markup exactly as `data-game` does now — the host chooses it, never the request.
- **A `ttt-host` binary** serving the game over the IPC socket and HTTP, with
  `--store <socket>` mounting a PEER store per game: `game(id, peer)` where `peer` is a
  `ModuleSpace` (or the face's own client space) over that socket. Check the peer with `check_store` before accepting it. ⚠ Socket paths must fit
  macOS's 104-byte `sun_path` (not the scratchpad).
- ⚠ **The chain does not cross the wire.** A game corridor whose store is a mount is fine —
  the store resolves `stored:…` itself and has no sub-requests. But a HOST that resolves a
  whole game remotely (the Python app asking the Rust host for `board` in game `a`) cannot
  send the chain: the wire's default refuses a non-empty chain. The game id would have to
  travel as something the far host turns into a corridor — its own authority decision.
- The ledger-563 fix and a scoped golden thread would make N games cost what one does; both
  are core.


## What increment 5 left for the next ones

### The names

All under `urn:iki:tutorial:ttt:`, all bound in `space_with_store`, so they exist in every
game through the corridor with no wiring:

| name | endpoint | verbs | answers |
|---|---|---|---|
| `template:{name}` | `ttt-template` | Source | a template, `text/html`, `Never`; unknown name → `NotFound` |
| `view:board` | `composeOver` over `template:board` | Source | the board as HTML, cacheable |
| `view:square:{x}:{y}` | `composeOver` over `template:square` | Source | one square: open, taken or closed |
| `view:status` | `composeOver` over `template:status` | Source | `X to play.` / `O has won.` / `A draw.` — HTML text, cacheable |
| `view:reply` | `composeOver` over `template:reply` | Source | the argument `message`, then the status |
| `view:game:{game}` | `composeOver` over `template:game` | Source | game `{game}`'s page shell |
| `view:play:{x}:{y}` | `ttt-view-play` | **Sink only** | Sinks `move:{x}:{y}`; answers `view:reply` |
| `view:reset` | `ttt-view-reset` | **Sink only** | Sinks `reset`; answers `view:reply` |
| `reset` | `ttt-reset` | **Sink only** | Deletes every square a `LINES` rule passes through; `The board is clear` |

The views need one name the game does not bind, `urn:iki:fn:conditional` (`ikigai-fn`).
`conditional_space()` binds it alone; `kernel()` and `ttt-host` mount it beside the game, and
the book's page already has it from Part I's `ikigai_fn::space()`.

`view:board` reads the templates and the nine square views; each square reads its cell and,
while empty, the winner (not the board text: a cell's answer is the exact mark, and the board
concatenates marks, which is ambiguous for a multi-character mark). `view:status` reads the
winner and, while the game is on, the turn. All are composites, so they are cached per game
and a move recomputes them through what it touched. A view's Sink answer is not cacheable; the
reply READS `view:status`, so after a play the status is cached again (the new one) and the
board is not until someone asks.

Until the views were templates (2026-09-29), they were `{{slot}}` templates that Rust filled.
The bytes did not change: `tests/parity.rs` plays fresh, in-progress, won, drawn and
hostile-mark games through the views and through the old filler (kept in the test, over the
old templates frozen in `tests/reference/`) and compares every board, status, reply and page
shell.

### The template language — what every host that fills these templates implements

Templates are the files in `templates/` (`TEMPLATES` in `lib.rs`), served at
`template:{name}`. A file's final newline is not part of the template. They are written in
`ikigai-fn`'s template language (`ikigai_fn::Compose`'s docs are the full grammar); a host
that serves the views from the Rust kernel implements nothing. A host that fills them itself
(the Python and Deno apps) implements the SUBSET these templates use, stated here:

- **Scan** left to right. `$$` is a literal `$`. `$a{`, `$r{` or `$h{` opens a MARKER, which
  ends at the matching `}`: count `{` and `}` inside it (an argument `{x}` nests), and skip
  any `"…"` span (where `\"` and `\\` are escapes). A marker that never closes is literal
  text, `$` included. Any other `$`, `{` or `}` is literal. The marker's BODY is the text
  between the braces, trimmed.
- **Body.** `{name}` alone (a letter or `_`, then letters, digits, `_`, `-`) is an ARGUMENT:
  splice its value, resolve nothing. Anything else is a REQUEST: `IRI` or `IRI?k=v&k=v`. In the
  IRI, each `{name}` is replaced by the argument's value percent-encoded as RFC 6570 does a
  simple variable (every byte outside `A-Z a-z 0-9 - . _ ~` as `%XX`, upper case). In an
  unquoted value, each `{name}` is replaced verbatim (and the result is still ONE value); a
  `"quoted"` value is literal. Every request is a `Source`. (`||` fallbacks exist in the
  language; these templates use none, and a filler may refuse them.)
- **Splice.** `$h`: the answer as text, with `&` `<` `>` `"` `'` replaced by `&amp;` `&lt;`
  `&gt;` `&quot;` `&#39;` and nothing else changed. ⚠ `&#39;`, not Python's `&#x27;`.
  `$r`: the answer as it is. `$a`: the answer's own markers expanded, with the SAME
  arguments, then spliced. `$a{{name}}` is refused (an argument is never a template). Only
  `$a` ever expands: a `$h` or `$r` value is never scanned, so a mark holding
  `$a{urn:…}` renders as those characters.
- **Arguments** are the variables a view's name captures (`x`, `y` in `view:square:{x}:{y}`;
  `game` in `view:game:{game}`) and, failing that, the request's own arguments (`message` for
  `view:reply`). A missing one fails the view.
- **Failure.** A marker that fails fails the whole view.
- **`urn:iki:fn:conditional?if=A&equals=V&then=B&else=C`** — the only function the
  templates call. Source `A`; if its text, trimmed, is exactly `V`, source and answer `B`,
  else `C`. Only the chosen side is sourced. It answers the chosen TEMPLATE, which the `$a`
  around it then expands. A host that forwards only the game's names (as `ttt-host`'s
  gateway does) will not answer `urn:iki:fn:conditional`, so a filler implements it itself.
- **Views are names.** A `$r{urn:iki:tutorial:ttt:view:square:1:0}` inside the board is a
  view like the board. A filler either asks the host for it (it is bound in every game) or
  composes it itself, by the table above: `view:square:{x}:{y}` is `template:square` with
  those arguments.
- **The game.** Every name in a template is spelled for the root game. A filler serving game
  `id` over `ttt-host`'s IPC face resolves `urn:iki:tutorial:ttt:{rest}` as
  `urn:game:{id}:iki:tutorial:ttt:{rest}` (the root game is `urn:game:root:…`).

The cases below are the spec, and `tests/templates.rs`
(`the_template_language_cases_in_the_readme_hold`) reads THIS block and runs every line
through `ikigai-fn`'s compose, so the spec and the language cannot drift. Each case is filled
with the arguments `x` = `1`, `y` = `-2`, `message` = `it's <b>`, over these resources:
`urn:t:mark` = `<b>"&'$a{urn:t:secret}`, `urn:t:html` = `<i>ok</i>`, `urn:t:dash` = ` -` and a
newline, `urn:t:inner` = `[$h{{x}}]`, `urn:t:cell:{x}:{y}` = its two coordinates joined by
`.`, and nothing else (`urn:t:secret` is not bound). `fill` gives a template and what it
fills to, separated by a tab; `refuse` is a template whose filling fails.

<!-- template-cases: begin -->
```text
fill    plain {x} } { $ $x{a} $	plain {x} } { $ $x{a} $
fill    $h{urn:t:mark}	&lt;b&gt;&quot;&amp;&#39;$a{urn:t:secret}
fill    $r{urn:t:mark}	<b>"&'$a{urn:t:secret}
fill    $h{urn:t:html}|$r{urn:t:html}	&lt;i&gt;ok&lt;/i&gt;|<i>ok</i>
fill    $$h{urn:t:mark} $$$$ $$	$h{urn:t:mark} $$ $
fill    $h{ urn:t:html }	&lt;i&gt;ok&lt;/i&gt;
fill    $h{{x}},$h{{y}}	1,-2
fill    $h{{message}}	it&#39;s &lt;b&gt;
fill    $r{{message}}	it's <b>
fill    $h{urn:t:cell:{x}:{y}}	1.-2
fill    $r{urn:t:inner}	[$h{{x}}]
fill    $a{urn:t:inner}	[1]
fill    $a{urn:iki:fn:conditional?if=urn:t:dash&equals=-&then=urn:t:inner&else=urn:t:html}	[1]
fill    $a{urn:iki:fn:conditional?if=urn:t:dash&equals=X&then=urn:t:inner&else=urn:t:html}	<i>ok</i>
fill    $a{urn:iki:fn:conditional?if=urn:t:cell:{x}:{y}&equals=1.-2&then=urn:t:inner&else=urn:t:html}	[1]
fill    $h{urn:t:mark	$h{urn:t:mark
refuse  $h{urn:t:secret}
refuse  $h{{nope}}
refuse  $a{{x}}
refuse  $h{urn:t:{nope}}
```
<!-- template-cases: end -->

Which template a view shows is no longer a rule a filler knows: it is in the templates.
`square` asks whether the cell is `-` (then `square-empty`, else `square-taken`), and
`square-empty` asks whether the winner is `-` (then `square-open`, else `square-closed`);
`status` asks whether the winner is `-` (then `status-turn`, else `status-over`), and
`status-over` whether it is `draw` (then `status-draw`, else `status-won`). The reply's
`message` is what the write answered, **or the error's Display on a refusal** (the view never
looks inside an error — so ledger #575's typed precondition variant will render with no view
change). The page shell's `game` is the game's id as the host names it (`root` for the root
game).

### The path ↔ IRI rule — what every host of this markup implements

The markup's `hx-get` / `hx-post` values are **relative** paths and name **no game and no
host**: `iki/tutorial/ttt/view/board`, `iki/tutorial/ttt/view/play/1/1`,
`iki/tutorial/ttt/view/status`, `iki/tutorial/ttt/view/reset`
(`the_markup_names_no_game_and_no_host` pins it).

- A request for the relative path `a/b/c` is a request for `urn:a:b:c` — the segments joined
  by `:` after `urn:` (ikigai-web's mechanical `/noun/partition/key` default, made relative).
- `GET` → `Source`, `POST` → `Sink`, `DELETE` → `Delete`. The body is ignored by every view.
- **The game is where the page is.** The part of the URL in front of the relative path is
  the host's; the host turns it into the game's corridor. It is never read from the request
  body, a query parameter, or a header.
- **How `ttt-host` really reads a path** — its HTTP face is `ikigai-web`'s generic edge, as
  hardened in cli **0.1.30** (ledger items 80 and 591), and an app that answers every path the
  way the host does must copy it. `the_edge_decodes_a_path_segment_by_segment_and_refuses_a_malformed_escape`
  in `crates/ttt-host/tests/host.rs` pins each decoding line of this list over a real socket,
  and the bounds as the host's config (they were measured by hand against the running host
  on 2026-09-28):
  - The target is decoded as **bytes**. A `%` must be followed by exactly two hex digits:
    `%`, `%4`, `%zz`, `%+1` are **`400 malformed percent-escape`**, never guessed at, in the
    path and in the query alike. Bytes that are not UTF-8 once decoded (`%FF`) are
    **`400 not UTF-8 once decoded`**.
  - The path is **split on `/` first**, and each segment is decoded on its own (RFC 3986),
    so `%2F` is **data inside its segment**, never a separator: `…/play/1%2F1/0` asks for
    the coordinate `1/1` (refused, `400`), and `/game%2Fa/…` is the IRI `urn:game/a:…`,
    which is no game (`404`) — not game `a`.
  - **`+` in the path is a `+`.** `…/play/+1/0` reaches the game, and the coordinate rule
    refuses it as it refuses every second spelling of an integer: ``400 invalid argument
    `x`: `+1` is not an integer in its plain form``. `%2B1` is the same request. Only the
    **query** is form-encoded, so there `+` is a space; the views read no query.
  - A well-formed escape of an ordinary character is that character: `…/play/%31/0` is a
    play at `1,0`.
  - **Empty segments are dropped** (`/game/a//iki/…` is `/game/a/iki/…`).
  - **`.` and `..` are segments like any other** — nothing normalizes them — and they name
    no game resource, so they are `404` (`…/view/../board` asks for
    `urn:…:view:..:board`).
  - ⚠ **A `:` inside a segment is not escaped on the way to the IRI**: the mechanical
    mapping joins segments with `:`, so `/game/a/iki%3Atutorial/ttt/view/status` and
    `/game/a/iki:tutorial/ttt/view/status` both answer as `/game/a/iki/tutorial/ttt/view/status`,
    and `/game/a:b/…` is `urn:game:a:b:…` (no game, `404`). A client cannot add a PATH
    segment by encoding `/`, but it can add an IRI segment with `:`.
  - **Time and connections are bounded** (ikigai-web's defaults, which `ttt-host` keeps):
    request line and headers within **10 s**, a declared body within **30 s** — each
    **`408`** otherwise — the response written within **30 s**, and at most **256**
    connections at once; the next is **`503` with `Retry-After: 1`**, sent before a
    lingering close so it arrives instead of a reset.

  The markup only ever sends plain relative paths, so none of this reaches a board.

Hosts, as built or planned:

| host | where the game comes from | status |
|---|---|---|
| the book page (`js/ttt.js`) | `data-game` on the `.ttt-play` element holding the board | done |
| `ttt-host` over HTTP | the path prefix `/game/{id}/` (the page's `<base href>`); `/` is the root game | done |
| Python / Deno apps | their own path prefix, same rule (`/game/{id}/`, `/` the root) | done — `ikigai-python` / `ikigai-deno` `examples/tictactoe_app.*` |

### The in-page shim

- `js/ttt.js` loads the vendored htmx (`books/ikigai/src/vendor/htmx-2.0.4.min.js`, byte-
  identical to `ikigai-web/assets/htmx.min.js`, sha256 `e209dda5…fb447`, 0BSD) only on a
  page with a board, puts the page shell `view:game:{game}` into each
  `.ttt-play[data-game]`, and `htmx.process`es it. It fills no template.
- **Mechanism:** `htmx:beforeRequest` → `preventDefault()` for requests from inside a board
  → path → IRI → `issueAsync(game, verb, iri)` (a new wasm export over `Page::issue`, which
  issues in the SAME `Scope` the game's cells run in) → `htmx.swap(target, html,
  {swapStyle: "innerHTML"})`, which fires afterSwap/afterSettle as a real response would. It
  does not implement `hx-swap` (the markup uses only innerHTML).
- **Ids are namespaced per game** by the shim (`a-ttt-square-1-1`): htmx restores focus by
  id after a swap, and two boards on one page would otherwise duplicate every id.
- **The DOM is a copy the kernel cannot cut.** `run.js` now dispatches `ikigai:cell-ran`
  with the cell's game after every run, and the shim re-fetches that board's status (its own
  `hx-get`), which re-renders the board. `run_chapter_in_page_order` replays both the boards'
  initial loads and these refreshes, so the chapter's pinned outputs include them.
- `run.js` exposes its loader as `window.ikigaiBookKernel`, so the board and the cells share
  one wasm instance.

### Accessibility, as built

Buttons for squares (keyboard-operable with no code); every square named (`2,0: empty,
play here`, `X at 1,1`, `0,1: empty, the game is over`); unplayable squares are
`aria-disabled="true"`, not `disabled`, so they stay focusable and focus survives the
re-render; the status line is `role="status"` and is the target of every play and reset, so
what a move did is announced once. Its content is plain text on purpose: an element inside it
would fire extra `htmx:afterSettle` events and re-trigger the board. Checked with axe-core in
a real browser (light, rust, navy themes: no violations in the boards) and with jsdom
`a11y/axe.mjs` + `book-a11y`. ⚠ CI still never sees a LIVE board: the page is linked now, so
`pages.yml`'s axe scans it, but jsdom cannot load the wasm, so it only ever sees the fallback
line (`a11y/no-kernel.mjs` checks that a board with no kernel is a message with no squares).
The in-browser check was re-run on parts I–VI when the arc was linked; re-run it after any
change to the markup, `ttt.css` or `ttt.js`.

### `ttt-host` (`crates/ttt-host`)

```text
ttt-host [--http <addr>] [--socket <path>] [--store <socket>] [--game <id>[=<socket>]]… [-c <line>]…
  --http 127.0.0.1:8070 (default)   --socket $TMPDIR/ttt-host.sock (default; must fit 104 bytes)
  --store <socket>   the ROOT game's store is a peer     --game <id>[=<socket>]   a game, in memory or a peer
  no --game → games a and b, in memory; `root` is the root game's id, so --game refuses it
  -c <line>   run a REPL line in the root game on the game's own kernel, print it, exit (repeatable)
```

The startup line names the address the listener BOUND (`--http 127.0.0.1:0` prints the port
the system chose), and lists the games sorted.

- **One game kernel**: `space_with_store(root store)` + a `game(id, store)` corridor per
  `--game`. **Two front kernels** forward into it, because `ikigai-web` and `ikigai-ipc` both
  issue with `Kernel::issue` (no chain) and only the kernel's holder can `issue_in`:
  - `Gateway`: `urn:game:{id}:iki:tutorial:ttt:{rest}` → `urn:iki:tutorial:ttt:{rest}` issued
    in game `id`'s corridor. Over HTTP that is the mechanical mapping of
    `/game/{id}/iki/tutorial/ttt/{rest…}`; over IPC it is the name itself.
  - **The root game's gateway name is `urn:game:root:…`** (`/game/root/…` over HTTP, and a
    page at `/game/root/`): the same answers as its plain names, issued with no corridor, so
    a client can spell every game `urn:game:{id}:` with no special case. The plain names
    keep working. `root` is reserved (`--game root` is refused). The catalog lists the
    root's names ONCE, plainly — `urn:game:root:…` is a second spelling, left out the way
    an alias is — so the catalog's `urn:game:{id}:` entries remain exactly the `--game`
    games, which is what the apps' navigation lists.
  - **Games are sorted** (`Host::games`, a `BTreeMap`): the startup line, every page's
    navigation and the catalog's gateway names are in id order whatever order the `--game`
    flags came in (`the_games_are_listed_sorted_whatever_order_the_flags_came_in`). This was
    already so; the test pins it.
  - `Forward`: the root game's `urn:iki:tutorial:ttt:*` as they are.
  - A forwarded answer is re-marked `Expiry::Always` in the front: the front never saw the
    reads it depends on, so it could never cut a copy. The game kernel caches and cuts.
    (`ikigai-web` then says `Cache-Control: no-store`, which is right for a live board.)
  - **HTTP does not serve `stored:`** (404): a browser reaches the store through the rules.
    **IPC does** (a socket client is the host's own user). Unknown game → 404 / Unresolved.
- **Pages**: routes `/` (root game) and `/game/{id}` → a document around `view:game:{id}` with
  `<base href="/game/{id}/">`, `/static/htmx-2.0.4.min.js` (the book's vendored copy,
  `include_str!`), `/static/ttt.css` (the book's `css/ttt.css`, the ONE stylesheet for the
  markup), `/static/host.css` (the color variables mdbook gives the book). Page routes set
  CSP `base-uri 'self'` — `ikigai-web`'s default `base-uri 'none'` blocks the `<base>`, and
  then `/game/a` (no slash) sends every request to `/game/iki/…`. htmx is configured by a
  `<meta name="htmx-config">`: `allowEval:false`, `includeIndicatorStyles:false` (the CSP
  would refuse htmx's inline style), `historyEnabled:false`.
- **A peer store**: `ikigai_ipc::connect` → `MountedRemote::overriding(…,
  "urn:iki:tutorial:ttt:stored:", …)` — the same mount as `ikigai --override
  urn:iki:tutorial:ttt:stored:=<socket>`, NOT an alias `--mount` (ledger #578: the faces
  strip at most the first segment of an alias prefix) — then `check_store` on it, and a
  refusal names the clause: `the store at … breaks the store contract: …`. Checked live on
  2026-09-27 against `ikigai-python` and `ikigai-deno`'s `examples/tictactoe_store.*`: both
  pass (Exists included) and play over HTTP; a Python store keeps its game across a host
  restart.
- **Tests** (`tests/host.rs`, hermetic: sockets in `temp_dir()` with a length assert, HTTP on
  `127.0.0.1:0`): an HTTP move re-renders that game's board and no other; the pages, the
  static files, 404 for `stored:` and unknown games, 405 for GET on a play; a Rust
  `stored_space` served by `ikigai_ipc::serve` in the test passes the contract and plays a
  won game whose marks are in the PEER; a peer answering empty for an unplayed square is
  refused at startup; a missing socket is refused; the IPC face serves the root's and game
  `a`'s names; argument parsing.

## What increment 6 inherits

- **The markup is fixed.** A Python or Deno app that serves the board implements three
  things, all stated above: the template slot format and its choice rules, the path ↔ IRI
  rule, and a page around `template:game` with a `<base>` at the game's path. It loads the
  vendored htmx and `ttt.css` (same files, same versions).
- **Where the app gets its answers is the open choice.**
  (a) Source `view:*` from `ttt-host` over IPC by edge name (`urn:game:{id}:…`): no rendering
  in the app at all, and the board is exactly the Rust one — but then the app is a proxy.
  (b) Fill the templates itself from `template:*`, `cell:*`, `winner` and `turn` read over
  IPC: the "a dozen lines of any language" claim gets exercised, which is the point of a
  showcase. Either way a game's state must be written THROUGH the host, never to a store
  directly (no golden thread over the wire).
- ⚠ **The chain still does not cross the wire.** `ttt-host`'s `urn:game:{id}:…` edge names
  are the workaround, and they are the host's addressing, not the resource model. An
  `ikigai-web` `ScopeFn` (a request → a `Scope`, the chain twin of `CapFn`) would retire the
  HTTP front; ledger #564's "context crosses as values" would retire the IPC one.
- ⚠ **ids and two boards.** Squares carry game-free ids for htmx's focus restoration. One
  board per page is fine; two need the ids namespaced (the book's shim prefixes the game).
- ⚠ **ledger #575** will change the refusal TEXT the reply shows (the variant's Display), not
  the view code; the chapter's pinned refusal lines (they start `invalid argument`) will move with it.


## What increment 6 left

- **Linked.** Parts I–VI are the book's "Applied: tic-tac-toe" part, between "Beyond one
  host" and the polyglot tracks. The track's families sections link to part VI and back.
- **`ttt-host -c <line>`** (repeatable) runs REPL lines against the ROOT game on the game's
  own kernel, prints them the way the book's cells do, and exits without serving. It exists
  because the IPC face cannot show a verdict: its front kernel re-marks every forwarded
  answer `Expiry::Always`, so a client of the socket sees `[uncacheable]` for everything.
  `ikigai_ipc::serve` takes the `Kernel` by value, so ONE kernel cannot face both HTTP (which
  takes an `Arc`) and IPC; an `Arc`-taking serve would let the root game's names be served
  with their real verdicts. (Reported, not built.)
- **`scripts/ttt-polyglot-demo.sh`** reruns part VI for real: clones `ikigai-python` and
  `ikigai-deno` at main into `/tmp/ttt6demo/work`, builds `ttt-host`, starts the chapter's
  processes from the chapter's own `<!-- demo: start -->` blocks, reruns every
  `<!-- demo: run -->` block and diffs it, and checks each `<!-- excerpt: <repo> <path> @
  <commit> -->` block is still verbatim at main. Not in CI (python3, deno, the ikigai CLI,
  network, ports 8070–8072); it skips, exit 0, naming what is missing.
- **The root game's gateway name** `urn:game:root:…` exists (increment 6, PR #47); the apps
  still use their prefix switch. They can drop it, but they keep their `/` page special case
  (the host's page for `/` has `<base href="/">` and the title "the root game").
- ⚠ **ledger items 583 → 585**: when `Conflict` crosses the wire and the move uses it, part
  VI's refusal transcript (today `invalid argument`, then `conflict:`) and parts III and V
  move with it. The demo script will say so.

## The views as templates (after increment 6)

- **No view code.** `ikigai-fn` 0.3.0 made compose a view template language, and ikigai-fn
  PR #7 proved the board could be pure composition byte for byte. Every view that READS is
  now a template bound at a name (`view`, `compose_over`): `view:board`,
  `view:square:{x}:{y}`, `view:status`, `view:reply`, `view:game:{game}`. The `{{slot}}`
  format, `fill`, `slots`, `escape`, `Slot` and `Fill` are gone from the crate; the
  JS shim and `ttt-host` fill nothing (each sources `view:game:{game}`).
- **What stays code, and why.** `view:play:{x}:{y}` and `view:reset` are a write and then
  a read of `view:reply`, in that order. A template's markers are all `Source`, so a
  sequence cannot be one. The move, the reset and the rest of the game are unchanged.
- **Byte-identical**, three ways: `tests/parity.rs` (every state, against the old filler
  over `tests/reference/`); `ttt-host`'s whole HTTP face captured before and after (68
  requests across fresh, in-progress, won, drawn, refused and reset games, plus `-c` lines
  with a hostile mark), `cmp` equal; and the book's pinned outputs, which did not move except
  where a cell shows a template's own source.
- ⚠ **The Python and Deno apps fill the OLD `{{slot}}` templates.** Against a `ttt-host`
  built from this commit they pass the markers through as text, so part VI's app transcripts
  and its three-digest board comparison fail until each app implements the template
  language above ("The template language"). The chapter says so. The transcripts were NOT
  changed to match stale apps.
