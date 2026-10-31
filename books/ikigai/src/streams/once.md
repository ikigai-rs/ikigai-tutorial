# Streams, III: a body read once, and its bound

[The second streams page](replay.md) ended on the body it could not do much with: `Body::Once`, a
one-shot stream, which the kernel would not cache and `urn:stream:range` would not slice. This
page is about that body. It covers what a one-shot stream is and why it is never cached, how one
is collected into a buffer under the run's **byte budget**, how one is decoded as text without
being collected, and the refusal past the bound. The decoding runs in this page's kernel. The page
has no one-shot source of its own, so the listings that make one run in `mdbook test`.

## A stream that can be read once

A one-shot body is a stream of chunks that ends when its producer stops: a generator, a response
still arriving, an upload in flight. Each chunk is handed to whoever is reading and then it is
gone. Nothing in the stream can start it again from the first byte, so the kernel never stores
one: the next caller would find it already read. The endpoint runs again for each request
instead, and a trace's uncached note gives the reason as `one-shot body`.

The listing below answers the shop's stocktake, counted as it is read, as a one-shot body of one
chunk per row. It counts how often the count is taken, and runs in `mdbook test`:

<!-- urn-gate: unbound urn:shop:count — bound by the listing below, in a space of its own. -->

```rust
# extern crate async_trait;
# extern crate futures;
# extern crate ikigai_core;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use futures::executor::block_on;
use futures::StreamExt;
use ikigai_core::{
    Body, Bytes, Capability, Endpoint, EndpointSpace, Exact, Invocation, Iri, Kernel, ReprType,
    Representation, Request, Result, TraceEvent, Tracer, Verb, UNCACHED_NOTE,
};

const ROWS: [&str; 5] = [
    "item,shelf,count\n",
    "flour,A,12\n",
    "sugar,A,0\n",
    "rice,B,30\n",
    "lentils,B,7\n",
];

/// The stocktake: one chunk per row as it is counted, counting the counts.
struct Count(Arc<AtomicUsize>);

#[async_trait]
impl Endpoint for Count {
    async fn invoke(&self, inv: &Invocation<'_>) -> Result<Representation> {
        let body = self.invoke_body(inv).await?;
        body.collect_within(inv.memory_budget().expect("the kernel mints one")).await
    }

    async fn invoke_body(&self, _inv: &Invocation<'_>) -> Result<Representation<Body>> {
        self.0.fetch_add(1, Ordering::SeqCst);
        let rows = ROWS.map(|row| Ok(Bytes::from_static(row.as_bytes())));
        let body = Body::once(futures::stream::iter(rows));
        Ok(Representation::from_body(ReprType::new("text/csv"), body).cacheable())
    }
}

let counts = Arc::new(AtomicUsize::new(0));
let space = EndpointSpace::new().bind(Exact::new("urn:shop:count"), Count(counts.clone()));
let kernel = Kernel::new(Arc::new(space));
let root = Capability::root();
let read = || Request::new(Verb::Source, Iri::parse("urn:shop:count").unwrap());

// The body API hands the stream over unread, with no length: nobody knows it yet.
let answer = block_on(kernel.issue_body(read(), &root)).unwrap();
assert!(answer.bytes.is_one_shot());
assert_eq!(answer.bytes.len_hint(), None);

// Reading it takes the chunks one at a time, five rows, and then it is spent.
let chunks: Vec<Bytes> = block_on(answer.bytes.into_stream().map(|c| c.unwrap()).collect());
assert_eq!(chunks.len(), 5);
assert_eq!(chunks.concat(), ROWS.concat().as_bytes());

// Marked cacheable and still never cached, so every request takes the count again.
assert!(!kernel.is_cached(&read(), &root));
let again = block_on(kernel.issue(read(), &root)).unwrap();
assert_eq!(again.bytes, ROWS.concat().as_bytes());
assert_eq!(counts.load(Ordering::SeqCst), 2);

// A trace of a third request says why it was not kept.
struct Notes(Mutex<Vec<TraceEvent>>);
impl Tracer for Notes {
    fn record(&self, event: TraceEvent) {
        self.0.lock().unwrap().push(event);
    }
}
let notes = Arc::new(Notes(Mutex::new(Vec::new())));
block_on(kernel.issue_traced(read(), &root, notes.clone())).unwrap();
let events = notes.0.lock().unwrap();
let event = events.iter().find(|e| e.target == "urn:shop:count").unwrap();
let why = event.notes.iter().find(|(k, _)| k == UNCACHED_NOTE).map(|(_, v)| v.as_str());
assert_eq!(why, Some("one-shot body"));
```

The second request used `Kernel::issue`, the bytes API every page before this part used. It
collected the five chunks into one buffer and handed back a `Representation` as before. It is
still not cached: the bytes were collected for this caller, after the kernel had decided not to
keep a stream it could not read twice. The third request was traced, and the trace's note for the
count says `one-shot body`.

## Collecting under the run's byte budget

Collecting is where a stream turns back into a buffer, and a buffer costs memory. A stream does
not say how long it is, so the kernel cannot check the size before it reads. It checks as it
reads instead. Every request a caller issues runs with a **byte budget**, one pool for the request
and every sub-request below it, and each chunk is drawn from the pool before it is held. The first
chunk the pool cannot cover ends the collect with `OverBudget`. The caller does not get the rows
that fit, and what the collect drew is given back.

The pool's size comes from the caller's capability, the way the retry budget's does:

- a host sets a **base** and a **ceiling** with `Kernel::with_stream_memory(base, ceiling)`, both
  64 MiB unless it says otherwise;
- a capability with no grant gets the base, and root gets the ceiling;
- a grant `urn:cap:stream:memory:<bytes>` (`cap_stream_memory(bytes)`) raises its holder's budget,
  never above the ceiling and never below the base. Attenuating a capability can only remove the
  grant, so it can only lower the budget.

The listing below runs the same stocktake on a host with a base of 32 bytes and a ceiling of 1 KiB.
The count is 60 bytes. A caller with no grant is refused, a caller granted 64 bytes is not, and
root is not:

<!-- urn-gate: unbound urn:shop:count — bound by the listing below, in a space of its own. -->

```rust
# extern crate async_trait;
# extern crate futures;
# extern crate ikigai_core;
# use std::sync::Arc;
# use async_trait::async_trait;
# use ikigai_core::{
#     Body, Bytes, Endpoint, EndpointSpace, Exact, Invocation, ReprType, Representation, Result,
# };
# const ROWS: [&str; 5] = [
#     "item,shelf,count\n",
#     "flour,A,12\n",
#     "sugar,A,0\n",
#     "rice,B,30\n",
#     "lentils,B,7\n",
# ];
# struct Count;
# #[async_trait]
# impl Endpoint for Count {
#     async fn invoke(&self, inv: &Invocation<'_>) -> Result<Representation> {
#         let body = self.invoke_body(inv).await?;
#         body.collect_within(inv.memory_budget().expect("the kernel mints one")).await
#     }
#     async fn invoke_body(&self, _inv: &Invocation<'_>) -> Result<Representation<Body>> {
#         let rows = ROWS.map(|row| Ok(Bytes::from_static(row.as_bytes())));
#         let body = Body::once(futures::stream::iter(rows));
#         Ok(Representation::from_body(ReprType::new("text/csv"), body).cacheable())
#     }
# }
use futures::executor::block_on;
use ikigai_core::{
    cap_stream_memory, Capability, Error, FailureClass, Iri, Kernel, MemoryLimits, Request, Verb,
};

let space = EndpointSpace::new().bind(Exact::new("urn:shop:count"), Count);
let kernel = Kernel::new(Arc::new(space)).with_stream_memory(32, 1024);
let read = || Request::new(Verb::Source, Iri::parse("urn:shop:count").unwrap());

// No grant: the base, 32 bytes. The third row does not fit, and the answer is the refusal.
let plain = Capability::scoped(["urn:cap:read"]);
let refused = block_on(kernel.issue(read(), &plain)).unwrap_err();
assert!(matches!(refused, Error::OverBudget { .. }));
assert_eq!(refused.class(), FailureClass::OverBudget);
assert!(refused.to_string().contains("4 byte(s) left, at least 38"));

// A grant of 64 bytes covers the 60, and root gets the ceiling.
let granted = Capability::scoped([cap_stream_memory(64)]);
assert_eq!(block_on(kernel.issue(read(), &granted)).unwrap().bytes.len(), 60);
assert_eq!(block_on(kernel.issue(read(), &Capability::root())).unwrap().bytes.len(), 60);

// The rule the kernel applies: a grant above the ceiling is clamped to it, so asking for
// more than the host allows is not a way past it.
let limits = MemoryLimits::new(32, 1024);
assert_eq!(limits.for_capability(&plain), 32);
assert_eq!(limits.for_capability(&Capability::scoped([cap_stream_memory(u64::MAX)])), 1024);
```

The refusal says `over budget`, that 4 bytes were left and at least 38 were needed, and that a
`urn:cap:stream:memory:<bytes>` grant raises the budget up to the host's ceiling. The header and
flour's row are 28 bytes, so four of the 32 were left when sugar's 10-byte row arrived. The collect refused at the first chunk that did not fit, read nothing after it, and did
not hand back the 28 bytes it had. `OverBudget` is a failure class of its own, and nothing in the
kernel retries it: the same request under the same budget fails the same way. Only a caller with a
larger grant, making a new request, gets a different answer.

The budget belongs to the run, not to one collect. Two endpoints in the same request that each
collect a stream draw from the same pool, so a request cannot get past its budget by splitting its
reads. A body that is already a buffer, such as the shop's stock in this book's page kernel, draws
nothing: it is in memory already, and refusing it would free nothing.

## Decoding a body as text

`urn:stream:text` is the kernel's text decoder. It reads the body given as `content`, checks that
it is UTF-8, and answers it as `text/plain;charset=utf-8`. The charset comes from the `charset`
argument if there is one, else from the body's own media type, else it is UTF-8. UTF-8 is the only
charset it decodes. Any other is refused by name rather than guessed at.

The shop's stock is UTF-8, so the first line below answers it as text and the second is a cache
hit. The third gzips it first, as [the first streams page](bytes.md) did, and gzip's bytes are not
text. The fourth asks for a charset the decoder does not have:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:shop:stock | urn:stream:text
source urn:iki:tutorial:shop:stock | urn:stream:text
source urn:iki:tutorial:shop:stock | urn:compress:gzip | urn:stream:text
source urn:iki:tutorial:shop:stock | urn:stream:text charset=iso-8859-1'>
<pre class="ikigai-run-expected">item,shelf,count
flour,A,12
sugar,A,0
rice,B,30
lentils,B,7
[computed]
item,shelf,count
flour,A,12
sugar,A,0
rice,B,30
lentils,B,7
[cached]
error: invalid argument `content`: the body is not valid UTF-8 (at byte 1)
       `content`: the body to decode
error: unsupported: urn:stream:text decodes UTF-8 only; the body's charset is `iso-8859-1`</pre>
</div>

The gzip refusal names the offset of the first byte that is not UTF-8. A gzip stream starts with
the bytes `1F 8B`. The first is a control character, which UTF-8 allows. The second can only
continue a character, never start one, so the decoder stops at byte 1. A body that is not valid
text is refused whole. The decoder does not drop the bad bytes or replace them with `�`, because
the text it would answer is not what the body says.

A buffer is checked whole. A stream is decoded **incrementally**: each chunk is checked as it
arrives and passed on, and the stream is never collected. A UTF-8 character can be split across
two chunks, and the decoder holds the first part of it, at most three bytes, until the next chunk
completes it. Whatever is invalid ends the stream with an error, after the text before it was
passed on. So a reader of a decoded stream has the same duty as a reader of any stream: a stream
that ends with an error is a failure, and the chunks before it are not the whole.

The listing below decodes a count that sells jalapeños, with the `ñ` split across two chunks, and
then a count whose `ñ` was written in Latin-1, as the one byte `F1`, which UTF-8 never uses alone:

<!-- urn-gate: unbound urn:shop:jalapenos — bound by the listing below, in a space of its own. -->
<!-- urn-gate: unbound urn:shop:latin1 — bound by the listing below, in a space of its own. -->

```rust
# extern crate async_trait;
# extern crate futures;
# extern crate ikigai_core;
use std::sync::Arc;

use async_trait::async_trait;
use futures::executor::block_on;
use futures::StreamExt;
use ikigai_core::{
    ArgRef, Body, Bytes, Capability, Endpoint, EndpointSpace, Error, Exact, Invocation, Iri,
    Kernel, ReprType, Representation, Request, Verb,
};

/// A one-shot `text/csv` body over these chunks.
struct Chunks(&'static [&'static [u8]]);

#[async_trait]
impl Endpoint for Chunks {
    async fn invoke(&self, inv: &Invocation<'_>) -> ikigai_core::Result<Representation> {
        let body = self.invoke_body(inv).await?;
        body.collect_within(inv.memory_budget().expect("the kernel mints one")).await
    }

    async fn invoke_body(&self, _: &Invocation<'_>) -> ikigai_core::Result<Representation<Body>> {
        let chunks = self.0.iter().map(|chunk| Ok(Bytes::from_static(chunk)));
        let body = Body::once(futures::stream::iter(chunks.collect::<Vec<_>>()));
        Ok(Representation::from_body(ReprType::new("text/csv"), body))
    }
}

// "jalapeños" in UTF-8 is `jalape` C3 B1 `os`: the first chunk ends after C3.
let split = Chunks(&[b"item,shelf,count\njalape\xC3", b"\xB1os,C,4\n"]);
// Latin-1 writes the same letter as the one byte F1.
let latin1 = Chunks(&[b"item,shelf,count\n", b"jalape\xF1os,C,4\n"]);
let kernel = Kernel::new(Arc::new(
    EndpointSpace::new()
        .bind(Exact::new("urn:shop:jalapenos"), split)
        .bind(Exact::new("urn:shop:latin1"), latin1),
));
let root = Capability::root();
let text = |of: &str| {
    Request::new(Verb::Source, Iri::parse("urn:stream:text").unwrap())
        .with_arg("content", ArgRef::Reference(Iri::parse(of).unwrap()))
};

// Decoded as a stream: still one-shot, never collected, and the split letter arrives whole.
let decoded = block_on(kernel.issue_body(text("urn:shop:jalapenos"), &root)).unwrap();
assert_eq!(decoded.repr_type.media_type, "text/plain");
assert!(decoded.bytes.is_one_shot());
let chunks: Vec<Bytes> = block_on(decoded.bytes.into_stream().map(|c| c.unwrap()).collect());
assert_eq!(chunks.len(), 2);
assert!(chunks[0].ends_with(b"jalape"));
assert!(chunks[1].starts_with("ñ".as_bytes()));

// The Latin-1 count: the header passes, then the stream ends with the refusal, not a short text.
let decoded = block_on(kernel.issue_body(text("urn:shop:latin1"), &root)).unwrap();
let read: Vec<Result<Bytes, Error>> = block_on(decoded.bytes.into_stream().collect());
assert_eq!(read.len(), 2);
assert_eq!(read[0].as_ref().unwrap(), &b"item,shelf,count\n"[..]);
let refused = read[1].as_ref().unwrap_err();
assert_eq!(
    refused.to_string(),
    "invalid argument `content`: the body is not valid UTF-8 (at byte 23)"
);

// Collected through the bytes API, the same refusal is the whole answer.
assert!(matches!(
    block_on(kernel.issue(text("urn:shop:latin1"), &root)),
    Err(Error::InvalidArgument { .. })
));
```

The refusal names byte 23, counted from the start of the body: the header's 17 bytes, then
`jalape`, then the `F1`. The header had already been passed on when the refusal arrived, which is
why a reader must not take what came before an error as the whole.

## What a one-shot body still cannot do

A one-shot body can now be collected, under a bound that refuses rather than truncates, and
decoded as it streams. It still cannot be read again, so it still cannot be cached, sliced with
`urn:stream:range`, or handed to two readers. The kernel's answer to that is
`urn:stream:spill`, which writes a one-shot body through to the disk as it arrives, under a disk
budget of its own, and answers a body that can be read again. That, and serving a range of a
resource over HTTP, are what a later page in this part takes up (ledger #2228).
