# Introduction

gonk is one binary that holds a durable RDF store and serves a **work ledger** from it:
items, comments, labels, links, claims, and a ranked answer to *what should I do next*. Around
that it can serve a repository browser, explanations of what it browses from a model it
mounts, a review queue that a person drains, and three doors to all of it: HTTP on loopback
with a browser face, an owner-only Unix socket, and QUIC for other machines.

None of that is gonk's own code in the sense that matters. The ledger is
[`ikigai-ledger`](https://github.com/ikigai-rs/ikigai-ledger), the store is
[`ikigai-store`](https://github.com/ikigai-rs/ikigai-store), the browser is
[`ikigai-browse`](https://github.com/ikigai-rs/ikigai-browse), and the doors are ikigai's own
transports. gonk is the host that composes them, which is why this book can show each one
**twice**: as modules in a kernel you build, and as the server.

## The ways to use it on its own

Each part is one way gonk is used **standalone**, and each builds on the one before it.

- **A ledger of your own.** File, comment, link, claim, close and ask `next` — in this page,
  in a kernel of your own, and against a gonk you start. Then the same ledger in a browser.
- **An agent's ledger.** The ledger reached from another process over the socket, the way an
  agent's tool calls reach it; and over MCP, once gonk projects its manifold that way
  (tracked as ledger item 782; marked *coming* until it lands).
- **The repository browser.** A repository's tree and files as resources, and an explanation
  of each derived once per content version, from a model gonk mounts rather than links.
- **Review without a reviewer.** One-off review, what a finding is, and the audit protocol
  that replaced per-commit review — with the measurement that retired it.
- **A spec in the repository.** [OpenSpec](https://github.com/Fission-AI/OpenSpec) keeps
  requirements, scenarios, changes and tasks as Markdown in an `openspec/` tree. Lifted to RDF
  with an `ikigai-markdown` mapping, a change's tasks become ledger items and `next` ranks them.
- **Bridges.** Filing roborev findings as ledger items; importing a kata ledger; checking out
  the repositories gonk browses and keeping them current; and all three in order on a fresh
  machine.
- **Several machines.** The three doors, the grant each runs under, and a QUIC client.
- **Embedding.** The ledger, the store and browse in a kernel of your own, with no gonk at all.

Chapters not yet written are listed in the sidebar without a link, so the shape of the book
is visible before all of it is.

## Three kinds of example, and how each is checked

A tutorial's examples rot silently, and the first reader to hit a stale one assumes they are
the problem. So every example in this book is one of three kinds, and each kind has a check
that fails when the example stops telling the truth.

| kind | looks like | checked by |
| --- | --- | --- |
| **Runnable in the page** | a cell with a **Run** button | the page's own kernel (`crates/gonk-book-wasm`: the ledger over an in-memory store, compiled to wasm). A test runs every cell natively, in page order, and compares its output with the output the page shows |
| **A Rust example** | a `rust` block, or a listing included from `crates/` | `mdbook test` compiles and runs every one against the crates under `crates/` |
| **A shell transcript** | a `console` block of `$ ` lines and their output | `scripts/test-transcripts.sh` installs gonk at the exact git revision this book pins, runs each page's commands in a scratch home, and compares what they print with what the page says |

The third kind is new in this repository. A transcript names the check it is under in a
comment above it, and one that cannot be run (opening a browser, say) says so and why — a
transcript nobody runs fails the check rather than passing it. Where an answer contains
something minted at run time, an item's id or a timestamp, the page prints `…` in its place,
and the check reads `…` as "anything here".

## Which gonk

gonk is not on crates.io yet. This book pins it to one git revision, held in one place
(`books/gonk/gonk.rev`) that the book, the transcript check and CI all read, and installs it
from there:

<!-- transcript: manual — the check installs this revision itself (scripts/test-transcripts.sh), reading the same file -->
```console
$ cargo install --locked --git https://github.com/ikigai-rs/ikigai-gonk --rev {{#include ../gonk.rev}}
```

The modules it links are the published crates, and the book's own crates use the same
versions: `ikigai-ledger` 0.3.0, `ikigai-store` 0.2.6 and `ikigai-browse` 0.18.0. Where a
chapter needs a model, its kernel binds a **stub** at `urn:llm:` that answers the same thing
every time: no example in this book calls a real model or the network.

From [Over the socket](agent/socket.md) on, a page also runs `ikigai`, the command-line host,
as the other process that reaches gonk. It IS on crates.io, so it is pinned to one exact
version, in one file (`books/gonk/ikigai-cli.version`) that the check and CI read the same
way. The version is the release whose transports the pinned gonk was built with, so the client
speaks the wire the server does:

<!-- transcript: manual — the check installs this version itself (scripts/test-transcripts.sh), reading the same file -->
```console
$ cargo install --locked ikigai-cli --version {{#include ../ikigai-cli.version}}
```

## How to read this book

The published copy is at <https://ikigai-rs.github.io/ikigai-tutorial/gonk/>, rebuilt from
`main` on every merge, beside [The ikigai Book](https://ikigai-rs.github.io/ikigai-tutorial/),
which teaches the kernel this one stands on. You do not need to have read it; where a chapter
leans on an idea it explains, it links there.

```bash
# read it (mdbook serve never builds the in-page kernel, so Run is disabled there)
mdbook serve books/gonk --open

# check that it still tells the truth
./scripts/test-books.sh          # every Rust example, and the accessibility gates
./scripts/test-transcripts.sh    # every shell transcript, against the pinned gonk
```

⚠ **Nothing in this book touches a gonk you already run.** Every transcript runs a scratch gonk
on port 1070 with its own home directory, and the check refuses any command that names 1060, a
real gonk's port, before it runs.
