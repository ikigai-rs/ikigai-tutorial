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
  agent's tool calls reach it; and over MCP, where `ikigai mcp` projects the ledger's actions as
  an agent's tools, through a door that names the agent.
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
- **Scripts and sentences.** A query saved at a name and run by a person and a worker; a query
  drafted from a sentence by a machine and published by a person; and what each door may do with
  either.
- **Ways of working.** Processes rather than features: one person and an agent on one machine; a
  dispatcher and remote workers; review on every commit; hub and spoke; spec first; and periodic
  unled audits. Each chapter answers the same questions in the same order (who holds which role,
  what runs where, the loop step by step, what holds it, and what it costs), and links back to the
  chapters that teach its machinery rather than teaching it again. The last, harness engineering,
  is about the controls around any of them: the rules and checks as a graph, a log of every time
  a check fires, and the questions that say which rules nothing checks and which checks have
  never fired.
- **Embedding.** The ledger, the store and browse in a kernel of your own, with no gonk at all.

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

The modules it links are published crates: `ikigai-core` 0.1.99, `ikigai-ledger` 0.11.0,
`ikigai-store` 0.2.19, `ikigai-browse` 0.18.8, `ikigai-script` 0.3.3 and `ikigai-nl` 0.4.0. The
book's own crates (the ones its Rust listings and in-page cells run) take the same releases, so a
`console` block checked against the pinned gonk and a listing or a cell checked against the book's
crates are answered by the same code. (0.4.0 of the ledger added the keyed append [part 5](spec/ledger.md)
syncs with and the [bridges](bridges/roborev.md) file through; 0.9.0 added `count`, the number of
items a filter admits without the listing, which part 5's walk asks.) It serves scripts and
drafting at `urn:script:*` and `urn:nl:*`.

What the pin's last move (from 2598192) changed for a reader, each where the page that shows it
says so: a refusal at a family scope now also says what satisfies it, ``; any grant under
`urn:cap:ledger:delete:` satisfies it`` ([Your own ledger](ledger/your-own.md), [Over the
socket](agent/socket.md), [A QUIC client](machines/quic-client.md)); a draft whose repair repeats
an earlier attempt says the drafting stopped there, rather than naming a bound it did not reach
([Scripts and sentences in gonk](scripts/in-gonk.md)); and a mounted peer that cannot be reached
says how many dials failed and how long the next is held off ([Explain, with a mounted
model](browse/explain.md)). The `ikigai` the pages run moved with it, from 0.1.52 to 0.1.56. The
move before it (from 9f2e89e) gave the drafter `focus=` (its older `graph=` still works for one
release), read only the parts of the grounding a query needs, sent a draft whose preview is empty
back to the model with a warning naming the pattern that matches nothing and saved it MARKED if it
stayed empty, told a draft's author who runs it that it is a draft, and let `ikigai` read a pipe
into a `source` whose one required argument is unfilled, which is how that chapter hands the
owner's query to the check. The move before that (from 9704b54) took the ledger from 0.6.2 to
0.9.0, so the agent's tools gain `ledger-count__source` ([Over MCP](agent/mcp.md)) and part 5's
count can be asked of gonk itself ([In gonk](spec/ledger.md#in-gonk-and-not-in-this-page)); a
close also releases the item's claim, and the ledger's per-verb grants arrive as a fifth
`--ledger` authority, `worker` (claim and comment, never close), which the hint a client with no
grant prints now names ([A QUIC client](machines/quic-client.md)). No page here closes a claimed
item or hands one back.

Where a chapter needs a model, its kernel binds a **stub** at `urn:llm:` that answers the same thing
every time: no example in this book calls a real model or the network.

From [Over the socket](agent/socket.md) on, a page also runs `ikigai`, the command-line host,
as the other process that reaches gonk. It IS on crates.io, so it is pinned to one exact
version, in one file (`books/gonk/ikigai-cli.version`) that the check and CI read the same
way. It is the newest release, 0.1.56, one past the transports the pinned gonk was built with
(0.1.55): it speaks the same wire, and its own `ikigai-core` is the one that words a refusal at a
family scope the way gonk does, which [Over the socket](agent/socket.md) prints. ⚠ One difference
a reader with a real model can meet: a 0.1.56 client pings a QUIC door it is waiting on, and a
0.1.55 door does not answer those pings while it works, so a call that is silent for more than 15
seconds (a real model drafting or explaining) can be lost over `quic://`. The book's stub model
answers at once, and the socket is not affected; the difference closes when gonk takes the 0.1.56
transports.

<!-- transcript: manual — the check installs this version itself (scripts/test-transcripts.sh), reading the same file -->
```console
$ cargo install --locked ikigai-cli --version {{#include ../ikigai-cli.version}} --features quic
```

`--features quic` is for [A QUIC client](machines/quic-client.md), where `ikigai` reaches gonk
from "another machine"; without it the command line refuses a `quic://` address, and everything
before that chapter works either way.

Each [Ways of working](ways/solo.md) chapter ends with the [gonk
starter](https://github.com/ikigai-rs/ikigai-gonk-starter), a template repository whose
`tools/new.py` generates a repository set up for that process, and [Harness
engineering](ways/harness.md) uses the harness it generates. The starter is pinned the same way,
to one git revision in one file (`books/gonk/starter.rev`), and those pages clone it and check that
revision out themselves. Its last move was to a revision that carries the harness; in the
other six chapters that shows as `tools/harness.py` and the harness files among what is generated,
and one `harness` row in what `init` reports. It needs Python 3.11 or later and nothing else:

<!-- transcript: manual — the Ways of working pages run these two commands themselves, reading the same file -->
```console
$ git clone https://github.com/ikigai-rs/ikigai-gonk-starter
$ git -C ikigai-gonk-starter checkout {{#include ../starter.rev}}
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
on port 1070 whose config and data homes are inside the page's own directory (`--config-home`,
`--data-home`), and the check refuses any command that names 1062, a real gonk's port, before it
runs (or 1060, gonk's own until 2026-10-12).
