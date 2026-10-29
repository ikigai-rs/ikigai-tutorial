# Streams, II: a body you can read again

[The first streams page](bytes.md) ended on what a representation could not do: its `bytes` were
one buffer, held whole before anyone read the first byte. The kernel now has the start of an
answer, and this page covers the part of it that can be read more than once: a **body**, which may
be a buffer or something that opens a fresh reader each time it is asked, and a **byte range**,
which reads part of a resource without reading the rest. The ranges run in this page's kernel. Files do not, because the page has no file system,
so the listings that read a file run in `mdbook test`.

## Three shapes

`ikigai-core`'s `Body` is what a `Representation<Body>` carries where a `Representation` carries a
`Vec<u8>`. It has three shapes, and an endpoint answers with the weakest one it can honor:

- **`Body::Bytes`**: a buffer, as before, behind a shared handle, so a cache hit hands it out
  without copying it.
- **`Body::Replay`**: something that opens a fresh reader from the first byte every time it is
  asked. A file is one, and so is an object in a store. It may also open a reader at an offset,
  and it may know its length before anyone reads it.
- **`Body::Once`**: a one-shot stream, such as a generator, a live source or an upload in flight.
  It can be read once.

Two questions are kept apart. The expiry says how long the bytes are *true*, and the body's shape
says whether they can be *read again*. The kernel caches a `Bytes` or a `Replay` answer exactly as
it caches bytes, and never caches a `Once` answer: what it keeps has to be readable by the next
caller, and a one-shot stream has been read by the first.

An endpoint written before bodies existed still answers a `Vec<u8>`, and the kernel hands it on as
`Body::Bytes`. An endpoint that has a body to offer implements `invoke_body` beside `invoke`, and a
caller that wants the body rather than a buffer calls `Kernel::issue_body`. `Kernel::issue`, which
every page so far has used, collects whatever body comes back into one buffer, under the run's
byte budget.

## A body of your own

`Replay` is a trait with one required method, `open`, and three optional ones: `open_at`, for a
reader that starts at an offset; `len_hint`, the length when it is known before reading; and
`validator`, the content id of exactly these bytes when the producer knows it. The listing below
answers the shop's stock as a `Replay` that counts how often it is opened, and runs in
`mdbook test`:

<!-- urn-gate: unbound urn:shop:stock — bound by the listing below, in a space of its own. -->

```rust
# extern crate async_trait;
# extern crate futures;
# extern crate ikigai_core;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use futures::executor::block_on;
use futures::StreamExt;
use ikigai_core::{
    Body, ByteStream, Bytes, Capability, Endpoint, EndpointSpace, Exact, Invocation, Iri, Kernel,
    Replay, ReprType, Representation, Request, Result, Verb,
};

const STOCK: &[u8] = b"item,shelf,count\nflour,A,12\nsugar,A,0\nrice,B,30\nlentils,B,7\n";

/// The stock, read again on every open, counting the opens.
struct Shelf(Arc<AtomicUsize>);

impl Replay for Shelf {
    fn open(&self) -> ByteStream {
        self.0.fetch_add(1, Ordering::SeqCst);
        // Two chunks, as a file read in pieces would yield them.
        let (head, rest) = STOCK.split_at(17);
        let chunks = [head, rest].map(|chunk| Ok(Bytes::from_static(chunk)));
        Box::pin(futures::stream::iter(chunks))
    }

    fn len_hint(&self) -> Option<u64> {
        Some(STOCK.len() as u64)
    }
}

struct Stock(Arc<AtomicUsize>);

#[async_trait]
impl Endpoint for Stock {
    async fn invoke(&self, inv: &Invocation<'_>) -> Result<Representation> {
        let body = self.invoke_body(inv).await?;
        body.collect_within(inv.memory_budget().expect("the kernel mints one")).await
    }

    async fn invoke_body(&self, _inv: &Invocation<'_>) -> Result<Representation<Body>> {
        let body = Body::replay(Shelf(self.0.clone()));
        Ok(Representation::from_body(ReprType::new("text/csv"), body).cacheable())
    }
}

let opens = Arc::new(AtomicUsize::new(0));
let space = EndpointSpace::new().bind(Exact::new("urn:shop:stock"), Stock(opens.clone()));
let kernel = Kernel::new(Arc::new(space));
let root = Capability::root();
let read = || Request::new(Verb::Source, Iri::parse("urn:shop:stock").unwrap());

// The body API hands the replay over unread: nothing has opened it yet.
let answer = block_on(kernel.issue_body(read(), &root)).unwrap();
assert!(matches!(answer.bytes, Body::Replay(_)));
assert_eq!(answer.bytes.len_hint(), Some(60));
assert_eq!(opens.load(Ordering::SeqCst), 0);

// Every open is a fresh reader from the first byte, so it can be read twice.
let Body::Replay(replay) = answer.bytes else { unreachable!() };
let read_all = |stream: ByteStream| {
    block_on(stream.map(|chunk| chunk.unwrap()).collect::<Vec<_>>()).concat()
};
assert_eq!(read_all(replay.open()), STOCK);
assert_eq!(read_all(replay.open()), STOCK);
assert_eq!(opens.load(Ordering::SeqCst), 2);

// It was cached as a replay: the second request is a hit that opens nothing new.
assert!(kernel.is_cached(&read(), &root));
let again = block_on(kernel.issue_body(read(), &root)).unwrap();
assert!(matches!(again.bytes, Body::Replay(_)));
assert_eq!(opens.load(Ordering::SeqCst), 2);

// The bytes API collects it, one more open.
assert_eq!(block_on(kernel.issue(read(), &root)).unwrap().bytes, STOCK);
assert_eq!(opens.load(Ordering::SeqCst), 3);
```

The kernel did not read the body to cache it. It stored the replay, as it would have stored a
buffer, and the cached answer opens a fresh reader for whoever asks. Nothing in the kernel holds
the 60 bytes unless a caller collects them.

A one-shot body is the other case. The same endpoint answering `Body::once` over the same two
chunks is never cached:

<!-- urn-gate: unbound urn:shop:live — bound by the listing below, in a space of its own. -->

```rust
# extern crate async_trait;
# extern crate futures;
# extern crate ikigai_core;
use std::sync::Arc;

use async_trait::async_trait;
use futures::executor::block_on;
use ikigai_core::{
    Body, Bytes, Capability, Endpoint, EndpointSpace, Exact, Invocation, Iri, Kernel, ReprType,
    Representation, Request, Result, Verb,
};

struct Live;

#[async_trait]
impl Endpoint for Live {
    async fn invoke(&self, inv: &Invocation<'_>) -> Result<Representation> {
        let body = self.invoke_body(inv).await?;
        body.collect_within(inv.memory_budget().expect("the kernel mints one")).await
    }

    async fn invoke_body(&self, _inv: &Invocation<'_>) -> Result<Representation<Body>> {
        let chunks = [&b"item,shelf,count\n"[..], b"flour,A,12\n"]
            .map(|chunk| Ok(Bytes::from_static(chunk)));
        let body = Body::once(futures::stream::iter(chunks));
        Ok(Representation::from_body(ReprType::new("text/csv"), body).cacheable())
    }
}

let kernel = Kernel::new(Arc::new(EndpointSpace::new().bind(Exact::new("urn:shop:live"), Live)));
let root = Capability::root();
let read = || Request::new(Verb::Source, Iri::parse("urn:shop:live").unwrap());

let answer = block_on(kernel.issue_body(read(), &root)).unwrap();
assert!(answer.bytes.is_one_shot());
// Marked cacheable, and still not cached: a one-shot body cannot be read by the next caller.
assert!(!kernel.is_cached(&read(), &root));
```

## A range of a resource

A body that can open a reader at an offset can be read in part. The kernel answers
`urn:stream:range` itself, as it answers `urn:kernel:*`, so no module can bind it and every
resource can be asked for one: `of` names the resource, `start` is the first byte's offset, and
`end`, which may be left out, is the offset just past the last. The stock's header is 17 bytes and
its first row is 11, so its first row is bytes 17 to 28. The second line asks again:

<div class="ikigai-run" data-cmd='source urn:stream:range of=urn:iki:tutorial:shop:stock start=17 end=28
source urn:stream:range of=urn:iki:tutorial:shop:stock start=17 end=28'>
<pre class="ikigai-run-expected">flour,A,12
[computed]
flour,A,12
[cached]</pre>
</div>

The range read the stock as a sub-request, under the caller's capability, and kept its media type.
It also hangs from the stock's threads, so a write to the stock cuts every cached range of it.
Bytes 38 to 48 are rice's row. The second line below writes the stock without its empty shelf,
as [Transreption](../building/transreption.md) did, and the third asks for the same bytes again:

<div class="ikigai-run" data-cmd='source urn:stream:range of=urn:iki:tutorial:shop:stock start=38 end=48
source urn:json:query src=urn:iki:tutorial:shop:stock q="map(select(.count | tonumber > 0))" | urn:json:to-csv | sink urn:iki:tutorial:shop:stock
source urn:stream:range of=urn:iki:tutorial:shop:stock start=38 end=48'>
<pre class="ikigai-run-expected">rice,B,30
[computed]
ok
[2 computed · 1 uncacheable]
0
lentils
[computed]</pre>
</div>

The write cut the cached range, so the third line was computed, from the new table. Its answer is
not rice's row, though, and not a whole row at all. Sugar's row is gone, and `urn:json:to-csv`
wrote the table with CRLF line ends, as Transreption noted, so every line left is one byte longer
than it was. Bytes 38 to 48 are now the last digit of rice's count, the `\r\n` after it, and the
first seven bytes of lentils' row. A range is offsets into bytes. It knows nothing about rows, lines or characters, and an offset taken
from one version of a resource says nothing about the next.

The table is 54 bytes now. A range that starts at or past its end is refused, which is HTTP's
`416`. One that ends past it reads to the end, so the last row, which starts at byte 41, is
`start=41` with any `end` beyond 54. And one that ends before it starts is refused before
anything is read:

<div class="ikigai-run" data-cmd='source urn:stream:range of=urn:iki:tutorial:shop:stock start=54
source urn:stream:range of=urn:iki:tutorial:shop:stock start=41 end=1000
source urn:stream:range of=urn:iki:tutorial:shop:stock start=20 end=10'>
<pre class="ikigai-run-expected">error: invalid argument `start`: the range starts at 54, past the end of `urn:iki:tutorial:shop:stock` (54 bytes)
       `start`: the first byte's offset
lentils,B,7
[computed]
error: invalid argument `end`: the range ends before it starts (20..10)
       `end`: the offset just past the last byte (exclusive)</pre>
</div>

The stock in this page is a buffer, so slicing it copies nothing and reads nothing. A `Replay`
is read from `start` through `open_at`, and only as far as `end`: a range of a large file reads the
range, not the file. A body that cannot open at an offset, a one-shot stream or a replay whose
`open_at` answers `None`, is refused as `Unsupported`. `urn:stream:range` does not collect a whole
stream in order to slice it, because that is the cost it exists to avoid.

## A file is a seekable body

On a native host, [`ikigai-fs`](../repositories.md)'s `Source` answers a file as a `Replay`. Each
reader opens a fresh descriptor through the same jailed walk the bytes path takes, reads chunks of
at most 64 KiB, and can start at any offset, and the replay knows the file's length from the moment
it was opened. A file read this way is not limited to the 16 MiB its bytes path holds; whoever
collects it is bounded by their run's byte budget instead.

The listing writes the stock to a file in a fresh directory, mounts the directory with
`cacheable_space`, and reads it the three ways:

```rust
# extern crate futures;
# extern crate ikigai_core;
# extern crate ikigai_fs;
use std::sync::Arc;

use futures::executor::block_on;
use futures::StreamExt;
use ikigai_core::{ArgRef, Body, Capability, Error, Iri, Kernel, Request, Verb};

let dir = std::env::temp_dir().join(format!("ikigai-book-streams-{}", std::process::id()));
std::fs::create_dir_all(&dir).unwrap();
let stock = "item,shelf,count\nflour,A,12\nsugar,A,0\nrice,B,30\nlentils,B,7\n";
std::fs::write(dir.join("stock.csv"), stock).unwrap();

let kernel = Kernel::new(Arc::new(ikigai_fs::cacheable_space(&dir)));
let root = Capability::root();
let file = || Request::new(Verb::Source, Iri::parse("urn:file:stock.csv").unwrap());
let range = |start: &str, end: &str| {
    Request::new(Verb::Source, Iri::parse("urn:stream:range").unwrap())
        .with_arg("of", ArgRef::Reference(Iri::parse("urn:file:stock.csv").unwrap()))
        .with_arg("start", ArgRef::Inline(start.as_bytes().to_vec()))
        .with_arg("end", ArgRef::Inline(end.as_bytes().to_vec()))
};
let read_all = |stream: ikigai_core::ByteStream| {
    block_on(stream.collect::<Vec<_>>())
        .into_iter()
        .collect::<Result<Vec<_>, Error>>()
        .map(|chunks| chunks.concat())
};

// A file is a replay: its length known before a byte is read, read from the start as
// often as asked, and from any offset. ikigai-fs has no type for `.csv`, so a file of
// UTF-8 is `text/plain`.
let answer = block_on(kernel.issue_body(file(), &root)).unwrap();
assert_eq!(answer.repr_type.media_type, "text/plain");
let Body::Replay(replay) = answer.bytes else { panic!("a file is a replay") };
assert_eq!(replay.len_hint(), Some(60));
assert_eq!(read_all(replay.open()).unwrap(), stock.as_bytes());
assert_eq!(read_all(replay.open()).unwrap(), stock.as_bytes());
assert_eq!(read_all(replay.open_at(48).unwrap()).unwrap(), b"lentils,B,7\n");

// A range of the file, read from its offset, and cached.
assert_eq!(block_on(kernel.issue(range("17", "28"), &root)).unwrap().bytes, b"flour,A,12\n");
assert!(kernel.is_cached(&range("17", "28"), &root));

// A write through the kernel cuts the file's thread, and every range that hangs from it.
let without_sugar = "item,shelf,count\nflour,A,12\nrice,B,30\nlentils,B,7\n";
let sink = Request::new(Verb::Sink, Iri::parse("urn:file:stock.csv").unwrap())
    .with_arg("content", ArgRef::Inline(without_sugar.as_bytes().to_vec()));
block_on(kernel.issue(sink, &root)).unwrap();
assert!(!kernel.is_cached(&range("17", "28"), &root));
assert_eq!(block_on(kernel.issue(range("28", "38"), &root)).unwrap().bytes, b"rice,B,30\n");

// A file changed in place behind a reader's back: the reader refuses, never a torn body.
let held = block_on(kernel.issue_body(file(), &root)).unwrap();
let Body::Replay(held) = held.bytes else { panic!("a file is a replay") };
std::fs::write(dir.join("stock.csv"), "item,shelf,count\n").unwrap();
assert!(matches!(read_all(held.open()), Err(Error::Conflict(_))));

std::fs::remove_dir_all(&dir).unwrap();
```

The last four lines are the promise a `Replay` makes. It records the file's version when it is
opened, its size, inode and times, and checks it when each reader opens and again at the end. A
file written in place while a reader is open ends that reader with `Conflict` ("changed while
read"), never with a short body or one that is half old and half new. A `Sink` through the kernel
does not do that to anyone: it writes a staging file and renames it over the old one, so a reader
already open keeps the old file and reads it whole, and the cut sends every later lookup to the
new one.

## What a body still cannot do

Everything on this page could be read again. A buffer can, a file can, and a range of either is a
buffer. What is left is the third shape, the body that can be read once: a response still
arriving, a generator, an upload. The kernel already refuses to cache one, and
`urn:stream:range` already refuses to slice one. Holding one as something that can be read again,
by writing it through to the disk as it arrives (`urn:stream:spill`), and serving a range of a
resource over HTTP, are what the next page in this part takes up (ledger #2228).
