# A page is a composed resource

A page in this book is a name. Ask the kernel for `urn:iki:tutorial:inspect:page:cache` and it
answers with HTML, the way it answers any other name: it finds the door bound to the pattern,
the endpoint behind the door runs, and the answer comes back as a representation with a media
type. Nothing about it is special to user interfaces. What makes it a page is only that its
media type is `text/html` and a browser will show it.

What makes it worth building this way is what the page is made *from*. It is not written by a
function that fetches the cache and formats a string. It is a **template** filled with other
resources: a header, a title and the view itself, each a name of its own. The kernel resolves
each part, caches the whole, and remembers which parts it read, so a change to any part
recomputes the page and nothing else.

## A template, and what it is filled with

The smallest view in the inspector is a card that shows a note. The card's template is itself a
resource; read it as text:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:template:card'>
<pre class="ikigai-run-expected">&lt;div id="note-card" class="inspect-card" hx-get="/inspect/card" hx-trigger="ikigai:cut from:body" hx-swap="outerHTML" data-ui-quiet=""&gt;
&lt;p&gt;$h{urn:iki:tutorial:inspect:demo:note}&lt;/p&gt;
&lt;/div&gt;
[computed]</pre>
</div>

Three things in it are not HTML. `$h{urn:iki:tutorial:inspect:demo:note}` is a **marker**: when
the template is filled, compose (`ikigai-fn`'s template language) resolves the name inside the
braces and splices the answer in its place. The `h` says how: as **text, escaped for HTML**, so
whatever the note holds can never become markup. The others are htmx attributes, which the
chapter on live data explains.

The card is the template, filled. It is bound at its own name, `demo:card`, by a `composeOver`:
compose with its template fixed, so the view is a resource like any other.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:demo:card'>
<pre class="ikigai-run-expected">&lt;div id="note-card" class="inspect-card" hx-get="/inspect/card" hx-trigger="ikigai:cut from:body" hx-swap="outerHTML" data-ui-quiet=""&gt;
&lt;p&gt;Write me, then watch the cache.&lt;/p&gt;
&lt;/div&gt;
[computed]</pre>
</div>

Read it again, and the answer comes from the cache: nothing was recomputed, because nothing it
was made from has changed.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:demo:card'>
<pre class="ikigai-run-expected">&lt;div id="note-card" class="inspect-card" hx-get="/inspect/card" hx-trigger="ikigai:cut from:body" hx-swap="outerHTML" data-ui-quiet=""&gt;
&lt;p&gt;Write me, then watch the cache.&lt;/p&gt;
&lt;/div&gt;
[cached]</pre>
</div>

## The one atom

Almost nothing in the inspector holds state. The views are compositions, the stylesheets are
artifacts, and the kernel's readouts belong to the kernel. The note is the exception, so it is
the one place the inspector has code that keeps something: an **atom**. Write it, with something
in it that would be markup if it were not escaped:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:inspect:demo:note A &lt;b&gt;bold&lt;/b&gt; claim'>
<pre class="ikigai-run-expected">written
[uncacheable]</pre>
</div>

The write cut the note's golden thread. The card was cached hanging from it, so the next read
of the card is computed again, from the new note, and the tags arrive as text:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:demo:card'>
<pre class="ikigai-run-expected">&lt;div id="note-card" class="inspect-card" hx-get="/inspect/card" hx-trigger="ikigai:cut from:body" hx-swap="outerHTML" data-ui-quiet=""&gt;
&lt;p&gt;A &amp;lt;b&amp;gt;bold&amp;lt;/b&amp;gt; claim&lt;/p&gt;
&lt;/div&gt;
[computed]</pre>
</div>

That is the whole model of a page, at its smallest: a template that is a resource, filled with
resources, cached, and recomputed exactly when one of its parts is written.

## The page around a view

A full page needs more than a view: a document, a stylesheet, scripts, landmarks, a skip link,
a region for status messages. Every page in every ikigai app needs the same ones, so they are
not the inspector's to write. They come from **ikigai-ui**, which serves a shared page shell, a
token stylesheet with tested themes, one first-party script, and a vendored htmx, all as
resources. A host binds ikigai-ui's space and asks it for a **family of pages** over its own
names:

```rust,ignore
{{#include ../../../../crates/inspector/src/lib.rs:kernel}}
```

`ikigai_ui::pages("inspect", "urn:iki:tutorial:inspect:")` binds
`urn:iki:tutorial:inspect:page:{name}`: the shell, filled with this family's
`urn:iki:tutorial:inspect:header`, `…:title:{name}` and `…:fragment:{name}`. The page's title is
a resource too:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:title:note'>
<pre class="ikigai-run-expected">A note to watch · Inspector
[computed]</pre>
</div>

and the page is the shell over the note's fragment, which in turn includes the card:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:page:note'>
<pre class="ikigai-run-expected">&lt;!doctype html&gt;
&lt;html lang="en" data-theme="auto"&gt;
&lt;head&gt;
&lt;meta charset="utf-8"&gt;
&lt;meta name="viewport" content="width=device-width, initial-scale=1"&gt;
&lt;meta name="htmx-config" content='{"allowEval":false,"allowScriptTags":false,"includeIndicatorStyles":false,"h…
&lt;title&gt;A note to watch · Inspector&lt;/title&gt;
&lt;link rel="stylesheet" href="/ui/tokens.css"&gt;
&lt;script src="/ui/shell.js"&gt;&lt;/script&gt;
&lt;script src="/ui/htmx" integrity="sha384-HGfztofotfshcF7+8n44JQL2oJmowVChPTg48S+jvZoztPfvwD79OC/LTtG6dMp+" defer&gt;&lt;/script&gt;
&lt;/head&gt;
&lt;body&gt;
&lt;a class="ui-skip" href="#main"&gt;Skip to content&lt;/a&gt;
&lt;header&gt;
&lt;link rel="stylesheet" href="/inspect/inspector.css"&gt;
&lt;nav aria-label="Inspector"&gt;
&lt;a href="/inspect/cache" hx-get="/inspect/fragment/cache" hx-target="#main" hx-push-url="/inspect/cache"&gt;Cache&lt;/a&gt;
&lt;a href="/inspect/threads" hx-get="/inspect/fragment/threads" hx-target="#main" hx-push-url="/inspect/threads"&gt;Threads&lt;/a&gt;
&lt;a href="/inspect/topology" hx-get="/inspect/fragment/topology" hx-target="#main" hx-push-url="/inspect/topology"&gt;Topology&lt;/a&gt;
&lt;a href="/inspect/catalog" hx-get="/inspect/fragment/catalog" hx-target="#main" hx-push-url="/inspect/catalog"&gt;Catalog&lt;/a&gt;
&lt;a href="/inspect/scheduler" hx-get="/inspect/fragment/scheduler" hx-target="#main" hx-push-url="/inspect/scheduler"&gt;Scheduler&lt;/a&gt;
&lt;a href="/inspect/note" hx-get="/inspect/fragment/note" hx-target="#main" hx-push-url="/inspect/note"&gt;Note&lt;/a&gt;
&lt;a href="/inspect/planets-json" hx-get="/inspect/fragment/planets-json" hx-target="#main" hx-push-url="/inspect/planets-json"&gt;Planets (JSON)&lt;/a&gt;
&lt;a href="/inspect/planets-csv" hx-get="/inspect/fragment/planets-csv" hx-target="#main" hx-push-url="/inspect/planets-csv"&gt;Planets (CSV)&lt;/a&gt;
&lt;/nav&gt;
&lt;script src="/inspect/live.js" defer&gt;&lt;/script&gt;
&#32;
&lt;/header&gt;
&lt;div id="status" role="status" aria-live="polite"&gt;&lt;/div&gt;
&lt;main id="main" tabindex="-1"&gt;
&lt;section id="view" class="inspect"&gt;
&lt;h1&gt;A note to watch&lt;/h1&gt;
&lt;p&gt;Writing the note cuts its golden thread. Open the cache view in another tab and watch the card's entry turn &lt;code&gt;cut&lt;/code&gt;.&lt;/p&gt;
&lt;div id="note-card" class="inspect-card" hx-get="/inspect/card" hx-trigger="ikigai:cut from:body" hx-swap="outerHTML" data-ui-quiet=""&gt;
&lt;p&gt;A &amp;lt;b&amp;gt;bold&amp;lt;/b&amp;gt; claim&lt;/p&gt;
&lt;/div&gt;
&#32;
&lt;form method="post" action="/inspect/note/write" hx-post="/inspect/note/write" hx-swap="none"&gt;
&lt;label for="note-text"&gt;The note&lt;/label&gt;
&lt;textarea id="note-text" name="content" rows="3" maxlength="4096" required&gt;&lt;/textarea&gt;
&lt;button type="submit"&gt;Write it&lt;/button&gt;
&lt;/form&gt;
&lt;/section&gt;
&#32;
&lt;/main&gt;
&lt;/body&gt;
&lt;/html&gt;
[computed]</pre>
</div>

Two things about that page are rules, not accidents.

**The fragment and the page are separate names.** `fragment:note` is the view alone, what htmx
swaps into the page when you follow a link; `page:note` is the document a reload or a bookmark
gets. The page composes the fragment, so the two cannot drift apart, and no URL ever answers
two different things depending on who asks (UI conventions D8).

**The page splices what the host says, never what the request says.** The shell splices the
fragment raw, as trusted markup, so which fragment it splices is fixed by the page's own name.
No argument on the request can point the shell at a different resource.

## In Rust

The same reads, compiled and run by `mdbook test`. The capability is not root: a viewer gets
the inspector's viewer scopes, and every part of the page is resolved under them.

```rust
# extern crate ikigai_core;
# extern crate inspector;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

let kernel = inspector::kernel(Vec::new());
let viewer = Capability::root().attenuate(inspector::VIEWER_SCOPES);
let read = |name: &str| {
    let rep = block_on(kernel.issue(Request::new(Verb::Source, Iri::parse(name).unwrap()), &viewer))
        .unwrap();
    String::from_utf8(rep.bytes).unwrap()
};

let card = read("urn:iki:tutorial:inspect:demo:card");
assert!(card.contains("<p>Write me, then watch the cache.</p>"));

block_on(kernel.issue(
    Request::new(Verb::Sink, Iri::parse("urn:iki:tutorial:inspect:demo:note").unwrap())
        .with_arg("content", ArgRef::Inline(b"A <b>bold</b> claim".to_vec())),
    &viewer,
))
.unwrap();
let card = read("urn:iki:tutorial:inspect:demo:card");
assert!(card.contains("<p>A &lt;b&gt;bold&lt;/b&gt; claim</p>"));

let page = read("urn:iki:tutorial:inspect:page:note");
assert!(page.starts_with("<!doctype html>"));
assert!(page.contains("<title>A note to watch · Inspector</title>"));
assert!(page.contains(&card));
```

## What was built, and what it rules out

The card, its template, the note and the page are resources; only the note is code that holds
state. What this rules out is the thing most web code does: Rust (or any language) assembling
HTML strings per view (UI conventions D1). There is no `format!` building markup anywhere in the
inspector, and the escaping lives in exactly one place, compose's `$h`.

Next, the views whose input is a graph: [Graph to page with XSLT](xslt.md).
