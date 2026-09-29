# A spreadsheet, VI: a sheet is a space, and spaces stack

Five parts have composed *resources*: a value is a function of other values, a view is a template
over names, and the kernel keeps the dependencies. This part composes the *context* the names are
read in. The same name, `urn:iki:tutorial:sheet:cell:B2`, answers one thing on the shared sheet,
another in the sheet one person is playing with ("what if we sold twelve?"), and a third in the
sheet as it stood last Tuesday. None of the three is a copy of the sheet, and not one line of the
code that computes a value knows which of them it is computing in.

Each is a **corridor**: a space pushed in front of the shared names for the length of a request,
answering a few of them its own way and letting every other name through. Tic-tac-toe used one per
game ([Part IV](tic-tac-toe-4.md) of that arc). This part builds two kinds, a personal one and a
temporal one, and then stacks them.

## The model, on paper

One atom changes, and two corridors are added:

| the space | its name | what it answers | built from |
|---|---|---|---|
| the shared sheet | the root | every name, as Parts I–V built it | the input, which now keeps every write it was given |
| a person's **scenario** | `urn:iki:tutorial:sheet:scenario:{who}` | `input:{ref}` for the cells that person overrode, and nothing else | a store of overrides, behind a capability of their own |
| the sheet **as of** an instant | `urn:ctx:time:{instant}` | every `input:{ref}` as it stood then; `now` and `today` at that instant | the input's history, and the time chapter's clock |
| both at once | a stack of the two | whatever the innermost corridor answers, then the next | `Scope::stack` |

A corridor's name is not a resource: nothing binds it, and nothing resolves it. It is the name the
request's chain carries, and the cache files every answer computed in the corridor under it.

<!-- urn-gate: illustration urn:iki:tutorial:sheet:scenario:{who} — a scenario corridor's name,
     which nothing binds; the chain carries it. -->
<!-- urn-gate: illustration urn:ctx:time:{instant} — a temporal corridor's name, which the
     engine's `as-of=` gives it and nothing binds. -->
<!-- urn-gate: illustration urn:ctx:time:2026-09-22T18:00:00Z — likewise, one instant's. -->

## A clock the story keeps

This part is about "as of last Tuesday", so its page cannot use your clock: on your page, last
Tuesday would be a date before anything here was typed, and every cell below would answer an empty
sheet. So the page keeps a **story clock**. Its markup carries `data-clock='2026-09-22T12:00Z'`, an
instant where Part IV's page carried a bare `data-clock`, and the page's kernel is built with a
clock stopped there. A cell that says when it runs (`data-at`) moves the clock before it runs, and
says so beside its label:

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:story_clock}}
```

## A week of the sheet

It is Tuesday, 22 September, at noon. A price, a quantity and their product; a cost, and the
profit; and, in `C1`, the time:

<div class="ikigai-run" data-at='2026-09-22T12:00Z' data-cmd='sink urn:iki:tutorial:sheet:input:A1 100
sink urn:iki:tutorial:sheet:input:A2 10
sink urn:iki:tutorial:sheet:input:A3 =A1*A2
sink urn:iki:tutorial:sheet:input:B1 600
sink urn:iki:tutorial:sheet:input:B2 =A3-B1
sink urn:iki:tutorial:sheet:input:C1 =NOW()
source urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">100
[uncacheable]
10
[uncacheable]
=A1*A2
[uncacheable]
600
[uncacheable]
=A3-B1
[uncacheable]
=NOW()
[uncacheable]
400
[computed]</pre>
</div>

A week later, the price goes up:

<div class="ikigai-run" data-at='2026-09-29T14:05Z' data-cmd='sink urn:iki:tutorial:sheet:input:A1 110
source urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">110
[uncacheable]
500
[computed]</pre>
</div>

## The sheet as it was

The input used to keep only what a cell held now. It keeps every write now, each stamped with the
kernel's clock, and a clear is a write of nothing:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:store}}
```

That is the only change to the sheet the temporal view needs. The rest is a corridor. Inside it,
`input:{ref}` is answered from the history, as of the instant the corridor is pinned to:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:as_of}}
```

A line of the REPL asks for one with `as-of=`: the engine builds a corridor named for the instant,
`urn:ctx:time:2026-09-22T18:00:00Z`, over doors the host hands it, with a clock stopped at that
instant. Which names the corridor rebinds is the host's business, so the page hands the engine the
sheet's doors and the time chapter's:

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:as_of_doors}}
```

Here is the profit as it stood on Tuesday evening, twice, and at eleven that morning, before
anything had been typed:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:sheet:cell:B2 as-of=2026-09-22T18:00Z
source urn:iki:tutorial:sheet:cell:B2 as-of=2026-09-22T18:00Z
source urn:iki:tutorial:sheet:cell:B2 as-of=2026-09-22T11:00Z'>
<pre class="ikigai-run-expected">400
[computed]
400
[cached]
[computed]</pre>
</div>

The price was 100 on Tuesday, so the profit was 400. The formulas, the compiler, the evaluator and
the views are the code Parts I–V wrote, and they read `input:{ref}` as they always did. In this
corridor, that name means "then". At eleven there was nothing to read, and an empty cell's value is
empty.

The answer is cached, and it hangs from the input's golden thread, as every value does. For an
instant in the past that is more than it needs: a write today cannot change what a cell held last
Tuesday, but it cuts the thread, and the value as of Tuesday is computed again, the same. It is
never less than it needs, which is the direction that matters.

And the past is read-only. A write in the corridor is a `Conflict`: the state says no. The sheet
as it was is something to read, and a change belongs to the live sheet, or to a scenario.

## Inside the corridor, there is one now

`C1` holds `=NOW()`. Read it as of Tuesday evening:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:sheet:cell:C1 as-of=2026-09-22T18:00Z
source urn:iki:tutorial:sheet:cell:C1 as-of=2026-09-22T18:00Z
source urn:iki:tutorial:sheet:cell:C1'>
<pre class="ikigai-run-expected">2026-09-22T18:00Z
[computed]
2026-09-22T18:00Z
[cached]
2026-09-29T14:05Z
[cached]</pre>
</div>

`NOW()` reads the time chapter's `instant`, which reads `today` and `now`. The corridor binds both,
pinned to its instant, so every read of the time in the corridor gets the same one:

```rust,ignore
{{#include ../../../../crates/time-resource/src/lib.rs:as_of_doors}}
```

Two things follow. The first is the answer to the torn reading [Part IV](spreadsheet-4.md) guarded
against by hand, the date read on one side of midnight and the time on the other: inside a temporal
corridor there is only one instant to read, however many reads a composite makes. The second is that
a "then" is cached. Live, `NOW()` is good until the next minute and computed again after it; as of
Tuesday evening it is good for as long as the corridor, because Tuesday evening does not change.
(The live value above says `[cached]` because the sheet at the bottom of the page read it after the
last cell.)

## A scenario: what if we sold twelve?

Alice wants to know what twelve units would make, without changing the sheet everybody else reads.
Her scenario is a corridor too, and what it holds is a store of the cells she overrode:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:overrides}}
```

As a space, it answers `input:{ref}` for the cells in that store, and misses for every other read,
so the resolution falls through to whatever is outside the corridor. A write is always the
scenario's: overriding a cell nobody has overridden is how an override starts, and it must never
reach the shared sheet:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:scenario_space}}
```

A cell with `data-scenario='alice'` runs in her scenario, and says so beside its label. Like a game,
the scenario is markup, not something a line can choose: whoever may push a corridor chooses what
the names in it mean.

<div class="ikigai-run" data-scenario='alice' data-cmd='sink urn:iki:tutorial:sheet:input:A2 12
source urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">12
[uncacheable]
720
[computed]</pre>
</div>

110 × 12 − 600. On the shared sheet, nothing moved:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:sheet:cell:B2
source urn:iki:tutorial:sheet:input:A2'>
<pre class="ikigai-run-expected">500
[computed]
10
[cached]</pre>
</div>

Still 500, and `A2` is still 10. But look at the verdict: the shared `B2` was *computed*, not
served, although nothing on the shared sheet changed. Alice's write cut it. A golden thread is a
name, `input:A2`, and a write in her corridor cuts that name's thread for everybody: for her
values, which needed it, and for the shared sheet's, which did not. The shared value is computed
again and comes out the same, so nothing is ever wrong. It is only work, and the next section
says what it costs.

## One name, a cache entry in each space

Every answer is filed under the chain it was computed in, so `cell:B2` on the shared sheet and
`cell:B2` in Alice's scenario are two entries. A write to the shared sheet cuts both, because both
read the shared `B1`:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:B1 650
cache urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">650
[uncacheable]
not cached</pre>
</div>

<div class="ikigai-run" data-scenario='alice' data-cmd='cache urn:iki:tutorial:sheet:cell:B2
source urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">not cached
670
[computed]</pre>
</div>

Alice's value followed the shared cost without her doing anything: her scenario holds the one cell
she changed, and reads every other one from the sheet everybody reads. It is a corridor, not a copy.

A write in her scenario should cut only her own values. It does not, yet:

<div class="ikigai-run" data-scenario='alice' data-cmd='sink urn:iki:tutorial:sheet:input:A2 15
source urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">15
[uncacheable]
1000
[computed]</pre>
</div>

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:sheet:cell:B2
source urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">not cached
450
[computed]</pre>
</div>

This is the one thing a personal space should buy that the kernel does not deliver yet. It is
sound: the kernel only ever computes too much, never serves a wrong answer. But the waste grows
with the number of people. A hundred people with scenarios over one sheet, each editing, would
each cut the shared values, and every other scenario's, for every name they touch. The fix is for a
thread to know which corridor answered the read it hangs from, so that a write in Alice's corridor
cuts only what her corridor answered (ledger item 581). The crate's tests pin the behavior as it
is, so the day it changes, a test fails and this section is rewritten.

## Spaces stack

Alice's scenario and the sheet as of Tuesday are two corridors. A request can carry both, and
then the order matters, because the innermost corridor is asked first. The page's engine for a
scenario cell holds her chain, and a line's `as-of=` brings another; the page stacks the line's
inside hers (`Scope::stack`, core 0.1.82):

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:with_line}}
```

So a line that says `as-of=` in her scenario is Tuesday's sheet, whole. The temporal corridor
answers every input, hers included, before her scenario is asked:

<div class="ikigai-run" data-scenario='alice' data-cmd='source urn:iki:tutorial:sheet:cell:B2 as-of=2026-09-22T18:00Z'>
<pre class="ikigai-run-expected">400
[computed]</pre>
</div>

The other order puts her scenario inside Tuesday: her overrides first, and Tuesday's sheet for
everything else. That is "what would fifteen units have made last week?", and the page builds it
for a cell that says both `data-scenario='alice'` and `data-as-of`:

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:chains}}
```

<div class="ikigai-run" data-scenario='alice' data-as-of='2026-09-22T18:00Z' data-cmd='source urn:iki:tutorial:sheet:cell:B2
sink urn:iki:tutorial:sheet:input:A1 95
source urn:iki:tutorial:sheet:cell:B2'>
<pre class="ikigai-run-expected">900
[computed]
95
[uncacheable]
825
[computed]</pre>
</div>

Tuesday's price and cost, 100 and 600, and her fifteen units: 900. And a write in this chain goes
to her, since her corridor is the innermost: a price of 95 is her override now, and last Tuesday's
sheet is untouched. In the first order, the same write would have reached Tuesday's corridor first,
and been refused, because the past is read-only. The crate's test says all of this at once:

```rust,ignore
{{#include ../../../../crates/spreadsheet/tests/spaces.rs:stack_test}}
```

Neither corridor knows the other exists. The scenario was written for the shared sheet, and the
temporal view for the live one; stacked, each answers what it binds and misses the rest, and the
kernel asks them in order.

## Who may read a scenario

Alice's overrides are hers. The endpoint that answers them declares a capability named for her,
and the kernel enforces what an endpoint declares before it runs it:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:scenario_names}}
```

Narrow this cell's capability to Bob's, and read her profit in her scenario:

<div class="ikigai-run" data-scenario='alice' data-cmd='cap urn:cap:iki:tutorial:sheet:scenario:bob
source urn:iki:tutorial:sheet:cell:B2
source urn:iki:tutorial:sheet:cell:B1
cap reset'>
<pre class="ikigai-run-expected">narrowed — capability: urn:cap:iki:tutorial:sheet:scenario:bob
error: denied: capability does not grant `urn:cap:iki:tutorial:sheet:scenario:alice` (declared by `urn:iki:tutorial:sheet:input:A1`)
650
[computed]
reset to identity — capability: root (full authority)</pre>
</div>

Bob can stand in Alice's corridor, and the door is still locked. `B2` reads her price, which is an
override, so it is refused; `B1` she never overrode, so it is the shared sheet's, and needs nothing.
Two authorities are at work, and they are different ones. Who may *create* a scenario, or put a
reader in one, is whoever may push a corridor: the host, here the page. Who may *read* one is
whoever holds its capability. A corridor decides what a name means; a capability decides who may
hear the answer.

## The shared sheet

The sheet everybody reads, at the story's current instant. Its edits land on the shared sheet, and
Alice's scenario reads them through.

<div class="sheet-play" data-clock='2026-09-22T12:00Z' data-shell='urn:iki:tutorial:sheet:template:page'><p class="sheet-unavailable">The sheet appears here when the in-page kernel has loaded.</p></div>

## What was built

| the set | the name | what it is |
|---|---|---|
| the input's history | `urn:iki:tutorial:sheet:input:{ref}` | code: the same atom, now keeping every write |
| an input as of an instant | `input:{ref}` in `urn:ctx:time:{instant}` | code: the history, read at the corridor's clock; writes refused |
| the time, pinned | `urn:iki:tutorial:time:now`, `…:today` in `urn:ctx:time:{instant}` | code: the corridor's clock, as text, cacheable for as long as the corridor |
| a person's override | `input:{ref}` in `urn:iki:tutorial:sheet:scenario:{who}` | code: the second atom, a store per person, behind their capability |
| a scenario | `urn:iki:tutorial:sheet:scenario:{who}` | a corridor: a space that misses on every read it does not hold |
| a scenario over an instant | a stack of the two | `Scope::stack`: no code |

Not one line in the formulas, the compiler, the evaluator or the views changed, and none of them
knows about people or time. The whole of what this part added is two atoms' worth of state (a
history and a store of overrides), three small doors, and the arrangement: which corridors a request
carries, and in what order. That arrangement is where the new behavior came from: a "what if" that
costs one cell, a past that caches, a single now, and a capability that locks a person's cells
without locking the sheet. The one thing it promised and could not deliver, a personal edit that
leaves everybody else's cache alone, is a kernel change, and it has a name.
