# A view over HTTP: the remaining step

You have asked a recorded week questions, given its data names, built composites,
replayed scripts, and named requests. [A river of feeds](rss.md#6-one-answer-four-faces)
also builds HTML: its Run cells let you inspect the answer and see it rendered as a page.
The final step is to share a view through an HTTP server, so another browser can ask for it.

**This walkthrough is waiting on the HTTP view sample (ledger #2450).** The part does not
have a tested, end-to-end HTTP finale yet. Rendering an answer inside this book does not
prove that the CLI's HTTP door can serve the same composition. Until the sample is checked
against the book's pinned CLI, there are no server commands or expected HTTP responses to
replay here.

## The resources a view needs

Keep the same separation you used for the recorded week:

- **Data**: the recorded GeoJSON file, or the live feed read through `urn:http:get`.
  [Somebody else's resource](http.md) explains the live feed's freshness and authority.
- **Questions and templates**: files that describe how to select the data and turn an
  answer into a representation. The river's saved searches and HTML stylesheet are a
  working example; [Reshaping JSON with jq](jq.md#4-a-template-that-asks-questions) fills
  a smaller headline template over Quakes.
- **A named composition**: the request a caller asks for, with its fixed arguments kept
  in the binding. [Naming a composition](naming.md) shows what `bind` can name today,
  including its limits for the river's build pipeline.
- **An HTTP address**: a route to the view, served with the authority needed to read its
  inputs. This is the boundary the pending sample must establish.

An HTML answer and an HTTP address have different jobs. The composition decides what the
answer contains. The server decides how a browser reaches it and what the request may read.
A file copied out by a refresh script also has a different dependency boundary from a view
that reads its inputs through the kernel on each request. The finale needs to demonstrate
which one it serves, how an input change reaches the next answer, and what is cached.

## What remains to be checked

The HTTP sample must show which resources the served space exposes, how the view receives
its arguments, and how the server gets permission to read its files and reach the API.
It must also show an input changing and the next HTTP request answering from the changed
data. A successful read in the REPL alone does not establish any of those behaviors over
HTTP.

Request bindings were another gap when this part began (ledger #2447). Their usable form
is now covered by [Naming a composition](naming.md); the remaining pipeline limitations
are marked there. The HTTP finale remains pending separately. This page does not assume
that closing one gap closes the other.

For now, you can run and change the river's HTML views in [A river of feeds](rss.md),
follow [When a rule is not enough](beyond-rules.md) into Lisp and a Python peer, or continue
to [Resolution](../getting-started/resolution.md) to see how these requests work underneath.
