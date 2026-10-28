# Introduction

A resource-oriented computing kernel in Rust, taught from the outside in.

## What ikigai is, in one paragraph

In ikigai, data and computations are **resources**, each named by an IRI
(Internationalized Resource Identifier). To **resolve** a name is to ask the **kernel**
to route a request to an **endpoint**, code that handles it. The answer is a
**representation**: bytes plus a media type, such as `text/plain`. Requests use five verbs: `Source` (read), `Sink`
(write), `Exists`, `Delete`, and `Meta` (describe the resource). **Transreptors** convert
one representation into another, such as text into HTML. **Golden threads** record a
result's dependencies so a change can invalidate cached answers that depend on it.
**Capabilities** specify which requests a caller may make; a caller can pass narrower
permissions to the resources it calls.

That is the whole model. The rest is consequences — and the first four of them, run
rather than described, are [What resolution buys you](getting-started/payoff.md).

## Why bother

Because naming a thing and *resolving* a thing are different acts, and separating them
buys you a lot at once. If a computation has a name, it can be cached, traced,
substituted, authorized, converted, and moved to another machine without its caller
knowing. An ordinary function call does not provide those services on its own.

The wager of this project is that a system where every step is a resolvable name is
*cheaper to reason about at scale* than one where the steps are opaque calls — and that
this matters most now that programs are being assembled by agents, which need exactly
what resolution gives you: a machine-readable catalog of what can be done, an enforceable
boundary on what may be done, and provenance for what was done.

## What is in here

**[The front door](front-door/cli.md)** needs no Rust: twenty minutes with the `ikigai` CLI that show the
model in use: list available resources, pass results between them, change formats, reuse
and invalidate cached answers, and trace a request. [The REPL grammar](front-door/grammar.md)
then explains the command syntax. A REPL is an interactive prompt that reads a command,
runs it, prints its result, and waits for another. Run buttons let you try commands
where the page's own kernel can answer.

**Part I — Getting started** builds an endpoint and links it into a kernel you compose
yourself. It is the 95% case, and everything else assumes it.

**Building endpoints** is the rest of the builder's toolkit, one chapter per move: a
transreptor of your own and the kernel finding it, an endpoint with three verbs and a
capability that is declared *and* enforced, the catalog queried as a graph, and an
endpoint that reads the clock tested without one.

**Part II — Loadable modules** covers the other shape: a `space()` compiled separately
and routed to at runtime, and the callback that makes a module something quite different
from a remote peer.

**Part III — Beyond one host** takes the step Part II stops at. A kernel behind a socket,
the authority a certificate mints when the socket becomes a network, the three honest
things a mount can mean by "resolve this over there" — and then two clients, one human and
one machine, that build their command surface by reading the catalog rather than being
told.

**Applied** builds one small application end to end, the way resource-oriented design says
to: name the resources first, write code only for what has to hold something, and compose the
rest. The application is tic-tac-toe, in seven parts numbered on their own, I to VII. They take
the game from a model on paper to atoms, lines, rules, many games at once, a board you can
play in the page, the same game held and served from Python and TypeScript, and finally its
arrangement written down as data, one file the kernel builds the game from. Then
**Time is a resource** takes the first step past composition: an answer that goes out of date
with nobody writing, a clock the host gives the kernel, and a page that polls it. A
**spreadsheet**, in seven more parts, builds something a little bigger the same way: cells as
atoms, a formula as code that is itself a resource, compiled by a transreptor, recalculation that
is nothing but golden threads, cells that read the clock, a feed that something outside the
sheet writes, the sheet as a space (a person's scenario and the sheet as it was, each a corridor,
stacked), and finally push: the page is told when a value changes instead of asking.

**Polyglot tracks** mirror Part I chapter for chapter from Python and from TypeScript —
a decorated function served on a socket, the signature as the contract, a mount as a
binding, the kernel's cache and trace seen from outside, a scoped connect refused — and
close with a notebook that queries the catalog as a graph. They say plainly what an L0
peer cannot do: compose.

Read [the front door](front-door/cli.md) if you have not run ikigai before, then Part I — or at least
[Resolution](getting-started/resolution.md) and [Binding](getting-started/binding.md).
Building endpoints assumes Part I and nothing else;
Part II assumes you know what a `space()` is and why binding is separate from defining;
Part III assumes both, and leans hardest on capabilities. The tic-tac-toe parts assume Part I,
and its last part assumes the polyglot tracks as well. The spreadsheet assumes the tic-tac-toe
parts, and its fourth part assumes *Time is a resource*.

## How to read this book

The published copy is at <https://ikigai-rs.github.io/ikigai-tutorial/>, rebuilt from `main`
on every merge.

The book changes often. [What's new](whats-new.md) lists each change, most recent first, with
the pages to read for it.

[The library](library/index.html) lists the companion books, with who each is for and
where to start.

`mdbook test` compiles and runs the runnable Rust examples. It skips fragments marked
`ignore`; the longer code listings are **included from the crates that compile them**
rather than copied. A book that paraphrases its own examples is a book that will eventually
be wrong about them.

```bash
# read it (mdbook serve never builds the in-page kernel, so Run is disabled there)
mdbook serve books/ikigai --open

# read it with runnable cells: builds the kernel, serves on 127.0.0.1:3000
./scripts/serve-with-drafts.sh

# check that it still tells the truth
./scripts/test-books.sh
```

The code lives in
[`crates/`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates) — one crate per
part, plus crates that build and check the books.

## Conventions

Shell commands assume you are at the root of the `ikigai-tutorial` repository. Anything
marked ⚠ is a trap that has actually caught somebody.
