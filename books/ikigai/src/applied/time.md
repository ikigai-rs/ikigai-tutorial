# Time is a resource

Everything the book has built so far changes only when something writes. A `Sink` plays a
mark, the kernel cuts the stored cell's golden thread, and whatever was computed from that
cell is computed again the next time somebody asks. Nothing changes on its own.

Time does. The answer to "what time is it?" goes out of date with nobody writing anything,
so a golden thread cannot say when to let it go. The kernel has a second way to let an
answer go, and this chapter is about it: **expiry**. A representation can say *keep me
until this instant*, and the kernel serves it from its cache until then and not a
millisecond longer.

The chapter builds a clock, a date, a named event and a countdown to it, and ends with a
clock on the page that asks for the time every second. The lesson is in how often the
kernel actually does any work when it is asked that often.

## The model, on paper

As in the [tic-tac-toe](tic-tac-toe-1.md) parts, the names come first:

| the set | its name | what it is |
|---|---|---|
| **now** | `urn:iki:tutorial:time:now` | the time of day, `HH:MM`, read from the kernel's clock; good until the next minute |
| **today** | `urn:iki:tutorial:time:today` | the date, `YYYY-MM-DD`; good until the next midnight |
| an **event** | `urn:iki:tutorial:time:event:{name}` | a named instant, `YYYY-MM-DDTHH:MMZ`, set with `Sink` |
| the **instant** | `urn:iki:tutorial:time:instant` | `today` and `now` as one reading, `YYYY-MM-DDTHH:MMZ`; good until the next minute |
| a **countdown** | `urn:iki:tutorial:time:until:{name}` | minutes from now until the event |
| a **template** | `urn:iki:tutorial:time:template:{name}` | the chapter's HTML |
| the **clock view** | `urn:iki:tutorial:time:view:clock` | the time as HTML, for a page to show |

Only the event holds state: it is the one name a `Sink` writes, and it holds an instant
until somebody moves it. `now` and `today` hold nothing. They read the clock, and the clock
is not theirs.

```rust,ignore
{{#include ../../../../crates/time-resource/src/lib.rs:names}}
```

All the times in this chapter are UTC. A time zone is not part of what time it *is*; it is
part of where the person asking is standing, which makes it context rather than a name,
and this chapter leaves it out.

## A kernel has a clock only if its host gives it one

The core crate reads no clock of its own. A host that wants its kernel to know the time
injects a clock with `Kernel::with_clock`, the way it injects a space or a Meta renderer,
and an endpoint asks for the time with `Invocation::now`. So "what time is it?" is a
question the host answers, which is what lets a test answer it with a clock that stands
still, and a replay answer it with the instant being replayed.

Almost every page of this book runs a kernel with no clock. That is why a trace
in [What resolution buys you](../getting-started/payoff.md) prints `—` where a duration
would go. This page asks for one, and so do [A spreadsheet, IV](spreadsheet-4.md), whose
cells read the time, and [A spreadsheet, VI](spreadsheet-6.md), which keeps a story clock so its
week of history reads the same for every reader. This page's markup carries a `data-clock` attribute, and when the
page sees it, it builds its kernel with the browser's clock before anything resolves:

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:page_kernel_with_clock}}
```

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:browser_clock}}
```

Without a clock, the time endpoints do not guess. They refuse, and say why:

```rust,ignore
{{#include ../../../../crates/time-resource/src/lib.rs:clock}}
```

On any other page of this book, `source urn:iki:tutorial:time:now` answers with that
refusal.

## Now, until the next minute

```rust,ignore
{{#include ../../../../crates/time-resource/src/lib.rs:now}}
```

The last line is the whole idea. The time is `14:05` from 14:05:00 to 14:05:59.999, so the
answer says so: `cacheable_until` the start of the next minute. That is an `Expiry::At`, a
deadline, and the kernel checks it against the same clock the answer was read from.

Ask twice. Your times will be your own; the outputs under each cell are from a test that
stopped a clock at 14:05:20 UTC on 29 September 2026.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:time:now
source urn:iki:tutorial:time:now'>
<pre class="ikigai-run-expected">14:05
[computed]
14:05
[cached]</pre>
</div>

The second answer ran nothing. Now wait for the minute to turn, and ask whether the kernel
still holds it:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:time:now
source urn:iki:tutorial:time:now'>
<pre class="ikigai-run-expected">not cached
14:06
[computed]</pre>
</div>

Nobody wrote anything and no thread was cut. The deadline passed, so the answer went. The
crate's tests put the boundary exactly where the code says it is, with a clock the test
holds and moves by hand:

```rust,ignore
{{#include ../../../../crates/time-resource/tests/time.rs:minute}}
```

`today` is the same resource with a longer deadline: the date, cacheable until the next
midnight.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:time:today
cache urn:iki:tutorial:time:today'>
<pre class="ikigai-run-expected">2026-09-29
[computed]
cached</pre>
</div>

## An event, and a countdown to it

The event is the atom: a map from a name to an instant, in memory, like the tic-tac-toe
stored cell. `Sink` takes an instant spelled `YYYY-MM-DDTHH:MMZ`, or `+N` for N minutes from
now, and answers the instant it now holds. `+90` does not store "ninety minutes from
whenever you ask"; it reads the time once, when you set it, and stores where that lands.

The countdown is a composite. It reads the event, `today` and `now`, all through the
kernel, and subtracts:

```rust,ignore
{{#include ../../../../crates/time-resource/src/lib.rs:until}}
```

It says `.cacheable()`, with no deadline, as if its answer were good forever. Set an event
ninety minutes out and count down to it:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:time:event:tea +90
source urn:iki:tutorial:time:until:tea
source urn:iki:tutorial:time:until:tea'>
<pre class="ikigai-run-expected">2026-09-29T15:36Z
[uncacheable]
90
[computed]
90
[cached]</pre>
</div>

Now let the minute turn again, and ask:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:time:until:tea
cache urn:iki:tutorial:time:event:tea
source urn:iki:tutorial:time:until:tea'>
<pre class="ikigai-run-expected">not cached
cached
89
[computed]</pre>
</div>

The countdown went at the minute, and the event it was computed from did not. Nothing in
the countdown's code mentions a minute. The kernel met the countdown's own expiry, *never*,
with the expiry of everything it read, and took the soonest: `today`'s midnight, `now`'s
next minute, and the event's *never* come to `now`'s next minute. A composite is never
fresher than the least fresh thing it read. ikigai's own working rules put it as
*cacheability is only as good as the least cacheable dependency*, and here it is working
for you: the countdown is exactly as cacheable as `now`, and not because anybody said so.

The rule has a second edge, and it cuts the other way. Add a clock to a composite that
used to be cached for good, and it is cached for a minute from then on, with nothing in its
code or its types to show for it. That is a performance change, even when it is not a
correctness one, and the only way to see it is to measure the read.

The event's golden thread still works as it always did. Move the event, and the countdown
goes at once, in the middle of the minute, while `now` stays cached:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:time:event:tea 2026-09-29T14:37Z
cache urn:iki:tutorial:time:until:tea
cache urn:iki:tutorial:time:now
source urn:iki:tutorial:time:until:tea'>
<pre class="ikigai-run-expected">2026-09-29T14:37Z
[uncacheable]
not cached
cached
30
[computed]</pre>
</div>

Two ways for an answer to go, then, and a composite gets both from what it reads: a thread
that a write cuts, and a deadline that a clock passes.

```rust,ignore
{{#include ../../../../crates/time-resource/tests/time.rs:countdown}}
```

### Two readings of one clock

There is one trap in reading the date and the time as two resources, and it is worth a
paragraph because every composition over time meets it. They are read at two different
instants. Read `today` at 23:59:59.999 and `now` a millisecond later, and you have
yesterday's date with today's time: a countdown a whole day out. It is not *cached* a day
out: the date it read expired at the midnight it straddled, so the kernel never serves that
answer again. But it has already been served once, wrong.

So the countdown reads the date on both sides of the time. If the date is the same both
times, the time was read on that date:

```rust,ignore
{{#include ../../../../crates/time-resource/src/lib.rs:this_minute}}
```

The test for it starts a clock that moves on a millisecond every time anybody reads it,
just before midnight, two hundred times over, so that some resolution straddles the
boundary. With the second read of the date taken out, it fails, a day out; with it, it
passes.

A guard that every composite has to remember is a guard some composite will forget, so the
crate also gives the guarded reading a name of its own, `urn:iki:tutorial:time:instant`: the
current minute as one instant, `YYYY-MM-DDTHH:MMZ`. Whatever wants the date and the time
together reads that one name and cannot tear them. It declares no deadline of its own, and is
cached until the next minute, which it inherits from `now`:

```rust,ignore
{{#include ../../../../crates/time-resource/src/lib.rs:instant}}
```

The spreadsheet's `NOW()` reads it, in [A spreadsheet, IV](spreadsheet-4.md).

## A page that polls

Here is the clock. It is paused until you start it, so the cells above answer as they
say; press **Poll** and it asks the kernel for the time once a second, and
counts what that cost.

<div class="ikigai-view" data-clock data-shell='urn:iki:tutorial:time:template:face'><p class="ikigai-view-unavailable">The clock appears here when the in-page kernel has loaded.</p></div>

Watch the count. The page asks sixty times a minute, and the kernel computes the time
once: the first request after each minute boundary. The other fifty-nine are answered
from the cache, which is a lookup, not a computation. So a poll is cheap when the answer
is cached, and the thing that decides when work happens is the answer's expiry, not how
often anybody asks. Poll every tenth of a second and the kernel still computes the time
once a minute.

The same thing as a test, with a clock the test moves one second at a time:

```rust,ignore
{{#include ../../../../crates/time-resource/tests/time.rs:poll}}
```

### The markup

The clock's markup is two templates, served as resources like tic-tac-toe's. The one the
page loads is the poller:

```text
{{#include ../../../../crates/time-resource/templates/face.html}}
```

It asks for `iki/tutorial/time/view/clock` on load and every second after, by the same
path rule as the board in [Tic-tac-toe, V](tic-tac-toe-5.md), and puts the answer into the
`<span>` inside it: `hx-target` names a child, not the element itself. The web demo's
first poller targeted itself, and its host processed the swap target again after every
answer, which broke the poll cycle. A child is never the element htmx polls from, so
whatever a host does with the target, the poller is left alone and only what it shows
changes.

The view is the other template, and it has no view code of its own to speak of:

```text
{{#include ../../../../crates/time-resource/templates/clock.html}}
```

A `$h{…}` marker names a resource, and the generic composer from `ikigai-fn` resolves every
name it finds and splices the answer in, escaped for HTML. That is the template language
[Tic-tac-toe, V](tic-tac-toe-5.md#the-template-language) teaches, and the clock uses the smallest
part of it. The template names what it shows, so nothing fills it from code, and the view is the
template bound at a name of its own with `compose_over`, the way the game's board and squares are:

```rust,ignore
{{#include ../../../../crates/time-resource/src/lib.rs:view_clock}}
```

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:time:view:clock
cache urn:iki:tutorial:time:now'>
<pre class="ikigai-run-expected">&lt;time datetime=&quot;14:07&quot;&gt;14:07&lt;/time&gt;
[computed]
cached</pre>
</div>

Compose knows nothing about time. It declares its answer cacheable and lets the kernel
meet that with what it read, so the view expires with `now`, like the countdown.

There is no view code: `compose_over` is the composer with its template fixed, so binding it at
`view:clock` makes the view a resource like any other, with nothing of its own to run. (Before
`ikigai-fn` 0.3.0 the view was one request of glue, because a name could not carry the argument
that tells compose which template to fill.)

`$h` escapes what it splices. Here that changes nothing, since the only thing spliced is the
kernel's own clock, but it is the marker to reach for by default: a value that is text somebody
else chose, a tic-tac-toe mark or a spreadsheet cell, must never be spliced in as markup.

## What was built

| the set | the name | what it is |
|---|---|---|
| now, today | `urn:iki:tutorial:time:now`, `…:time:today` | code: small functions of the kernel's clock, each with a deadline |
| the instant | `urn:iki:tutorial:time:instant` | code: the guarded reading of `today` and `now`, with `now`'s deadline inherited |
| an event | `urn:iki:tutorial:time:event:{name}` | code: the one atom |
| a countdown | `urn:iki:tutorial:time:until:{name}` | code: a small function of three names, with its deadline inherited |
| templates | `urn:iki:tutorial:time:template:{name}` | resources: HTML, one of which names what it shows |
| the clock view | `urn:iki:tutorial:time:view:clock` | a template bound at a name: no code |

Nothing here polls on the server side, nothing sets a timer, and nothing invalidates a
cache. The page asks as often as it likes; each answer says how long it is good for; the
kernel does the arithmetic. What is left for later is the time *zone*, which is context
rather than a name, and making things happen *at* a time rather than answering when asked.
