# Remote workers

A dispatcher files work; unattended **worker** machines take it, do it and hand it back with
evidence; the dispatcher checks the evidence and closes. The difference from
[Solo on one machine](solo.md) is not the steps, which are the same, but who may take which of
them. A worker reaches gonk over QUIC with an identity of its own, and its grant lets it read,
claim, renew, comment and release. It **cannot close**, and it cannot merge: whatever it says
about its own work stays a claim until someone else checks it.

## Who

| role | who | holds |
| --- | --- | --- |
| the dispatcher | a person (or a coordinating agent) on the gonk machine | the owner's socket: everything, including close |
| a worker | a machine running an agent, unattended | a certificate, and the ledger's `worker` role: read, claim, comment |
| the forge | GitHub, say | the worker pushes and opens pull requests as a machine user that cannot merge |

## What runs where

```text
  gonk machine                                   worker machines (each its own bundle)
  ┌────────────────────────────────┐             ┌───────────────────────────────────┐
  │ dispatcher ── socket ──▶ gonk ◀┼── QUIC ─────┤ worker-1: next, claim, comment,   │
  │                         │      │  UDP 1070   │           release; a worktree per │
  │                         store  │             │           item, an agent in it    │
  └────────────────────────────────┘             ├───────────────────────────────────┤
                                                 │ worker-2: the same, its own cert  │
                                                 └───────────────────────────────────┘
          both sides ──▶ the forge: a branch and a pull request per item; CI
```

This page plays every machine on one, the way [A QUIC client](../machines/quic-client.md) does:
the server's homes are in the page's directory, and `worker-1/` and `worker-2/` stand in for the
two workers. On real machines only the address changes, and the bundle is moved, not copied.

## The loop, step by step

### 0. Once: an identity per worker

`client add … --worker <ledger>` mints a certificate and enrolls it under the ledger's worker role
(`--ledger default=worker` says the same):

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ ikigai-gonk client add worker-1 --worker default --port 1070 --config-home "$PWD/.config/ikigai"
client `worker-1`  …/.config/ikigai/gonk/quic/clients/worker-1
  fingerprint  …
  principal    urn:iki:host:gonk:client:…
  enrolled     grant `worker-1` (5 scopes) in …/.config/ikigai/gonk/grants.json
…
$ ikigai-gonk client add worker-2 --worker default --port 1070 --config-home "$PWD/.config/ikigai"
client `worker-2`  …/.config/ikigai/gonk/quic/clients/worker-2
…
  enrolled     grant `worker-2` (5 scopes) in …/.config/ikigai/gonk/grants.json
…
$ jq -r '.["worker-1"][]' .config/ikigai/gonk/grants.json
urn:cap:ledger:read:default
urn:cap:store:read:graph:urn:iki:ledger:graph:default
urn:cap:ledger:claim:default
urn:cap:ledger:comment:default
urn:cap:store:write:graph:urn:iki:ledger:graph:default
```

Read, claim and comment on one ledger; the store write token is the one the ledger makes its own
writes with, and gonk refuses a worker's raw write to the ledger's graph
([A QUIC client](../machines/quic-client.md#writing-the-graph-raw)). No `close`, no `write`, no
`delete`. Each worker gets its own identity, so the log, the claims and the comments can tell them
apart.

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  quic    udp 0.0.0.0:1070 — 2 trusted certificate(s), 2 enrolled
…
```

Each bundle goes to its worker (here, a copy into the directory that plays it):

<!-- transcript: run -->
```console
$ mkdir -p worker-1 worker-2
$ cp -R .config/ikigai/gonk/quic/clients/worker-1 worker-1/gonk
$ cp -R .config/ikigai/gonk/quic/clients/worker-2 worker-2/gonk
```

### 1. The dispatcher files, with a brief

An unattended worker has nothing to go on but the item, so the item carries the brief: what to
do, what done means, and what evidence to bring back. The first line is the title and the rest is
the body.

<!-- transcript: file brief-1.txt -->
```text
initials panics on an empty word

Done when initials("Ada  Lovelace") is "AL" and a test says so.
Evidence: the pull request, with CI green.
```

<!-- transcript: file brief-2.txt -->
```text
Document initials in the README

One example and one sentence on what an empty name returns.
Evidence: the pull request, with CI green.
```

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=worker' < brief-1.txt
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=2 labels=worker' < brief-2.txt
#2 urn:iki:ledger:default:item:…
[uncacheable]
```

(A content-less `sink` in a `-c` batch reads its content from standard input.)

### 2. Workers take work

Each worker asks `next` and claims the top item on a lease. The claim is the fence between them:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'source urn:iki:ledger:next'
 1.    #1  open    p1  initials panics on an empty word  [worker]
    p1 — priority 1; last updated …
 2.    #2  open    p2  Document initials in the README  [worker]
    p2 — priority 2; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 0
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=1 lease=30m worker-1'
#1 claimed by worker-1 until …
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'sink urn:iki:ledger:claim item=1 lease=30m worker-2'
error: conflict: #1 is already claimed by worker-1. Release it first (`delete urn:iki:ledger:claim item=1`), or take it up with them — a claim is a fence, and stealing one silently is how two workers end up in the same tree
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'delete urn:iki:ledger:claim item=1'
error: denied: #1 is held by worker-1 (as urn:iki:host:gonk:client:…), and this capability acts as `urn:iki:host:gonk:client:…`: a claim taken as a principal is released by that principal, or by a dispatcher holding `urn:cap:ledger:close:default` or `urn:cap:ledger:write:default` (the override). Naming the holder does not make the claim yours
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'source urn:iki:ledger:next'
 1.    #2  open    p2  Document initials in the README  [worker]
    p2 — priority 2; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 1
  not #1: claimed by worker-1
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'sink urn:iki:ledger:claim item=2 lease=30m worker-2'
#2 claimed by worker-2 until …
[uncacheable]
```

Two refusals, and they are different. The claim is a `conflict`: the item is held, and the answer
says by whom. The release is `denied`: a claim taken over QUIC records the claimant's
**principal**, so worker-2 cannot drop worker-1's claim by spelling its name. Over the socket in
the previous chapter there was no principal, and a holder was only a name.

### 3. The work, on the worker

<!-- transcript: manual — a worker machine running an agent, and a forge; this book's checks have neither -->
```console
$ git worktree add ../item-1 -b item-1-empty-word
$ cd ../item-1 && your-agent "$(ikigai --connect quic://gonk.local:1062 --cert-dir ~/gonk -c 'source urn:iki:ledger:item:1')"
$ git push -u origin item-1-empty-word && gh pr create --fill
```

One working tree per item, so two agents never share files; the item's text is the agent's brief;
the push and the pull request are made as a machine user the forge lets open pull requests and
not merge them. While the agent works, the worker renews its lease, which renews and never
takes:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=1 renew=true lease=30m worker-1'
#1 renewed by worker-1 until …
[uncacheable]
```

A worker that dies stops renewing. Its lease runs out, `next` says so, naming it, and the item is
not handed to anyone else until a deliberate takeover (`takeover=true from=worker-1`).

The starter repository, [`ikigai-gonk-starter`](https://github.com/ikigai-rs/ikigai-gonk-starter),
packages this loop as `tools/worker.py`, with the scripts that set up a worker machine's accounts.
The loop here uses none of it, so that every step stays visible; [With the
starter](#with-the-starter), at the end, shows what it generates.

### 4. Hand back with evidence

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:comment item=1 handback=true https://github.com/you/demo/pull/8 adds the test, CI green' -c 'delete urn:iki:ledger:claim item=1 from=worker-1'
commented on #1 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
[uncacheable]
#1 released
[uncacheable]
— batch: 2 commands · 2 uncacheable
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:close item=1 reason=done'
error: denied: this capability holds neither `urn:cap:ledger:close:default` nor `urn:cap:ledger:write:default`, and this resource takes either. A ledger grant names exactly one ledger, and a grant for another verb (a worker's claim and comment, say) does not reach this one
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'sink urn:iki:ledger:comment item=2 handback=true https://github.com/you/demo/pull/9 README example, CI green' -c 'delete urn:iki:ledger:claim item=2 from=worker-2'
commented on #2 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
[uncacheable]
#2 released
[uncacheable]
— batch: 2 commands · 2 uncacheable
```

The hand-back rides on the one write a worker has, a comment, and makes the item wait for its
dispatcher; the release lets go of the claim. The worker's attempt to close its own work is
refused at the grant. (The pull request URLs are the shape step 3 would print; this page has no
forge.)

### 5. The dispatcher checks, and closes or sends it back

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
nothing is ready (2 open item(s) excluded: #2 handed back, awaiting the dispatcher; #1 handed back, awaiting the dispatcher)
policy: priority-recency
[uncacheable]
```

Nothing is offered: both items are the dispatcher's. One pull request holds, and is closed; the
other is red on the main branch after it merged, so the dispatcher **rejects** the hand-back, with
the reason, and the item is ready again for whichever worker asks next:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=1 reason=done author=ada Checked: PR 8 merged, CI green, the test is in the diff.'
closed #1 (done)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:comment item=2 rejects=true author=ada PR 9 is red on main: the README example does not compile.'
commented on #2 (urn:iki:ledger:default:comment:…) cleared-awaiting
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
 1.    #2  open    p2  Document initials in the README  [worker]
    p2 — priority 2; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 0
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:2'
   #2  open    p2  Document initials in the README  [worker]
  iri:      urn:iki:ledger:default:item:…
  filed:    …
  updated:  …
  One example and one sentence on what an empty name returns.
  Evidence: the pull request, with CI green.
2 comment(s):
  … urn:iki:host:gonk:client:…
    https://github.com/you/demo/pull/9 README example, CI green
  … ada
    PR 9 is red on main: the README example does not compile.
[uncacheable]
```

The worker's comment is attributed to its certificate's principal, filled in by the QUIC door;
the dispatcher's `author=ada` is the owner's own word, because the socket proves nothing more.

### 6. What the server saw

<!-- transcript: run -->
```console
$ grep -E 'door=quic .*outcome=(conflict|denied)' gonk.log
… gonk:Access urn:iki:ledger:claim door=quic verb=sink outcome=conflict bytes=- dur=… principal=urn:iki:host:gonk:client:… q=-
… gonk:Access urn:iki:ledger:claim door=quic verb=delete outcome=denied bytes=- dur=… principal=urn:iki:host:gonk:client:… q=-
… gonk:Access urn:iki:ledger:close door=quic verb=sink outcome=denied bytes=- dur=… principal=urn:iki:host:gonk:client:… q=-
```

Every refusal above, each with the principal that asked.

## What holds it

The **guides**: the brief in each item, which is all an unattended agent knows about what to do;
the rules the agent works to (claim before writing, one working tree per item, reproduce before
fixing, report with evidence), in the repository it works in; and the forge's branch rules.

The **sensors**, and here they are enforced rather than habits:

- **The grant.** A worker cannot close, relabel, file or delete, and the door refuses it before
  the ledger is asked.
- **Claims bound to a principal.** One worker cannot release or renew another's claim; a takeover
  works only on an expired lease, and only as a compare-and-set against the holder it names.
- **The lease**, which turns a dead worker into a visible `lease-expired` rather than a silently
  stuck item.
- **The hand-back state**, which keeps finished work out of `next` until a dispatcher answers.
- **CI and the forge's merge rule**: the worker's machine user can open a pull request and not
  merge it.
- **The dispatcher's evidence check**, now a separate party from the worker, and the reject that
  sends work back with a reason.
- **The access log**, which names every worker on every request it made.

## What it costs, and when to choose it

Each worker is a certificate to mint, a bundle to move and rotate, a machine to keep, and an
agent's model calls to pay for; and the dispatcher's evidence check becomes the bottleneck,
because every item waits for it. A worker that misreads its brief spends a lease's worth of time
before anyone notices.

Choose it when there is more work than one machine's attention, when the work is well enough
specified to be done from a brief, and when you want the machine that writes code to be unable to
accept its own work. If the work is not yet specified that well, a coordinating session in front
of the workers is the next chapter, [Hub and spoke](hub-and-spoke.md).

## With the starter

The [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter)'s `workers` profile is this
chapter packaged: `tools/new.py --process workers` adds a "Worker machines" section to the
repository's `AGENTS.md`, carries `tools/worker.py` (the claim, work, report loop above, run
unattended) with its configuration's template and its service units, and the `just worker`
recipes that run it. The book checks the starter at the revision in `books/gonk/starter.rev`
([Solo](solo.md#with-the-starter) says more):

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ cd starter && python3 tools/new.py ../library --process workers
generated the library-lending sample (python, process workers) in …/library: 63 files
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
## Worker machines
## Optional: a hook for Claude Code
$ ls library/tools library/worker
library/tools:
gonk.py
harness.py
worker.py

library/worker:
units
worker.sample.toml
$ cat library/.gonk/process.toml
# How this repository's team works: the process profiles it was generated with, and what
# each one asks of it. Committed. Written by ikigai-gonk-starter's tools/new.py from its
# profiles/; `python3 tools/gonk.py init` and `status` report it and read nothing else to
# decide it. AGENTS.md holds the rules of exactly these profiles, so change both together.
version = 1
process = ["workers"]
hooks = ["claude:require-claim"]
checks = ["worker-config"]

[profile.workers]
summary = "a dispatcher and unattended worker machines: workers claim and report, and never close or merge"
roles = ["dispatcher: marks items for workers, checks each hand-back, merges and closes, or rejects", "worker: an unattended machine running tools/worker.py; claims, works, reports"]
```

On a worker machine, `init` takes the QUIC door and the bundle `client add` wrote, here
`worker-1`'s from step 0, and asks gonk as that worker:

<!-- transcript: run -->
```console
$ cd library && git init -q && python3 tools/gonk.py init --ledger default --quic quic://127.0.0.1:1070 --cert-dir ../worker-1/gonk --holder worker-1
gonk init: …/library
  ok       layout    openspec at openspec/; artifacts at artifacts/; queries at .gonk/queries/; scripts at .gonk/scripts/
  note     harness   4 harness question(s) in .gonk/queries/ are asked by `python3 tools/gonk.py harness` through gonk's store, not published: gonk runs a script under a ceiling of ledger reads, and they read the harness graph
  note     openspec  8 open task(s) in 2 change(s) in flight: `python3 tools/gonk.py seed` files them as ledger items (again is a no-op)
  ok       process   workers (.gonk/process.toml)
  note     workers   a dispatcher and unattended worker machines: workers claim and report, and never close or merge
  note     role      dispatcher: marks items for workers, checks each hand-back, merges and closes, or rejects
  note     role      worker: an unattended machine running tools/worker.py; claims, works, reports
  note     hook      the Claude Code claim hook is optional and off: merge hooks/claude/settings.sample.json into .claude/settings.json to turn it on
  note     worker    no .gonk/worker.toml, so this machine runs no worker: a worker machine copies worker/worker.sample.toml there (`python3 tools/worker.py plan` checks it)
  ok       ikigai    ikigai 0.1.61 at …/ikigai
  ok       ledger    `default` (.gonk/gonk.toml)
  ok       holder    `worker-1` (.gonk/local.toml, this working tree only)
  ok       identity  urn:iki:host:gonk:client:… (from …/worker-1/gonk/client.crt)
  ok       alias     older records name it urn:iki:gonk:client:…: gonk reads that spelling as this machine too, and never mints it again
  ok       gonk      quic quic://127.0.0.1:1070 (certificates in …/worker-1/gonk) answers
  ok       items     `default` holds 2 item(s), 1 open
  ok       scripts   gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  failed   scripts   urn:script:default-open-items-for-artifact was refused: error: denied: capability does not grant `urn:cap:script:write:*` (declared by `urn:script:default-open-items-for-artifact`; any grant under `urn:cap:script:write:` satisfies it)
  failed   scripts   urn:script:default-open-work-by-change was refused: error: denied: capability does not grant `urn:cap:script:write:*` (declared by `urn:script:default-open-work-by-change`; any grant under `urn:cap:script:write:` satisfies it)
  failed   scripts   urn:script:default-requirements-with-open-work was refused: error: denied: capability does not grant `urn:cap:script:write:*` (declared by `urn:script:default-requirements-with-open-work`; any grant under `urn:cap:script:write:` satisfies it)
  failed   scripts   urn:script:default-next-for-artifact was refused: error: denied: capability does not grant `urn:cap:script:write:*` (declared by `urn:script:default-next-for-artifact`; any grant under `urn:cap:script:write:` satisfies it)
  unknown  nl        could not ask gonk about `urn:nl:sparql`

this machine's QUIC identity, for the gonk operator to enroll (once):
…
agents over MCP (configure the agent with this command):
  ikigai mcp --override urn:iki:ledger:=quic://127.0.0.1:1070 --cert-dir …/worker-1/gonk

ready.
$ cd library && python3 tools/gonk.py status
process: workers
no live claim: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
```

The identity row is the name gonk's access log gives this machine, and the lines elided after it
are the `client add` and `grants.json` an operator would use to enroll it, which step 0 already
did. The `failed` rows are the worker grant at work: `init` offers to publish the sample's saved
queries, and a worker may not write scripts (the dispatcher's own `init`, as the owner over the
socket, publishes them, as [Solo](solo.md#with-the-starter) shows). The `unknown` row is this
page's gonk mounting no model: it binds no `urn:nl:sparql` (its banner says so), and the starter
reads that refusal as a gonk it could not ask. The `worker` row is what a worker machine still
needs, its `.gonk/worker.toml`; setting that machine up (its accounts, the boundary its agent runs
inside) is the starter's `worker/README.md`.

Profiles combine (`--process workers,spec` adds [spec first](spec-first.md)'s rules, and no
`--process` at all means `solo,workers,spec`); only [unled audits](audits.md) and [review every
commit](review-every-commit.md) refuse each other.
