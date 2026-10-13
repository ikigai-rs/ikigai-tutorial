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
This page uses none of it, so that every step stays visible.

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
