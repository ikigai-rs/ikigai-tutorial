# tic-tac-toe

The code taught by the book's applied arc: noughts and crosses designed as resources
before it is code, after *Resource Oriented Analysis and Design*, Parts 1–9, by Peter
Rodgers (NetKernel News 3.34–3.42, August–October 2012). The prose is
`books/ikigai/src/applied/tic-tac-toe-N.md`, one page per increment; the listings are
included from this crate by anchor.

The chapters are **drafts** until the whole arc is done: listed in
`books/ikigai/drafts.txt`, not in `SUMMARY.md`, because a merge to main deploys the book.
Read them with `./scripts/serve-with-drafts.sh`.

## The arc

Each increment is its own satellite, its own PR, and its own page. Brian reviews each on
the drafts server before the next is dispatched, and may reorder or cut what follows.

| Increment | Chapter content | Series parallel | Status |
|---|---|---|---|
| **1** | Name the sets on paper (the resource model); the atoms: a stored cell and the platonic empty cell; wire the game into the page's kernel; runnable cells | Parts 1–3 | done — `tic-tac-toe-1.md` |
| **2** | Composites: a line of cells by name, rows / columns / diagonals as zero-code `Alias`es, the board; one move recomputes the lines through it and the board (trace shows it) | Part 4 | done — `tic-tac-toe-2.md` |
| **3** | Rules as resources: CheckSet (a representation that is a list of IRIs), winner, whose turn; constraint at the edge, model kept loose | Parts 6–7 | done — `tic-tac-toe-3.md` |
| **4** | Many games: the game as context (a `Scope` corridor) versus the game in the name — argue it; the store as a swappable space | Part 8 | done — `tic-tac-toe-4.md` |
| **5** | The views and templates as resources; a playable board in the page over the same kernel (htmx, answered by the wasm kernel); `ttt-host`; the wrap-up | Parts 5, 9 | done — `tic-tac-toe-5.md`, `crates/ttt-host` |

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
  only through the lines it touched. The centre is on four lines, a corner on three, an
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
  `space_over(CellStore)` are unchanged in behaviour and are now `space_with_store(
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
| `view:board` | `ttt-view-board` | Source | the board as HTML, cacheable |
| `view:status` | `ttt-view-status` | Source | `X to play.` / `O has won.` / `A draw.` — HTML text, cacheable |
| `view:play:{x}:{y}` | `ttt-view-play` | **Sink only** | Sinks `move:{x}:{y}`; answers the `reply` template |
| `view:reset` | `ttt-view-reset` | **Sink only** | Sinks `reset`; answers the `reply` template |
| `reset` | `ttt-reset` | **Sink only** | Deletes every square a `LINES` rule passes through; `The board is clear` |

`view:board` reads the templates, the **winner** and the nine **cells** (not the board text:
a cell's answer is the exact mark, and the board concatenates marks, which is ambiguous for a
multi-character mark). `view:status` reads the winner and, while the game is on, the turn.
Both are composites, so they are cached per game and a move recomputes them through what it
touched. A view's Sink answer is not cacheable; the reply READS `view:status`, so after a
play the status is cached again (the new one) and the board is not until someone asks.

### The template format — what every host of this markup implements

Templates are the files in `templates/` (`TEMPLATES` in `lib.rs`), served at
`template:{name}`: `game`, `board`, `square-open`, `square-taken`, `square-closed`,
`status-turn`, `status-won`, `status-draw`, `reply`. A file's final newline is not part of
the template.

- A **slot** is `{{` NAME (` ` INTEGER)* `}}`: NAME is `[a-z][a-z-]*`, each INTEGER a plain
  integer (`0`, `-2`; not `01`, `+1`, `-0`), one space before each. The regex is
  `\{\{([a-z][a-z-]*)((?: -?[0-9]+)*)\}\}`. Rust refuses any other `{{` (`fill`/`slots`).
- A value is **text, HTML-escaped** (`& < > " '` → `&amp; &lt; &gt; &quot; &#39;`), except
  the two slots that take another template's output raw: `{{square X Y}}` and `{{status}}`.
- No loops, no conditions. Which template to use is chosen by the filler:
  - `board`: each `{{square X Y}}` is the square template for the cell at (X, Y), filled with
    `{{x}}`, `{{y}}`, `{{mark}}`: **`square-taken`** if the cell is not `-`; else
    **`square-open`** while the winner is `-`; else **`square-closed`**. The template, not the
    code, decides which squares exist (a test pins them to the `LINES` squares).
  - status: winner `-` → **`status-turn`** with `{{mark}}` = the turn; `draw` →
    **`status-draw`**; `X`/`O` → **`status-won`** with `{{mark}}` = the winner.
  - `reply` (a play's or reset's answer): `{{message}}` = what the write answered, **or the
    error's Display on a refusal** (the view never looks inside an error — so ledger #575's
    typed precondition variant will render with no view change), `{{status}}` = the status.
  - `game`: the page shell; `{{game}}` = the game's id as the host names it (text).
- The JS filler in `books/ikigai/js/ttt.js` (`fill`) is the reference in a dozen lines; a
  Python one is `re.sub` with the regex above and a callback.

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
- A path that is absolute, has an empty, `.` or `..` segment, or is a URL, is not answered.

Hosts, as built or planned:

| host | where the game comes from | status |
|---|---|---|
| the book page (`js/ttt.js`) | `data-game` on the `.ttt-play` element holding the board | done |
| `ttt-host` over HTTP | the path prefix `/game/{id}/` (the page's `<base href>`); `/` is the root game | done |
| Python / Deno apps | their own path prefix, same rule | increment 6 |

### The in-page shim

- `js/ttt.js` loads the vendored htmx (`books/ikigai/src/vendor/htmx-2.0.4.min.js`, byte-
  identical to `ikigai-web/assets/htmx.min.js`, sha256 `e209dda5…fb447`, 0BSD) only on a
  page with a board, fills `template:game` into each `.ttt-play[data-game]`, and
  `htmx.process`es it.
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
`a11y/axe.mjs` + `book-a11y` over a with-drafts build. ⚠ CI does not see this page's board:
drafts are not built by `pages.yml`, and jsdom cannot load the wasm, so CI's axe only ever
sees the fallback line. Re-run the in-browser check when the page is linked.

### `ttt-host` (`crates/ttt-host`)

```text
ttt-host [--http <addr>] [--socket <path>] [--store <socket>] [--game <id>[=<socket>]]…
  --http 127.0.0.1:8070 (default)   --socket $TMPDIR/ttt-host.sock (default; must fit 104 bytes)
  --store <socket>   the ROOT game's store is a peer     --game <id>[=<socket>]   a game, in memory or a peer
  no --game → games a and b, in memory
```

- **One game kernel**: `space_with_store(root store)` + a `game(id, store)` corridor per
  `--game`. **Two front kernels** forward into it, because `ikigai-web` and `ikigai-ipc` both
  issue with `Kernel::issue` (no chain) and only the kernel's holder can `issue_in`:
  - `Gateway`: `urn:game:{id}:iki:tutorial:ttt:{rest}` → `urn:iki:tutorial:ttt:{rest}` issued
    in game `id`'s corridor. Over HTTP that is the mechanical mapping of
    `/game/{id}/iki/tutorial/ttt/{rest…}`; over IPC it is the name itself.
  - `Forward`: the root game's `urn:iki:tutorial:ttt:*` as they are.
  - A forwarded answer is re-marked `Expiry::Always` in the front: the front never saw the
    reads it depends on, so it could never cut a copy. The game kernel caches and cuts.
    (`ikigai-web` then says `Cache-Control: no-store`, which is right for a live board.)
  - **HTTP does not serve `stored:`** (404): a browser reaches the store through the rules.
    **IPC does** (a socket client is the host's own user). Unknown game → 404 / Unresolved.
- **Pages**: routes `/` (root game) and `/game/{id}` → a document around `template:game` with
  `<base href="/game/{id}/">`, `/static/htmx-2.0.4.min.js` (the book's vendored copy,
  `include_str!`), `/static/ttt.css` (the book's `css/ttt.css`, the ONE stylesheet for the
  markup), `/static/host.css` (the colour variables mdbook gives the book). Page routes set
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
