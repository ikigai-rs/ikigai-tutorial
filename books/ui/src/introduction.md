# Introduction

This book builds user interfaces the ikigai way: **a view is a resource**. A page is not a
program that fetches data and assembles markup. It is a name the kernel resolves, composed from
other names, cached like any other answer and recomputed when what it was made from changes.
The same model is delivered to a browser as HTML, and later to a native window, a page running
its own kernel, or a terminal, without a second renderer written against a second idea of the
data.

The book teaches by building one small, real app: a **kernel inspector**. It shows a running
kernel to the person running it: what its cache holds, which golden threads have been cut, how
its spaces are arranged, what its endpoints say about themselves, and its scheduler. If you have
read The ikigai Book you have met every one of those as text. `source urn:kernel:cache` printed
the cache in [Twenty minutes, nothing compiled](../front-door/cli.html), and the threads and their
cuts were the subject of [Golden threads in practice](../getting-started/golden-threads.html).
Here they become pages that update while you watch.

## What you need

- The ikigai Book's first part, or the same ideas from somewhere else: resources, spaces,
  capabilities and golden threads. This book uses them and does not teach them again.
- A browser. Every chapter in Part I runs in the page: the cells talk to a kernel compiled to
  WebAssembly that holds the whole inspector, the same crates the rest of the book builds.
- Rust, for the listings and for serving the inspector yourself in Part II. Each listing is
  compiled and run by `mdbook test` whenever the book is built, so a listing that stopped
  matching the code would fail the build before it reached you.

## How the book is laid out

**Part I, the model**, is about views as resources, whatever delivers them: a page composed
from parts, a graph turned into a page by XSLT, and data that changes while it is shown. The
model does not know whether it will end up in a browser.

**Part II, delivery targets**, takes the same model to a place a person sees it. Server-rendered
HTML with htmx and a push stream is written; the others (GPUI, WebAssembly in the browser,
WebTransport and the terminal) are reserved, each with a page saying what it will cover and
what it waits on. A new target is a new thin crate and a new chapter, not a rewrite.

**Part III, components**, will cover the pieces you embed in a page you did not write.

Some chapters are not written yet. Their pages say so and say what they will cover; What's new
lists a chapter when it is written.

## The code

The inspector is two crates in this book's repository, split the way every target will be:

- `crates/inspector` is the **model and its artifacts**: the stylesheets, the templates, the
  route-to-view names, and the one atom that holds state. It is the same crate in every target.
- `crates/inspector-htmx` is a **thin renderer**: a host that serves the inspector over HTTP,
  through the same door library the cli uses.

The book's own in-page kernel, `crates/ui-book-wasm`, is a third, even thinner face: the
inspector's kernel with the cli's engine in front of it, so a cell's line means exactly what it
means at `ikigai`.

The rules every page follows (how a view is made, how its URL is spelled, what the door allows,
what accessible means and how it is checked) were decided before any code was written. Each
chapter names the ones it uses, and what they rule out.
