# A spreadsheet, V: data that changes outside the sheet

Everything the sheet has shown so far changes for one of two reasons: somebody typed into it,
or the clock moved. Real sheets read a third kind of thing, data that belongs to somebody else:
a stock price, a sensor reading, a figure from another team's system. It changes when *they*
change it, and the sheet is not told.

This part gives the sheet a **feed**: a name that something outside the sheet writes, and a
formula function, `FEED(name)`, that reads it. The claim is the one Part III made about
recalculation, pushed one step further. The sheet needs no machinery for outside data either.
A write to a feed is a `Sink` on a name, the kernel cuts that name's golden thread, and exactly
the values that read the feed are computed again the next time somebody looks. What is new is
the last four words. The page did not make the write, so the page does not know to look.

## The model, on paper

| the set | its name | what it is |
|---|---|---|
| a **feed** | `urn:iki:tutorial:sheet:feed:{name}` | the latest value written to a feed: the second **atom**, and the first the sheet never writes |
| **the market moves** | `urn:iki:tutorial:sheet:tick:{name}` | `Sink` only: writes the feed's next price to it; the outside world, simulated |
| `FEED(name)` | in a formula; `(feed name)` compiled | reads the feed |
| the **market page** | `urn:iki:tutorial:sheet:template:market` | the sheet, a grid that polls, and a market that writes a feed |

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:feed_names}}
```

A feed's name is a lower-case letter, then lower-case letters and digits: `acme`, `fx2`. As
with a cell, one spelling per feed, so one golden thread per feed. In a formula the name may
be written any case, `FEED(Acme)`, and compiles to the one spelling, `(feed acme)`.

## The feed: an atom somebody else writes

A feed is the input's shape again: a map from a name to a string, in memory, `Sink` to write,
`Delete` to clear, and `NotFound` for a feed nobody has written to:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:feed}}
```

The page has no price service, so it has an endpoint that plays one. `tick:{name}` reads the
feed, works out a next price, and writes it, all through the kernel:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:tick}}
```

The next price is a pure function of the current one, so this chapter's outputs are the same
every time it is run:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:next_price}}
```

None of that is the sheet. From the sheet's side a feed is a name, and the evaluator reads it
the way it reads a cell:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:read_feed}}
```

## A write the sheet did not make

Write a price to the feed `acme`, as a price service would, and build a small holding on it:
the price in `A1`, a number of shares in `A2`, their value in `A3`. `C1` is arithmetic that has
nothing to do with it.

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:feed:acme 100
sink urn:iki:tutorial:sheet:input:A1 =FEED(acme)
sink urn:iki:tutorial:sheet:input:A2 10
sink urn:iki:tutorial:sheet:input:A3 =A1*A2
sink urn:iki:tutorial:sheet:input:C1 =6*7
source urn:iki:tutorial:sheet:cell:A3'>
<pre class="ikigai-run-expected">100
[uncacheable]
=FEED(acme)
[uncacheable]
10
[uncacheable]
=A1*A2
[uncacheable]
=6*7
[uncacheable]
1000
[computed]</pre>
</div>

Now the price changes. The line below writes the feed, and nothing in it mentions the sheet:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:feed:acme 101.5
cache urn:iki:tutorial:sheet:cell:A1
cache urn:iki:tutorial:sheet:cell:A2
cache urn:iki:tutorial:sheet:cell:A3
cache urn:iki:tutorial:sheet:cell:C1
source urn:iki:tutorial:sheet:cell:A3'>
<pre class="ikigai-run-expected">101.5
[uncacheable]
not cached
cached
not cached
cached
1015
[computed]</pre>
</div>

The write cut the feed's thread. `A1` read the feed and `A3` read `A1`, so both went; `A2` and
`C1` read neither and stayed. It is Part III's edit again, with one difference that is the whole
point of this part: the write came from outside the sheet, and the sheet's code has no idea it
happened. It does not need one. The dependency was recorded when `A1` read the feed, and the cut
found it.

The market moving is the same write, made by an endpoint rather than by hand. Let it move, and
trace the holding:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:tick:acme
trace urn:iki:tutorial:sheet:cell:A3'>
<pre class="ikigai-run-expected">100.34
[uncacheable]
trace  urn:iki:tutorial:sheet:cell:A3
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:sheet:cell:A3   sheet-cell · computed · ThreadId(1) · —   → 6b  1003.4
├─ urn:iki:tutorial:sheet:input:A3   sheet-input · cached · ThreadId(1) · —
├─ urn:iki:tutorial:sheet:precedents:A3   sheet-precedents · cached · ThreadId(1) · —
├─ urn:iki:tutorial:sheet:formula:A3   sheet-formula · cached · ThreadId(1) · —
├─ urn:iki:tutorial:sheet:cell:A1   sheet-cell · computed · ThreadId(1) · —
│  ├─ urn:iki:tutorial:sheet:input:A1   sheet-input · cached · ThreadId(1) · —
│  ├─ urn:iki:tutorial:sheet:precedents:A1   sheet-precedents · cached · ThreadId(1) · —
│  ├─ urn:iki:tutorial:sheet:formula:A1   sheet-formula · cached · ThreadId(1) · —
│  └─ urn:iki:tutorial:sheet:feed:acme   sheet-feed · computed · ThreadId(1) · —
└─ urn:iki:tutorial:sheet:cell:A2   sheet-cell · cached · ThreadId(1) · —</pre>
</div>

The feed was read and computed again, because the market wrote it. `A1`'s value read the feed,
`A3`'s read `A1`'s, and `A2`'s was served. Nothing in the trace is about the market: the sheet
sees a name whose thread was cut, and never who cut it.

## A feed nobody has written to

A formula can name a feed before anything has been written to it. The feed answers `NotFound`,
and the value is `#N/A`, the sheet's word for "not available":

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:B1 =FEED(globex)
source urn:iki:tutorial:sheet:cell:B1
source urn:iki:tutorial:sheet:cell:B1
sink urn:iki:tutorial:sheet:feed:globex 55
cache urn:iki:tutorial:sheet:cell:B1
source urn:iki:tutorial:sheet:cell:B1'>
<pre class="ikigai-run-expected">=FEED(globex)
[uncacheable]
#N/A
[computed]
#N/A
[cached]
55
[uncacheable]
not cached
55
[computed]</pre>
</div>

`#N/A` was cached, and the first write to the feed cut it. When a request fails, the kernel
records the failure as a dependency of whatever asked, so an answer built on a feed's absence
hangs from the feed's thread, and the first write to the feed cuts it.

That has only been true all the way down since core 0.1.82, and the story of the gap is worth
knowing. Before it, a failure was recorded under the thread of the name that was *asked for*, and
nothing else. `B1` asked for `feed:globex` itself, the atom, and the atom's own thread is exactly
what a write to it cuts, so `B1` was safe. Had it asked for some composite that read the feed, and
fallen back on *that* composite's `NotFound`, the `#N/A` would have hung from the composite's
thread, which no write ever cuts, and stayed `#N/A` after the feed was written. The sheet was built
around the gap: Part II's evaluator reads a cell's input before its compiled formula, never falling
back on the formula's `NotFound`. Building it is how the gap was found (ledger item 611), and
0.1.82 closed it: a failed request now carries the dependencies it recorded on its way to failing,
so a fallback hangs from the atom's thread however far down the `NotFound` came from.

The rule the sheet follows is still a good one, for a different reason: **fall back on an atom's own
`NotFound`**, because that is the one that says exactly what is missing. A composite's `NotFound`
may mean the atom is empty, or that it holds the wrong kind of thing (a cell holding a number has no
compiled formula), and a fallback cannot tell which.

```rust,ignore
{{#include ../../../../crates/spreadsheet/tests/live.rs:unwritten}}
```

## Bad data is a value

A feed can hold anything its writer sends, and one day that will not be a number. Say the
exchange halts trading and the feed says so:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:feed:acme halted
source urn:iki:tutorial:sheet:cell:A3
source urn:iki:tutorial:sheet:cell:A3
sink urn:iki:tutorial:sheet:tick:acme
sink urn:iki:tutorial:sheet:feed:acme 100'>
<pre class="ikigai-run-expected">halted
[uncacheable]
#VALUE
[computed]
#VALUE
[cached]
error: conflict: the feed `acme` holds `halted`, which is not a price
100
[uncacheable]</pre>
</div>

`A3` is `#VALUE`, and the second read is served from the cache. That is a choice, and the
other choice is worse than it looks. Had the evaluator refused, the refusal would never have
been cached: the kernel records every error except `NotFound` as already expired, since most
errors are transient. So `A3`, and the grid that shows it, would be computed again on every read
for as long as the feed said `halted`, which could be all afternoon. When the honest answer is
"this input is wrong", say it as a value, and it caches like any other.

The market itself does refuse, with a `Conflict`: it will not move a price that is not a price.
That refusal is right where it is, because it answers a write, and a write is never cached. The
last line puts the price back, for the page below.

## Finding out: polling

The runnable cells above redraw the sheet at the bottom of the page after they run, as they did
in earlier parts. That is the page reacting to something the page did. The market is different.
Press **Run the market** below and two things start, both driven by the page's timers and both
paused until you press it: the market writes a new price to `acme` every three seconds, and the
grid asks for itself again every two. Neither knows about the other. The market does not redraw
the grid; it only writes the feed. The grid finds out the way anything that reads a name finds
out about a write it did not make: it asks again.

Put `=FEED(acme)` somewhere, and something that reads it, and watch the count and the table
under the grid. Most polls are answered entirely from the cache, and cost a lookup. The poll
after each write computes exactly the values that read the feed, and serves the rest.

<div class="sheet-play" data-cache data-shell='urn:iki:tutorial:sheet:template:market'><p class="sheet-unavailable">The sheet appears here when the in-page kernel has loaded.</p></div>

**The market moves** writes once, by hand. The grid does not change until its next poll, which
is the lesson rather than a bug: the button is somebody else, and the grid is not told.

The same thing as a test, with the market writing every third second and the grid polled every
second:

```rust,ignore
{{#include ../../../../crates/spreadsheet/tests/live.rs:poll}}
```

A poll answered from the cache is cheap in the page, and a host serving the sheet over HTTP makes
it cheaper still. ikigai's web edge gives a cached answer a strong `ETag`, and answers a poll
that sends it back (`If-None-Match`) with `304 Not Modified` and no body: the grid is not even
sent again. That is the whole cost of polling a resource whose answer has not changed: a lookup,
and a few bytes of headers. It is why polling is a reasonable first answer, and the time
chapter's clock, polled every second and computed once a minute, was the same argument.

Its cost is latency and waste at the edges. A write is seen up to one polling interval late, and
every poll between writes is a request that learns nothing. For a sheet that must show a price
the moment it moves, and for a thousand readers of one feed, the answer is the other direction:
the kernel tells the page.

## Next: push

This part stops at polling on purpose. Push needs pieces this book does not have yet, and it is
worth saying exactly what they are, and which of them the kernel already provides.

**From the kernel: somebody to tell.** A cut moves a counter on, and by default tells nobody;
that laziness is what makes an edit cost nothing until somebody reads (Part III). Push needs a host
to be able to say "tell me when this thread is cut", for an exact name or for a prefix like
`urn:iki:tutorial:sheet:feed:`, and to be told, with the thread's name, when it is. This is
NetKernel's golden-thread listener, and since core 0.1.82 the kernel has one, `Kernel::listen`,
built to the constraints a kernel with no runtime puts on it. It cannot call anyone back on a task
of its own, and a cut runs inside the write that caused it, so it calls nobody: it appends the cut to
a queue the host drains when it chooses, and issues no request from inside a cut. Listening is an
authority, so it is a capability, and a listener hears only the names its own reads were already
given. And the queue is bounded: a cut that finds it full is counted as dropped, so a slow host
learns that it missed something rather than acting on half a history.

**From the host: what to redraw.** Hearing that `feed:acme` was cut is not yet knowing that the grid
changed, but the listener says that too: each cut comes with the cached answers it invalidated, the
grid among them when the grid read the feed. So a host that hears a cut can push the views it has
handed out that are on that list, or read them again straight away, so the recomputation is done
before the reader asks: a recalculating golden thread, which NetKernel also had. The host half is
[Part VII](spreadsheet-7.md): there the page registers a listener with its kernel and drains it.

**From the page: a channel.** In this book the host is the page, so being told is a function
call. A sheet served by a real host needs a connection that the server can write to: server-sent
events or a WebSocket, which htmx can already consume. The markup would change from "ask every
two seconds" to "listen for `grid`", and nothing else in the sheet would.

**Time does not cut.** A value that goes because its deadline passed was never cut, so a listener
never hears about it. A `NOW()` cell pushed to a page needs the host to set a timer for the
earliest deadline among the answers it has handed out; the kernel knows each deadline, and would
have to say what it is.

**And the same answer twice.** A cut is by name, not by content, so the market writing the price
the feed already holds cuts it anyway, and push would send a grid identical to the last. Stopping
there needs the recomputed answer compared with the old one before anything downstream is told:
early cutoff, which Part III noted the kernel does not do.

## What was built

| the set | the name | what it is |
|---|---|---|
| a feed | `urn:iki:tutorial:sheet:feed:{name}` | code: the second atom, written from outside the sheet |
| the market | `urn:iki:tutorial:sheet:tick:{name}` | code: the outside world, simulated; a `Sink` that reads the feed and writes the next price |
| `FEED(name)` | in a formula; `(feed name)` compiled | a word for the compiler, and one read of a name for the evaluator |
| the market page | `urn:iki:tutorial:sheet:template:market` | a template: the sheet, a grid that polls, and the market's controls |

The sheet gained an atom and learned to read it, and nothing about recalculation changed. The
market is the only new code that does anything, and it is not the sheet's: in a real deployment
it is somebody else's system, writing a name. A cell's value is still a plain name,
`cell:{ref}`, and so are an input and a feed, which is what [the next part](spreadsheet-6.md) needs. A *personal*
scenario, my own numbers laid over the shared sheet's, is a corridor in front of `input:{ref}`,
and the sheet reads the names it always read.
