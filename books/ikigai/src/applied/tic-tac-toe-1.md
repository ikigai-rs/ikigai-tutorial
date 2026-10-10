# Tic-tac-toe, I: the resource model and the atoms

Everything so far has been one idea at a time. This part builds one small application
end to end, in the order resource-oriented design says to: name the resources first, write
code for the few that have to *hold* something, and get the rest by composing names. The
application is tic-tac-toe (noughts and crosses, in Britain), because everyone already <!-- spelling: quote (Britain's name for the game, a note) -->
knows its rules, so every ounce of attention goes to the shape rather than the domain.

The design is not new. It follows *Resource Oriented Analysis and Design*, a nine-part
series by Peter Rodgers in NetKernel News, issues 3.34 to 3.42 (August to October 2012),
reprinted in 1060 Research's 2013 collection of NetKernel articles. The series builds the
same game on NetKernel, resources first and code last, and argues each step as it goes.
This part keeps its order and its main decisions. The names, the code and the sentences
are ikigai's own, and where ikigai does something differently the text says so, because
the differences are where the lessons are.

It comes in six parts, one increment each. This one draws the whole model on paper and
then builds only the atom: a single cell.

> **This is deliberately more machinery than tic-tac-toe needs.** A game this small could be forty lines of ordinary code, and the parts that follow will sometimes feel like overkill: a name for every cell, row and rule, templates that are resources, a game that is a corridor rather than an object. The game is the excuse, not the point. What these parts build is a way of working in which only the few things that have to *hold* something are code, and everything else is a name composed from other names. That is what pays for itself as a system grows. Because every intermediate result is a resource, the kernel caches it, knows what it was computed from, and recomputes what a change affects, with no caching code in the application. A new rule, view or question is a new name, not an edit to the code already there. The same composition answers whether the state lives in this page, in a host process, or in a Python or TypeScript peer, and a thousand games share one set of rules. Change the domain and the shape stays: the board becomes a portfolio, a line becomes a group of positions, "who won" becomes a risk figure, and resolution, caching, invalidation and authority work exactly as they do here. Tic-tac-toe is small enough to hold the whole model in your head while you watch that happen.
>
> Part IV makes each game a *corridor*, a term the [Scope and alias](../getting-started/scope-and-alias.md) chapter introduces; a look there first will help.

## The model, on paper

Before any code, list the sets. A game of tic-tac-toe is small enough to name
every kind of thing in it:

| the set | its name | what it is | built in |
|---|---|---|---|
| a **cell** | `urn:iki:tutorial:ttt:cell:{x}:{y}` | the mark at one place, or empty | this part |
| a **line** of cells | `urn:iki:tutorial:ttt:cells:{list}` | any cells, in order, named by their coordinates | part II |
| a **row** | `urn:iki:tutorial:ttt:row:0` (and `:1`, `:2`) | a line of cells, by name — no code | part II |
| a **column** | `urn:iki:tutorial:ttt:column:0` (and `:1`, `:2`) | the same, the other way | part II |
| a **diagonal** | `urn:iki:tutorial:ttt:diagonal:0` (and `:1`) | the same, corner to corner | part II |
| the **board** | `urn:iki:tutorial:ttt:board` | every row | part II |
| which lines a cell is on | `urn:iki:tutorial:ttt:checkset:{x}:{y}` | a list of the names of other resources | part III |
| **whose turn** | `urn:iki:tutorial:ttt:turn` | a question about the board | part III |
| **who won** | `urn:iki:tutorial:ttt:winner` | another | part III |
| a **move** | `urn:iki:tutorial:ttt:move:{x}:{y}` | a mark played, if the rules allow it | part III |
| many games | — | the game as context: a corridor around the names, not a name | part IV |
| a board you can **play** | `urn:iki:tutorial:ttt:view:board` (and `view:status`, `view:play:{x}:{y}`) | HTML over all of the above, from templates that are resources too | part V |

Read down the second column and notice how little of it is code. Only the cell holds
state: everything above it is a set of cells, or a question about sets of cells, and a set
of names can be *named* rather than computed. That is the bet this part and the next are
testing. The names past the first row are plans, not promises; each part settles its own.

## Two cells, not one

The cell is really two resources, because two different questions get asked of it.

The **stored cell**, `urn:iki:tutorial:ttt:stored:{x}:{y}`, answers "what has been
played here?" It holds a mark in memory. `Source` reads it, `Sink` plays one, `Delete`
clears it — and where nothing has been played, `Source` answers `NotFound`, because
nothing is stored there.

The **platonic cell**, `urn:iki:tutorial:ttt:cell:{x}:{y}`, answers "what does this cell
look like?", and it is the one anything else will read. Where a mark has been played it
answers the mark; where none has, it answers the empty cell, `-`. So every cell of every
board already exists, before any game has started: nothing has to create a board before
reading one, and a fresh game is simply the absence of stored marks. (The name is the
series' own: the empty cell is the idea of a cell, the one every played cell is a
departure from.)

The empty cell is a hyphen rather than a space for one practical reason: it gets read as
text. A lone space is invisible in a terminal and in this page, where it looks exactly
like nothing came back; a hyphen is visible, and it still lines up in a monospaced board.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:names}}
```

Writes go to the stored cell and reads go to the platonic one. That split is deliberate,
and the next section is why.

## The stored cell

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:store}}
```

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:stored}}
```

The state lives in a `CellStore` the host holds, the same shape as Part I's `title`, and
for the same reason: a test can hold the read counter and prove that an answer was
*served* rather than recomputed.

Look at what the `Source` arm does not say. It is `.cacheable()` and names no golden
thread. The kernel does both halves of the bookkeeping itself: since `ikigai-core` 0.1.73
every cacheable read hangs from the thread named after the resource it read, and every
successful `Sink` or `Delete` cuts the thread named after its target. (Part I's `title`
still declares `.depends_on(TITLE)` by hand; it was written before 0.1.73, and the line is
now redundant.)

This is where ikigai and the original part company. In the series, the store attaches a
golden thread when it reads and cuts it when it writes, explicitly, in the store's own
code. It is an honest design, and it has an honest failure: the series itself records an
in-memory store that forgot to cut its thread on `DELETE`, so a cleared cell went on
being served with its old mark from the cache. Here there is no line to forget.

## The platonic cell

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:platonic}}
```

It resolves the stored cell through the kernel, with `inv.source`, exactly as a caller
outside would, and turns one error into an answer: `NotFound` becomes `-`. Any other
failure passes through untouched. A store that is down is not an empty cell, and
answering it as one would be a lie that caches.

### Why this is not a `Fallback`

The original gets its empty cell from a *resolution* fallback: a request that misses the
store falls through to a default resource that answers every cell. ikigai has the same
tool — [`Fallback`](../getting-started/scope-and-alias.md) tries spaces in order until one
resolves the name — and it is the wrong tool here, for a reason worth being exact about.

`Fallback` acts on **resolution**: it moves on only when a space does not bind the name.
The stored cell is bound by a template, `stored:{x}:{y}`, which binds *every* cell, played
or not. Resolution always succeeds. The miss happens a step later, at **invocation**, when
the endpoint looks in the store and finds nothing — and by then `Fallback` has already
made its choice. So the honest ikigai form is a composite that catches the invocation's
`NotFound`, and says so in its code.

### The failed read is a dependency

Here is the property that makes the composite more than a workaround. The empty answer is
`.cacheable()`. The next read of the same empty cell is served without the endpoint
running. And yet it is never stale:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/tests/atoms.rs:platonic_cut}}
```

The platonic cell's `inv.source` of the stored cell *failed*. Since `ikigai-core` 0.1.73 a
failed sub-request is recorded as a dependency exactly like a successful one: the cached
empty answer hangs from the stored cell's thread. When the stored cell is played, the
`Sink` cuts that thread, and the empty answer goes with it. Clearing the cell works the
same way in reverse. No code in either endpoint mentions the other's thread, and nothing
polls.

That is also why writes go to the stored cell rather than through the platonic one. If
the platonic cell accepted a `Sink` and passed it on, the kernel would cut the platonic
cell's own thread after the write, and the lesson would be hidden: the cache would be
invalidated by the name that was written, not by the dependency. Writing to the store
directly leaves the dependency as the only thing that can reach the platonic cell's cache
— and it does.

## Run it

These cells run against the book's own kernel, in this page — the same space the tests
above run against, under the same engine the `ikigai` CLI uses. One kernel serves every
cell on the page and keeps its cache between runs, so run them in order; running one again
answers differently the second time, and that is the point.

Read an empty cell, twice:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:cell:1:1
source urn:iki:tutorial:ttt:cell:1:1'>
<pre class="ikigai-run-expected">-
[computed]
-
[cached]</pre>
</div>

The first read ran the platonic cell, which asked the store and got `NotFound`. The second
ran nothing. The store itself, asked directly, still has nothing to say:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:stored:1:1'>
<pre class="ikigai-run-expected">error: not found: nothing has been played at 1,1</pre>
</div>

That is `NotFound`, not `no endpoint resolved`: the name is bound, and the thing it names
is absent. The two errors are different on purpose, and the difference is the whole reason
`Fallback` could not help.

Now play the center. The cached empty cell is still cached until the write lands, and
then it is not:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:ttt:cell:1:1
sink urn:iki:tutorial:ttt:stored:1:1 X
cache urn:iki:tutorial:ttt:cell:1:1
source urn:iki:tutorial:ttt:cell:1:1'>
<pre class="ikigai-run-expected">cached
ok
[uncacheable]
not cached
X
[computed]</pre>
</div>

The `Sink` named only the stored cell. The platonic cell's cached answer went because it
had *failed* to read the stored cell, and that failure was recorded. Clear the cell and it
comes back empty — computed once, then served:

<div class="ikigai-run" data-cmd='delete urn:iki:tutorial:ttt:stored:1:1
source urn:iki:tutorial:ttt:cell:1:1
source urn:iki:tutorial:ttt:cell:1:1'>
<pre class="ikigai-run-expected">ok
[uncacheable]
-
[computed]
-
[cached]</pre>
</div>

Both endpoints describe themselves, so the page can tell you what a cell takes. `x` and
`y` are **bindings** — captured from the name by the template, not passed as arguments:

<div class="ikigai-run" data-cmd='describe urn:iki:tutorial:ttt:cell:0:0 text/turtle'>
<pre class="ikigai-run-expected">@prefix ik: &lt;https://ikigai-rs.dev/ns#&gt; .
&#32;
&lt;urn:ikigai:endpoint:ttt-cell&gt; a ik:Endpoint ;
    ik:id "ttt-cell" ;
    ik:title "Cell" ;
    ik:summary "The cell at (x, y): the mark played there, or - where none has been." ;
    ik:verb "Source", "Meta" ;
    ik:output "text/plain;charset=utf-8" ;
    ik:input &lt;urn:ikigai:endpoint:ttt-cell:input:x&gt; ;
    ik:input &lt;urn:ikigai:endpoint:ttt-cell:input:y&gt; ;
    ik:action &lt;urn:ikigai:contract:ttt-cell:source:b3:…&gt; .
&#32;
&lt;urn:ikigai:endpoint:ttt-cell:input:x&gt; ik:inputName "x" ;
    ik:source "binding" ;
    ik:required true ;
    ik:summary "the column — any integer" ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
&#32;
&lt;urn:ikigai:endpoint:ttt-cell:input:y&gt; ik:inputName "y" ;
    ik:source "binding" ;
    ik:required true ;
    ik:summary "the row — any integer" ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
&#32;
&lt;urn:ikigai:contract:ttt-cell:source:b3:…:input:x&gt; ik:inputName "x" ;
    ik:source "binding" ;
    ik:required true ;
    ik:summary "the column — any integer" ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
&#32;
&lt;urn:ikigai:contract:ttt-cell:source:b3:…:input:y&gt; ik:inputName "y" ;
    ik:source "binding" ;
    ik:required true ;
    ik:summary "the row — any integer" ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
&#32;
&lt;urn:ikigai:contract:ttt-cell:source:b3:…&gt; a ik:Action ;
    ik:verb "Source" ;
    ik:output "text/plain;charset=utf-8" ;
    ik:input &lt;urn:ikigai:contract:ttt-cell:source:b3:…:input:x&gt; ;
    ik:input &lt;urn:ikigai:contract:ttt-cell:source:b3:…:input:y&gt; .
[computed]</pre>
</div>

The `b3:…` in the contract's name stands for a digest of the contract, which this book does not
spell out; [What resolution buys you](../getting-started/payoff.md) says why.

## Loose on purpose

`{x}` and `{y}` are any integers — `-1`, `3`, a cell far off the board — and not 0 to 2.
Writing the board's edges into the grammar would make the model watertight for
tic-tac-toe and useless for anything else (Connect Four is a board of cells too), so the
atoms stay loose and the rules arrive at the edge of the application, in part III.

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:coordinate}}
```

Loose about *which* integers, strict about how each one is spelled. `01` and `1` are the
same number, and two names for one cell would be two threads over one piece of state:
playing through one name would cut its own thread and leave a cached read under the other
serving the old mark. One spelling per cell keeps one thread per cell.

## Next

[Part II](tic-tac-toe-2.md) gathers cells into lines by name, makes the rows, columns and
diagonals from those names with no code at all, and shows that one move recomputes the
lines through it and the board — and nothing else.
