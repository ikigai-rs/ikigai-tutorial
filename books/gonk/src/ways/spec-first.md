# Spec first

Behavior is written down before it is built. A change starts as a proposal in the repository, with
the requirements it adds or modifies and a list of tasks; once that is agreed, the tasks become
ledger items in the order the list gives them, `next` hands them out one at a time, each task's box
is checked in the same pull request as its code, and when the last one is done the change is
archived and the spec says what the system now does.

The machinery is [A spec in the repository](../spec/openspec.md): OpenSpec's `openspec/` tree,
lifted to RDF, and a sync that files and closes items from it. This chapter is the process around
that machinery, run against gonk.

## Who

| role | who | does |
| --- | --- | --- |
| the proposer | a person, or an agent asked to draft | writes the change: proposal, delta spec, tasks |
| the approver | a person | merges the change's pull request, which is the agreement to build it |
| the seeder | a host or a script, over the socket | files one item per open task, keyed by the task, in order |
| a worker | an agent or a person | takes the next task, implements it, checks its box in the same pull request |
| whoever archives | the proposer, usually | moves the change to the archive once every box is checked |

## What runs where

```text
  repository: openspec/                         gonk
  ┌──────────────────────────────────┐          ┌─────────────────────────────────┐
  │ specs/       what the system does│          │ ledger: one item per open task  │
  │ changes/add-undo/                │  seed ──▶│   key  = the task's IRI         │
  │   proposal.md, specs/, tasks.md  │          │   about = the task, the req.    │
  │ changes/archive/…                │◀─ boxes ─│   1.2 ─blocks─▶ 1.3 ─blocks─▶ 2.1│
  └──────────────────────────────────┘  checked └─────────────────────────────────┘
                 ▲                        in the same PR as the code
                 └──── one pull request per task ────── a worker, from `next`
```

One machine, the socket. The lift that turns the tree into RDF is not in gonk
([In gonk, and not in this page](../spec/ledger.md#in-gonk-and-not-in-this-page)), so the seeding
here is the requests a seeder sends, written out; the Rust host in the spec part sends the same
ones from a query over the lifted tree.

## The loop, step by step

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
```

### 1. Propose, and agree

The change is a folder: why (`proposal.md`), what the requirements become (a delta spec under
`specs/`), and how, as tasks. A task names the requirement it implements in a trailing parenthesis,
the book's one convention beyond OpenSpec's ([OpenSpec in one chapter](../spec/openspec.md)). This
is the tic-tac-toe change the spec part uses:

<!-- transcript: run -->
```console
$ git init -q -b main game
$ mkdir -p game/openspec/changes/add-undo
```

<!-- transcript: file game/openspec/changes/add-undo/proposal.md -->
```markdown
# Add undo

## Why

A misclick ends a game early. Players should be able to take back the last move.
```

<!-- transcript: file game/openspec/changes/add-undo/tasks.md -->
```markdown
# Tasks

## 1. Undo

- [x] 1.1 Keep the game's moves in order, as the history undo reads
- [ ] 1.2 Empty the last move's square and return the turn (The last move can be undone)
- [ ] 1.3 Refuse undo once the game is over (The last move can be undone)

## 2. Turns

- [ ] 2.1 Give the turn back after an undo (Players alternate)
```

<!-- transcript: run -->
```console
$ git -C game add openspec
$ git -C game -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Propose add-undo'
```

The pull request that adds this folder is the agreement. It changes no code, so it is cheap to
argue about, and arguing about it is the point: a requirement changed here costs a sentence, and
the same change found after the code is written costs the code.

### 2. Seed the tasks as items

Every open task becomes an item, **keyed by the task's IRI** so that seeding again files nothing,
**about** the task and the requirement it implements, and labeled with the change. Task 1.1 is
already checked, so it is never filed.

<!-- transcript: run -->
```console
$ printf "add-undo 1.2: Empty the last move's square and return the turn" | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append key=urn:openspec:change:add-undo:task:1.2 about="urn:openspec:spec:moves:requirement:the-last-move-can-be-undone urn:openspec:change:add-undo:task:1.2" labels=openspec,add-undo'
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ printf "add-undo 1.3: Refuse undo once the game is over" | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append key=urn:openspec:change:add-undo:task:1.3 about="urn:openspec:spec:moves:requirement:the-last-move-can-be-undone urn:openspec:change:add-undo:task:1.3" labels=openspec,add-undo'
#2 urn:iki:ledger:default:item:…
[uncacheable]
$ printf "add-undo 2.1: Give the turn back after an undo" | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append key=urn:openspec:change:add-undo:task:2.1 about="urn:openspec:spec:moves:requirement:players-alternate urn:openspec:change:add-undo:task:2.1" labels=openspec,add-undo'
#3 urn:iki:ledger:default:item:…
[uncacheable]
$ printf "add-undo 1.2: Empty the last move's square and return the turn" | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append key=urn:openspec:change:add-undo:task:1.2 about="urn:openspec:spec:moves:requirement:the-last-move-can-be-undone urn:openspec:change:add-undo:task:1.2" labels=openspec,add-undo'
#1 urn:iki:ledger:default:item:… existing open
[uncacheable]
```

The fourth line is a seeder run twice: `existing`, and nothing written. Then the order, as `blocks`
links in the order `tasks.md` lists the tasks, so `next` offers them one at a time:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:link item=1 type=blocks 2' -c 'sink urn:gk:iki:ledger:link item=2 type=blocks 3'
linked #1 blocks #2
[uncacheable]
linked #2 blocks #3
[uncacheable]
— batch: 2 commands · 2 uncacheable
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
 1.    #1  open    p-  add-undo 1.2: Empty the last move's square and return the turn  [add-undo openspec]
    unprioritized — no priority set, so it ranks below every item that has one; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 2
  not #2: blocked by #1
  not #3: blocked by #2
[uncacheable]
```

Because each item names its requirement, the ledger can be asked by requirement: how much open
work implements "The last move can be undone"?

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:count about=urn:openspec:spec:moves:requirement:the-last-move-can-be-undone'
2
[uncacheable]
```

### 3. One task, one pull request, one checked box

A worker takes the next task as in [Solo on one machine](solo.md): claim, branch, implement. The
box is checked **in the same commit as the code**, so the tree and the code never disagree about
whether the task is done:

<!-- transcript: run -->
```console
$ git -C game switch -q -c add-undo-1.2
$ sed -i.bak 's/- \[ \] 1.2 /- [x] 1.2 /' game/openspec/changes/add-undo/tasks.md && rm game/openspec/changes/add-undo/tasks.md.bak
$ git -C game -c user.name=Agent -c user.email=agent@example.com commit -q -am 'add-undo 1.2: empty the last square, return the turn'
$ grep '1.2' game/openspec/changes/add-undo/tasks.md
- [x] 1.2 Empty the last move's square and return the turn (The last move can be undone)
```

(The code itself is not on this page: `add-undo` is the spec part's example change, and building
undo is not what this chapter is about.) When the pull request merges, the item closes with the reason. The key finds
it, so whatever closes it needs only the task:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=key:urn:openspec:change:add-undo:task:1.2 reason=done checked off in tasks.md'
closed #1 (done)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
 1.    #2  open    p-  add-undo 1.3: Refuse undo once the game is over  [add-undo openspec]
    unprioritized — no priority set, so it ranks below every item that has one; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 1
  not #3: blocked by #2
[uncacheable]
```

### 4. Keep the spec true

When the work shows the spec was wrong (undo after a win turns out to be wanted, say), the fix is a
change to the delta spec and to `tasks.md`, **in the pull request that discovers it**, and the
seeder runs again: a new task files a new item, a removed one is closed by hand with
`reason=wontfix` and why. The spec is never left describing a system nobody built.

### 5. Archive

When every box is checked the change is archived: its folder moves under `changes/archive/` with
the date (OpenSpec's `openspec archive` also folds the delta into `specs/`, which is then what the
system does). Here the last two tasks are done in one go, and the move is a commit of its own:

<!-- transcript: run -->
```console
$ sed -i.bak 's/- \[ \] /- [x] /' game/openspec/changes/add-undo/tasks.md && rm game/openspec/changes/add-undo/tasks.md.bak
$ git -C game -c user.name=Agent -c user.email=agent@example.com commit -q -am 'add-undo 1.3 and 2.1'
$ mkdir -p game/openspec/changes/archive
$ git -C game mv openspec/changes/add-undo openspec/changes/archive/2026-10-13-add-undo
$ git -C game -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Archive add-undo'
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=key:urn:openspec:change:add-undo:task:1.3 reason=done archived with add-undo on 2026-10-13' -c 'sink urn:gk:iki:ledger:close item=key:urn:openspec:change:add-undo:task:2.1 reason=done archived with add-undo on 2026-10-13'
closed #2 (done)
[uncacheable]
closed #3 (done)
[uncacheable]
— batch: 2 commands · 2 uncacheable
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
nothing is ready (0 open item(s) excluded: the ledger has no open items matching the filter)
policy: priority-recency
[uncacheable]
```

The items still say what they were about, by IRI, after the folder has moved: a later reader can
ask which requirement a piece of work implemented without the tree.

## What holds it

The **guides**: the spec itself, which says what the system does; the change's proposal and delta,
which say what it will do; `tasks.md`, which says in what order; and two conventions, a task names
its requirement, and a box is checked in the same pull request as its code.

The **sensors**:

- **The key**, so seeding is safe to repeat on every commit, and two seeders at once file once.
- **`blocks` and `next`**, so tasks are taken in the order the change gives them.
- **`about`**, so "what implements this requirement" and "which open item is about a requirement a
  change has since removed" are queries ([What `about` buys](../spec/ledger.md#what-about-buys)).
- **The archive**, which a sync over the lifted tree turns into closes (the spec part's walk does
  exactly that), so an item cannot outlive its change unnoticed.
- **CI**, as everywhere, on each task's pull request.

## What it costs, and when to choose it

A proposal and a delta spec are writing nobody does in the other processes, and a seeder is one
more thing to run. The spec has to be kept true by people, in pull requests: a spec that drifts is
worse than none, because agents read it as the truth. And the lift is not in gonk, so the seeding
and the closes are a host's or a script's to run.

Choose it when behavior matters more than speed: a public interface, a protocol, something several
people or agents will build against. It pairs well with [Hub and spoke](hub-and-spoke.md), where a
change's tasks are the arcs. For a one-line fix, file an item and fix it.

## With the starter

The [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter)'s `spec` profile adds a "Spec
first" section to `AGENTS.md` and nothing else, because what it needs is already in every
repository the starter generates: an `openspec/` tree (the library-lending sample's, with two
changes in flight) and `tools/gonk.py seed`, the seeder step 2 wrote out by hand. Spec first says
how work is *defined*, not who does it, so it has no roles of its own and is meant to be combined:
here with [Solo](solo.md), as `--process solo,spec`. The book checks the starter at the revision
in `books/gonk/starter.rev` ([Solo](solo.md#with-the-starter) says more):

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ cd starter && python3 tools/new.py ../library --process solo,spec
generated the library-lending sample (python, process solo, spec) in …/library: 59 files
next, in that directory:
  git init, then commit everything it wrote (the one commit where every file is yours)
  python3 scripts/test.py                      every test, as CI runs them
  python3 tools/gonk.py init --ledger <name> --socket ~/.ikigai/gonk.sock --holder <you>
  python3 tools/gonk.py seed                   file the open changes' tasks as items
  python3 tools/gonk.py next                   what to do first, and why

AGENTS.md holds this process's rules; README.md walks through the sample.
$ grep '^## ' library/AGENTS.md
## The loop
## 1. Claim the item before you write
## 2. One agent per claimed item, one agent per working tree
## 3. Reproduce before you fix
## 4. Every change through a pull request with green CI
## 5. Report with evidence; the dispatcher closes
## 6. Never a bare `#N`
## Specs, artifacts, queries and scripts
## Spec first
## Optional: a hook for Claude Code
$ cat library/.gonk/process.toml
# How this repository's team works: the process profiles it was generated with, and what
# each one asks of it. Committed. Written by ikigai-gonk-starter's tools/new.py from its
# profiles/; `python3 tools/gonk.py init` and `status` report it and read nothing else to
# decide it. AGENTS.md holds the rules of exactly these profiles, so change both together.
version = 1
process = ["solo", "spec"]
hooks = ["claude:require-claim"]
checks = []

[profile.solo]
summary = "one person and their agent: you are your own dispatcher, and close after checking your own evidence"
roles = ["you: both the dispatcher and whoever does the work"]

[profile.spec]
summary = "spec first: every change in behavior starts as an openspec change, seed files its tasks, next orders them"
roles = []
```

`seed --dry-run` reads the changes in flight and says what it would send, asking gonk nothing: one
item per open task, keyed by the task's IRI, and `blocks` links in `tasks.md` order. Run without
`--dry-run`, each item is also `about` the requirement its task cites and labeled with its change,
and a second run files nothing (the key answers `existing`).

<!-- transcript: run -->
```console
$ cd library && git init -q && python3 tools/gonk.py init --ledger default --socket ../.ikigai/gonk.sock --holder agent
gonk init: …/library
  ok       layout    openspec at openspec/; artifacts at artifacts/; queries at .gonk/queries/; scripts at .gonk/scripts/
  note     harness   4 harness question(s) in .gonk/queries/ are asked by `python3 tools/gonk.py harness` through gonk's store, not published: gonk runs a script under a ceiling of ledger reads, and they read the harness graph
  note     openspec  8 open task(s) in 2 change(s) in flight: `python3 tools/gonk.py seed` files them as ledger items (again is a no-op)
  ok       process   solo, spec (.gonk/process.toml)
  note     solo      one person and their agent: you are your own dispatcher, and close after checking your own evidence
  note     role      you: both the dispatcher and whoever does the work
  note     spec      spec first: every change in behavior starts as an openspec change, seed files its tasks, next orders them
  note     hook      the Claude Code claim hook is optional and off: merge hooks/claude/settings.sample.json into .claude/settings.json to turn it on
  ok       ikigai    ikigai 0.1.61 at …/ikigai
  ok       ledger    `default` (.gonk/gonk.toml)
  ok       holder    `agent` (.gonk/local.toml, this working tree only)
  ok       identity  the socket's owner: every request over it is `owner`, so the log cannot tell this tree from any other process of yours. A machine that must be named uses QUIC
  ok       gonk      socket …/.ikigai/gonk.sock answers
  ok       items     `default` holds 3 item(s), 0 open
  ok       scripts   gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  ok       scripts   published 4: urn:script:default-open-items-for-artifact, urn:script:default-open-work-by-change, urn:script:default-requirements-with-open-work, urn:script:default-next-for-artifact
  unknown  nl        could not ask gonk about `urn:nl:sparql`

agents over MCP (configure the agent with this command):
  ikigai mcp --override urn:iki:ledger:=…/.ikigai/gonk.sock

ready.
$ cd library && python3 tools/gonk.py seed --dry-run
would file  add-holds 1.1: Accept a hold on a title whose copies are all out
            key=urn:openspec:change:add-holds:task:1.1
would file  add-holds 1.2: Keep a returned copy for the first holder
            key=urn:openspec:change:add-holds:task:1.2
would file  add-holds 1.3: Refuse a renewal while a hold waits
            key=urn:openspec:change:add-holds:task:1.3
would file  add-holds 1.4: Add a vector to openspec/vectors.json for every new scenario
            key=urn:openspec:change:add-holds:task:1.4
would file  add-holds 2.1: Add `library.py hold MEMBER ISBN`
            key=urn:openspec:change:add-holds:task:2.1
would file  add-holds 2.2: Show who a kept copy waits for in `library.py copy`
            key=urn:openspec:change:add-holds:task:2.2
would file  cap-late-fees 1.1: Stop a late return's fee at 1000 cents
            key=urn:openspec:change:cap-late-fees:task:1.1
would file  cap-late-fees 1.2: Add a vector for the capped return to openspec/vectors.json
            key=urn:openspec:change:cap-late-fees:task:1.2
would link  add-holds 1.1 blocks 1.2
would link  add-holds 1.2 blocks 1.3
would link  add-holds 1.3 blocks 1.4
would link  add-holds 1.4 blocks 2.1
would link  add-holds 2.1 blocks 2.2
would link  cap-late-fees 1.1 blocks 1.2

seed --dry-run: 8 open task(s) would be filed, in 6 order link(s); 0 checked off
$ cd library && python3 tools/gonk.py status
process: solo, spec
no live claim: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
```

The `unknown` row is this page's gonk mounting no model, so it binds no `urn:nl:sparql` (its
banner says so); the starter reads that refusal as a gonk it could not ask. The two changes in
flight seed two chains, and `next` offers the head of each: two tasks that can go in parallel,
and the rest of each chain one at a time behind its head.

Any profile that says who does the work combines with this one (`--process workers,spec`,
`hub,spec`); only [unled audits](audits.md) and [review every commit](review-every-commit.md)
refuse each other.
