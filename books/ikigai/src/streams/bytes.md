# Streams, I: bytes are a representation

Every answer in this book so far has been text: a title, a CSV table, Turtle, an SVG picture.
That is a habit of the examples, not a rule of the kernel. A representation is two things, a
**media type** and **bytes**, and nothing requires the bytes to be text. This part of the book is
about bytes: how they travel, how big they get, and what a representation can and cannot do with
them. This first page needs only what the kernel already has, and its cells run in this page.

## Two fields

`ikigai-core`'s `Representation` holds a `repr_type`, whose `media_type` says what the bytes are
(`text/csv`, `image/svg+xml`, `application/gzip`), and `bytes`, a `Vec<u8>`. It also carries an
expiry and its golden threads, which is how the cache treats it, but those say how long the
answer is good for, not what it is. The kernel does not look inside `bytes`. It caches them,
passes them through a pipe, and hands them to the next endpoint. Whether they are UTF-8 matters
only to whoever reads them.

So binary is ordinary. A gzip stream, a PNG or a PDF is a representation exactly as the shop's
stock is, and resolving one is the same `Source`.

## Bytes that are not text

The page's kernel links [`ikigai-compress`](../repositories.md), which binds gzip and zlib as
resources: `urn:compress:gzip` compresses whatever is piped into it and answers
`application/gzip`, and `urn:decompress:gzip` reads it back. The shop's stock from
[Transreption](../building/transreption.md#a-format-that-is-not-rdf) is the input, first as it
is and then compressed:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:shop:stock
source urn:iki:tutorial:shop:stock | urn:compress:gzip'>
<pre class="ikigai-run-expected">item,shelf,count
flour,A,12
sugar,A,0
rice,B,30
lentils,B,7
[computed]
error: output is not UTF-8 text (83 bytes) — pipe it into a `sink` to store the bytes</pre>
</div>

The second line resolved. The refusal is the cell's, not the kernel's: a cell, like the REPL,
prints text, and 83 bytes of gzip are not text, so it names how many there are instead of
printing them. The compressed table is also larger than the 60 bytes it came from. gzip's header
and trailer cost 18 bytes, and a table this small has too little repetition to win them back.

Those 83 bytes were computed, and cached, like any other answer. Pipe them on to
`urn:decompress:gzip` and the table comes back. A gzip stream records no media type for what it
holds, so the line says what it is with `as=`:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:shop:stock | urn:compress:gzip | urn:decompress:gzip as=text/csv'>
<pre class="ikigai-run-expected">item,shelf,count
flour,A,12
sugar,A,0
rice,B,30
lentils,B,7
[2 cached · 1 computed]</pre>
</div>

Two of the three stages were cached: the stock, and the gzip stream the cell above could not
print. That is only possible because `ikigai-compress` writes the same bytes for the same input
every time. gzip's header has a timestamp and an operating-system byte, and the crate fixes both,
so a cached stream is the stream a second compression would have made. Without `as=` the bytes
come back labeled `application/octet-stream`, which means "bytes, of a type nobody stated". The
label is the caller's claim: `as=` says what the bytes are, and nothing checks it.

## Transrepted by type

[Transreption](../building/transreption.md) described a transreptor as an endpoint that declares
what it converts between, so the kernel can plan a route through it without anyone naming it.
The four gzip and zlib endpoints declare exactly that: `urn:decompress:gzip` converts
`application/gzip` to the payload's types, losslessly. The Rust below runs in `mdbook test`. It
compresses the same table natively, looks at the bytes, and asks the kernel for a route out of
`application/gzip`:

```rust
# extern crate futures;
# extern crate ikigai_compress;
# extern crate ikigai_core;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request, Verb};

let kernel = Kernel::new(Arc::new(ikigai_compress::space()));
let root = Capability::root();
let stock = "item,shelf,count\nflour,A,12\nsugar,A,0\nrice,B,30\nlentils,B,7\n";
let call = |iri: &str, args: &[(&str, &[u8])]| {
    let mut request = Request::new(Verb::Source, Iri::parse(iri).unwrap());
    for (name, value) in args {
        request = request.with_arg(*name, ArgRef::Inline(value.to_vec()));
    }
    block_on(kernel.issue(request, &root))
};

// Compressed: a media type, and 83 bytes that start with gzip's magic number.
let gz = call("urn:compress:gzip", &[("content", stock.as_bytes())]).unwrap();
assert_eq!(gz.repr_type.media_type, "application/gzip");
assert_eq!(gz.bytes.len(), 83);
assert_eq!(gz.bytes[..2], [0x1f, 0x8b]);
assert!(String::from_utf8(gz.bytes.clone()).is_err());

// The same input, the same bytes: what makes the stream safe to cache.
let again = call("urn:compress:gzip", &[("content", stock.as_bytes())]).unwrap();
assert_eq!(again.bytes, gz.bytes);

// Read back: the bytes are the table, and the label is whatever the caller says.
let back = call("urn:decompress:gzip", &[("content", &gz.bytes)]).unwrap();
assert_eq!(back.repr_type.media_type, "application/octet-stream");
assert_eq!(back.bytes, stock.as_bytes());
let csv = call("urn:decompress:gzip", &[("content", &gz.bytes), ("as", b"text/csv")]).unwrap();
assert_eq!(csv.repr_type.media_type, "text/csv");

// And a route out of gzip that nobody named: the planner found it by its declaration.
let plan = kernel.select_transreptor("application/gzip", "text/csv").unwrap();
assert_eq!(plan.len(), 1);
assert_eq!(plan[0].endpoint, "urn:decompress:gzip");
assert!(plan[0].lossless);
```

## The bound refuses

A few hundred bytes of gzip can expand to gigabytes, and a decompressor is handed bytes from
somewhere else. So `urn:decompress:gzip` takes `max-bytes`, 64 MiB by default, which a caller may
lower, or raise to at most 1 GiB. Past the bound, it refuses. It does not return the first `max-bytes`
of the payload, because a truncated CSV table, or a truncated Turtle document, is often still
well formed, and nothing downstream could tell it from the real one. The table is 60 bytes, so a
bound of 59 refuses, and 60 is enough:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:shop:stock | urn:compress:gzip | urn:decompress:gzip as=text/csv max-bytes=59
source urn:iki:tutorial:shop:stock | urn:compress:gzip | urn:decompress:gzip as=text/csv max-bytes=60'>
<pre class="ikigai-run-expected">error: invalid argument `content`: decompressed output exceeds the 59-byte limit set by `max-bytes`; refusing rather than returning a truncated representation (raise `max-bytes`, up to 1073741824)
       `content`: the gzip stream to expand — arbitrary bytes, not necessarily UTF-8; filled from the pipe
item,shelf,count
flour,A,12
sugar,A,0
rice,B,30
lentils,B,7
[2 cached · 1 computed]</pre>
</div>

The refusal names the limit, and the bound that refused is the one the line set. The decompressor
stops reading one byte past `max-bytes`, so refusing a bomb costs one byte over the limit, not the
whole expansion. The same listing natively:

```rust
# extern crate futures;
# extern crate ikigai_compress;
# extern crate ikigai_core;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Kernel, Request, Verb};

let kernel = Kernel::new(Arc::new(ikigai_compress::space()));
let root = Capability::root();
let stock = "item,shelf,count\nflour,A,12\nsugar,A,0\nrice,B,30\nlentils,B,7\n";
let issue = |request: Request| block_on(kernel.issue(request, &root));
let gzip = Request::new(Verb::Source, Iri::parse("urn:compress:gzip").unwrap())
    .with_arg("content", ArgRef::Inline(stock.as_bytes().to_vec()));
let gz = issue(gzip).unwrap().bytes;
let gunzip = |max: &str| {
    Request::new(Verb::Source, Iri::parse("urn:decompress:gzip").unwrap())
        .with_arg("content", ArgRef::Inline(gz.clone()))
        .with_arg("max-bytes", ArgRef::Inline(max.as_bytes().to_vec()))
};

let refused = issue(gunzip("59")).unwrap_err();
assert!(matches!(refused, Error::InvalidArgument { .. }));
assert!(refused.to_string().contains("exceeds the 59-byte limit"));
assert_eq!(issue(gunzip("60")).unwrap().bytes, stock.as_bytes());
```

## What a representation cannot do yet

Everything above worked because every representation on this page is small. Look at what the
two fields cannot say:

- **It is one buffer.** `bytes` is a `Vec<u8>`: the whole answer, in memory, before anyone can
  read the first byte. The decompressor has to hold all of its output before it answers, which is
  one reason its bound exists.
- **It has no beginning before its end.** The first line of the table is ready as soon as it is
  decompressed, but nothing can be handed on until the last line is. A pipe of three stages holds
  three whole buffers.
- **The bound is the only limit.** Anything bigger than memory, or bigger than a sensible
  `max-bytes`, cannot be a representation at all, and a stream with no end, such as a log that is
  still being written, never finishes being one.

That is where this part goes next: a representation whose bytes arrive in pieces, so an answer
can start before it ends and a large one never has to be held whole (ledger #2228). It is not in
the kernel yet, and the pages that follow will say what it adds as it arrives.
