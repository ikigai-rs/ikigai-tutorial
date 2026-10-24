# Golden threads and the cost of a read

The Web caches by URL: a resource carries a time-to-live or a validator, and a client re-fetches
when the clock says so or the validator changes. That is a guess about staleness, made per
resource, in advance. ikigai does something exact instead. The kernel tracks **what each answer was
made from** — its **golden threads** — and when something an answer depended on changes, it cuts
precisely the threads that lead to it, and nothing else. This chapter is the system's accounting:
caching, golden threads, and why "add a source to this resource" is a line in the ledger, not a
free gesture.

## Watch a thread cut and a view recompute

File two items, with priorities, so the ledger's ranked view has something to rank:

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:append priority=0 Translate the research note'>
<pre class="ikigai-run-expected">#1 urn:iki:ledger:default:item:…
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:append priority=1 Wire the live cells'>
<pre class="ikigai-run-expected">#2 urn:iki:ledger:default:item:…
[uncacheable]</pre>
</div>

Ask for `next`, the ranked ready-set. The first read is **computed** — the kernel did the work:

<div class="ikigai-run" data-cmd='source urn:iki:ledger:next'>
<pre class="ikigai-run-expected"> 1.    #1  open    p0  Translate the research note
    p0 — priority 0; last updated …
 2.    #2  open    p1  Wire the live cells
    p1 — priority 1; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 0
[computed]</pre>
</div>

Ask again, unchanged. The second read is **cached** — nearly free, because `next` is a pure
function of the ledger graph and nothing the graph depends on has moved:

<div class="ikigai-run" data-cmd='source urn:iki:ledger:next'>
<pre class="ikigai-run-expected"> 1.    #1  open    p0  Translate the research note
    p0 — priority 0; last updated …
 2.    #2  open    p1  Wire the live cells
    p1 — priority 1; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 0
[cached]</pre>
</div>

Now **close** item #2. A write is uncacheable by nature, and — this is the whole point — it cuts the
golden threads that led to `next`:

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:close item=2 Wired.'>
<pre class="ikigai-run-expected">closed #2 (done)
[uncacheable]</pre>
</div>

Ask for `next` one more time. It is **computed** again, and #2 is gone from the ready-set — and
nobody told `next` to refresh:

<div class="ikigai-run" data-cmd='source urn:iki:ledger:next'>
<pre class="ikigai-run-expected"> 1.    #1  open    p0  Translate the research note
    p0 — priority 0; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 0
[computed]</pre>
</div>

That is the mechanism in six cells. `next` never subscribed to anything, never polled, and holds no
refresh logic. It declared, by being computed from the graph, that it depends on the graph; closing
an item wrote the graph; the kernel cut exactly the threads that reached `next` and recomputed it on
the next read. A view that depended on something the close did *not* touch would still be served
from cache. This is cache invalidation that is neither a guess nor a manual chore — it is a
consequence of naming dependencies honestly.

## Cacheability is only as good as the least-cacheable dependency

Here is the trap, and it is the one thing from this chapter to carry away. Effective cacheability
**propagates from dependencies**. A resource is cacheable only if everything it is made from is;
join one uncacheable source into an otherwise-cached resource and the cached resource **silently
stops being cached** — every read recomputes, forever. Nothing in the types changes and every test
can still pass; the only signal is that reads got slower.

So **adding a source to a cached resource is a performance change even when it is a correctness
no-op.** The remedy is twofold: the kernel will *tell* you why an answer was not cached (it names
the uncacheable dependency rather than leaving you to guess), and the discipline is to join mutable
data into a resource that is *already* uncacheable, not into the expensive cached one. Measure a
hot read before and after you add a source to it; that difference is the only thing that will warn
you.

Relatedly: a **refusal** is never cached. A resource that refuses bad input by erroring makes
everything that reads it recompute on every read. Where "this input is wrong" is a legitimate
answer, return it as a *value* the caller can see, not as an error — so the cache can keep it.

## The cache is also a confidentiality boundary

The quietest crossover in the whole book is here. The cache keys on the **capability fingerprint** —
the principal the answer was computed for. So the index that makes repetition cheap is *the same
index* that keeps one person's answer from ever being served to another: there is no cache entry
reachable across principals. Performance and isolation are one mechanism. You cannot accidentally
leak a cached answer across a tenancy boundary, because a cached answer computed under one authority
is not addressable under a different one.

## The throughput reading of the governors

The last chapter framed the overlays as *governance* — constraints made explicit and enforceable.
Caching and golden threads are the **throughput** half of the same theory-of-constraints picture.
The bottleneck in a resolution is the expensive computation; the cache **exploits** it by reclaiming
a prior result at near-zero cost, the golden thread keeps that exploitation honest by cutting the
cache exactly when it must, and the scheduler **subordinates** everything else to the constraint by
parking work rather than blocking on it. One drum-buffer-rope picture, drawn once: the governors
make the constraint explicit, and the cache spends it wisely.
