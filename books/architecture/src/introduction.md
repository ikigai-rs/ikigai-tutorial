# Introduction

This book is about how ikigai is put together, and about one claim in particular: **ikigai does
not have a user-experience layer and a security layer that were designed separately and bolted
together.** It has one substrate, and the patterns a person *operates* are the same patterns that
*authorize and confine* them. The grant that a consent screen renders is the grant the kernel
checks. The action list a page draws is the action list an agent is handed. The cache that makes a
second read cheap is the boundary that keeps one person's answer from reaching another. Each of
these is one mechanism with two faces, and this book teaches the mechanism once and then turns it
to the light twice.

Before any of that, though, it is worth saying plainly what kind of system ikigai is, because the
shape of everything after follows from it.

## Resource-oriented computing

ikigai is a **resource-oriented** system. Everything it deals with — a file, a calculation, a
ledger item, a capability, the catalog of what the kernel can do — is a **resource**: a value with
a name, reached the same way whatever it is. You do not call a function that returns a ledger item;
you *resolve the name of* the item and get back a representation of it. You do not open a
connection and send a command; you ask one of a small, fixed set of verbs of a name, and the kernel
routes the request to whatever is bound to answer for it.

That idea is not new, and this book does not pretend it is. Resource-oriented computing was worked
out in detail in **NetKernel**, designed by **Peter Rodgers and Tony Butterfield**, and ikigai owes
it the core of its thinking. You do not need to know NetKernel to read this book — everything is
taught here from the ground — but it is honest to name where the ideas come from.

What does a reader get from a system built this way? A few things, and they compound:

- **One uniform interface.** There are five verbs and no more: **Source** (read), **Sink** (write),
  **Exists**, **Delete**, and **Meta** (describe yourself). Every resource answers the same five, so
  there is one thing to learn, not one API per subsystem.
- **Composition by default.** A resource can be built from other resources by name. A page is a
  template filled with the names it needs; a ledger's ranked view is a query over the ledger's
  graph. Nothing special connects them — they are all just names the kernel resolves.
- **Determinism and cheap repetition.** When a resource is a pure function of its inputs, the kernel
  can cache it, and — because it tracks what each answer was made from — it can know exactly when the
  cached answer has gone stale and no sooner. Reading the same thing twice is nearly free, and that
  is not an optimization bolted on top; it is a property of naming things honestly.
- **Machine-legibility.** Every endpoint describes itself, and the kernel publishes a catalog of
  what it can do and an action list scoped to who is asking. A program — or an agent — can read the
  system's own description of itself rather than being told about it out of band.

That last point is why ikigai keeps turning up in the same conversations as agentic AI. An agent
needs to know what actions are available, do them safely, and cite where its answers came from.
A resource-oriented kernel offers exactly those as first-class facts: the action list is the tool
manifold, capabilities are the safety model, and provenance is the citation. The substrate an agent
needs and the substrate a careful application needs turn out to be the same substrate.

## The Web is resource-oriented computing with a fixed vocabulary

The quickest way to understand ikigai is to notice that you already use a resource-oriented system
every day. The Web is one. It names things (URLs), it has a small fixed set of verbs
(`GET`, `POST`, `PUT`, `DELETE`), it negotiates representations (an `Accept` header asks for HTML or
JSON of the same resource), and it caches by URL (with `ETag` and `Cache-Control`). Those are the
same four ideas — naming, verbs, representation, caching — that this book is about.

The Web made each of those choices in one particular way, and froze it. ikigai takes each axis and
**generalizes it**:

| axis | the Web | ikigai |
|---|---|---|
| **naming** | `http(s)` URLs | any `urn:` scheme — a file, a function, a ledger item, a capability, the catalog itself all have names in the same space |
| **verbs** | four, and not uniform (much sits behind `POST`) | five — Source, Sink, Exists, Delete, Meta — over **every** resource, uniformly |
| **representation** | content negotiation on a mostly-fixed set of types | **transreption**: any representation converts to any other through converters, with one canonical hub (Turtle), so a new face is a converter, not a new endpoint |
| **caching** | by URL, with time-to-live and validators | **golden threads**: the kernel tracks what each answer depended on and invalidates exactly those, so a write cuts precisely the derived answers it should and no others |
| **authority** | bolted on per application (cookies, sessions, OAuth) | **capabilities**: authority is a resource too, attenuated down every call, and the kernel's capability check is the only authorization |

None of this makes the Web wrong. The Web's fixed vocabulary is why it scaled to the planet. The
point is that its shape is one *instance* of a more general shape, and once you see the general
shape you can apply it where the Web's particular freezing of it gets in the way: to a calculation,
to a private tenant's data, to an agent's tool list, to a cache that must respect who is asking.

### A note on RDF and XSLT

ikigai leans on RDF (a graph data model) and, in places, XSLT (a transform language for turning one
representation into another). A reader coming from a mainstream background may read that as a cost
of entry. It is worth being clear that it is a **choice with a reason**, not a tax.

The reason is that the data in this system *already is a graph*. An endpoint's self-description, the
catalog of what the kernel can do, the provenance trail behind an answer, a ledger item and its
comments and links — these are nodes and edges whether or not you write them down that way. RDF is
how you write down a graph without inventing a bespoke shape for each one, and it is why two
different subsystems' descriptions can be read by one tool. XSLT earns its place for the same
reason the Web uses it: turning a graph into a page is a transform from one representation to
another, which is exactly what a transreptor is.

And it is not a requirement for using the system. A view can be composed from a JSON document or a
CSV table just as well as from a graph; the plain-text face of a resource is the default, and a JSON
face is a converter away. RDF is the hub because a hub has to be *something*, and a self-describing
graph is the honest something for a system whose data is already shaped like one. Where this book
shows a graph face, it says so, and it does not ask you to reach for RDF where a simpler
representation is the natural one.

## How this book is organized, and what it assumes

This is the fourth book in [the library](../library/index.html). It assumes you have met the core
ideas — resources, the five verbs, transreption, golden threads, capabilities — in
[The ikigai Book](../index.html), and it names them as it builds on them rather than teaching them
again. It is shorter than the others on purpose: it is a tour of the architecture for a reader who
can already write against the kernel and now wants to see how the pieces fit, and why the pieces
that confine are the same pieces that serve.

It carries **one worked resource throughout**: a **ledger item**, the kind of thing
[The gonk Book](../gonk/index.html) builds a whole host around. An item is a good choice because it
is already everything at once — a page, a name the kernel resolves, a set of actions gated by
capabilities, and a record with provenance. Each chapter turns that one resource to the light from
a new angle.

Where a demonstration can run, it runs. This book's pages carry **runnable cells**: the same work
ledger The gonk Book teaches, compiled to WebAssembly and running in your browser. When you press
Run, a real kernel files the item, ranks it, and refuses what your capability does not allow — in
the page, for real, nothing recorded. A few patterns in the later chapters cannot run in a page yet
(they need parts of the system a browser has no sockets for); those are drawn as pictures, and the
text says plainly which is which and promises the runnable version as the tooling for it lands.

The chapters, in order:

1. **One substrate, two faces** — the thesis, on the worked item.
2. **The capability spine** — authority resolved once per request, and the user interface that falls
   out of it.
3. **The door** — the one place where the spine meets a browser.
4. **The overlay chain** — the governors that wrap a resource to make it resilient and bounded.
5. **The economics** — golden threads and caching, and why they are the system's accounting.
6. **Closing the loop** — the human-in-the-loop action and the record it leaves.
