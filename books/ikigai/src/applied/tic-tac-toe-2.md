# Tic-tac-toe, II: lines, aliases and the board

[Part I](tic-tac-toe-1.md) built the atom: a stored cell that holds a mark, and a platonic
cell that answers `-` where nothing has been played. Everything else in the game is a
gathering of cells, or a question about one, and this part builds the gatherings: a line
of cells, the eight lines a board has, and the board. It follows the fourth part of the
series Part I credits, which is where the original makes the same move.

Only one of the three new things is an endpoint worth reading. The eight lines are not
code at all, and the board is a dozen lines that read three names. The lesson is what that
arrangement does when a mark is played.

## A line of cells has a name

A line of cells is named by its members, in order:
`urn:iki:tutorial:ttt:cells:0.0,1.0,2.0` is the cells at (0,0), (1,0) and (2,0), read in
that order. Each member is `x.y`, and members are joined by commas.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:line_names}}
```

It is a sequence rather than a set, because a row reads left to right: the same three cells
named the other way round are another line, and answer the other way round. And the model
stays as loose as the cell's. Any coordinates, any length, the same cell twice if you like
— nothing in the name knows a board is three by three.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:members}}
```

The one-spelling rule from Part I carries over, and for the same reason. Two spellings of
one line would be two names over one answer, so two cache entries, and a move would have to
find both. So a member is two integers in their plain form joined by one `.`, members are
joined by one `,`, and there is nothing else: no spaces, no empty member, no trailing
comma. Both separators are safe everywhere the name travels. RFC 8141 allows `,` and `.`
in a URN. The engine's grammar treats only whitespace, quotes, `|`, `(`, `)`, `;` and a
`..` standing alone as syntax. And a template's last variable captures the rest of the
name, commas included.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:line}}
```

The endpoint is a composite over the platonic cell and nothing more. It sources each member
through the kernel, as any caller would, and puts the marks side by side. It is
`.cacheable()` and names no golden thread. Its dependencies are its sub-requests, and
[Part I](tic-tac-toe-1.md) showed what the kernel does with those.

As in Part I, one kernel serves every cell on this page and keeps its cache between runs,
so run them in order. A page starts on a fresh board. Here is a row of three empty cells,
and a stranger line of two, one of them far off any board:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:cells:0.0,1.0,2.0
source urn:iki:tutorial:ttt:cells:1.1,-4.9'>
<pre class="ikigai-run-expected">---
[computed]
--
[computed]</pre>
</div>

And the second spellings, refused. A trailing comma is an empty member, and `01` is `1`
spelled another way:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:cells:0.0,1.0,2.0,
source urn:iki:tutorial:ttt:cells:0.0,01.0'>
<pre class="ikigai-run-expected">error: invalid argument `list`: an empty member — cells are x.y, joined by single commas
error: invalid argument `list`: `01` is not an integer in its plain form (e.g. 0, 2, -1)</pre>
</div>

## Rows, columns and diagonals: eight names, no code

A row is a line of cells with a shorter name. So it is written as exactly that: a name for
a name. Eight exact rules, as text:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:lines}}
```

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:space}}
```

The game's space is its bound endpoints wrapped in an
[`Alias`](../getting-started/scope-and-alias.md) that rewrites through that table. Four of
them are this part's and the one before it; the last four are the rules, which
[Part III](tic-tac-toe-3.md) builds, reading the same table. No
endpoint implements a row. `urn:iki:tutorial:ttt:row:0` resolves because the table
rewrites it to `urn:iki:tutorial:ttt:cells:0.0,1.0,2.0`, and the line endpoint answers
that. The table parses from text, so it could as easily have been read from a file.
Nothing about it is compiled logic.

This table is also where "three by three" lives, and nowhere else does. The cell takes any
coordinates and the line takes any cells, so the size of the board is the one thing the
game has to say for itself, and it says it here, as data. Connect Four is a board of seven
columns and six rows, won by four in a line, and it has sixty-nine such lines. It would be
another table over the same two endpoints, not another crate. That is what keeping Part I's
model loose was for: the atoms do not change when the game does, only the names laid over
them.

`y` runs down the board, so row 0 is the top row. A line reads left to right, or top to
bottom, and diagonal 1 runs from the top-right corner to the bottom-left.

## One name for the row, one cache entry

An alias is a second name, not a second resource. The `Alias` reports the name it rewrote
to, and the kernel files the answer under that name before it computes the cache key. So
`row:1` and the line it names are one cache entry, and a read through either name serves
the other:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/lines.rs:one_entry}}
```

Read the row by its short name, then ask about the line by its own:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:row:1
cache urn:iki:tutorial:ttt:cells:0.1,1.1,2.1
source urn:iki:tutorial:ttt:cells:0.1,1.1,2.1'>
<pre class="ikigai-run-expected">---
[computed]
cached
---
[cached]</pre>
</div>

The line was never read under its own name, and it is cached. That is what a zero-code
alias over a set buys in ikigai: the eight short names add nothing to the cache and nothing
to invalidate. Whatever reaches the line reaches the row, because they are one thing.

Two consequences, both of them core's decisions rather than this game's. Ask the row what
it is (`describe urn:iki:tutorial:ttt:row:0 text/turtle`) and it describes `ttt-cells`,
because a description reports the backing resource. And probe the line's cache by the
*line's* name, as the cell above does. `cache` asks without resolving, and a rewrite made
inside a space is known only by resolving through it, so `cache urn:iki:tutorial:ttt:row:1`
answers `not cached` even while the entry is there. The probe under-reports a hit; it never
invents one. (A table installed on the kernel itself with `Kernel::with_aliases` is known
to the probe. This one travels with the game's space instead, so that the page and every
other host get the rows with no wiring of their own.)

Nor does the catalog list the aliases. It lists what is bound — `cells:{list}` once, and
`board` — and the table is data of its own: in `urn:kernel:topology`, the game's space is an
`ik:Alias` node carrying eight `ik:RewriteRule`s, each with its logical and canonical name,
above the `EndpointSpace` that binds the templates. Type `source urn:kernel:topology`
into any cell on this page to see the whole arrangement as a graph.

## The board

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:board}}
```

The board reads the three rows by their short names, through the kernel, like any other
caller. It does not know that a row is a line, or that a line is cells, and it does not
need to. It answers them top to bottom, one row per line of text:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">---
---
---
[computed]</pre>
</div>

The board was computed, but not all of it was. Row 1 was already cached from the cell
above, so the board's read of it ran nothing.

## One move, and what it recomputes

Here is what the arrangement is for. The board depends on three rows, each row on three
cells, each cell on its stored mark, and the kernel recorded every one of those
dependencies as it resolved them. So a move reaches exactly what was derived from the cell
it changed:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/lines.rs:normalized}}
```

Run it here. First read some lines the board does not: the column and the diagonal through
the top-left corner, and one of each that miss it:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:column:0
source urn:iki:tutorial:ttt:column:1
source urn:iki:tutorial:ttt:diagonal:0
source urn:iki:tutorial:ttt:diagonal:1'>
<pre class="ikigai-run-expected">---
[computed]
---
[computed]
---
[computed]
---
[computed]</pre>
</div>

Now play the corner. The four probes are row 0, column 0, diagonal 0 and the board, the
four things that pass through (0,0). None of them is cached any more:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:stored:0:0 X
cache urn:iki:tutorial:ttt:cells:0.0,1.0,2.0
cache urn:iki:tutorial:ttt:cells:0.0,0.1,0.2
cache urn:iki:tutorial:ttt:cells:0.0,1.1,2.2
cache urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">ok
[uncacheable]
not cached
not cached
not cached
not cached</pre>
</div>

And the ones that miss it are row 1, row 2, column 1 and diagonal 1. All four are still
cached:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:ttt:cells:0.1,1.1,2.1
cache urn:iki:tutorial:ttt:cells:0.2,1.2,2.2
cache urn:iki:tutorial:ttt:cells:1.0,1.1,1.2
cache urn:iki:tutorial:ttt:cells:2.0,1.1,0.2'>
<pre class="ikigai-run-expected">cached
cached
cached
cached</pre>
</div>

The `Sink` named one stored cell. Nothing in any endpoint named a row, a column or the
board, and nothing walked the board looking for what to throw away. The dependencies were
recorded when the answers were computed, and the cut followed them.

Now read the board again, traced, to see what it recomputes:

<div class="ikigai-run" data-cmd='trace urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">trace  urn:iki:tutorial:ttt:board
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:ttt:board   ttt-board · computed · ThreadId(1) · —   → 11b  X--…
├─ urn:iki:tutorial:ttt:cells:0.0,1.0,2.0   ttt-cells · computed · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:0 -&gt; urn:iki:tutorial:ttt:cells:0.0,1.0,2.0
│  ├─ urn:iki:tutorial:ttt:cell:0:0   ttt-cell · computed · ThreadId(1) · —
│  │  └─ urn:iki:tutorial:ttt:stored:0:0   ttt-stored · computed · ThreadId(1) · —
│  ├─ urn:iki:tutorial:ttt:cell:1:0   ttt-cell · cached · ThreadId(1) · —
│  └─ urn:iki:tutorial:ttt:cell:2:0   ttt-cell · cached · ThreadId(1) · —
├─ urn:iki:tutorial:ttt:cells:0.1,1.1,2.1   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:1 -&gt; urn:iki:tutorial:ttt:cells:0.1,1.1,2.1
└─ urn:iki:tutorial:ttt:cells:0.2,1.2,2.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:2 -&gt; urn:iki:tutorial:ttt:cells:0.2,1.2,2.2</pre>
</div>

Read it from the top. The board was computed. Row 0 was computed, and inside it only the
corner went back to the store: its two neighbours were served from cache. Rows 1 and 2 were
served whole, and their six cells were never asked, so they are not in the tree at all.
Each row carries a note, `alias=… -> …`, which is the name the board asked for beside the
name the kernel filed it under. Of nine stored cells, one was read.

That is the normalized-recompute property, and it comes from the shape of the design rather
than from any cleverness in the code. Each resource depends only on the resources it is made
of, so a change travels up exactly the paths that lead to it. (The corner shows its stored
cell as a child because it has been played. An unplayed cell that recomputed would show as
one line: its failed read of the store is recorded as a dependency, but a trace in this
release does not draw it.)

One more move, in the centre, and the board again: computed once, then served.

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:stored:1:1 O
source urn:iki:tutorial:ttt:board
source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">ok
[uncacheable]
X--
-O-
---
[computed]
X--
-O-
---
[cached]</pre>
</div>

The centre is on four lines (row 1, column 1 and both diagonals), a corner on three, and an
edge on two. That count is about to matter.

## Next

Part III asks questions of the board. Which lines a cell is on becomes a resource of its
own: a representation that is a list of the names of other resources. Whose turn it is and
who has won are questions about those lines, and the rules of noughts and crosses arrive
there, at the edge of the application, with the model underneath still loose.
