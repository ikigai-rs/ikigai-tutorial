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
| 3 | Rules as resources: CheckSet (a representation that is a list of IRIs), winner, whose turn; constraint at the edge, model kept loose | Parts 6–7 | |
| 4 | Many games: the game as context (a `Scope` corridor) versus the game in the name — argue it | Part 8 | |
| 5 | A playable board in the page over the same kernel; the wrap-up | Parts 5, 9 | |

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
