# Graph to page with XSLT

The card in the last chapter was filled from named parts. Most of what the inspector shows is
not like that: the kernel describes its cache, its threads, its arrangement and its endpoints
as **graphs**, and a page about a graph has to walk it. There is a standard for turning a
document into another document by walking it, and ikigai uses it: **XSLT**. When the input is a
graph the page is an XSLT transform, and when the input is named arguments the page is a
compose template (UI conventions D2).

The inspector uses RDF and XSLT because of where its data comes from, not because a view needs
them. The kernel already publishes its introspection as Turtle, so the shortest path from a
readout to a page is a stylesheet over that graph. A view composes whatever form its data already
has. D2 says what to do with data that is ALREADY a graph; it is not a rule that data must become
one before it can be shown. Your data may be a JSON document from an API or a CSV export from a
spreadsheet, and the two chapters after this one build views over exactly those, with no RDF in
them: [Views from JSON](views-from-json.md) and [Views from CSV: one model, two
sources](views-from-csv.md).

This chapter builds the threads view, then reads the four others built the same way.

## The input: a readout, as a graph

Golden threads were introduced in [Golden threads in practice](../../getting-started/golden-threads.html):
a cached answer hangs from the threads of what it read, and a write cuts a thread. The kernel
lists the threads that have been cut at `urn:kernel:threads`. So that there is one to see, read
the card and write the note:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:demo:card'>
<pre class="ikigai-run-expected">&lt;div id="note-card" class="inspect-card" hx-get="/inspect/card" hx-trigger="ikigai:cut from:body" hx-swap="outerHTML" data-ui-quiet=""&gt;
&lt;p&gt;Write me, then watch the cache.&lt;/p&gt;
&lt;/div&gt;
[computed]</pre>
</div>

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:inspect:demo:note Threads, please'>
<pre class="ikigai-run-expected">written
[uncacheable]</pre>
</div>

The readout's default face is text, for a person at a terminal. Its graph face is Turtle:

<div class="ikigai-run" data-cmd='source urn:kernel:threads as=text/turtle'>
<pre class="ikigai-run-expected">@prefix ik: &lt;https://ikigai-rs.dev/ns#&gt; .
@prefix xsd: &lt;http://www.w3.org/2001/XMLSchema#&gt; .
&#32;
&lt;urn:ikigai:thread:urn:iki:tutorial:inspect:demo:note&gt; a ik:GoldenThread ;
    ik:threadName "urn:iki:tutorial:inspect:demo:note" ;
    ik:cutCount "1"^^xsd:nonNegativeInteger .
[uncacheable]</pre>
</div>

That graph is what the view is made from. Two steps stand between it and a page.

## Step one: the graph a stylesheet can read

XSLT reads XML, and RDF has an XML syntax, so the first step is a transreption: the same graph,
in RDF/XML. ikigai-rdf serves that as a resource, `urn:rdf:transrept`; the inspector does not
write a serializer. It does add one thing first. A thread's page has a URL, and building a URL
means percent-encoding the thread's name, which XSLT 1.0 cannot do. So the link arrives as data
on the node it belongs to, computed once, by the shared encoder. That is `graph:{view}`, the one
piece of the views that is code:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:graph:threads'>
<pre class="ikigai-run-expected">&lt;?xml version="1.0" encoding="UTF-8"?&gt;
&lt;rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#" xmlns:its="http://www.w3.org/2005/11/its"&gt;
	&lt;GoldenThread xmlns="https://ikigai-rs.dev/ns#" rdf:about="urn:ikigai:thread:urn:iki:tutorial:inspect:demo:note"&gt;
		&lt;cutCount xmlns="https://ikigai-rs.dev/ns#" rdf:datatype="http://www.w3.org/2001/XMLSchema#nonNegativeInteger"&gt;1&lt;/cutCount&gt;
		&lt;threadName xmlns="https://ikigai-rs.dev/ns#"&gt;urn:iki:tutorial:inspect:demo:note&lt;/threadName&gt;
		&lt;fragment xmlns="urn:iki:tutorial:inspect:term#"&gt;/inspect/fragment/thread/urn%3Aiki%3Atutorial%3Ainspect%3Ademo%3Anote&lt;/fragment&gt;
		&lt;page xmlns="urn:iki:tutorial:inspect:term#"&gt;/inspect/thread/urn%3Aiki%3Atutorial%3Ainspect%3Ademo%3Anote&lt;/page&gt;
	&lt;/GoldenThread&gt;
&lt;/rdf:RDF&gt;
[uncacheable]</pre>
</div>

It reads the kernel's readout under the **caller's** capability. A viewer who may not inspect
the kernel is refused by the kernel, for the kernel's reason; the inspector shows only what its
viewer may see, and it never decides that itself.

## Step two: the stylesheet

The stylesheet is an artifact, a resource the inspector serves at
`urn:iki:tutorial:inspect:xsl:threads`. It is short enough to read whole:

```xml
{{#include ../../../../crates/inspector/artifacts/threads.xsl}}
```

Values leave it only through `xsl:value-of` and attribute value templates, which escape what
they write. Nothing disables output escaping, and a test fails if any stylesheet ever does.
Every link is a real `href` that works without a script, plus `hx-get` for the fragment and
`hx-push-url` for the page, so htmx can swap the view in place and the address bar still names
what you see.

## The view is the two, composed

The fragment is a compose template whose one marker asks `urn:xslt:transform` for the transform
of the graph by the stylesheet:

```html
{{#include ../../../../crates/inspector/artifacts/view.html}}
```

and the inspector binds it at `fragment:{view}`, so every graph view is the same three lines of
binding over a different stylesheet:

```rust,ignore
{{#include ../../../../crates/inspector/src/lib.rs:views}}
```

Here is the threads view, in this page's kernel:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:threads'>
<pre class="ikigai-run-expected">&lt;section class='inspect' id='view'&gt;&lt;span data-ui-quiet='' hidden='' hx-get='/inspect/fragment/threads' hx-swap…
[uncacheable]</pre>
</div>

The stylesheet, the template and the transform of an unchanged input are cached. The readout
is not: what the kernel holds changes from one read to the next, so the view over it is
recomputed every time, and it says so.

## The other views

The cache, the topology and the catalog are built the same way, each with its own stylesheet
over its own readout. The topology's graph states order as RDF lists (a chain's layers, a
space's doors), which a stylesheet cannot walk without features xrust does not run reliably, so
`graph:topology` gives each node its place in resolution order as data, the same way it gives a
thread its link.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:cache'>
<pre class="ikigai-run-expected">&lt;section class='inspect' id='view'&gt;&lt;span data-ui-quiet='' hidden='' hx-get='/inspect/fragment/cache' hx-swap='…
[uncacheable]</pre>
</div>

A detail is a view too. `fragment:thread:{name}` is the thread and the entries hanging from it,
and `fragment:entry:{id}` is one cache entry and the threads it hangs from. Their graphs keep
only the focus and what is linked to it, and a detail the cache does not hold is refused as not
found, with the reason, rather than shown as an empty page:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:entry:0123456789abcdef'>
<pre class="ikigai-run-expected">error: not found: the cache holds no entry 0123456789abcdef for you (it may have been evicted)</pre>
</div>

## The scheduler, the stopgap

`urn:kernel:scheduler` has a text face and no Turtle face in the kernel this book runs on, so
there is no graph for a stylesheet to read. Its view shows the text, escaped, and says so
rather than pretending to be a graph view:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:scheduler'>
<pre class="ikigai-run-expected">&lt;section id="view" class="inspect"&gt;
&lt;h1&gt;Scheduler&lt;/h1&gt;
&lt;p class="inspect-stopgap"&gt;Shown as the kernel's text: &lt;code&gt;urn:kernel:scheduler&lt;/code&gt; has no Turtle face in…
&lt;pre&gt;scheduler
  backend    single
  threads    1
&lt;/pre&gt;
&lt;/section&gt;
[uncacheable]</pre>
</div>

When a Turtle face arrives, the scheduler becomes one more stylesheet.

## In Rust

```rust
# extern crate ikigai_core;
# extern crate inspector;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Request, Verb};

let kernel = inspector::kernel(Vec::new());
let viewer = Capability::root().attenuate(inspector::VIEWER_SCOPES);
let read = |name: &str, cap: &Capability| {
    block_on(kernel.issue(Request::new(Verb::Source, Iri::parse(name).unwrap()), cap))
        .map(|rep| String::from_utf8(rep.bytes).unwrap())
};
read("urn:iki:tutorial:inspect:demo:card", &viewer).unwrap();
block_on(kernel.issue(
    Request::new(Verb::Sink, Iri::parse("urn:iki:tutorial:inspect:demo:note").unwrap())
        .with_arg("content", ArgRef::Inline(b"Threads, please".to_vec())),
    &viewer,
))
.unwrap();

let threads = read("urn:iki:tutorial:inspect:fragment:threads", &viewer).unwrap();
assert!(threads.contains("<h1>Golden threads</h1>"));
assert!(threads.contains(">urn:iki:tutorial:inspect:demo:note</a>"));

// A viewer who may write the note but not inspect the kernel: the kernel says no.
let writer = Capability::root().attenuate([inspector::CAP_NOTE]);
let refused = read("urn:iki:tutorial:inspect:fragment:threads", &writer).unwrap_err();
assert!(matches!(refused, Error::Denied(_)), "{refused}");
```

## What was built, and what it rules out

Five views, four of them graph views, and one small piece of code that turns a readout into the
graph a stylesheet reads. Ruled out: walking a graph in Rust to emit HTML when a stylesheet
would do, and `disable-output-escaping` anywhere.

Next, the same kind of view over data that is not a graph: [Views from JSON](views-from-json.md).
