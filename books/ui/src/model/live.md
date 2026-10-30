# Live data and history

A page about a cache is out of date the moment something is written. The usual answer is to ask
again every second. ikigai has a better one, because the kernel already knows exactly when an
answer goes stale: that is what golden threads are for. This chapter makes the inspector's
views follow the kernel as it changes, and makes every view a URL that Back returns to.

## What a write does to the cache

In this page's kernel, read the card, then look at the cache's text face, the one
[Twenty minutes with the CLI](../../front-door/cli.html) printed:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:demo:card'>
<pre class="ikigai-run-expected">&lt;div id="note-card" class="inspect-card" hx-get="/inspect/card" hx-trigger="ikigai:cut from:body" hx-swap="outerHTML" data-ui-quiet=""&gt;
&lt;p&gt;Write me, then watch the cache.&lt;/p&gt;
&lt;/div&gt;
[computed]</pre>
</div>

<div class="ikigai-run" data-cmd='source urn:kernel:cache'>
<pre class="ikigai-run-expected">cache
  entries  3 / 4096 (0 stale)
  size     402 B / 64.0 MB
  urn:iki:tutorial:inspect:demo:card      live     text/html; charset=utf-8      182 B  4 threads   root
  urn:iki:tutorial:inspect:demo:note      live     text/plain; charset=utf-8       31 B  1 thread    root
  urn:iki:tutorial:inspect:template:card  live     text/html; charset=utf-8      189 B  2 threads   root
[uncacheable]</pre>
</div>

The card and the note are both `live`: a read would be served from the cache. Now write the
note:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:inspect:demo:note Watch this'>
<pre class="ikigai-run-expected">written
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='source urn:kernel:cache'>
<pre class="ikigai-run-expected">cache
  entries  3 / 4096 (2 stale)
  size     402 B / 64.0 MB
  urn:iki:tutorial:inspect:demo:card      cut      text/html; charset=utf-8      182 B  4 threads   root
  urn:iki:tutorial:inspect:demo:note      cut      text/plain; charset=utf-8       31 B  1 thread    root
  urn:iki:tutorial:inspect:template:card  live     text/html; charset=utf-8      189 B  2 threads   root
[uncacheable]</pre>
</div>

Both entries are `cut`: still resident, but stale, so the next read recomputes them. The cut
happened at one instant, for one reason, to a known set of entries. A page showing either of
them does not need to guess when to look again; it needs to be **told**.

## Being told: the push stream

The cli's HTTP door already tells. [Spreadsheet, VII: push](../../applied/spreadsheet-7.html)
built on it: one `GET` of the door's push path answers `text/event-stream`, and every cut that
invalidated something the page's own reads rest on arrives as one `cut` event. It is heard under
the same capability the page's reads were made with, so a viewer hears only cuts of what that
viewer cached, and nobody else's (UI conventions D9).

The inspector's part is small. A script, `urn:iki:tutorial:inspect:live.js`, opens the stream
and turns every `cut` into one event on the page:

```js
{{#include ../../../../crates/inspector/artifacts/live.js}}
```

and a view that shows changing data refetches itself when it hears that event. The cache view's
stylesheet puts this at the top of the view:

```html
<span hidden="" data-ui-quiet="" hx-get="/inspect/fragment/cache"
      hx-trigger="ikigai:cut from:body" hx-target="closest section" hx-swap="outerHTML"></span>
```

htmx does the rest: on a cut it asks for the fragment again and swaps the whole view in place.
The row for the card turns `cut` while you watch, and a moment later, when the card is read
again, `live`. The note's card listens the same way, so writing the note in one tab updates
the card in every tab that shows it.

Three choices in that line are rules:

- **`data-ui-quiet`.** ikigai-ui's shell script announces the result of what *you* did in the
  page's status region. A view refreshing itself is not something you did, so it is quiet: it
  never writes the status region and never moves your focus.
- **The trigger is a cut, not a clock.** Polling a cacheable fragment on a timer is ruled out;
  the door knows when to refetch, and an unchanged fragment would answer `304` anyway.
- **The fragment is fetched, not the page.** The page composes the fragment, so refreshing the
  fragment is refreshing exactly what changed.

⚠ `live.js` is a stopgap. The shared shell script is meant to carry this mapping itself (UI
conventions D14: one first-party script per page), and ikigai-ui 0.1.0 does not yet, so the
inspector serves its own and says so.

## Every view is a URL

Live views are only half of a page that changes. The other half is history: following a link
inside the inspector swaps the view in place, and Back must still work, and a reload or a link
someone pastes must land on the same view.

So every view has a page URL and a fragment URL, and every link carries both:

```html
<a href="/inspect/threads" hx-get="/inspect/fragment/threads"
   hx-target="#main" hx-push-url="/inspect/threads">Threads</a>
```

With a script, htmx fetches the fragment, swaps it into `<main>`, and pushes the page's URL onto
the history; ikigai-ui's shell script then moves focus to the new view's heading, so a keyboard
or screen reader user lands where the content changed. Without one, the `href` is an ordinary
link to the whole page. A reload, a bookmark and a history miss all fetch the page, which
composes the same fragment, so there is no state a reload can lose (UI conventions D10).

A detail is a view with a URL like any other. Each thread's link carries its name,
percent-encoded, as one path segment:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:thread:urn:iki:tutorial:inspect:demo:note'>
<pre class="ikigai-run-expected">&lt;section class='inspect' id='view'&gt;&lt;span data-ui-quiet='' hidden='' hx-get='/inspect/fragment/thread/urn%3Aiki…
[uncacheable]</pre>
</div>

## Why it is cheap

A push stream rides the cache the reads already built. The door registers one listener per
open page, in that page's cache partition, and the kernel tells it about a cut only when an
entry that page's reads made depends on the thread that was cut. Nothing polls. The page asks
again only when its answer is known to have changed.

## In Rust

The cut, seen through the kernel's own readout, compiled and run by `mdbook test`:

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
let card_row = |cache: &str| {
    let at = cache.find(">urn:iki:tutorial:inspect:demo:card</a>").unwrap();
    cache[cache[..at].rfind("<tr").unwrap()..at].to_string()
};

read("urn:iki:tutorial:inspect:demo:card");
assert!(card_row(&read("urn:iki:tutorial:inspect:fragment:cache")).contains("state-live"));

block_on(kernel.issue(
    Request::new(Verb::Sink, Iri::parse("urn:iki:tutorial:inspect:demo:note").unwrap())
        .with_arg("content", ArgRef::Inline(b"Watch this".to_vec())),
    &viewer,
))
.unwrap();
assert!(card_row(&read("urn:iki:tutorial:inspect:fragment:cache")).contains("state-cut"));
```

The stream itself is the door's, so its test is the door's too: `crates/inspector-htmx` starts
the real binary, opens `/_ikigai/push`, writes the note through the door, and reads the `cut`
event that names the card.

## In this page, and what is next for it

The cells above run in this page's own kernel, so they show the cache changing as text. This
page does not yet show the cache *view* rendered and changing in place. Nothing in the kernel
stops it: a kernel compiled to WebAssembly can listen for its own cuts, and
[Spreadsheet, VII: push](../../applied/spreadsheet-7.html) does exactly that in its page. What
is missing is this book's half: its kernel does not export a listener to the page yet, and no
script here renders a view's HTML in place of a cell's text. That is the next step for this
book's kernel; until then, [htmx: server-rendered HTML with push](../delivery/htmx.md) runs
the real thing.
