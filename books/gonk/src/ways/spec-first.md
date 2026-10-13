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
