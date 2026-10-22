# Hub and spoke

One coordinating session, the **hub**, holds a goal that spans several repositories. It splits the
goal into **arcs**, one repository each, writes a brief for every arc, and dispatches each arc to
its own agent, a **spoke**, working in its own tree. A spoke does its arc, merges its own pull
request once CI is green, and reports with the evidence. The hub checks the evidence and closes
the arc. The hub's *own* changes are the exception: it asks a person before merging them.

[Remote workers](workers.md) handed well-specified items to machines. Hub and spoke is what sits in
front of that when the work is not yet items: the hub's job is the splitting, the ordering and the
checking, and a spoke may be a remote worker, an agent on the hub's own machine, or a person.

This chapter shows the pattern with gonk's primitives only: items, `parent` and `blocks` links,
claims with leases as the fence, and `next` ordering the arcs. The authors run this pattern with
private tooling of their own around it; none of that is needed, or shown, here.

## Who

| role | who | does |
| --- | --- | --- |
| the person | whoever owns the goal | sets it; answers the hub when it asks before merging its own work |
| the hub | one coordinating session (an agent, or a person) | splits, briefs, dispatches, checks evidence, closes |
| a spoke | one agent per arc, in its own working tree | claims one arc, does it, self-merges on green, hands back |
| CI | the forge | the gate a spoke's self-merge waits on |

A spoke may merge its own work because that authority was **delegated through the brief**: the
arc's item says so. A session that started some other way (by hand, from a suggestion) holds no
such brief, and stops at a green pull request and reports.

## What runs where

```text
                      ┌──────────────── gonk (the ledger) ────────────────┐
                      │  #1 goal (claimed by the hub)                     │
                      │   ├─ #2 arc: lib  ──blocks──▶ #3 arc: docs        │
                      │   └─ #3 arc: docs                                 │
                      └───────────▲──────────────▲──────────────▲─────────┘
                                  │              │              │
      hub (files, links, checks, closes)   spoke-lib          spoke-docs
                                           worktree of lib    worktree of docs
                                           PR ─▶ CI ─▶ merge  PR ─▶ CI ─▶ merge
```

This page runs one gonk and plays the hub and both spokes over its socket. Spokes on other
machines reach it over QUIC with identities of their own, exactly as in
[Remote workers](workers.md).

## The loop, step by step

In a new empty directory, as in [every chapter](../introduction.md#a-gonk-per-chapter): if the gonk
and the `model-stub` from [Review every commit](review-every-commit.md) are still running, stop them
first, since this gonk takes the same port.

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
```

### 1. The hub files the goal, and splits it into arcs

An arc is one repository's worth of the goal, small enough for one agent to finish and one pull
request to hold. Its item is the spoke's whole brief: what to do, in which repository, what done
means, what evidence to bring back, and whether the spoke may merge its own work.

<!-- transcript: file goal.txt -->
```text
initials handles every name

Two arcs: the library first, then its documentation.
```

<!-- transcript: file arc-lib.txt -->
```text
lib: initials skips empty words

Repository: lib. Done when initials("Ada  Lovelace") is "AL" and a test says so.
Self-merge on green CI; hand back with the pull request.
```

<!-- transcript: file arc-docs.txt -->
```text
docs: document initials

Repository: docs. One example, and one sentence on what an empty name returns.
Self-merge on green CI; hand back with the pull request.
```

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=goal' < goal.txt
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=arc' < arc-lib.txt
#2 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=arc' < arc-docs.txt
#3 urn:iki:ledger:default:item:…
[uncacheable]
```

Then the shape, as links: both arcs are children of the goal, and the library arc blocks the
documentation arc, because the docs describe what the library does once it is fixed. The hub
claims the goal itself, with no lease: it holds it until the goal is done.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:link item=2 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=3 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=2 type=blocks 3'
linked #2 parent #1
[uncacheable]
linked #3 parent #1
[uncacheable]
linked #2 blocks #3
[uncacheable]
— batch: 3 commands · 3 uncacheable
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:claim item=1 hub'
#1 claimed by hub
[uncacheable]
```

### 2. `next` orders the arcs

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
 1.    #2  open    p1  lib: initials skips empty words  [arc]
    p1 — priority 1; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 2
  not #1: claimed by hub
  not #3: blocked by #2
[uncacheable]
```

One arc is ready. The goal is the hub's, and the docs arc waits on the library arc, with the reason
in words. The hub does not have to remember the order: it is in the links, and any spoke that asks
`next` gets the same answer.

### 3. Dispatch: one spoke, one arc, one tree

The hub starts a spoke on the ready arc, in a working tree of its own (`git worktree add`), so that
two agents never share files. The spoke's first act is the claim, on a lease long enough for the
arc; the claim is the fence that keeps a second spoke off the same arc:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:claim item=2 lease=90m spoke-lib'
#2 claimed by spoke-lib until …
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:claim item=2 lease=90m spoke-docs'
error: conflict: #2 is already claimed by spoke-lib. Release it first (`delete urn:iki:ledger:claim item=2`), or take it up with them — a claim is a fence, and stealing one silently is how two workers end up in the same tree
```

Over the socket a holder is a name, so this fence holds among spokes that claim honestly; spokes
with identities of their own ([Remote workers](workers.md)) make it a fence no spoke can lift for
another.

### 4. The spoke works, and merges on green

<!-- transcript: manual — an agent, a forge and its CI, which this book's checks do not have -->
```console
$ git -C lib worktree add ../lib-arc-2 -b arc-2-empty-words
$ gh pr create --fill
https://github.com/you/lib/pull/12
$ gh pr checks 12 --watch
$ gh pr merge 12 --squash
```

The merge waits on CI and nothing else: the hub reviews when it checks the evidence, not as a gate
in front of every spoke's merge, which would put every arc back in a single line. Then the spoke
hands back:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:comment item=2 author=spoke-lib handback=true https://github.com/you/lib/pull/12 merged on green' -c 'delete urn:gk:iki:ledger:claim item=2 from=spoke-lib'
commented on #2 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
[uncacheable]
#2 released
[uncacheable]
— batch: 2 commands · 2 uncacheable
```

### 5. The hub checks, closes, and the next arc opens

The hub opens the pull request, sees it merged with CI green and the test in the diff, and closes
the arc with what it checked. Closing the blocker is what makes the next arc ready:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=2 reason=done author=hub Checked: lib PR 12 merged, CI green.'
closed #2 (done)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
 1.    #3  open    p1  docs: document initials  [arc]
    p1 — priority 1; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 1
  not #1: claimed by hub
[uncacheable]
```

The docs arc goes the same way: a spoke claims it, works in its own tree, merges on green, and
hands back; the hub checks and closes.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:claim item=3 lease=90m spoke-docs' -c 'sink urn:gk:iki:ledger:comment item=3 author=spoke-docs handback=true https://github.com/you/docs/pull/5 merged on green' -c 'delete urn:gk:iki:ledger:claim item=3 from=spoke-docs'
#3 claimed by spoke-docs until …
[uncacheable]
commented on #3 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
[uncacheable]
#3 released
[uncacheable]
— batch: 3 commands · 3 uncacheable
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=3 reason=done author=hub Checked: docs PR 5 merged, CI green.'
closed #3 (done)
[uncacheable]
```

### 6. The hub's own work: ask first

Some of the goal is the hub's to do itself: here, a line in the changelog saying what the arcs
changed. The hub files and claims it like any arc, and does it. But when its own pull request is
green it does **not** merge: a merge is the moment work becomes shared state for everyone else, and
the person who owns the goal gets one beat to redirect first. The hub hands its own item back with
the question:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=hub changelog: initials handles every name' -c 'sink urn:gk:iki:ledger:link item=4 type=parent 1' -c 'sink urn:gk:iki:ledger:claim item=4 hub'
#4 urn:iki:ledger:default:item:…
[uncacheable]
linked #4 parent #1
[uncacheable]
#4 claimed by hub
[uncacheable]
— batch: 3 commands · 3 uncacheable
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:comment item=4 author=hub handback=true https://github.com/you/lib/pull/13 is green. Merge?'
commented on #4 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:comment item=4 author=ada Yes, merge it.'
commented on #4 (urn:iki:ledger:default:comment:…) cleared-awaiting
[uncacheable]
```

The person's answer, from a caller who may close, clears the wait. The hub merges, closes its own
item, and with every child done, closes the goal:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=4 reason=done author=hub Merged lib PR 13 on the answer above.' -c 'sink urn:gk:iki:ledger:close item=1 reason=done author=hub All three children closed with evidence.'
closed #4 (done), released hub
[uncacheable]
closed #1 (done), released hub
[uncacheable]
— batch: 2 commands · 2 uncacheable
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:3'
   #3  closed  p1  docs: document initials  [arc]  (done)
  iri:      urn:iki:ledger:default:item:…
  filed:    …
  updated:  …
  parent:   urn:iki:ledger:default:item:…
  Repository: docs. One example, and one sentence on what an empty name returns.
  Self-merge on green CI; hand back with the pull request.
2 comment(s):
  … spoke-docs
    https://github.com/you/docs/pull/5 merged on green
  … hub
    Checked: docs PR 5 merged, CI green.
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:doctor'
doctor: default (lifecycle kata-flight): no problems
checked: orphaned, abandoned, lease-expired, unknown-state, two-states, awaiting-closed at …
[uncacheable]
```

Each arc's item is its whole record: the parent it belongs to, the brief, the spoke's evidence and
the hub's check, in that order. The close released the hub's own claims as it closed them, so the
doctor has nothing to report.

## What holds it

The **guides**:

- **The brief**, one per arc, in the item: the repository, what done means, the evidence to bring
  back, and the authority the spoke has (here, to merge its own pull request on green). It is the
  only channel from hub to spoke, so a vague brief is the commonest way for an arc to go wrong.
- **One repository per arc, one tree per spoke.** An arc that touches two repositories is two arcs
  with a `blocks` link between them.
- **The rules every spoke works to**: claim before writing, reproduce a reported bug first, every
  change through a pull request with green CI, hand back with evidence.
- **The hub asks before merging its own work.** Spokes self-merge because their brief says so; the
  hub's authority to merge its own work is not something it can grant itself.

The **sensors**:

- **Claims with leases**, the fence between spokes. A second claim is a `conflict` naming the
  holder; a spoke that dies stops renewing and shows up as `lease-expired` rather than as an arc
  quietly never finished.
- **`next` over the links**, which keeps a blocked arc from being started early and says why.
- **CI**, the only gate on a spoke's merge.
- **The hub's evidence check**, and the hand-back state that holds every finished arc out of
  `next` until the hub has answered it.
- **The doctor**, for claims nobody is working.

## What it costs, and when to choose it

The hub is a bottleneck by design: every arc's brief is written by it and every arc's evidence is
checked by it, so its attention bounds how many spokes can run at once. Briefs cost time, and a
brief that is wrong costs a spoke's whole arc. Spokes that self-merge put changes on main without
anyone's review before the merge; what makes that acceptable is CI that actually tests what
matters, and a hub that reads the evidence after.

Choose it when a goal spans repositories, when the arcs can run in parallel once ordered, and when
CI in each repository is trusted enough to be the gate. For one repository and one agent,
[Solo on one machine](solo.md) is cheaper; for work that is already items, a dispatcher and
[remote workers](workers.md) is enough.

## With the starter

The [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter)'s `hub` profile packages the
hub's two acts: `tools/hub.py arc` files an arc whose body is its brief (shaped by
`.gonk/templates/brief.md`) and links it `parent` and `blocks`, as step 1 did by hand, and
`tools/hub.py dispatch` gives an arc its own `git worktree` and holder, `arc-<item>`, as step 3
describes; `just arc` and `just dispatch` run them. `AGENTS.md` gains a "Hub and spoke" section,
beside the section 7 every generated repository carries (read its `ACCESSIBILITY.md` before
changing what people see).
The book checks the starter at the revision in `books/gonk/starter.rev` ([Solo](solo.md#with-the-starter)
says more):

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ cd starter && python3 tools/new.py ../library --process hub
generated the library-lending sample (python, process hub) in …/library: 62 files
next, in that directory:
  git init -b main, then commit everything it wrote (the one commit where every file is yours; -b main because the worker loop branches from origin/main, and git's own default is still master where init.defaultBranch is unset)
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
## 7. Read ACCESSIBILITY.md before changing what people see
## Specs, artifacts, queries and scripts
## Hub and spoke
## Optional: a hook for Claude Code
$ ls library/tools library/.gonk/templates
library/.gonk/templates:
brief.md

library/tools:
gonk.py
harness.py
hub.py
$ cat library/.gonk/process.toml
# How this repository's team works: the process profiles it was generated with, and what
# each one asks of it. Committed. Written by ikigai-gonk-starter's tools/new.py from its
# profiles/; `python3 tools/gonk.py init` and `status` report it and read nothing else to
# decide it. AGENTS.md holds the rules of exactly these profiles, so change both together.
version = 1
process = ["hub"]
hooks = ["claude:require-claim"]
checks = []

[profile.hub]
summary = "a hub splits work into single-repository arcs with written briefs, each dispatched to its own working tree and holder"
roles = ["hub: writes briefs as items, links them, dispatches each arc, closes on the evidence, asks a person before merging its own work", "arc agent: works one arc from its brief in its own working tree, merges on green, reports"]
```

<!-- transcript: run -->
```console
$ cd library && git init -q && python3 tools/gonk.py init --ledger default --socket ../.ikigai/gonk.sock --holder hub
gonk init: …/library
  ok       layout    openspec at openspec/; artifacts at artifacts/; queries at .gonk/queries/; scripts at .gonk/scripts/
  note     harness   4 harness question(s) in .gonk/queries/ are asked by `python3 tools/gonk.py harness` through gonk's store, not published: gonk runs a script under a ceiling of ledger reads, and they read the harness graph
  note     openspec  8 open task(s) in 2 change(s) in flight: `python3 tools/gonk.py seed` files them as ledger items (again is a no-op)
  ok       process   hub (.gonk/process.toml)
  note     hub       a hub splits work into single-repository arcs with written briefs, each dispatched to its own working tree and holder
  note     role      hub: writes briefs as items, links them, dispatches each arc, closes on the evidence, asks a person before merging its own work
  note     role      arc agent: works one arc from its brief in its own working tree, merges on green, reports
  note     hook      the Claude Code claim hook is optional and off: merge hooks/claude/settings.sample.json into .claude/settings.json to turn it on
  ok       ikigai    ikigai 0.1.67 at …/ikigai
  ok       ledger    `default` (.gonk/gonk.toml)
  ok       holder    `hub` (.gonk/local.toml, this working tree only)
  ok       identity  the socket's owner: every request over it is `owner`, so the log cannot tell this tree from any other process of yours. A machine that must be named uses QUIC
  ok       gonk      socket …/.ikigai/gonk.sock answers
  ok       items     `default` holds 4 item(s), 0 open
  ok       scripts   gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  ok       scripts   published 4: urn:script:default-open-items-for-artifact, urn:script:default-open-work-by-change, urn:script:default-requirements-with-open-work, urn:script:default-next-for-artifact
  absent   nl        gonk does not offer `urn:nl:sparql` yet, so a query is written by hand rather than drafted from a sentence (ikigai-nl); nothing to do until it does

agents over MCP (configure the agent with this command):
  ikigai mcp --override urn:iki:ledger:=…/.ikigai/gonk.sock

ready.
$ cd library && python3 tools/gonk.py status
process: hub
no live claim: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
```

The `absent nl` row is this page's gonk mounting no model, so it binds no `urn:nl:sparql` (its
banner says so); `init` says a query is written by hand until it does, and that there is
nothing to do about it here. The profile adds no
sensor the chapter lacks: the fence is still the claim, and the arc agent's merge on green and the
hub's question before merging its own work (step 6) are rules in the generated `AGENTS.md`, held
by the forge and by whoever reads them.

Profiles combine: a hub usually works over [remote workers](workers.md) or [spec-first](spec-first.md)
tasks, so `--process hub,workers,spec` is a list, not a new profile. Only [unled audits](audits.md)
and [review every commit](review-every-commit.md) refuse each other.
