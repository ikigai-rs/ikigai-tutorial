# Tic-tac-toe, IV: many games, and where the game belongs

Parts [I](tic-tac-toe-1.md) to [III](tic-tac-toe-3.md) play one game. There is one board
per kernel, which is why each page starts on a fresh one. Two players in two games need two
boards, and the question this part answers is where the game goes. It could go in every
name, so that a cell is a cell *of game 7*. Or it could go around the names, as the context
they are resolved in, so that a cell stays a cell and the game is where you are standing
when you ask for it. The eighth part of the series Part I credits takes the second answer,
with contextual configuration. This part takes it too, argues against the first, and
changes no name and no endpoint above the atom to do it.

## Only the store holds a game

Look back over the three parts for what holds state. The stored cell does. Everything
above it is a pure function of names it sources: the platonic cell reads the stored cell,
a line reads cells, the board reads rows, the winner reads lines, the turn reads the winner
and the board, and the move reads four of those before it writes the stored cell. So a game
*is* what `urn:iki:tutorial:ttt:stored:{x}:{y}` resolves to. Change that, for one request
and everything it gives rise to, and the whole game changes with it. No code above the
store has to know.

That makes two small changes, and they are the whole of this part's code.

## The store is a space

The game's space used to bind the stored cell beside everything else. Now it takes the
store as an argument, and the store is any space at all:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:space}}
```

`space_with_store` binds every composite, then puts the store after them in a `Fallback`,
and wraps the lot in Part II's table. The composites reach the store by name, through the
kernel, so they cannot tell which space answered. `space()` and `space_over()` are what they
were in Parts I–III, spelled as the special case: the in-memory store, `stored_space`, over a
`CellStore`.

## A game is a corridor

The second change is a game: a resolution chain whose one corridor binds the stored cell to
that game's own store.

<!-- urn-gate: illustration urn:iki:tutorial:ttt:game:a — the corridor name of game a, as
     the page's trace and the prose print it. Not a resource. -->
<!-- urn-gate: illustration urn:iki:tutorial:ttt:game:b — likewise, game b. -->

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:game}}
```

[Scope and alias](../getting-started/scope-and-alias.md) showed what a corridor does. It
sits ahead of the root, and a name it binds shadows the root's binding for the request
*and every sub-request it gives rise to*. Here the corridor binds exactly one template, the
stored cell's. Ask for the board in game `a`, and the board reads rows, the rows read
cells, and every cell asks for its stored cell, which game `a`'s corridor answers. Every
other name in that chain of requests missed the corridor and was answered by the root, by
the same endpoints that answered it in Part III.

Two things about the corridor are rules rather than conveniences.

**Its name is a claim.** `urn:iki:tutorial:ttt:game:a` is not a resource: nothing binds it
and nothing resolves it. It is the name the chain carries, and the cache files every answer
computed in game `a` under it. `with_named` asserts *any corridor with this name holds these
doors*, so a game's corridor is built once and kept, and two stores never share an id.

**Injecting it is authority.** Whoever may push a corridor ahead of the root chooses what
the stored cell means, for every sub-request below, so whoever may push one chooses the
game. That is why the choice belongs to the host and not to a line of the REPL.

## Choosing a game in this page

The engine's grammar has no way to name a chain, and that is right: a line that could name
one would let whoever wrote the line decide what every name in it means. So on this page
the game is markup. A cell the chapter wrote with `data-game='a'` says so beside its label,
*Command, in game a*, and the page's own host code runs each of its lines in game `a`'s chain.
You can edit the command. You cannot edit the game.

The page does it with the engine it already had. The engine takes any resolver, so the page
gives it one that issues every request in one chain, on the same kernel every other cell
uses:

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:in_game}}
```

One engine per game, each built the first time a cell names that game and kept for the life
of the page, each with its own empty in-memory store. One kernel under all of them, so one
cache and one set of golden threads, partitioned by the chain each request carries. A cell
with no `data-game` plays the root's game, the one every earlier part played.

## Two games, side by side

X opens in game `a`, in the centre:

<div class="ikigai-run" data-game='a' data-cmd='sink urn:iki:tutorial:ttt:move:1:1
source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">X plays 1,1
[uncacheable]
---
-X-
---
[computed]</pre>
</div>

Game `b` has its own board. It is empty, and its first move is X's, because its turn is read
from its own board:

<div class="ikigai-run" data-game='b' data-cmd='source urn:iki:tutorial:ttt:board
sink urn:iki:tutorial:ttt:move:0:0
source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">---
---
---
[computed]
X plays 0,0
[uncacheable]
X--
---
---
[computed]</pre>
</div>

And the root's game, which no cell has named yet on this page, is untouched by either:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">---
---
---
[computed]</pre>
</div>

The same names, the same endpoints, three boards. The crate says the same:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/games.rs:two_games}}
```

## A move stays in its game

The move is the resource that could get this wrong. It reads four resources before it
writes (the CheckSet, the winner, the cell and the turn) and then issues a `Sink`. Every one
of those five requests has to resolve in the move's own game. If any of them escaped to
another game, a player would be refused because *someone else's* game was over, or told a
square was taken because someone else had taken it.

They cannot escape, because the move does not choose where they go. It issues them through
its invocation, and the invocation hands its chain to every sub-request. So finish game `a`.
X takes the top-left corner, which in game `b` is already X's, and in game `a` is free:

<div class="ikigai-run" data-game='a' data-cmd='sink urn:iki:tutorial:ttt:move:1:0
sink urn:iki:tutorial:ttt:move:0:0
sink urn:iki:tutorial:ttt:move:2:0
sink urn:iki:tutorial:ttt:move:2:2
source urn:iki:tutorial:ttt:winner'>
<pre class="ikigai-run-expected">O plays 1,0
[uncacheable]
X plays 0,0
[uncacheable]
O plays 2,0
[uncacheable]
X plays 2,2
[uncacheable]
X
[computed]</pre>
</div>

Game `a` is over, so a move in game `a` is refused:

<div class="ikigai-run" data-game='a' data-cmd='sink urn:iki:tutorial:ttt:move:0:2'>
<pre class="ikigai-run-expected">error: invalid argument `x, y`: the game is over — X has won</pre>
</div>

The same move in game `b` plays, and game `b` goes on:

<div class="ikigai-run" data-game='b' data-cmd='sink urn:iki:tutorial:ttt:move:0:2
source urn:iki:tutorial:ttt:winner
source urn:iki:tutorial:ttt:turn'>
<pre class="ikigai-run-expected">O plays 0,2
[uncacheable]
-
[computed]
X
[computed]</pre>
</div>

The crate pins it both ways round, and counts reads on each game's store to show that
nothing game `b` asked touched game `a`'s:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/games.rs:stays}}
```

## Two games, two caches

The cache key carries the chain's fingerprint, so an answer computed in one game is never
served in another. For the board that is exactly right. For the CheckSet it is a cost.

Ask game `a` which lines pass through the middle of the right-hand column, twice:

<div class="ikigai-run" data-game='a' data-cmd='source urn:iki:tutorial:ttt:checkset:2:1
source urn:iki:tutorial:ttt:checkset:2:1'>
<pre class="ikigai-run-expected">urn:iki:tutorial:ttt:column:2
urn:iki:tutorial:ttt:row:1
[computed]
urn:iki:tutorial:ttt:column:2
urn:iki:tutorial:ttt:row:1
[cached]</pre>
</div>

Game `b` has not got it, and computes the same two names again:

<div class="ikigai-run" data-game='b' data-cmd='cache urn:iki:tutorial:ttt:checkset:2:1
source urn:iki:tutorial:ttt:checkset:2:1'>
<pre class="ikigai-run-expected">not cached
urn:iki:tutorial:ttt:column:2
urn:iki:tutorial:ttt:row:1
[computed]</pre>
</div>

Nor has the root:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:ttt:checkset:2:1'>
<pre class="ikigai-run-expected">not cached</pre>
</div>

[Part III](tic-tac-toe-3.md) called the CheckSet platonic in the strong sense: it depends on
its name and a constant table, nothing a move can change, so it is computed once in the
life of the kernel. Under games that is no longer true. It is computed once in the life of
each *game*. The crate measures it: every CheckSet of the board, read in the root and in
three games, is nine distinct answers filed thirty-six times.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/games.rs:partitions}}
```

Nothing is wrong. Keying on the whole chain is sound and over-partitioned: two chains that
differ only in a corridor a request never touched do not share. The CheckSet never touches
the game's corridor. It is answered by the root, and the chain it was asked in made no
difference to the answer. The fix is to key an entry on the part of the chain that could
have answered it, which since core 0.1.78 the kernel can see: a resolution reports the space
that answered it. Keying the cache on that is a decision taken and not yet built (the
resolution context, ledger item 563), so for now each game computes its own copy of
something that is the same in every game.

## A golden thread is a name

There is a second cost, the other way round. The key is partitioned by game, but a golden
thread is not. A thread is a name, and a move cuts the thread named after the square it
played, in every game at once.

It has already happened on this page. Game `b` read its top row with its board, several
cells back, and nothing in game `b` has touched that row since. Yet read it now and it is
computed, not served:

<div class="ikigai-run" data-game='b' data-cmd='source urn:iki:tutorial:ttt:row:0
cache urn:iki:tutorial:ttt:cells:0.0,1.0,2.0'>
<pre class="ikigai-run-expected">X--
[computed]
cached</pre>
</div>

Game `a` played three moves on the top row in between, and each one cut the thread of the
square it played, in game `b` as well. Now the row is cached again, so watch it go on
purpose.

The root's game plays two moves, the second of them on the top-left corner:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:move:1:1
sink urn:iki:tutorial:ttt:move:0:0'>
<pre class="ikigai-run-expected">X plays 1,1
[uncacheable]
O plays 0,0
[uncacheable]</pre>
</div>

Nothing in game `b` changed, and game `b`'s top row is no longer cached. Traced, game `b`'s
corner shows why it was recomputed, and whose store answered it:

<div class="ikigai-run" data-game='b' data-cmd='cache urn:iki:tutorial:ttt:cells:0.0,1.0,2.0
trace urn:iki:tutorial:ttt:cell:0:0'>
<pre class="ikigai-run-expected">not cached
trace  urn:iki:tutorial:ttt:cell:0:0
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process · chain urn:iki:tutorial:ttt:game:b root
&#32;
urn:iki:tutorial:ttt:cell:0:0   ttt-cell · computed · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:b root   → 1b  X
└─ urn:iki:tutorial:ttt:stored:0:0   ttt-stored · computed · ThreadId(1) · — · answered-by=urn:iki:tutorial:ttt:game:b scope=urn:iki:tutorial:ttt:game:b root</pre>
</div>

Read the two nodes. Both ran in game `b`'s chain (`scope=`). The platonic cell was answered
by the root, like every name but one. The stored cell was answered by game `b`'s corridor
(`answered-by=`), and it still says X, which is game `b`'s mark, not the root's O. The root's
move cut `stored:0:0`, and the cut reached every entry hanging from that thread, whatever
game computed it. So game `b` recomputed a corner that had not changed, and got the same
answer.

That is sound, because a cut can only make the kernel compute something again, never serve
something stale. And it is a cost that grows with the number of games: a move in any game
recomputes the lines through that square in all of them. A thread named by the square *in
its game* would stop it. That is the same missing piece as the CheckSet's, seen from the
other side, and it waits on the same decision.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/games.rs:cut}}
```

## The game in the name

The other answer puts the game in every name. The stored cell becomes
`urn:iki:tutorial:ttt:game:{g}:stored:{x}:{y}`, and no corridor is needed.

<!-- urn-gate: illustration urn:iki:tutorial:ttt:game:{g}:stored:{x}:{y} — the design this
     section argues AGAINST: the stored cell with the game in its name. Nothing binds it,
     on purpose. -->

It does not stop at the stored cell. Every composite that names a cell now has to name the
game too, so every template above the store gains `{g}`: the cell, the line, the board, the
CheckSet, the winner, the turn and the move. Each endpoint has to read `g` out of its own
name and spell it into every name it builds. Part II's aliases are worse off. They are
*exact* rules, eight of them, one per line of one board, and an exact rule does not
parameterize. A prefix rule rewrites a prefix and not the middle of a name, so either the
table grows eight rules per game, or the game moves to the end of every name, or the
composites stop reading rows by their aliases and build line names themselves. Every one of
those makes the code above the store know about games, which is the thing Parts I–III were
built not to need.

Keeping the game as context keeps every name a game-free name, and keeps the one thing that
differs between games in the one place it lives: the store.

## A store you can swap

A game's store is any space that binds the stored cell and keeps its contract. The contract
is short, and the crate states it as a check that any store can be run against:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:contract}}
```

Each clause is there because something above the store depends on it. `NotFound` for an
unplayed square is what the platonic cell turns into `-`; an empty answer instead would make
an empty square read as nothing, and a line of `X` and two empty cells would read `X` rather
than `X--`. The mark's exact bytes are what the line concatenates, so a trailing newline
would break every row. A mark that is not cacheable would leave nothing above the store
cacheable, because a composite is only as cacheable as its least cacheable part. And one
spelling per coordinate is Part I's rule: two names for one square would be two golden
threads over one piece of state.

A store written a different way, marks kept in a map by the cell's name, keeps the same
contract and plays the same game:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/games.rs:contract_ok}}
```

That is the point of making the seam a `Space`. A game backed by the in-memory store and a
game backed by a store in another process differ only in what their corridor binds. Stores
written in Python and TypeScript, served over the wire by those languages' ikigai faces, are
next: the same contract, and the same game above them.

## Three boards, one kernel

Everything played on this page, in each game:

<div class="ikigai-run" data-game='a' data-cmd='source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">XOO
-X-
--X
[computed]</pre>
</div>

<div class="ikigai-run" data-game='b' data-cmd='source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">X--
---
O--
[computed]</pre>
</div>

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">O--
-X-
---
[computed]</pre>
</div>

## Next

Part V puts a board in this page that you can play by clicking, over the same kernel and the
same resources, and draws the arc together.
