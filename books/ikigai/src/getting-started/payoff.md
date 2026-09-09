# What resolution buys you

[Hello, resource](hello-resource.md) built an endpoint that camel-cases text, and if you
are honest about it, a function would have done the same job in fewer lines. This chapter
is where the difference shows up. Four things happen to a resolution that never happen to
a function call, and you run each of them.

Everything here resolves through the tutorial host — `hello_camel::kernel()`, a kernel
over this book's `space()` — which you have not built yet. [Binding, and a host of your
own](binding.md) is where you read it in full; for now it is the thing that answers.

Two more endpoints join `camel-case` for this chapter, because a pure function of its
arguments cannot show most of what follows. `title` is a string the host holds in memory,
readable and writable; `camel-title` is the camel-cased form of whatever `title` currently
says. Here is `title`:

```rust,ignore
{{#include ../../../../crates/hello-camel/src/lib.rs:title}}
```

Two things to notice before it runs. The `Source` arm says `.cacheable()` **and**
`.depends_on(TITLE)` — "cache this, and treat it as valid until the thread named
`urn:iki:tutorial:title` is cut." And the `Sink` arm cuts nothing: the kernel does that
itself, after any successful write, to the thread named after the write's target. The two
names are the same string on purpose.

And `camel-title`, which is the interesting one:

```rust,ignore
{{#include ../../../../crates/hello-camel/src/lib.rs:camel_title}}
```

It takes no argument. It *resolves* `title` — `inv.source(..)` — which is the same act a
caller performs from outside, made from inside an invocation. That one line is what makes
it a composite, and the rest of this chapter is the consequences of that line.

The four demonstrations are tests in `crates/hello-camel/tests/payoff.rs`. Each is included
below from the file that runs it. To see their output rather than take the book's word:

```bash
cargo test -p hello-camel --test payoff -- --nocapture --test-threads 1
```

Or press **Run**. Each section below ends in a cell: a command you can edit, and the
output the listing produces, shown as *expected* until you run it against **this book's
own kernel, in this page** — `hello_camel::kernel()` compiled to WebAssembly, under the
same engine the `ikigai` CLI uses, so the lines are the CLI's grammar and the answers are
real resolutions. Nothing runs until you press Run (or Enter in the command). One kernel
serves every cell on the page and keeps its cache and its golden threads between runs;
that is the point, so the order you run them in shows, running a cell twice answers
differently the second time, and every run stays under the cell so the two can be
compared. Edit a command and see what changes — a name the kernel does not bind answers
`error: no endpoint resolved`, which is the CLI's answer too. Nothing is answered in
advance: the listing's expected output sits behind a disclosure under each cell, and
stands in for the result only if the kernel did not load — the cell says so. Reset puts
the whole cell back the way the chapter shipped it.

## 1. Cached once

Resolve `title` twice through one kernel. The second answer is served without the
endpoint running — and *served* is the claim, so the test does not settle for the same
bytes twice (a recompute would give those too). It counts.

```rust,ignore
{{#include ../../../../crates/hello-camel/tests/payoff.rs:cached}}
```

```text
cached once: "resource oriented computing" served twice, endpoint ran 1 time
```

`kernel.is_cached(..)` is a probe, not a resolution: it answers "would this be served from
cache right now" without resolving anything, which is what lets a test ask the question
without changing the answer.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:title
source urn:iki:tutorial:title'>
<pre class="ikigai-run-expected">resource oriented computing
[computed]
resource oriented computing
[cached]</pre>
</div>

The bracketed word is the engine's verdict on each line: on your first run the first
resolution is computed and the second served. Press Run again and both say `[cached]` —
the history under the cell keeps both runs so you can see the change. Then edit the
name to one that is not bound and run that.

The cache is keyed on the request *and the capability* — a result computed under one
authority is never handed to a caller holding another. You will not feel that here, where
everything runs as root, but it is why `.cacheable()` is safe to say on an endpoint whose
answer depends on who is asking.

## 2. A golden thread, cut

This is the one the introduction promised. `camel-title` is cached after its first
resolution. Then `title` is written — and `camel-title`'s cache entry is gone, even though
`camel-title` never mentioned a thread.

```rust,ignore
{{#include ../../../../crates/hello-camel/tests/payoff.rs:cut}}
```

```text
thread cut: camel-title recomputed after a Sink to title
```

Follow the thread. `title`'s `Source` declared `.depends_on(TITLE)`. `camel-title` resolved
`title` through the kernel, and the kernel recorded that: a composite inherits the golden
threads of everything it resolved. The `Sink` to `title` succeeded, so the kernel cut the
thread named `urn:iki:tutorial:title`. Every cached representation depending on that thread
— directly, or transitively through composition — stopped being valid at that instant.

Nothing polled. No timeout expired. No one wrote an invalidation. The write invalidated
exactly what was derived from the thing written, because derivation was *recorded* rather
than guessed at — and the last assertion is the one to sit with: the recompute went all
the way down, reading `title` again rather than reusing a stale copy of it.

> ⚠ The thread's name is the resource's IRI, by convention and by the kernel's own choice:
> after a mutating verb it cuts the thread named after the *target*. An endpoint that
> declares `.depends_on("some-other-name")` is not wrong, but nothing will cut that thread
> unless something explicitly does — [The file workspace](file-workspace.md) has a
> watcher doing exactly that for files that change out from under the kernel.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:camel-title
cache urn:iki:tutorial:camel-title
sink urn:iki:tutorial:title golden threads cut
cache urn:iki:tutorial:camel-title
source urn:iki:tutorial:camel-title'>
<pre class="ikigai-run-expected">resourceOrientedComputing
[computed]
cached
ok
[uncacheable]
not cached
goldenThreadsCut
[computed]</pre>
</div>

`cache` is `is_cached` from the REPL — a probe. Between the two probes is one `Sink`, and
the composite went from `cached` to `not cached` without anyone naming it. (The Sink's own
verdict is `[uncacheable]`: a write is never served from a cache, by definition.)

## 3. Traced

A resolution is a tree — the request you issued, and every sub-request made on its behalf
— and the kernel will show you the tree. Hand `issue_traced` something that implements
`Tracer`:

```rust,ignore
{{#include ../../../../crates/hello-camel/tests/payoff.rs:tracer}}
```

```rust,ignore
{{#include ../../../../crates/hello-camel/tests/payoff.rs:traced}}
```

```text
span 1 parent Some(0)  urn:iki:tutorial:title  cache_hit=false
span 0 parent None  urn:iki:tutorial:camel-title  cache_hit=false
```

The child prints first, because it *finished* first — an event is recorded when its
invocation completes. `(span, parent)` pairs reconstruct the tree: `title` ran as span 1,
inside span 0. Each event also records whether the cache served it, which worker thread it
ran on, and the capability it ran under, so an attenuation down a call chain is visible
node by node. Run the traced resolution a second time on the same kernel and both spans
report `cache_hit=true`.

The events are plain, serializable data. That matters in [Part
III](../beyond/socket.md), where a remote kernel records its own events and ships them back
to be stitched into the caller's tree.

<div class="ikigai-run" data-cmd='sink urn:kernel:cut urn:iki:tutorial:title
trace urn:iki:tutorial:camel-title'>
<pre class="ikigai-run-expected">cut urn:iki:tutorial:title
[uncacheable]
trace  urn:iki:tutorial:camel-title
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:camel-title   camel-title · computed · ThreadId(1) · —   → 25b  resourceOrientedComputing
└─ urn:iki:tutorial:title   title · computed · ThreadId(1) · —</pre>
</div>

The first line cuts `title`'s thread by hand — `urn:kernel:cut` is the resource for
cutting somebody else's thread — so the trace shows a real resolution rather than a cache
hit with no children. `trace` is `issue_traced` with the engine's own tree renderer: the
child is indented under its parent, and each node says whether it was computed or served.
(The `—` is the duration: this kernel has no clock, and it says so rather than guessing;
`ThreadId(1)` is the browser's one thread. Run it after the cell above and the byte count
and the text change, because the title did — same kernel.)

## 4. Described

Everything so far was about *resolving* a name. `Meta` is about *asking* it. Through the
tutorial host's renderer, the answer is a graph:

```rust,ignore
{{#include ../../../../crates/hello-camel/tests/payoff.rs:described}}
```

```text
@prefix ik: <https://ikigai-rs.dev/ns#> .

<urn:ikigai:endpoint:camel-case> a ik:Endpoint ;
    ik:id "camel-case" ;
    ik:title "Camel-case" ;
    ik:summary "Camel-cases the UTF-8 text supplied in the `in` argument." ;
    ik:verb "Source", "Meta" ;
    ik:output "text/plain;charset=utf-8" ;
    ik:input <urn:ikigai:endpoint:camel-case:input:in> ;
    ik:action <urn:ikigai:endpoint:camel-case:action:source> .

<urn:ikigai:endpoint:camel-case:input:in> ik:inputName "in" ;
    ik:source "argument" ;
    ik:required true ;
    ik:summary "the text to camel-case" ;
    ik:class <http://www.w3.org/2001/XMLSchema#string> .
```

Three triples are worth reading slowly, and the test asserts all three so this page cannot
quietly drift from the code:

- `<urn:ikigai:endpoint:camel-case> a ik:Endpoint` — the endpoint is a *node* with a
  stable IRI, not a blob of documentation.
- `ik:input <urn:ikigai:endpoint:camel-case:input:in>` — so is each input. No blank nodes:
  every node has a name you can point a query at, and two catalogs diff cleanly.
- `ik:class <http://www.w3.org/2001/XMLSchema#string>` — the `.class(..)` you declared on
  the `ArgSpec` is where "what can I do with a string?" gets its answer from.

And the whole host at once — `urn:kernel:catalog`, one graph over every binding:

```rust,ignore
{{#include ../../../../crates/hello-camel/tests/payoff.rs:catalog}}
```

`title` declared two verbs with two contracts, and it appears as two `ik:Action` nodes —
that per-verb view, not the endpoint, is the unit an agent's tool list is built from.
[Why an endpoint describes itself](self-description.md) is about why this is
load-bearing rather than decorative.

<div class="ikigai-run" data-cmd='describe urn:iki:tutorial:camel-case text/turtle'>
<pre class="ikigai-run-expected">@prefix ik: &lt;https://ikigai-rs.dev/ns#&gt; .
&#32;
&lt;urn:ikigai:endpoint:camel-case&gt; a ik:Endpoint ;
    ik:id "camel-case" ;
    ik:title "Camel-case" ;
    ik:summary "Camel-cases the UTF-8 text supplied in the `in` argument." ;
    ik:verb "Source", "Meta" ;
    ik:output "text/plain;charset=utf-8" ;
    ik:input &lt;urn:ikigai:endpoint:camel-case:input:in&gt; ;
    ik:action &lt;urn:ikigai:endpoint:camel-case:action:source&gt; .
&#32;
&lt;urn:ikigai:endpoint:camel-case:input:in&gt; ik:inputName "in" ;
    ik:source "argument" ;
    ik:required true ;
    ik:summary "the text to camel-case" ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#string&gt; .
&#32;
&lt;urn:ikigai:endpoint:camel-case:action:source&gt; a ik:Action ;
    ik:verb "Source" ;
    ik:output "text/plain;charset=utf-8" ;
    ik:input &lt;urn:ikigai:endpoint:camel-case:input:in&gt; .
[computed]</pre>
</div>

`describe … text/turtle` is `Meta` with `as=text/turtle`, and the graph it prints is the
one the test above asserts three triples of.

```bash
cargo run -p hello-camel -- --catalog
```

prints the same graph from the tutorial binary.

## The same four things from a shell

None of this is a property of Rust. The `ikigai` CLI drives another host with the same
kernel in it, and the REPL grammar reaches all four. Start `ikigai` with no arguments and
type these in one session — the cache lives in the process, so one-shot `-c` runs would
each start empty:

```text
ikigai> source urn:iki:fn:toUpper in="a b" | urn:iki:fn:reverseList
A B
[2 computed]
ikigai> cache urn:iki:fn:toUpper in="a b"
cached
ikigai> source urn:iki:fn:toUpper in="a b"
A B
[cached]
ikigai> trace urn:iki:fn:toUpper in="a b"
trace  urn:iki:fn:toUpper
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process

urn:iki:fn:toUpper   toUpper · cached · main · 0ms   → 3b  A B
ikigai> describe urn:iki:fn:toUpper text/turtle
@prefix ik: <https://ikigai-rs.dev/ns#> .

<urn:ikigai:endpoint:toUpper> a ik:Endpoint ;
    ik:id "toUpper" ;
    …
ikigai> sink urn:file:notes.txt remember the milk
wrote 17 bytes to notes.txt
[uncacheable]
ikigai> source urn:file:notes.txt
remember the milk
[computed]
ikigai> cache urn:file:notes.txt
cached
ikigai> sink urn:kernel:cut urn:file:notes.txt
cut urn:file:notes.txt
[uncacheable]
ikigai> cache urn:file:notes.txt
not cached
ikigai> source urn:kernel:threads
threads (cut generations)
  urn:file:notes.txt  gen 2
```

Line by line: `|` pipes one resolution's output into the next one's unnamed argument, and
the `[2 computed]` tally is the kernel counting invocations. `cache` is `is_cached` from a
shell. `trace` is `issue_traced` with the CLI's own tracer rendering the tree. `describe
… text/turtle` is `Meta` with `as=text/turtle`. The file is the `title` of this chapter,
one level up: its endpoint declared a thread named after the file, so the `sink` that wrote
it cut that thread, and `sink urn:kernel:cut` cuts it again by hand — the resource for
doing what a write does, on somebody else's behalf — and the cached read is gone. Note
what a cut does *not* do: `urn:iki:fn:toUpper` declared no thread (a pure function has
nothing to hang one on), so cutting a thread by that name would leave its entry exactly as
cached as before. `urn:kernel:threads` shows every thread that has ever been cut, with its
generation — the file's twice.

> ⚠ The Turtle the CLI prints may differ in shape from the tutorial host's: which
> `ikigai-vocab` renders it is the CLI's choice, made on the CLI's release schedule, and
> older ones write inputs as blank nodes rather than the named ones above. The *triples*
> mean the same thing; the node names are the newer projection's improvement.

## What you have now

A name was cached, and you proved the endpoint did not run. A write upstream invalidated
a derived result that had never heard of the thing written. A resolution showed you its
own tree. An endpoint answered "what are you?" with a graph you could query. Four
properties, zero lines of code in `camel-case` to get any of them — they came from
resolving a *name* rather than calling a function.

The next two chapters are about the two things that made the fourth one possible: [the
description](self-description.md) an endpoint carries, and [the host](binding.md) that
knows how to render it.

## Try it

The cut, in twelve lines, compiled by this page:

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

let kernel = hello_camel::kernel();
let composite = Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:camel-title").unwrap());
let root = Capability::root();

block_on(kernel.issue(composite.clone(), &root)).unwrap();
assert!(kernel.is_cached(&composite, &root));

let write = Request::new(Verb::Sink, Iri::parse("urn:iki:tutorial:title").unwrap())
    .with_arg("content", ArgRef::Inline(b"try it".to_vec()));
block_on(kernel.issue(write, &root)).unwrap();

assert!(!kernel.is_cached(&composite, &root));
let repr = block_on(kernel.issue(composite, &root)).unwrap();
assert_eq!(String::from_utf8_lossy(&repr.bytes), "tryIt");
```
