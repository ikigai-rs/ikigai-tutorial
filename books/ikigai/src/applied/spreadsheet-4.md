# A spreadsheet, IV: cells that read the clock

[Part III](spreadsheet-3.md) showed one way for a cached value to go: something writes, the
kernel cuts a golden thread, and every value that read what was written is computed again the
next time somebody looks. [Time is a resource](time.md) showed the other way: an answer says how
long it is good for, and the kernel lets it go at that instant, with nobody writing anything.

A spreadsheet needs both. `=NOW()` is out of date a minute after it is computed, and so is
every cell that reads it, however far down the chain. This part adds `NOW()` and `TODAY()` to
the formula language and shows that nothing else changes: the recalculation that golden threads
gave the sheet in Part III, expiry gives it here, and a formula that never reads the time is
never computed again because of it.

## The model, on paper

No new set, and no new atom. Two new functions in the formula language, each of which reads a
name the time chapter already serves:

| in a formula | compiles to | reads | good until |
|---|---|---|---|
| `NOW()` | `(now)` | `urn:iki:tutorial:time:instant`, the date and the time as one reading | the next minute |
| `TODAY()` | `(today)` | `urn:iki:tutorial:time:today`, the date | the next midnight |

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:time_names}}
```

The sheet reads these names; it does not serve them. The time chapter's crate does, and the
page binds both side by side, as it binds tic-tac-toe beside Part I. That is the whole of the
integration: the sheet names a resource another chapter built, and the kernel finds it.

`NOW()` is the date and the time, as a spreadsheet's `NOW()` is, written the way the time
chapter writes an instant: `2026-09-29T14:05Z`. `TODAY()` is the date alone. Both are text,
so a formula that does arithmetic on them gets `#VALUE`; this part is about when they change,
not about what can be done with them.

## Evaluating the time

The compiler learns two words and the evaluator one function. Reading the time is reading a
name through the kernel, exactly as reading a cell is:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:read_time}}
```

There is nothing here about expiry. The evaluator does not say how long a value that read the
time is good for, any more than it said in Part III which cells a value depends on. It reads
the name, and the kernel records what it read: a golden thread for every name, and a deadline
for every name that had one.

This page's kernel has a clock: the page carries a `data-clock` attribute, as the time
chapter's does, and so it is built with the browser's. The outputs under each cell are from a
test that stopped a clock where the cell says; your times will be your own.

## The minute turns

Four cells: the time, the date, a cell that reads the time without saying so, and some
arithmetic that has nothing to do with it. It is 14:05:20 UTC on 29 September 2026.

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A1 =NOW()
sink urn:iki:tutorial:sheet:input:A2 =TODAY()
sink urn:iki:tutorial:sheet:input:B1 =A1
sink urn:iki:tutorial:sheet:input:C1 =6*7
source urn:iki:tutorial:sheet:formula:A1
source urn:iki:tutorial:sheet:cell:B1'>
<pre class="ikigai-run-expected">=NOW()
[uncacheable]
=TODAY()
[uncacheable]
=A1
[uncacheable]
=6*7
[uncacheable]
(now)
[computed]
2026-09-29T14:05Z
[computed]</pre>
</div>

`B1`'s formula is `=A1`. It says nothing about time. Twenty seconds later, everything is still
cached, `B1` included, because the grid at the bottom of the page read every value after the
cell above ran:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:sheet:cell:A1
cache urn:iki:tutorial:sheet:cell:B1
source urn:iki:tutorial:sheet:cell:B1'>
<pre class="ikigai-run-expected">cached
cached
2026-09-29T14:05Z
[cached]</pre>
</div>

Now let the minute turn. It is 14:06:05, nobody has typed anything, and no thread has been cut:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:sheet:cell:A1
cache urn:iki:tutorial:sheet:cell:B1
cache urn:iki:tutorial:sheet:cell:A2
cache urn:iki:tutorial:sheet:cell:C1
cache urn:iki:tutorial:sheet:formula:A1
cache urn:iki:tutorial:sheet:view:grid
source urn:iki:tutorial:sheet:cell:B1'>
<pre class="ikigai-run-expected">not cached
not cached
cached
cached
cached
not cached
2026-09-29T14:06Z
[computed]</pre>
</div>

`A1` went at the minute, because it read `instant`, which read `now`, whose answer was good
until 14:06. `B1` went with it, because it read `A1`. The date stayed, and so did `C1`, and so
did the compiled formula: compiling `=NOW()` reads the formula's text, not the time, so the
compiled form is cached for as long as nobody edits the cell. And the grid went, because it
read `B1`.

That is the rule the time chapter stated for a countdown, now on a sheet: **a value is never
fresher than the least fresh thing it read.** `B1` declares itself cacheable with no deadline,
like every value, and the kernel meets that with the deadline of everything it read. Here is
what it read, one minute further on:

<div class="ikigai-run" data-cmd='trace urn:iki:tutorial:sheet:cell:B1'>
<pre class="ikigai-run-expected">trace  urn:iki:tutorial:sheet:cell:B1
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:sheet:cell:B1   sheet-cell · computed · ThreadId(1) · 0ms   → 17b  2026-09-29T14:07Z
├─ urn:iki:tutorial:sheet:input:B1   sheet-input · cached · ThreadId(1) · 0ms
├─ urn:iki:tutorial:sheet:precedents:B1   sheet-precedents · cached · ThreadId(1) · 0ms
├─ urn:iki:tutorial:sheet:formula:B1   sheet-formula · cached · ThreadId(1) · 0ms
└─ urn:iki:tutorial:sheet:cell:A1   sheet-cell · computed · ThreadId(1) · 0ms
   ├─ urn:iki:tutorial:sheet:input:A1   sheet-input · cached · ThreadId(1) · 0ms
   ├─ urn:iki:tutorial:sheet:precedents:A1   sheet-precedents · cached · ThreadId(1) · 0ms
   ├─ urn:iki:tutorial:sheet:formula:A1   sheet-formula · cached · ThreadId(1) · 0ms
   └─ urn:iki:tutorial:time:instant   time-instant · computed · ThreadId(1) · 0ms
      ├─ urn:iki:tutorial:time:today   time-today · cached · ThreadId(1) · 0ms
      ├─ urn:iki:tutorial:time:now   time-now · computed · ThreadId(1) · 0ms
      └─ urn:iki:tutorial:time:today   time-today · cached · ThreadId(1) · 0ms</pre>
</div>

Read it from the bottom. `now` was computed, since its minute had passed; `today` was served,
twice: `instant` reads the date on both sides of the time, which is a later section's subject.
`instant`, `A1`'s value and `B1`'s value were computed because they read `now`. The inputs, the
compiled formulas and the precedents were served, since nothing wrote to them. (On this page a
trace has a clock to time itself by, so it prints a duration where Part III's printed `—`. The
test's clock stood still, which is why it says `0ms`; yours will not.)

Midnight is the same thing with a longer deadline. It is 00:00:10 on 30 September:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:sheet:cell:A2
cache urn:iki:tutorial:sheet:cell:C1
source urn:iki:tutorial:sheet:cell:A2'>
<pre class="ikigai-run-expected">not cached
cached
2026-09-30
[computed]</pre>
</div>

The crate's tests say all of this with a clock the test holds:

```rust,ignore
{{#include ../../../../crates/spreadsheet/tests/live.rs:minute}}
```

## One `NOW()` in a sheet is a performance change

The grid read every value on the sheet, so the grid expires on the minute as soon as any one
cell reads the time. Put `=NOW()` in a corner of a sheet that was cached until somebody typed,
and the whole grid is computed again every minute from then on. Nothing in the grid's code, or
the grid's types, or the formula's, shows it. It is the least-cacheable-dependency rule working
against you, the edge the time chapter warned about.

What keeps it cheap is the same thing that made Part III's recalculation cheap. The grid is
computed again, but the grid is a composition of twenty-four values, and twenty-two of them are
still cached. Drawing it again is one computation of each value that read the time, and
twenty-two lookups. The expensive part of a sheet, the formulas over ranges, is only as
volatile as the cells it actually reads.

## Two readings of one clock, again

The time chapter found a trap in reading the date and the time as two resources: read `today`
just before midnight and `now` just after, and the pair names a minute a day away from the real
one. A spreadsheet's `NOW()` is the date and the time, so the trap is in exactly the place this
part would have put it.

It is not here, because `NOW()` reads one name, `instant`, and `instant` is the guarded reading:
the date read on both sides of the time. The guard is written once, in the resource, and every
reader of that name is safe without knowing there was a trap. The crate's test runs `=NOW()` two
hundred times across midnight, on a clock that moves a millisecond every time it is read:

```rust,ignore
{{#include ../../../../crates/spreadsheet/tests/live.rs:torn}}
```

A sheet can still tear the two by hand. Put `=TODAY()` in one cell and a time in another, and
the grid, which reads both, can be drawn once across midnight with yesterday's date beside
today's time. It is drawn that way once, never twice: the date it read expired at the midnight
it straddled, so the grid it was composed into expired then too, and the next read computes it
again. A torn reading is an answer that is wrong once, not an answer the kernel keeps. (The
time chapter's own test for this is `an_unguarded_reading_across_midnight_is_wrong_once_and_never_cached`.)

## A kernel with no clock

Every other page of this book runs a kernel with no clock, and the time chapter's names refuse
there. The sheet does not pass the refusal on. A formula that reads the time on a kernel with no
clock is `#N/A`, a value, like the sheet's other errors, and the evaluator does not even ask:

- A refusal is never cached. The kernel records every error except `NotFound` as already
  expired, so a value that caught one would be computed again on every read, and so would the
  grid.
- The answer cannot change. A kernel's clock is chosen when it is built, so a kernel with no
  clock will never have one. `#N/A` is true for as long as the kernel lives, and the kernel
  caches it for exactly that long.

## The sheet, polling

Here is the sheet again, with one difference from Part III's: it can poll. Press **Poll** and
the grid is asked for again every two seconds, as the time chapter's clock was. Each poll is
counted: answered from the cache, or computed. Type `=NOW()` into a cell and watch the count.
The grid is computed once a minute, on the first poll after the minute turns, and the table under
the grid marks exactly the values that read the time.

<div class="sheet-play" data-clock data-cache data-shell='urn:iki:tutorial:sheet:template:live'><p class="sheet-unavailable">The sheet appears here when the in-page kernel has loaded.</p></div>

Polling is the only way this grid can find out that the minute turned. Nothing wrote, so there
was no edit for the page to react to. The answer went because its deadline passed, and a
deadline passing is not an event anybody is told about: the next read finds it gone. So
something has to read. That is also true of a value that goes because something *wrote*, when
the something is not the page, and that is [Part V](spreadsheet-5.md).

## What was built

| the set | the name | what it is |
|---|---|---|
| `NOW()`, `TODAY()` | in a formula; `(now)`, `(today)` compiled | two words for the compiler, and one read of a name each for the evaluator |
| the instant | `urn:iki:tutorial:time:instant` | the time chapter's guarded reading of `today` and `now`, given a name ([Time is a resource](time.md)) |
| the live sheet | `urn:iki:tutorial:sheet:template:live` | Part I's page, with a grid that polls |

No endpoint was added to the sheet. The evaluator reads two more names, and the deadlines, the
recalculation on the minute and the cells that are left alone all come from the kernel's rule
for composites, which the sheet was already following. One thing was added to the time
chapter's crate, and it was not code for the sheet: it was a guard that every composite over
the date and the time would otherwise have had to remember, moved into a resource so that
nothing has to.
