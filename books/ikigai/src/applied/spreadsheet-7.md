# A spreadsheet, VII: push

[Part V](spreadsheet-5.md) ended with a page that found out about a write it did not make the
only way a reader of a name can: it asked again, every two seconds, whether anything had changed
or not. It also listed what the other direction needs, the kernel telling the page. There were
four pieces. Somebody to tell: the kernel has had that since core 0.1.82, `Kernel::listen`. What
to redraw: the host's business. A channel: in this book the host is the page, so being told is a
function call. And a timer, because time does not cut.

This part builds the host's half, in the page. It needs no server and no new core, and the sheet
gains one template, the page that asks each cell to listen, and nothing else: not a line of its
formulas, its evaluator or its views changes.

## The model, on paper

| the set | its name | what it is |
|---|---|---|
| a **cut listener** | `urn:iki:tutorial:push:listener:{name}` | `Sink` a prefix to listen for cuts of every thread under it; `Source` takes what it has heard; `Delete` stops it. The state is the kernel's queue |
| what a read **carries** | `urn:iki:tutorial:push:carries` | the golden threads an answer hangs from, and when it expires: a question about another read, answered by making it |
| the **views drawn** | not a name: the page's own memory | for each view the page drew, what its answer carried |
| the **listening page** | `urn:iki:tutorial:sheet:template:push` | the sheet with each cell a view of its own, drawn again when the page says it is stale |

```rust,ignore
{{#include ../../../../crates/push/src/lib.rs:names}}
```

A listener is a resource so that it can be shown in a cell, and so that listening is checked the
way everything else is: by the kernel, against what the resource declares. The page's own push
uses the same listener, and waits on it with a function call rather than reading it.

## Listening is an authority

A listener learns the name of every thread cut under its prefix, and a thread is named after the
resource that was written. That is information, so listening needs a capability,
`urn:cap:kernel:listen`, and the listener declares it on every verb:

```rust,ignore
{{#include ../../../../crates/push/src/lib.rs:listener}}
```

Narrow this cell's capability to Bob's, from Part VI, and ask to listen. Then ask again with the
page's own:

<div class="ikigai-run" data-cmd='cap urn:cap:iki:tutorial:sheet:scenario:bob
sink urn:iki:tutorial:push:listener:mine urn:iki:tutorial:sheet:
cap reset
sink urn:iki:tutorial:push:listener:mine urn:iki:tutorial:sheet:
source urn:iki:tutorial:push:listener:mine'>
<pre class="ikigai-run-expected">narrowed — capability: urn:cap:iki:tutorial:sheet:scenario:bob
error: denied: capability does not grant `urn:cap:kernel:listen` (declared by `urn:iki:tutorial:push:listener:mine`)
reset to identity — capability: root (full authority)
listening for cuts under urn:iki:tutorial:sheet:, as root: every cut there
[uncacheable]
heard nothing
[uncacheable]</pre>
</div>

The refusal is the kernel's, made before the listener's code ran, because the listener declares
what it needs. `Kernel::listen` refuses too, in case something registers a listener without going
through the resource. And the refusal is an answer: a page whose reader may not listen says so,
in words, and keeps working, as the sheet at the bottom of this page does if its registration is
refused.

The listener holds the registration and nothing else. The queue is the kernel's:

```rust,ignore
{{#include ../../../../crates/push/src/lib.rs:listening}}
```

## A write, heard

Part V's holding again: a price in a feed, a quantity, their value, and some arithmetic that has
nothing to do with them. Then ask the listener what it heard:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:feed:acme 100
sink urn:iki:tutorial:sheet:input:A1 =FEED(acme)
sink urn:iki:tutorial:sheet:input:A2 10
sink urn:iki:tutorial:sheet:input:A3 =A1*A2
sink urn:iki:tutorial:sheet:input:C1 =6*7
source urn:iki:tutorial:sheet:cell:A3
source urn:iki:tutorial:push:listener:mine'>
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
[computed]
heard 5 cuts
cut urn:iki:tutorial:sheet:feed:acme
cut urn:iki:tutorial:sheet:input:A1
  invalidated urn:iki:tutorial:sheet:cell:A1
  invalidated urn:iki:tutorial:sheet:view:cell:A1
cut urn:iki:tutorial:sheet:input:A2
  invalidated urn:iki:tutorial:sheet:cell:A2
  invalidated urn:iki:tutorial:sheet:view:cell:A2
cut urn:iki:tutorial:sheet:input:A3
  invalidated urn:iki:tutorial:sheet:cell:A3
  invalidated urn:iki:tutorial:sheet:view:cell:A3
cut urn:iki:tutorial:sheet:input:C1
  invalidated urn:iki:tutorial:sheet:cell:C1
  invalidated urn:iki:tutorial:sheet:view:cell:C1
[uncacheable]</pre>
</div>

Five writes, five cuts, each with the cached answers it invalidated. The views are the sheet's at
the bottom of this page, which drew all twenty-four of its cells, empty, when the page loaded, and
four of those drawings just went out of date. The first cut invalidated nothing: nothing had read
the feed yet.

Reading the listener took what it had heard. Now the price moves, and nothing else happens:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:feed:acme 101.5
source urn:iki:tutorial:push:listener:mine'>
<pre class="ikigai-run-expected">101.5
[uncacheable]
heard 1 cut
cut urn:iki:tutorial:sheet:feed:acme
  invalidated urn:iki:tutorial:sheet:cell:A1
  invalidated urn:iki:tutorial:sheet:cell:A3
  invalidated urn:iki:tutorial:sheet:feed:acme
[uncacheable]</pre>
</div>

One write, one cut, and not one poll. The kernel's list names the feed and the two values that
read it, the ones the previous cell computed. It does not name the views of `A1` and `A3` that the
sheet below is showing, and that is not an omission. The list is the kernel's account of its
**cache**: what was stored and is now invalid. Those two views were already invalid, cut by the
previous cell's writes, and nobody has drawn them since, because the sheet below is not pushing.
They are still on the screen. A page cannot learn what its screen shows from the kernel's cache,
so it keeps its own account.

## From a cut to the views

What the page needs to know about each view it drew is what that view's answer rested on, and the
answer already says. The kernel puts on every answer the golden threads of everything read while
computing it, however far down, so a view carries the thread of every value, input, formula and
feed under it. Ask what a read carries:

```rust,ignore
{{#include ../../../../crates/push/src/lib.rs:carries}}
```

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:push:carries target=urn:iki:tutorial:sheet:view:cell:A3
source urn:iki:tutorial:push:carries target=urn:iki:tutorial:sheet:view:cell:C1'>
<pre class="ikigai-run-expected">urn:iki:tutorial:sheet:view:cell:A3
expires never: only a cut changes it
hangs from 17 threads
  urn:iki:tutorial:sheet:cell:A1
  urn:iki:tutorial:sheet:cell:A2
  urn:iki:tutorial:sheet:cell:A3
  urn:iki:tutorial:sheet:compile
  urn:iki:tutorial:sheet:feed:acme
  urn:iki:tutorial:sheet:formula:A1
  urn:iki:tutorial:sheet:formula:A3
  urn:iki:tutorial:sheet:input:A1
  urn:iki:tutorial:sheet:input:A2
  urn:iki:tutorial:sheet:input:A3
  urn:iki:tutorial:sheet:precedents:A1
  urn:iki:tutorial:sheet:precedents:A3
  urn:iki:tutorial:sheet:refs:A1
  urn:iki:tutorial:sheet:refs:A2
  urn:iki:tutorial:sheet:refs:A3
  urn:iki:tutorial:sheet:template:cell
  urn:iki:tutorial:sheet:view:cell:A3
[uncacheable]
urn:iki:tutorial:sheet:view:cell:C1
expires never: only a cut changes it
hangs from 8 threads
  urn:iki:tutorial:sheet:cell:C1
  urn:iki:tutorial:sheet:compile
  urn:iki:tutorial:sheet:formula:C1
  urn:iki:tutorial:sheet:input:C1
  urn:iki:tutorial:sheet:precedents:C1
  urn:iki:tutorial:sheet:refs:C1
  urn:iki:tutorial:sheet:template:cell
  urn:iki:tutorial:sheet:view:cell:C1
[uncacheable]</pre>
</div>

`A3`'s view hangs from the feed, three names down; `C1`'s does not. That is the whole rule for
push: **a cut of thread T makes stale exactly the views whose answers carried T.** The page
remembers, for each view it draws, what the answer carried, and nothing else:

```rust,ignore
{{#include ../../../../crates/push/src/lib.rs:drawn}}
```

There is no index of dependents anywhere, in the page or in the kernel. The dependency graph was
recorded when the view was computed, and handed to the page on the answer. The same thing as a
test, with every one of the sheet's twenty-four views drawn:

```rust,ignore
{{#include ../../../../crates/push/tests/push.rs:feed_write}}
```

Two of twenty-four. A poll would have asked all twenty-four, and been told twenty-two times that
nothing had changed.

## Time does not cut

`NOW()` changes every minute, and nobody writes anything when it does. The kernel lets the
answer go at its deadline, and a listener hears nothing, because there was no cut. So the page
reads each view's deadline off its answer too, and keeps one timer, at the earliest. The page's
kernel has the browser's clock (the sheet carries `data-clock`), so the outputs below are from a
test that stopped a clock at 14:05:20 UTC; your deadline is the next minute of your day.

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:D1 =NOW()
source urn:iki:tutorial:push:carries target=urn:iki:tutorial:sheet:view:cell:D1
source urn:iki:tutorial:push:listener:mine'>
<pre class="ikigai-run-expected">=NOW()
[uncacheable]
urn:iki:tutorial:sheet:view:cell:D1
expires at 2026-09-29T14:06Z
hangs from 11 threads
  urn:iki:tutorial:sheet:cell:D1
  urn:iki:tutorial:sheet:compile
  urn:iki:tutorial:sheet:formula:D1
  urn:iki:tutorial:sheet:input:D1
  urn:iki:tutorial:sheet:precedents:D1
  urn:iki:tutorial:sheet:refs:D1
  urn:iki:tutorial:sheet:template:cell
  urn:iki:tutorial:sheet:view:cell:D1
  urn:iki:tutorial:time:instant
  urn:iki:tutorial:time:now
  urn:iki:tutorial:time:today
[uncacheable]
heard 1 cut
cut urn:iki:tutorial:sheet:input:D1
  invalidated urn:iki:tutorial:sheet:cell:D1
  invalidated urn:iki:tutorial:sheet:view:cell:D1
[uncacheable]</pre>
</div>

The view is good until 14:06, and the only cut was the write. Wait for the minute to turn, then:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:push:listener:mine
cache urn:iki:tutorial:sheet:view:cell:D1'>
<pre class="ikigai-run-expected">heard nothing
[uncacheable]
not cached</pre>
</div>

The view is gone from the cache, and the listener heard nothing. A page that only listened would
show 14:05 forever. The timer is what draws it again, and only it, since no other view has a
deadline:

```rust,ignore
{{#include ../../../../crates/push/tests/push.rs:minute}}
```

So push has two triggers, and the page needs both: a **cut**, announced by the kernel, and an
**expiry**, which the page has to know about in advance, because nobody will announce it.

## A scenario's write is heard by everybody

Part VI admitted a limit: a golden thread is a name, not a name in a space, so Alice's override in
her scenario cuts the shared sheet's values too (ledger item 581). They are computed again and
come out the same, so nothing is ever wrong, only more work than necessary. With push, the work
becomes visible. Alice sells twelve:

<div class="ikigai-run" data-scenario='alice' data-cmd='sink urn:iki:tutorial:sheet:input:A2 12
source urn:iki:tutorial:sheet:cell:A3'>
<pre class="ikigai-run-expected">12
[uncacheable]
1218
[computed]</pre>
</div>

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:push:listener:mine'>
<pre class="ikigai-run-expected">heard 1 cut
cut urn:iki:tutorial:sheet:input:A2
  invalidated urn:iki:tutorial:sheet:cell:A2
  invalidated urn:iki:tutorial:sheet:cell:A3
  invalidated urn:iki:tutorial:sheet:input:A2
  invalidated urn:iki:tutorial:sheet:precedents:A3
  invalidated urn:iki:tutorial:sheet:refs:A2
  invalidated urn:iki:tutorial:sheet:view:cell:A3
[uncacheable]</pre>
</div>

Every entry on that list is the shared sheet's: Alice's scenario had read nothing before her write.
The shared view of `A3` is among them, so a page showing the shared sheet with push on draws `A2` and
`A3` again, to show the values they already showed. The test says it plainly:

```rust,ignore
{{#include ../../../../crates/push/tests/push.rs:scenario}}
```

Polling hides this, because a poll of an unchanged value is a cache lookup either way. Push turns it
into traffic: a redraw sent to every reader of the shared sheet, for a change none of them can see.
A hundred people with scenarios would each send the shared sheet's readers a redraw per edit. The
fix is still the kernel's, a thread that knows which corridor answered the read it hangs from, and
the page cannot make it. It can only show it, and on the sheet below you can watch it happen.

## A listener hears only what its reads rest on

The page's cells run as root, and a root listener hears every cut under its prefix. Any other
authority hears less: a cut of a thread only when some answer computed under that same capability
rests on it, and a list of invalidated answers from its own reads and nobody else's. It learns only
names it was already given.

<div class="ikigai-run" data-cmd='cap urn:cap:kernel:listen
sink urn:iki:tutorial:push:listener:session urn:iki:tutorial:sheet:
source urn:iki:tutorial:sheet:cell:A3
cap reset
sink urn:iki:tutorial:sheet:feed:acme 102
sink urn:iki:tutorial:sheet:input:B6 hello
cap urn:cap:kernel:listen
source urn:iki:tutorial:push:listener:session
cap reset'>
<pre class="ikigai-run-expected">narrowed — capability: urn:cap:kernel:listen
listening for cuts under urn:iki:tutorial:sheet:, under this capability: a cut there only when a read made under it rests on it
[uncacheable]
1015
[computed]
reset to identity — capability: root (full authority)
102
[uncacheable]
hello
[uncacheable]
narrowed — capability: urn:cap:kernel:listen
heard 1 cut
cut urn:iki:tutorial:sheet:feed:acme
  invalidated urn:iki:tutorial:sheet:cell:A1
  invalidated urn:iki:tutorial:sheet:cell:A3
  invalidated urn:iki:tutorial:sheet:feed:acme
[uncacheable]
reset to identity — capability: root (full authority)</pre>
</div>

The session read `A3`; somebody else moved the price and typed into `B6`. The session heard the
price, which its `A3` rests on, and not `B6`, which it never read. This is the shape a server wants:
one listener per session, registered under the session's own capability, so that it hears about
exactly what that session was sent.

## The page, pushing

The sheet below is Part V's, drawn a different way. Each cell is a view of its own, and each says,
in its markup, when it wants to be drawn: when the page loads, when the page says it is **stale**,
and every two seconds while polling is on. Here is the first row of the template:

```html
{{#include ../../../../crates/spreadsheet/templates/push.html:7}}
```

The markup changed from "ask every two seconds" to "listen for `stale`", as Part V said it would,
and nothing in the sheet's code changed with it. What the page adds is small. Its wasm module (the
page's host, `crates/book-wasm`) exports four things for it: a draw that remembers what the answer carried, a wait on a listener, and the
deadline and the expired views for the timer:

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:push_exports}}
```

And the page's side, in `js/sheet.js`: register, wait, find the cells that show the stale views,
trigger `stale` on each, and wait again; and keep one timer at the earliest deadline:

```js
{{#include ../../js/sheet.js:push_host}}
```

The kernel never calls the page. A cut runs inside the write that caused it, so it only appends to
the listener's queue and wakes the page's wait, and the page draws on its own turn of the event
loop. The queue is bounded, and a cut that finds it full is counted as dropped. A page that fell
behind does not know what it missed, so it draws every cell again, and says so:

```rust,ignore
{{#include ../../../../crates/push/tests/push.rs:dropped}}
```

Three toggles, all off to start. **Run the market** writes the feed every three seconds, as in Part
V. **Push** registers the page's own listener and draws what it is told to. **Poll** asks every cell
again every two seconds, which is Part V's way, kept here to compare. Put `=FEED(acme)` in a cell
and something that reads it in another (the cells above did, if you ran them), `=NOW()` in a third,
then run the market with push on. A cell the page drew again because it was told is outlined until
the next one is. With push and polling both off, the grid is the copy the page drew when it loaded,
and an edit from the formula bar changes nothing on it: a listening sheet never draws itself again
because the page made the write. Whoever writes, the listener hears it, or nobody does.

<div class="sheet-play" data-clock data-push data-shell='urn:iki:tutorial:sheet:template:push'><p class="sheet-unavailable">The sheet appears here when the in-page kernel has loaded.</p></div>

With push on, the cells above may answer differently from their expected output. The page draws a
stale view as soon as it hears the cut, so a value a cell expects to find `not cached` has often been
computed again already. That is the other half of a golden-thread listener, which NetKernel also
had: recompute before the first reader, so the next reader is served from the cache.

## Polling and push, compared

Neither is free, and the costs fall in different places.

**Polling** costs a request per view per interval, whether or not anything changed. The sheet above
polls twenty-four views every two seconds: 720 requests a minute, nearly all of them a cache lookup
(over HTTP, a `304 Not Modified` and no body). It sees a change up to one interval late. It needs no
state anywhere but the cache, and it degrades gently: a missed poll is made up by the next.

**Push** costs a listener, a memory of what was drawn, and a timer, and nothing at all while nothing
changes. It sees a change on the next turn of the event loop. But the costs it does have are the kind
that grow:

* **State per reader.** A listener (a bounded queue) and a record of every view sent, with its threads
  and its deadline. The page keeps one. A server keeps one per session.
* **Fan-out.** A cut of a name a thousand readers drew is a thousand redraws, at once. A poll spreads
  the same load over the interval.
* **Redraws that change nothing.** A cut is by name, not by content, so a write of the value a feed
  already holds makes its readers stale anyway, and each is drawn again, the same. Stopping that needs
  the recomputed answer compared with the old one before anyone is told: early cutoff, which the
  kernel does not do (Part III). Ledger item 581's over-reach, above, is another source of the same.
* **Two triggers.** A cut is announced and an expiry is not, so push is a listener *and* a timer. A
  poll covers both without knowing either exists.
* **Recovery.** A dropped cut means "draw everything", and a server's page that loses its connection
  must do the same when it reconnects.

So polling is the right first answer for a few readers of values that change slowly and cache well,
and push is the answer for many readers of values that change often, or when a second of latency
matters. They also combine: push as the fast path, and a slow poll as the net under it.

## What a server needs

This page is its own host, so push is a function call. A sheet served over HTTP needs the same four
pieces, each a little further apart:

* **One listener per session**, registered under that session's capability, so that it hears only
  about names that session read.
* **The session's memory of what it sent**: for each view, the threads and the deadline its answer
  carried. Server state, held as long as the session is.
* **A channel the server can write to.** Server-sent events suit it, and htmx can listen to one: the
  cell's trigger becomes an event on that channel instead of a DOM event, and nothing else in the
  markup changes.
* **A timer per session**, at the earliest deadline among the views it sent, and a full redraw when
  cuts were dropped or the channel reconnects.

## What was built

| the set | the name | what it is |
|---|---|---|
| a cut listener | `urn:iki:tutorial:push:listener:{name}` | code: a registration held by name over the kernel's own queue, behind `urn:cap:kernel:listen` |
| what a read carries | `urn:iki:tutorial:push:carries` | code: one read, and what its answer carries, in words |
| the views drawn | the page's memory (`push::Drawn`) | code: per view, the threads and the deadline its answer carried; the stale views for a cut, and the earliest deadline |
| the listening page | `urn:iki:tutorial:sheet:template:push` | a template: each cell a view that listens for `stale` |

The kernel already had the hard parts: the dependency graph, recorded as every answer was computed and
handed out on the answer, and a queue that tells a host when a thread is cut. Push needed a place to
keep what the page drew, a rule to map a cut onto it, and a timer for the one kind of change nobody
announces. The sheet's code did not change, beyond one template, and its values do not know whether
anybody is looking.
