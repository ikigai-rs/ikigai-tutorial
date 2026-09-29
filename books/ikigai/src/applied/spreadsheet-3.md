# A spreadsheet, III: recalculation is golden threads

Change one number in a spreadsheet, and every cell that reads it has to be computed again, and
every cell that reads those, and nothing else. Getting that right, and fast, is most of what a
spreadsheet engine is. The usual design keeps a **dependency graph**: for every cell, the cells
it reads (its precedents) and the cells that read it (its dependents). An edit marks the
edited cell dirty, follows the dependents to mark everything downstream dirty too, and then
computes the dirty cells in an order that puts every cell after the cells it reads. The graph
has to be kept true as formulas change: typing a new formula into a cell removes the edges its
old formula had and adds the new ones.

This sheet has none of that. [Part II](spreadsheet-2.md) built a value that reads other values
by name, and nothing in it writes an edge, marks a cell dirty or orders anything. This part
edits a cell and checks that exactly the right cells are computed again, then shows where the
graph went.

## A sheet with a chain in it

Six cells: a number, two formulas that chain off it, a total over all three, and a second
number with a formula of its own that has nothing to do with the first.

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A1 5
sink urn:iki:tutorial:sheet:input:A2 =A1*2
sink urn:iki:tutorial:sheet:input:A3 =A2+1
sink urn:iki:tutorial:sheet:input:B1 100
sink urn:iki:tutorial:sheet:input:B2 =SUM(A1:A3)
sink urn:iki:tutorial:sheet:input:C1 =B1/4
source urn:iki:tutorial:sheet:cell:B2
source urn:iki:tutorial:sheet:cell:C1'>
<pre class="ikigai-run-expected">5
[uncacheable]
=A1*2
[uncacheable]
=A2+1
[uncacheable]
100
[uncacheable]
=SUM(A1:A3)
[uncacheable]
=B1/4
[uncacheable]
26
[computed]
25
[computed]</pre>
</div>

`B2` is 5 + 10 + 11. After this cell runs, the sheet at the bottom of the page draws itself
again, as in the other parts, and that reads every value on the sheet, so all twenty-four are
now cached.

## Edit one cell

Change `A1`, and ask the cache about every cell that holds something:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A1 7
cache urn:iki:tutorial:sheet:cell:A1
cache urn:iki:tutorial:sheet:cell:A2
cache urn:iki:tutorial:sheet:cell:A3
cache urn:iki:tutorial:sheet:cell:B1
cache urn:iki:tutorial:sheet:cell:B2
cache urn:iki:tutorial:sheet:cell:C1'>
<pre class="ikigai-run-expected">7
[uncacheable]
not cached
not cached
not cached
cached
not cached
cached</pre>
</div>

`A1`, `A2`, `A3` and `B2` are gone from the cache: the edited cell, the chain that reads it, and
the total over the chain. `B1` and `C1` are still cached, because neither read `A1`. And the
edit itself computed nothing. The four values are computed again when something next reads
them, which here is the grid, straight after the cell. A spreadsheet engine recalculates when
you type; this sheet recalculates when you look.

Here is one of those reads, traced. Edit `A1` once more, and trace `A3`, the end of the chain:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A1 8
trace urn:iki:tutorial:sheet:cell:A3'>
<pre class="ikigai-run-expected">8
[uncacheable]
trace  urn:iki:tutorial:sheet:cell:A3
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:sheet:cell:A3   sheet-cell · computed · ThreadId(1) · —   → 2b  17
├─ urn:iki:tutorial:sheet:input:A3   sheet-input · cached · ThreadId(1) · —
├─ urn:iki:tutorial:sheet:precedents:A3   sheet-precedents · computed · ThreadId(1) · —
│  ├─ urn:iki:tutorial:sheet:refs:A3   sheet-refs · cached · ThreadId(1) · —
│  ├─ urn:iki:tutorial:sheet:refs:A2   sheet-refs · cached · ThreadId(1) · —
│  └─ urn:iki:tutorial:sheet:refs:A1   sheet-refs · computed · ThreadId(1) · —
│     └─ urn:iki:tutorial:sheet:input:A1   sheet-input · computed · ThreadId(1) · —
├─ urn:iki:tutorial:sheet:formula:A3   sheet-formula · cached · ThreadId(1) · —
└─ urn:iki:tutorial:sheet:cell:A2   sheet-cell · computed · ThreadId(1) · —
   ├─ urn:iki:tutorial:sheet:input:A2   sheet-input · cached · ThreadId(1) · —
   ├─ urn:iki:tutorial:sheet:precedents:A2   sheet-precedents · computed · ThreadId(1) · —
   │  ├─ urn:iki:tutorial:sheet:refs:A2   sheet-refs · cached · ThreadId(1) · —
   │  └─ urn:iki:tutorial:sheet:refs:A1   sheet-refs · cached · ThreadId(1) · —
   ├─ urn:iki:tutorial:sheet:formula:A2   sheet-formula · cached · ThreadId(1) · —
   └─ urn:iki:tutorial:sheet:cell:A1   sheet-cell · computed · ThreadId(1) · —
      └─ urn:iki:tutorial:sheet:input:A1   sheet-input · cached · ThreadId(1) · —</pre>
</div>

Read it from the bottom. `A1`'s value was computed, because its input changed. `A2`'s was
computed because it read `A1`'s. `A3`'s was computed because it read `A2`'s. Their compiled
formulas were served from the cache, since no formula changed, and so were their inputs.
The trace is the dependency graph, for one cell, drawn after the fact from what was actually
read.

## Where the graph is

Every cacheable answer the kernel keeps carries the set of golden threads it hangs from: the
threads of everything it read while it was being computed, gathered as it read them. `A3`'s
value read `A2`'s value, which read `A1`'s, which read `A1`'s input, so `A3`'s value carries
`A1`'s input thread, and the edit to `A1` cut it. That set is a cell's precedents, turned
inside out and stored on the answer rather than on the cell. Nothing keeps the dependents at
all. A cut does not go looking for them; it moves a counter on, and each cached answer, when
it is next asked for, checks whether any of its threads has moved since it was stored.

The counters are all the kernel keeps about the edits, and you can read them:

<div class="ikigai-run" data-cmd='source urn:kernel:threads'>
<pre class="ikigai-run-expected">threads (cut generations)
  urn:iki:tutorial:sheet:input:A1  gen 3
  urn:iki:tutorial:sheet:input:A2  gen 1
  urn:iki:tutorial:sheet:input:A3  gen 1
  urn:iki:tutorial:sheet:input:B1  gen 1
  urn:iki:tutorial:sheet:input:B2  gen 1
  urn:iki:tutorial:sheet:input:C1  gen 1
[uncacheable]</pre>
</div>

Only inputs appear, since inputs are the only things anything writes. `A1`'s thread has been
cut three times, once for each edit on this page; the others once, when they were first typed.

This is why there is no graph to keep true. Give `C1` a new formula, and it stops reading `B1`
and starts reading `A3`. An engine with a graph has to remove one edge and add another. Here
the next evaluation of `C1` reads what the new formula names, and the kernel records that:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:C1 =A3/4
source urn:iki:tutorial:sheet:cell:C1
sink urn:iki:tutorial:sheet:input:B1 200
cache urn:iki:tutorial:sheet:cell:C1
sink urn:iki:tutorial:sheet:input:A1 9
cache urn:iki:tutorial:sheet:cell:C1'>
<pre class="ikigai-run-expected">=A3/4
[uncacheable]
4.25
[computed]
200
[uncacheable]
cached
9
[uncacheable]
not cached</pre>
</div>

`C1` no longer cares about `B1`, and now cares about `A1`, through `A3` and `A2`. Nobody told
it. The dependencies are dynamic in a way a stored graph finds hard: they are whatever the
evaluation actually read, so a formula that read one of two cells depending on a third would
depend only on the one it read, each time.

## The sheet, and what an edit computed again

Here is the sheet, with one thing added under it: after each edit, before the grid is drawn
again, the page asks the kernel what drawing it will cost, and shows the answer as a table the
shape of the grid. A cell marked **computed** is a value the redraw will compute; the rest it will
be served. Type into `A1` and watch the chain; type into `D6`, which nothing reads, and watch one
cell.

<div class="sheet-play" data-cache data-shell='urn:iki:tutorial:sheet:template:page'><p class="sheet-unavailable">The sheet appears here when the in-page kernel has loaded.</p></div>

The table is a resource, `view:cost`, and it is composed like the grid, from templates and names,
with no code of its own. The question it asks of each cell is the one the `cache` command asks: if
somebody read this value right now, would the cache serve it? The kernel answers that question as a
resource, `urn:kernel:cached`, which resolves nothing and computes nothing: it looks up the key a
read by the asker would use, under the asker's own capability and chain, and says `true` or
`false`. A one-marker template gives it a name for each cell:

```text
{{#include ../../../../crates/spreadsheet/templates/cached.html}}
```

and each cell of the table branches on it with `urn:iki:fn:conditional`, which reads the name in
`if`, compares it with `equals`, and resolves only the branch it takes:

```text
{{#include ../../../../crates/spreadsheet/templates/cost-cell.html}}
```

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:cost_names}}
```

The view has to be read before the redraw, since the redraw is what computes the values again, and
it is never cached itself: `urn:kernel:cached` is a live question, so everything built on it is
asked again every time. The crate's test pins what it says:

```rust,ignore
{{#include ../../../../crates/spreadsheet/tests/sheet.rs:cost}}
```

Before core 0.1.82 this table was the page's own code, and that was a finding rather than a
choice. The kernel's cache readout, `urn:kernel:cache`, lists what the cache *holds*, and a cut is
lazy: an answer whose thread was cut stays in the cache until the next read of it finds it stale
and drops it. So right after an edit the readout listed every value the edit had invalidated
exactly as it did before, and the probe that knew the difference belonged to the host; no endpoint
could ask it. Now the readout marks each row `live`, `cut` or `expired`, and the probe is a
resource anybody holding `urn:cap:kernel:inspect` can read, so the viewer could become a
composition, and did.

## What it costs, and what it does not do

**An edit costs nothing until somebody reads.** The work is done on the next read of each value
that was cut, and only if something reads it. A cell nobody shows is never recomputed. That
suits a sheet on a page; it is the wrong shape for a sheet whose values must be pushed
somewhere the moment they change, which needs something to read them, and a later part's
subject.

**A cut is by name, not by content.** Type `9` into `A1` when it already holds `9`, and every
value that reads it is still computed again:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A1 9
cache urn:iki:tutorial:sheet:cell:B2
source urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">9
[uncacheable]
not cached
46
[computed]</pre>
</div>

The same answer, computed again. A write cuts the thread named after what it wrote, and every
answer hanging from that thread goes, whether or not it would come out different. Some build
systems stop early here, comparing a recomputed answer with the old one and leaving everything
downstream alone when they match. The kernel does not, and it shows in a subtler place too:
`refs:A1` reads `A1`'s input to learn that `A1` holds no formula, so changing the number in `A1`
cuts `A1`'s references, and with them the precedents of every cell downstream, though no
formula changed. The trace above shows it: `refs:A1` computed, `precedents:A2` and
`precedents:A3` computed with it. It is correct, and it is work a finer design could skip, by
putting "which kind of input is this" in a resource of its own.

**The cache is bounded.** The kernel evicts entries when it holds too many, and an evicted
value is simply computed again on its next read. Nothing depends on a value staying cached;
the cache only makes reads cheap.

## What was built

Nothing, in this part. No endpoint was added, and no line of the sheet's code is about
recalculation: not a dirty flag, not a dependents list, not an ordering. The recalculation engine
is the golden threads the kernel already had, and the evaluator is a function that reads other
values by name.

What comes next leans on that. A cell whose formula reads the time needs its value to expire
as well as be cut, which is the [time chapter](time.md)'s other way for an answer to go
([Part IV](spreadsheet-4.md)). A cell that reads outside data needs something outside to cut it
([Part V](spreadsheet-5.md)). And because a cell's value is a
name, a *personal* sheet, my overrides layered over a shared one, is a corridor in front of the
same names rather than a copy of the sheet.
