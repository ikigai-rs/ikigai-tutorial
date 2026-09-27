# Tic-tac-toe, III: the rules as resources

[Part II](tic-tac-toe-2.md) built the gatherings: a line of cells, the eight lines a board
has, and the board. None of it knows the rules of tic-tac-toe. This part adds
them, and adds them the same way as everything else: as resources. Which lines a square is
on, who has won and whose turn it is are each a name that answers. Then comes the one
resource that changes the game, the move, and the rules are enforced there and nowhere
else. It covers the ground of the sixth and seventh parts of the series Part I credits.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:rule_names}}
```

## Which lines a square is on

The center of the board is on four lines, a corner on three and an edge on two. That is a
fact about the board, and it can be a resource of its own: the **CheckSet**,
`urn:iki:tutorial:ttt:checkset:{x}:{y}`, which answers the names of the lines through a
square, one per line of text.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:checkset}}
```

Look at what the answer is. It is not marks, and it is not a board. It is **names of other
resources**, and each one can be resolved. A representation whose content is links is
what makes data linked, and nothing about it needed a new mechanism: a list of names is
text, and newline-separated text is how the ecosystem writes a list.

The list is not written anywhere by hand. The CheckSet reads Part II's table of eight
aliases and keeps each alias whose line holds the square. So "three by three" still lives
in that table and nowhere else. A board of another size, or Connect Four's sixty-nine
lines, is another table, and the CheckSet would answer for it with no change of code.

It answers the alias names, `row:1` rather than `cells:0.1,1.1,2.1`, because those read as
the game. Each one resolves, through the alias, to its line, and the line's own name is
the one the cache files the answer under.

As before, one kernel serves every cell on this page, and the page starts on a fresh
board. Ask the center, twice:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:checkset:1:1
source urn:iki:tutorial:ttt:checkset:1:1'>
<pre class="ikigai-run-expected">urn:iki:tutorial:ttt:column:1
urn:iki:tutorial:ttt:diagonal:0
urn:iki:tutorial:ttt:diagonal:1
urn:iki:tutorial:ttt:row:1
[computed]
urn:iki:tutorial:ttt:column:1
urn:iki:tutorial:ttt:diagonal:0
urn:iki:tutorial:ttt:diagonal:1
urn:iki:tutorial:ttt:row:1
[cached]</pre>
</div>

A corner, an edge, and a square off the board. The last answers the empty list. It is
still a cell (Part I's model takes any coordinates), but no line passes through it, and
only the table knows where the board is:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:checkset:0:0
source urn:iki:tutorial:ttt:checkset:1:0
source urn:iki:tutorial:ttt:checkset:3:1'>
<pre class="ikigai-run-expected">urn:iki:tutorial:ttt:column:0
urn:iki:tutorial:ttt:diagonal:0
urn:iki:tutorial:ttt:row:0
[computed]
urn:iki:tutorial:ttt:column:1
urn:iki:tutorial:ttt:row:0
[computed]
[computed]</pre>
</div>

### A platonic resource

The CheckSet depends on its own name and a constant table, and on nothing else. It issues
no sub-request, so it records no dependency that a `Sink` could cut. Its `.cacheable()` is
`Expiry::Never`, and here that is literally true: once computed, it is served for the life
of the kernel, whatever is played. The series has a name for a resource like this, a
*platonic resource*, computed once in the life of the system. The cell in Part I was
platonic in a weaker sense, because it answers for every square before anyone plays one.
The CheckSet is platonic in the strong one: no move can make it wrong, so none ever
recomputes it.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/rules.rs:platonic_checkset}}
```

The end of this page checks the same thing in the page's own kernel, after a whole game.

## Who has won, and whose turn it is

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:winner}}
```

The winner reads the eight lines by their alias names, from the same table, through the
kernel. A line is won when all three of its marks are one mark and not the empty cell. If
none is won and no line has an empty cell left, the board is full, and the answer is
`draw`. Otherwise it is `-`, for nobody yet.

It is a pure function of the lines, so it is `.cacheable()`, and like the board in Part II
it names no golden thread. Its dependencies are the eight lines it read. A move reaches it
through the lines that pass through the move's square, and through nothing else.

This is where the series puts the CheckSet to work. Checking for a win after a move, it
looks only at the lines through the square just played, because the other lines cannot
have changed. Here the kernel does that narrowing on its own. The winner asks for all eight
lines, and the ones a move did not touch answer from cache without running any code. The
CheckSet is still worth having, because the move below asks it where the board is. It is
just not needed to make the winner cheap.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:turn}}
```

Whose turn it is comes from the board. X opens, so X plays when the marks are level and O
plays when X is one ahead. Once the winner says the game is over, whether won or drawn,
nobody plays, and the turn answers `-`. The turn is a pure function of two resources, the
winner and the board, and it is cacheable for the same reason they are.

A fresh board: nobody has won, and X is to play.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:winner
source urn:iki:tutorial:ttt:turn'>
<pre class="ikigai-run-expected">-
[computed]
X
[computed]</pre>
</div>

A draw takes nine moves to reach, so this page does not play one. The crate's tests do: a
full board with no line answers `draw`, the turn answers `-`, and a tenth move is refused.

## The move: constraint at the edge

So far nothing in the game refuses anything. The stored cell takes any mark at any square,
the line takes any cells, and the winner and the turn only describe what is there. That is
deliberate. The rules arrive in one resource, the move,
`urn:iki:tutorial:ttt:move:{x}:{y}`, which a player writes to and nobody reads.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:move}}
```

The move enforces the rules by asking the resources the game already has. The CheckSet
says whether the square is on the board, the winner whether the game is over, the cell
whether the square is free, and the turn whose mark it is. Each refusal is an
`InvalidArgument` with a reason a player can act on. Only when all four agree does the move
play, and it plays by issuing a `Sink` to the stored cell, the same request a player could
make by hand.

`content` is optional and checked. Left out, the move plays whoever's turn it is. Named,
it must be the side to play, so a player who believes it is their turn and is wrong is told
so rather than silently playing for the other side.

This is the series' seventh part in one sentence: keep the model loose, and constrain it at
the boundary of the application. The stored cell does not know about turns, and the move
does not change it. The atoms are still the atoms Part I built. Another game on the same
cells (Connect Four, or tic-tac-toe on a bigger board) would write its own move
and its own table and reuse everything else. Put the rules into the stored cell, and the
store would stop being reusable.

The move can play through the stored cell because a `Sink` issued from inside an endpoint
is still a `Sink` to the kernel. It cuts the thread named after its target, exactly as a
`Sink` a player issues does, and the recompute story in Part II depends on that:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/rules.rs:inner_sink}}
```

Play the center, with no mark named. The turn passes to O:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:move:1:1
source urn:iki:tutorial:ttt:turn'>
<pre class="ikigai-run-expected">X plays 1,1
[uncacheable]
O
[computed]</pre>
</div>

Now each of the rules, once. Naming the wrong mark, then the right one:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:move:1:0 X
sink urn:iki:tutorial:ttt:move:1:0 O'>
<pre class="ikigai-run-expected">error: invalid argument `content`: it is O's turn, not X's
O plays 1,0
[uncacheable]</pre>
</div>

A square that is taken, and a square that is not on the board:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:move:1:1
sink urn:iki:tutorial:ttt:move:3:1'>
<pre class="ikigai-run-expected">error: invalid argument `x, y`: 1,1 is taken — X played there
error: invalid argument `x, y`: 3,1 is off the board — no line passes through it</pre>
</div>

The move describes itself like everything else, so a player (or an agent choosing a tool)
can see what it takes before it tries. `content` is `ik:required false`, and one of the
two marks:

<div class="ikigai-run" data-cmd='describe urn:iki:tutorial:ttt:move:0:0 text/turtle'>
<pre class="ikigai-run-expected">@prefix ik: &lt;https://ikigai-rs.dev/ns#&gt; .
&#32;
&lt;urn:ikigai:endpoint:ttt-move&gt; a ik:Endpoint ;
    ik:id "ttt-move" ;
    ik:title "Move" ;
    ik:summary "Play the side to move at (x, y), if the rules allow it: on the board, the game not over, the square free, and the mark (if given) the side to play." ;
    ik:verb "Sink", "Meta" ;
    ik:output "text/plain;charset=utf-8" ;
    ik:input &lt;urn:ikigai:endpoint:ttt-move:input:x&gt; ;
    ik:input &lt;urn:ikigai:endpoint:ttt-move:input:y&gt; ;
    ik:input &lt;urn:ikigai:endpoint:ttt-move:input:content&gt; ;
    ik:action &lt;urn:ikigai:endpoint:ttt-move:action:sink&gt; .
&#32;
&lt;urn:ikigai:endpoint:ttt-move:input:x&gt; ik:inputName "x" ;
    ik:source "binding" ;
    ik:required true ;
    ik:summary "the column — any integer" ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
&#32;
&lt;urn:ikigai:endpoint:ttt-move:input:y&gt; ik:inputName "y" ;
    ik:source "binding" ;
    ik:required true ;
    ik:summary "the row — any integer" ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
&#32;
&lt;urn:ikigai:endpoint:ttt-move:input:content&gt; ik:inputName "content" ;
    ik:source "argument" ;
    ik:required false ;
    ik:summary "the mark to play — checked against the turn; left out, the turn plays" ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#string&gt; ;
    ik:oneOf "X" ;
    ik:oneOf "O" .
&#32;
&lt;urn:ikigai:endpoint:ttt-move:action:sink&gt; a ik:Action ;
    ik:verb "Sink" ;
    ik:output "text/plain;charset=utf-8" ;
    ik:input &lt;urn:ikigai:endpoint:ttt-move:input:x&gt; ;
    ik:input &lt;urn:ikigai:endpoint:ttt-move:input:y&gt; ;
    ik:input &lt;urn:ikigai:endpoint:ttt-move:input:content&gt; .
[computed]</pre>
</div>

## One move, and what it recomputes

Part II showed that one move reaches the lines through its square and the board, and
nothing else. The rules sit one level above the lines, so the same holds for them:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/rules.rs:one_move}}
```

The refusals above already asked the winner, so it is cached, and so is every line under
it:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:winner'>
<pre class="ikigai-run-expected">-
[cached]</pre>
</div>

X plays the top-left corner. The corner is on row 0, column 0 and diagonal 0, so those
three lines are no longer cached, and neither are the winner and the turn built on them.
The probes name each line by its own name, not its alias: `cache` does not resolve, so it
cannot see a rewrite the game's space makes, and a probe of `row:0` would under-report. Row
1 misses the corner and is still cached. So is the corner's CheckSet, which the move itself
read:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:move:0:0
cache urn:iki:tutorial:ttt:cells:0.0,1.0,2.0
cache urn:iki:tutorial:ttt:cells:0.0,0.1,0.2
cache urn:iki:tutorial:ttt:cells:0.0,1.1,2.2
cache urn:iki:tutorial:ttt:winner
cache urn:iki:tutorial:ttt:turn
cache urn:iki:tutorial:ttt:cells:0.1,1.1,2.1
cache urn:iki:tutorial:ttt:checkset:0:0'>
<pre class="ikigai-run-expected">X plays 0,0
[uncacheable]
not cached
not cached
not cached
not cached
not cached
cached
cached</pre>
</div>

Now ask for the winner again, traced:

<div class="ikigai-run" data-cmd='trace urn:iki:tutorial:ttt:winner'>
<pre class="ikigai-run-expected">trace  urn:iki:tutorial:ttt:winner
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:ttt:winner   ttt-winner · computed · ThreadId(1) · —   → 1b  -
├─ urn:iki:tutorial:ttt:cells:0.0,1.1,2.2   ttt-cells · computed · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:diagonal:0 -&gt; urn:iki:tutorial:ttt:cells:0.0,1.1,2.2
│  ├─ urn:iki:tutorial:ttt:cell:0:0   ttt-cell · computed · ThreadId(1) · —
│  │  └─ urn:iki:tutorial:ttt:stored:0:0   ttt-stored · computed · ThreadId(1) · —
│  ├─ urn:iki:tutorial:ttt:cell:1:1   ttt-cell · cached · ThreadId(1) · —
│  └─ urn:iki:tutorial:ttt:cell:2:2   ttt-cell · cached · ThreadId(1) · —
├─ urn:iki:tutorial:ttt:cells:2.0,1.1,0.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:diagonal:1 -&gt; urn:iki:tutorial:ttt:cells:2.0,1.1,0.2
├─ urn:iki:tutorial:ttt:cells:0.0,0.1,0.2   ttt-cells · computed · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:column:0 -&gt; urn:iki:tutorial:ttt:cells:0.0,0.1,0.2
│  ├─ urn:iki:tutorial:ttt:cell:0:0   ttt-cell · cached · ThreadId(1) · —
│  ├─ urn:iki:tutorial:ttt:cell:0:1   ttt-cell · cached · ThreadId(1) · —
│  └─ urn:iki:tutorial:ttt:cell:0:2   ttt-cell · cached · ThreadId(1) · —
├─ urn:iki:tutorial:ttt:cells:1.0,1.1,1.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:column:1 -&gt; urn:iki:tutorial:ttt:cells:1.0,1.1,1.2
├─ urn:iki:tutorial:ttt:cells:2.0,2.1,2.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:column:2 -&gt; urn:iki:tutorial:ttt:cells:2.0,2.1,2.2
├─ urn:iki:tutorial:ttt:cells:0.0,1.0,2.0   ttt-cells · computed · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:0 -&gt; urn:iki:tutorial:ttt:cells:0.0,1.0,2.0
│  ├─ urn:iki:tutorial:ttt:cell:0:0   ttt-cell · cached · ThreadId(1) · —
│  ├─ urn:iki:tutorial:ttt:cell:1:0   ttt-cell · cached · ThreadId(1) · —
│  └─ urn:iki:tutorial:ttt:cell:2:0   ttt-cell · cached · ThreadId(1) · —
├─ urn:iki:tutorial:ttt:cells:0.1,1.1,2.1   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:1 -&gt; urn:iki:tutorial:ttt:cells:0.1,1.1,2.1
└─ urn:iki:tutorial:ttt:cells:0.2,1.2,2.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:2 -&gt; urn:iki:tutorial:ttt:cells:0.2,1.2,2.2</pre>
</div>

Read it from the top. The winner was computed, and asked for all eight lines. Three were
computed and five were served. The first line to need the corner, diagonal 0, recomputed
the corner's cell, which read the store. Column 0 and row 0 then found that cell already
recomputed, and cached. Of nine stored cells, one was read. That is the whole cost of a
move to the rules: the lines through the square, and the one square. (The lines appear in
the table's own order, longest alias first, which is the order the winner reads them.)

## The game, played out

O plays the middle of the right-hand column, which does not stop anything. X takes the
bottom-right corner, completing diagonal 0. The winner says so, and the turn says nobody
is to play:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:move:2:1
sink urn:iki:tutorial:ttt:move:2:2
source urn:iki:tutorial:ttt:winner
source urn:iki:tutorial:ttt:turn'>
<pre class="ikigai-run-expected">O plays 2,1
[uncacheable]
X plays 2,2
[uncacheable]
X
[computed]
-
[computed]</pre>
</div>

The last rule: the game is over, so any move is refused, whatever square it names.

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:ttt:move:0:2
source urn:iki:tutorial:ttt:board'>
<pre class="ikigai-run-expected">error: invalid argument `x, y`: the game is over — X has won
XO-
-XO
--X
[computed]</pre>
</div>

Finally, follow the links. The CheckSet answers names, and the engine's map, `..`, runs a
stage once for each line of its input. `urn:iki:fn:compose` sources the resource its `src`
names and expands any transclusion markers in it. A line has none, so here it does exactly
one thing: read the resource a name names. So this reads the four lines through the center,
the winning diagonal among them:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:checkset:1:1 .. urn:iki:fn:compose
cache urn:iki:tutorial:ttt:checkset:1:1'>
<pre class="ikigai-run-expected">OX-
XXX
-X-
-XO
[1 cached · 4 computed]
cached</pre>
</div>

The one cached answer in that tally is the CheckSet itself. It was computed in the first
cell on this page, and after five moves, one of them on the center square itself, it is
still cached, because nothing it depends on can change.

## Next

[Part IV](tic-tac-toe-4.md) plays more than one game at once, and asks where the game
belongs: in every name, or around them as context.
