# Remote workers

A dispatcher files work; unattended **worker** machines take it, do it and hand it back with
evidence; the dispatcher checks the evidence and closes. The difference from
[Solo on one machine](solo.md) is not the steps, which are the same, but who may take which of
them. A worker reaches gonk over QUIC with an identity of its own, and its grant lets it read,
claim, renew, comment and release. It **cannot close**. The forge must also prevent its machine
user from merging: whatever the worker says about its own work stays a claim until someone else
checks it.

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

In a new empty directory, as in [every chapter](../introduction.md#a-gonk-per-chapter): if the gonk
and the `model-stub` from [Solo on one machine](solo.md) are still running, stop them first, since
this page's gonk takes the same port (it needs no model).

`client add … --worker <ledger>` mints a certificate and enrolls it under the ledger's worker role
(`--ledger default=worker` says the same):

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ ikigai-gonk client add worker-1 --worker default --port 1070 --config-home "$PWD/.config/ikigai"
client `worker-1`  …/.config/ikigai/gonk/quic/clients/worker-1
  fingerprint  …
  principal    urn:iki:host:gonk:client:…
  enrolled     grant `worker-1` (9 scopes) in …/.config/ikigai/gonk/grants.json
…
$ ikigai-gonk client add worker-2 --worker default --port 1070 --config-home "$PWD/.config/ikigai"
client `worker-2`  …/.config/ikigai/gonk/quic/clients/worker-2
…
  enrolled     grant `worker-2` (9 scopes) in …/.config/ikigai/gonk/grants.json
…
$ jq -r '.["worker-1"][]' .config/ikigai/gonk/grants.json
urn:cap:ledger:read:default
urn:cap:store:read:graph:urn:iki:ledger:graph:default
urn:cap:store:read:graph:urn:iki:ledger:graph:default:session
urn:cap:ledger:claim:default
urn:cap:ledger:comment:default
urn:cap:ledger:session:default
urn:cap:store:write:graph:urn:iki:ledger:graph:default
urn:cap:store:write:graph:urn:iki:ledger:graph:default:session
urn:cap:store:read:graph:urn:collab:participants
```

Read, claim and comment on one ledger, and record a session's summary on the worker's own
hand-back. The `session` line is new in gonk PR 366, and the two `…:session` store tokens are new in
gonk PR 385 (ikigai-ledger 0.23.0): the summaries live in a graph of their own, and these are its
read and write ([Where the summaries live](#where-the-summaries-live)). A worker grant written before
then lacks those lines; `ikigai-gonk client refresh --write` adds them without touching anything else
([A QUIC client](../machines/quic-client.md#a-grant-from-an-older-gonk-client-refresh)). The other store write token is the one the ledger makes its own
writes with, and gonk refuses a worker's raw write to the ledger's graph
([A QUIC client](../machines/quic-client.md#writing-the-graph-raw)). No `close`, no `write`, no
`delete`. Each worker gets its own identity, so the log, the claims and the comments can tell them
apart.

One more identity, for whoever watches how the workers' runs go: it reads the ledger's session
summaries and nothing else. No role names that grant, so it is written with `--scope`, twice: the
ledger's `summaries` grant (ikigai-ledger 0.22.0) and the store read of the ledger's session graph,
`urn:iki:ledger:graph:default:session`, where the summaries are kept (ikigai-ledger 0.23.0):

<!-- transcript: run -->
```console
$ ikigai-gonk client add metrics --scope urn:cap:ledger:summaries:default --scope urn:cap:store:read:graph:urn:iki:ledger:graph:default:session --port 1070 --config-home "$PWD/.config/ikigai"
client `metrics`  …/.config/ikigai/gonk/quic/clients/metrics
…
  enrolled     grant `metrics` (2 scopes) in …/.config/ikigai/gonk/grants.json
…
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
  quic    udp 0.0.0.0:1070 — 3 trusted certificate(s), 3 enrolled
…
```

Each bundle goes to its worker (here, a copy into the directory that plays it):

<!-- transcript: run -->
```console
$ mkdir -p worker-1 worker-2 metrics
$ cp -R .config/ikigai/gonk/quic/clients/worker-1 worker-1/gonk
$ cp -R .config/ikigai/gonk/quic/clients/worker-2 worker-2/gonk
$ cp -R .config/ikigai/gonk/quic/clients/metrics metrics/gonk
```

### 1. The dispatcher files, with a brief

An unattended worker has nothing to go on but the item, so the item carries the brief: what to
do, what done means, and what evidence to bring back. The first line is the title and the rest is
the body. The `worker` label below marks the items for this page's two workers, which ask `next`
by hand; it is a convenience, not a route. A label records nobody who set it, and anyone who may
write to the ledger can add one, so the starter's unattended loop takes an item only when a
**router** it trusts has dispatched it, in a comment gonk attributes ([Routing work to
agents](routing.md), the next chapter).

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

Each worker asks `next` and claims the top item on a lease. The claim is the fence between them.
Its epoch identifies this take; the starter presents that epoch on subsequent writes, so a
worker holding an older take cannot write after the claim changes hands:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'source urn:iki:ledger:next'
 1.    #1  open    priority 1  initials panics on an empty word  [worker]
    p1 — priority 1; last updated …
 2.    #2  open    priority 2  Document initials in the README  [worker]
    p2 — priority 2; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 0
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=1 lease=30m worker-1'
#1 claimed by worker-1 at epoch 1 until …
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'sink urn:iki:ledger:claim item=1 lease=30m worker-2'
error: conflict: #1 is already claimed by worker-1. Release it first (`delete urn:iki:ledger:claim item=1`), or take it up with them — a claim is a fence, and stealing one silently is how two workers end up in the same tree
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'delete urn:iki:ledger:claim item=1'
error: denied: #1 is held by worker-1 (as urn:iki:host:gonk:client:…), and this capability acts as `urn:iki:host:gonk:client:…`: a claim taken as a principal is released by that principal, or by a dispatcher holding `urn:cap:ledger:close:default` or `urn:cap:ledger:write:default` (the override). Naming the holder does not make the claim yours
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'source urn:iki:ledger:next'
 1.    #2  open    priority 2  Document initials in the README  [worker]
    p2 — priority 2; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 1
  not #1: claimed by worker-1
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'sink urn:iki:ledger:claim item=2 lease=30m worker-2'
#2 claimed by worker-2 at epoch 1 until …
[uncacheable]
```

Two refusals, and they are different. The claim is a `conflict`: the item is held, and the answer
says by whom. The release is `denied`: a claim taken over QUIC records the claimant's
**principal**, so worker-2 cannot drop worker-1's claim by spelling its name. Over the socket in
[Solo on one machine](solo.md), every caller was the owner, so the holder name alone distinguished
one worker from another.

### 3. The worker does the work

On a real worker machine, start in a clone of the repository you want the agent to change. The
next commands use `gonk.local:1062` for the real server and `~/gonk` for that worker's certificate
bundle; replace both with your server's address and the bundle's location. `your-agent` stands
for your agent's command. These commands need the real repository and forge; the scratch setup
above checks only the ledger exchanges.

<!-- transcript: manual — a worker machine running an agent, and a forge; this book's checks have neither -->
```console
$ git worktree add ../item-1 -b item-1-empty-word
$ cd ../item-1 && your-agent "$(ikigai --connect quic://gonk.local:1062 --cert-dir ~/gonk -c 'source urn:iki:ledger:item:1')"
$ git push -u origin item-1-empty-word && gh pr create --fill
```

One working tree per item, so two agents never share files; the item's title and body are the
agent's brief; the push and the pull request are made as a machine user the forge lets open pull
requests and not merge them. The command above hands the agent everything `item:1` prints,
comments included, which is fine for a page whose ledger only you write to and is the hazard on a
shared one: a comment is whatever its writer typed. [Routing work to agents](routing.md) shows
how the starter's loop gives an agent the comments as quoted data, under their authors, instead.

While the agent works, the worker renews its lease. `renew=true` renews and never takes:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=1 renew=true lease=30m worker-1'
#1 renewed by worker-1 at epoch 1 until …
[uncacheable]
```

A worker that dies stops renewing. Its lease runs out, `next` says so, naming it, and the item is
not handed to anyone else until a deliberate takeover (`takeover=true from=worker-1`).

The starter repository, [`ikigai-gonk-starter`](https://github.com/ikigai-rs/ikigai-gonk-starter),
packages this loop as `tools/worker.py`, with the scripts that set up a worker machine's accounts.
The loop here uses none of it, so that every step stays visible; [With the
starter](#with-the-starter), at the end, shows what it generates, and [Routing work to
agents](routing.md) runs it: the dispatch it waits for, and one loop per agent.

### 4. The worker hands back, with evidence

The starter runs Claude Code with `--output-format json` and Codex with `exec --json`, and
reads that output for a **session summary**: the session id, machine name, harness, model,
exit status, wall-clock time, turns, tool-call count and tokens by category (input, cached
input, cache creation and output, where the harness reports them). It sends no cost, prompt,
transcript text, tool result or file content. The agent's words stay in the worker's local log.

After posting the hand-back, the starter records the summary on
`urn:iki:ledger:<ledger>:item:<N>:sessions`. Its hand-back ends with
`Session summary: <harness> session <id> (<model>, exit <N>), sent to the item's sessions after this hand-back.`
If it cannot make a summary, the last line is `No session summary: <why>.` A missing or
unparseable result, an unsupported harness or a run stopped at its budget leaves no summary.
Failure to make a summary does not itself fail the run. The summary is checked against the ledger's shape before it is
sent; a refusal to record it is logged and leaves the hand-back standing. Recording needs
ikigai-ledger 0.22.0 or newer, which the gonk this book pins links, and `urn:cap:ledger:session:<ledger>`
in the worker's grant, which step 0 showed.

This chapter runs the ledger exchanges by hand, without an agent harness. worker-1 plays the
starter's part in full: its hand-back ends with the starter's line, and it then sends a summary of
the kind the starter would make (the numbers are an example; no agent ran). worker-2's hand-back says
why it has none. Save the two hand-backs and the summary:

<!-- transcript: file handback-1.txt -->
```text
https://github.com/you/demo/pull/8 adds the test, CI green
Session summary: claude session 0b5e41aa (claude-opus-5-5, exit 0), sent to the item's sessions after this hand-back.
```

<!-- transcript: file session-1.json -->
```json
{"session": "0b5e41aa", "machineName": "worker-1", "harness": "claude", "model": "claude-opus-5-5",
 "exitStatus": 0, "wallClockMs": 1830000, "turns": 41, "toolCalls": 118, "inputTokens": 5120,
 "cachedInputTokens": 880000, "cacheCreationTokens": 64000, "outputTokens": 21000}
```

<!-- transcript: file handback-2.txt -->
```text
https://github.com/you/demo/pull/9 README example, CI green
No session summary: this chapter runs ledger commands by hand.
```

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:comment item=1 handback=true' -c 'delete urn:iki:ledger:claim item=1 from=worker-1' < handback-1.txt
commented on #1 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
[uncacheable]
#1 released
[uncacheable]
— batch: 2 commands · request outcomes: 2 uncacheable
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:item:1:sessions' < session-1.json
recorded session 0b5e41aa on #1's hand-back
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:close item=1 reason=done'
error: denied: this capability holds neither `urn:cap:ledger:close:default` nor `urn:cap:ledger:write:default`, and this resource takes either. A ledger grant names exactly one ledger, and a grant for another verb (a worker's claim and comment, say) does not reach this one
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'sink urn:iki:ledger:comment item=2 handback=true' -c 'delete urn:iki:ledger:claim item=2 from=worker-2' < handback-2.txt
commented on #2 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
[uncacheable]
#2 released
[uncacheable]
— batch: 2 commands · request outcomes: 2 uncacheable
```

The worker marks its comment as a hand-back, which makes the item wait for its dispatcher;
the release lets go of the claim. The summary is a separate request after the hand-back, and it is
recorded on the item's newest hand-back, so a refused summary would leave the hand-back standing. The worker's attempt to close its own work is refused at the
grant. (The pull request URLs are the shape step 3 would print; this page has no
forge.)

### 5. The dispatcher checks, and closes or sends it back

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
nothing is ready (2 open items excluded: #2 handed back, awaiting the dispatcher; #1 handed back, awaiting the dispatcher)
policy: priority-recency
[uncacheable]
```

Nothing is offered: both items are the dispatcher's. One pull request holds, and is closed; the
other is red on the main branch after it merged, so the dispatcher **rejects** the hand-back, with
the reason, and the item is ready again for whichever worker asks next:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=1 reason=done evidence=https://github.com/you/demo/pull/8 author=ada Checked: PR 8 merged, CI green, the test is in the diff.'
closed #1 done (1 evidence)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:comment item=2 rejects=true author=ada PR 9 is red on main: the README example does not compile.'
commented on #2 (urn:iki:ledger:default:comment:…) cleared-awaiting
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
 1.    #2  open    priority 2  Document initials in the README  [worker]
    p2 — priority 2; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 0
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:2'
   #2  open    priority 2  Document initials in the README  [worker]
  iri:      urn:iki:ledger:default:item:…
  filed:    …
  updated:  …
  One example and one sentence on what an empty name returns.
  Evidence: the pull request, with CI green.
2 comment(s):
  … urn:iki:host:gonk:client:…
    https://github.com/you/demo/pull/9 README example, CI green
    No session summary: this chapter runs ledger commands by hand.
  comment by ada (claimed, free text) at …:
  rejects: urn:iki:ledger:default:comment:…
    PR 9 is red on main: the README example does not compile.
[uncacheable]
```

The worker's comment is attributed to its certificate's principal, filled in by the QUIC door;
the dispatcher's `author=ada` is the owner's own word, because the socket proves nothing more.

The close's `evidence=` keeps the pull request on the close itself, and it is not fetched, so the
check is still the dispatcher's. An item's history shows who did what, in order, and it is the
shortest way to review a worker's run:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:1:history'
#1: 3 event(s)
…  claim worker-1  by urn:iki:host:gonk:client:…
…  release worker-1 (released)  by urn:iki:host:gonk:client:…
…  closed done, evidence https://github.com/you/demo/pull/8
[uncacheable]
```

The claim and the release were made over QUIC, so each ends with the principal the door verified. The
close was made over the socket, which has no name to give.

The ledger keeps every summary as one set too, `sessions`, newest first and paged (ikigai-ledger
0.22.0). It names each item by its number and IRI and never by its title, and it totals and prices
nothing. `metrics` holds the summaries grant and no other ledger grant, so it reads the set and is
refused the item the set names:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir metrics/gonk -c 'source urn:iki:ledger:sessions'
default: sessions, newest first, from offset 0
session 0b5e41aa recorded …
  item: #1 urn:iki:ledger:default:item:…
  harness: claude, model: claude-opus-5-5, machine: worker-1
  exit status: 0, wall clock: 1830000 ms
  turns: 41, tool calls: 118
  tokens: input 5120, cached input 880000, cache creation 64000, output 21000
  hand-back: urn:iki:ledger:default:comment:…
  iri: urn:agent:session:id:0b5e41aa
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir metrics/gonk -c 'source urn:iki:ledger:item:1'
error: denied: capability does not grant `urn:cap:ledger:read:*` (declared by `urn:iki:ledger:item:1`; any grant under `urn:cap:ledger:read:` satisfies it)
```

#### Where the summaries live

Since ikigai-ledger 0.23.0 (gonk PR 385) a ledger's summaries are in a named graph of their own,
`urn:iki:ledger:graph:default:session`, and nothing else is in it. A summary names its item by
number and IRI, so the set needs no read of the ledger's graph, where every title is. That is why
`metrics`'s one store token is the session graph's read: it reads the summaries, and no store read
it holds reaches a title.

Before 0.23.0 the summaries grant paired with the store read of the ledger's graph, which reads
every title, so `summaries` was then no boundary against a client querying the store directly. A worker records a summary under the session graph's store read and write tokens,
the two `…:session` lines in step 0's grant. Under a grant written before 0.23.0 the hand-back still
goes through and the summary is refused, naming both tokens;
[A QUIC client](../machines/quic-client.md#a-grant-from-an-older-gonk-client-refresh) shows that
refusal and `client refresh`, which adds them.

Summaries an older gonk recorded are in the ledger's graph, where the summaries grant no longer
looks, and nothing moves them at startup. The owner moves them once, through the socket. `plan`
lists them and writes nothing, and `move`, sent the same way, moves them. This page's gonk was never
older, so there is nothing to move:

<!-- transcript: run -->
```console
$ printf plan | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:default:sessions'
default: no session summaries in urn:iki:ledger:graph:default to move; nothing written
[uncacheable]
```

A move takes the ledger's write grant and the store read and write of both graphs, which the owner
holds. It writes each batch to the session graph before removing it from the ledger's graph, so an
interrupted move leaves a summary in both graphs, and running it again finishes the move.

[A lifecycle for your ledger](../ledger/lifecycle.md#what-happened-to-an-item) covers the rest of
what a history holds.

<a id="6-what-the-server-saw"></a>

### What the server saw

<!-- transcript: run -->
```console
$ grep -E 'door=quic .*outcome=(conflict|denied)' .config/ikigai/gonk/log/*.log
… log:Access urn:iki:ledger:claim seq=… door=quic verb=sink outcome=conflict dur=… principal=urn:iki:host:gonk:client:… credential=https://ikigai-rs.dev/ns/log#clientCertificate
… log:Access urn:iki:ledger:claim seq=… door=quic verb=delete outcome=denied dur=… principal=urn:iki:host:gonk:client:… credential=https://ikigai-rs.dev/ns/log#clientCertificate
… log:Access urn:iki:ledger:close seq=… door=quic verb=sink outcome=denied dur=… principal=urn:iki:host:gonk:client:… credential=https://ikigai-rs.dev/ns/log#clientCertificate
```

Every refusal above, each with the principal that asked.

## What holds it

The **guides**: the brief in each item, which is all an unattended agent knows about what to do;
the rules the agent works to (claim before writing, one working tree per item, reproduce before
fixing, report with evidence), in the repository it works in; and the forge's branch rules.

The **sensors**, and here they are enforced rather than habits:

- **Grant.** A worker cannot close, relabel, file or delete, and the door refuses it before
  the ledger is asked.
- **Claims bound to a principal.** One worker cannot release or renew another's claim; a takeover
  works only on an expired lease, and only as a compare-and-set against the holder it names.
- **Lease**, which turns a dead worker into a visible `lease-expired` rather than a silently
  stuck item.
- **Hand-back state**, which keeps finished work out of `next` until a dispatcher answers.
- **CI and the forge's merge rule**: the worker's machine user can open a pull request and not
  merge it.
- **Dispatcher's evidence check.** Someone other than the worker checks the evidence and
  rejects a hand-back that does not hold, sending work back with a reason.
- **Access log**, which names every worker on every request it made.

## What it costs, and when to choose it

Each worker is a certificate to mint, a bundle to move and rotate, a machine to keep, and an
agent's model calls to pay for; and the dispatcher's evidence check becomes the bottleneck,
because every item waits for it. A worker that misreads its brief spends a lease's worth of time
before anyone notices.

Choose it when there is more work than one machine's attention, when the work is well enough
specified to be done from a brief, and when you want the machine that writes code to be unable to
accept its own work. If the work is not yet specified that well, a coordinating session in front
of the workers is [Hub and spoke](hub-and-spoke.md), three chapters on.

## With the starter

The [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter)'s `workers` profile is this
chapter packaged: `tools/new.py --process workers` adds a "Worker machines" section to the
repository's `AGENTS.md`, carries `tools/worker.py` (the claim, work, report loop above, run
unattended) with its four helpers, `tools/worker_repos.py` (which enrolls an owner's repositories
in the worker's configuration and checks the worker's token may push to them),
`tools/worker_usage.py` (which reads an agent's usage against its plan's weekly limit between
items), `tools/worker_session.py` (which reads the session summary) and
`tools/worker_onboard.py` (which enrolls a new machine with a pairing code), its configuration's template and its service units, and the `just worker` recipes that
run it. The book checks the starter at the revision in `books/gonk/starter.rev`
([Solo](solo.md#with-the-starter) says more):

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ cd starter && python3 tools/new.py ../library --process workers
generated the library-lending sample (python, process workers) in …/library: 106 files
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
## Building in the ikigai style (optional)
## Writing text people skim (optional)
## Worker machines
## Optional: a hook for Claude Code
$ LC_ALL=C ls library/tools library/worker
library/tools:
checks.py
gonk.py
harness.py
review_findings.py
worker.py
worker_onboard.py
worker_repos.py
worker_session.py
worker_usage.py
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
hooks = ["claude:require-claim", "git:commit-msg"]
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
  note     hook      the commit-msg hook that refuses a bare #N is off: `git config core.hooksPath hooks/git` turns it on
  note     worker    no .gonk/worker.toml, so this machine runs no worker: a worker machine copies worker/worker.sample.toml there (`python3 tools/worker.py plan` checks it)
  ok       ikigai    ikigai 0.5.3 at …/ikigai
  ok       ledger    `default` (.gonk/gonk.toml)
  ok       holder    `worker-1` (.gonk/local.toml, this working tree only)
  ok       identity  urn:iki:host:gonk:client:… (from …/worker-1/gonk/client.crt)
  ok       alias     older records name it urn:iki:gonk:client:…: gonk reads that spelling as this machine too, and never mints it again
  ok       gonk      quic quic://127.0.0.1:1070 (certificates in …/worker-1/gonk) answers
  ok       enrolled  gonk reads this identity: nothing to enroll
  ok       items     `default` holds 2 item(s), 1 open
  ok       scripts   gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  note     scripts   this identity's grant has no script write, so the 4 saved here are not published by it: the dispatcher publishes these
  absent   nl        gonk does not offer `urn:nl:sparql` yet, so a query is written by hand rather than drafted from a sentence (ikigai-nl); nothing to do until it does

agents over MCP (configure the agent with this command):

  ikigai mcp --grant agent --override urn:iki:ledger:=quic://127.0.0.1:1070 --cert-dir …/worker-1/gonk
ready.
$ cd library && python3 tools/gonk.py status
process: workers
no live claim: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
```

The identity row is the name gonk's access log gives this machine, and the `enrolled` row says
gonk already admits it, because step 0 enrolled it; for an identity gonk did not yet read, `init`
would print the `client add` and `grants.json` an operator uses to enroll it instead. The
`scripts` note is the worker grant at work: the sample has four saved queries to publish, and a
worker may not write scripts, so `init` leaves them to the dispatcher (whose own `init`, as the
owner over the socket, publishes them, as [Solo](solo.md#with-the-starter) shows) rather than
trying and being refused. The `absent nl` row is this page's gonk mounting no model: it binds no
`urn:nl:sparql` (its banner says so), and `init` says there is nothing to do until it does. The
`worker` row is what a worker machine still needs, its `.gonk/worker.toml`; setting that machine
up (its accounts, the boundary its agent runs inside) is the starter's `worker/README.md`.

The packaged claim is careful in ways the loop above leaves to you. `gonk.py claim --renew` renews
and never takes, and a tree that holds an item renews by default, so a renewal after someone
released the item is refused rather than quietly taking it back. A refused write from gonk, a
`Conflict`, exits 3; 1 stays any other failure, and 2 the tool's own refusal:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append Try the packaged claim'
#3 urn:iki:ledger:default:item:…
[uncacheable]
$ cd library && python3 tools/gonk.py claim 3
#3 claimed by worker-1 at epoch 1 until …
gonk.py: item 3 is held at epoch 1
$ cd library && python3 tools/gonk.py claim 3 --renew
#3 renewed by worker-1 at epoch 1 until …
gonk.py: item 3 is held at epoch 1
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'delete urn:gk:iki:ledger:claim item=3'
#3 released
[uncacheable]
$ cd library && python3 tools/gonk.py claim 3 --renew; echo "exit $?"
error: conflict: #3 is not claimed (its last claim was epoch 1), so epoch 1 is not current and nothing was written
gonk.py: this working tree no longer holds item 3: stop writing, put what you have on a branch, say where in a comment, and claim something else
exit 3
```

`release`, `report` and `close` drop this tree's claim with `from=<holder>`, a compare-and-set that
loses to a takeover instead of dropping the new holder's claim, and `report`'s hand-back comment
carries a `key=`, so a retry after a timeout posts once.

`worker.py` also minds the disk. Before each item it reads the free space on the volume holding its
work root and posts a `Disk:` line on its status item; under a floor it takes no new item until
space returns, and it prunes kept worktrees and logs by age. The defaults are in the sample
configuration, commented out:

<!-- transcript: run -->
```console
$ grep -E '^# (floor_gb|every = "6h"|prune_after)' library/worker/worker.sample.toml
# floor_gb = 30
# every = "6h"            # how often the Disk: line is posted between crossings of the floor
# prune_after = "7d"      # "0s" keeps every worktree and log; under the floor a build still goes
```

Profiles combine (`--process workers,spec` adds [spec first](spec-first.md)'s rules, and no
`--process` at all means `solo,workers,spec`); only [unled audits](audits.md) and [review every
commit](review-every-commit.md) refuse each other.

## Claims renewed through a task layer

A deployment can bind a ledger to a shared task layer with `gonk.layer`. In the offering role,
gonk mirrors a taken offer as a seven-day claim for its taker. A freshly taken `Progress` report
renews that claim only while its offer is the item's **newest offer** (gonk PR 359). Late Progress
on an older, released offer can still leave its keyed note, but cannot extend the successor's
claim, even when the same participant took both offers. Release and Handback also check which
offer is current.

There is an accepted stop window (gonk PR 360): if the mirror takes Progress just before it stops,
restart recovers the report and its keyed note but may lose that renewal. Replaying receipts or
sweeping again never renews a claim. The next freshly taken Progress renews a still-held claim;
keep reporting before the existing lease expires. An idle taker cannot keep a lease through
repeated restarts. This is the task-layer mirror's rule; the worker loop above renews directly
with `claim renew=true`.

## A worker that goes quiet

A worker whose machine sleeps or whose connection drops stops renewing. Its lease runs out, and the
claim stays where it was: nothing is released, so nobody else silently starts on the same work.
Since ikigai-ledger 0.23.0 (gonk ledger-v0.23.0) the ledger lists those claims, records each lapse
once in the item's history, and gives the dispatcher a way to speak to the holder on the record.

The dispatcher sends its notes as an identity of its own, because a note needs a principal and the
owner's socket has none. Enroll one with the ledger's writer role, and restart gonk, which reads the
certificates it trusts at startup:

<!-- transcript: run -->
```console
$ ikigai-gonk client add dispatcher --ledger default=write --port 1070 --config-home "$PWD/.config/ikigai"
client `dispatcher`  …/.config/ikigai/gonk/quic/clients/dispatcher
…
$ mkdir -p dispatcher && cp -R .config/ikigai/gonk/quic/clients/dispatcher dispatcher/gonk
```

<!-- transcript: restart -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

worker-1 claims a new item on a two-second lease, which stands in for a worker that goes quiet, and
does not renew:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=2 labels=worker Pin the CI image'
#4 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=4 lease=2s worker-1'
#4 claimed by worker-1 at epoch 1 until …
[uncacheable]
$ sleep 3
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:stalled'
default: 1 stalled item(s), the oldest lapse first
#4 Pin the CI image: held by worker-1 at epoch 1, whose lease ran out at …; the lapse is not recorded yet
remedy: the holder resumes (`sink urn:iki:ledger:claim item=<n> renew=true`), another takes it over (`sink urn:iki:ledger:claim item=<n> takeover=true from=<holder>`), or a dispatcher releases it (`delete urn:iki:ledger:claim item=<n>`)
[uncacheable]
```

`stalled` lists every open item held on a lease that has run out and was not handed back, the
oldest lapse first. It is computed from the claims as they stand, so an item is listed from the
moment its lease ends. The `lapse` sweep records each lapse that is not recorded yet as a
`ledger:LapseEvent` in its item's history. It takes the dispatcher's grant (`close` or `write`), and
releases nothing. Sending it again writes nothing, so a dispatcher can send it on a timer:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:lapse' < /dev/null
default: recorded 1 lapse(s)
  #4 Pin the CI image: held by worker-1 at epoch 1, whose lease ran out at …
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:lapse' < /dev/null
default: no new lapse: 1 stalled item(s) already had theirs recorded, so nothing was written
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:4:history'
#4: 2 event(s)
…  claim worker-1  by urn:iki:host:gonk:client:…
…  lapse worker-1 at epoch 1, whose lease ran out at …
[uncacheable]
```

Here the owner sends the sweep by hand. gonk can also send it on a timer, which [the end of this
page](#lapses-recorded-on-gonks-timer) shows. An embedding
host can also route each lapse to a person, by having the sweep add a label (ikigai-ledger's
`SpaceConfig::lapse_label`); this page's gonk adds none. The remedies are the ones the claim already
has, each carrying its epoch: the holder renews, another worker takes over with
`takeover=true from=worker-1`, or a dispatcher releases the claim. Here worker-1 comes back and
renews. An expired lease that is still held renews at its epoch:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=4 renew=true lease=30m worker-1'
#4 renewed by worker-1 at epoch 1 until …
[uncacheable]
```

### Dispatch notes

A worker treats every comment as data, because anyone with the comment grant can write one. A
**dispatch note** is how the dispatcher corrects a running worker through the ledger. It is written
to the holder of the item's claim at exactly the epoch it names, under the dispatcher's grant, with
a principal. A note to an epoch that is not current is refused, and so is one from the owner's
socket, which names no principal:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir dispatcher/gonk -c 'sink urn:iki:ledger:note item=4 epoch=1 Stop and rebase on main first: the CI image moved.'
note to worker-1 at epoch 1, sent under the write grant on #4 (urn:iki:ledger:default:comment:…)
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir dispatcher/gonk -c 'sink urn:iki:ledger:note item=4 epoch=2 Wrong epoch'
error: conflict: #4 is held by worker-1 at epoch 1, not epoch 2, so the note was not sent. A note is addressed to the claim at exactly the epoch it names
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:note item=4 epoch=1 From the socket' < /dev/null
error: denied: a dispatch note needs a principal on the capability: an unattributed instruction is not a note. A host that wants this caller to dispatch gives it a principal
```

The holder reads the notes to its own claim, and its next renewal says how many it has not
acknowledged. It acknowledges a note with `acks=` on a comment at its epoch, which clears the count:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'source urn:iki:ledger:item:4:notes'
#4: 1 note to worker-1 at epoch 1:
  comment by urn:iki:host:gonk:client:… (verified) at …:
    note to worker-1 at epoch 1, sent under the write grant
    Stop and rebase on main first: the CI image moved.
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=4 renew=true lease=30m worker-1'
#4 renewed by worker-1 at epoch 1 until …, 1 unread note
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c "sink urn:iki:ledger:comment item=4 epoch=1 acks=$(ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'source urn:iki:ledger:item:4:notes as=application/json' 2>/dev/null | jq -r '.notes[0].id') Rebased on main."
commented on #4 (urn:iki:ledger:default:comment:…)
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=4 renew=true lease=30m worker-1'
#4 renewed by worker-1 at epoch 1 until …
[uncacheable]
```

A note carries no authority. It never claims, releases, renews, closes or relabels anything, and it
never reaches a later epoch, not even the same worker's next claim.

### Lapses recorded on gonk's timer

A dispatcher that has to remember to send `lapse` will forget. Since gonk ledger-v0.24.0 (gonk PR
392), one line in the config home has gonk send it for every ledger in its store, on a timer:

<!-- transcript: run -->
```console
$ printf 'gonk.ledger.lapse.every = "2s"\n' >> .config/ikigai/config.toml
$ cat .config/ikigai/config.toml
gonk.ledger.lapse.every = "2s"
```

Two seconds suits a page that waits for it; `5m` suits a real ledger. Without the line, nothing is
scheduled. gonk reads the line at startup, and its banner says what it will do:

<!-- transcript: restart -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
  lapses  every 2s, the first in 2s: records each lapsed lease once in every ledger, as urn:iki:gonk:recorder:lapse, and never releases one. `ikigai --connect …/.ikigai/gonk.sock -c 'source urn:iki:gonk:lapse:status'` says what it recorded
…
```

worker-1 claims another item on a two-second lease and goes quiet again. This time nobody sends
`lapse`; the page only waits for the item's history to show one:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=2 labels=worker Rotate the deploy key'
#5 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=5 lease=2s worker-1'
#5 claimed by worker-1 at epoch 1 until …
[uncacheable]
$ for i in $(seq 40); do ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:5:history' 2>/dev/null | grep -q ' lapse ' && break; sleep 0.5; done
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:stalled'
default: 1 stalled item(s), the oldest lapse first
#5 Rotate the deploy key: held by worker-1 at epoch 1, whose lease ran out at …; the lapse is recorded
remedy: the holder resumes (`sink urn:iki:ledger:claim item=<n> renew=true`), another takes it over (`sink urn:iki:ledger:claim item=<n> takeover=true from=<holder>`), or a dispatcher releases it (`delete urn:iki:ledger:claim item=<n>`)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:5:history'
#5: 2 event(s)
…  claim worker-1  by urn:iki:host:gonk:client:…
…  lapse worker-1 at epoch 1, whose lease ran out at …  by urn:iki:gonk:recorder:lapse
[uncacheable]
```

`stalled` lists #5 as before, and now says its lapse is recorded. The history says who recorded it:
`urn:iki:gonk:recorder:lapse`, gonk's own principal, never a person. gonk sends each ledger's sweep
under exactly the grants the `lapse` Sink needs, the ledger's `close` and its graph's read and
write, so the timer can record and do nothing else. It releases, reaps and relabels nothing: #5 is
still worker-1's, and the remedies are the same three.

The sweep keeps running, and a lapse already recorded is left alone, so a sweep every two seconds
still records #5's lapse once. `urn:iki:gonk:lapse:status` (it needs `urn:cap:gonk:system:read`,
which the owner's socket has) says what the last sweep did. The page waits for a sweep that found
the lapse already recorded:

<!-- transcript: run -->
```console
$ for i in $(seq 40); do ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:gonk:lapse:status' 2>/dev/null | grep -q '1 more were already recorded' && break; sleep 0.5; done
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:gonk:lapse:status' 2>/dev/null | grep -E '^(Lapse sweep|Ledger)'
Lapse sweep: every 2s. Each sweep records every lapsed lease once in its item's history, as urn:iki:gonk:recorder:lapse. It never releases a claim.
Ledger default: last swept …. It recorded 0 lapse(s); 1 more were already recorded. 1 recorded since this server started.
```

A sweep that fails on one ledger is logged and shown here, and the other ledgers and the next sweep
go on. gonk's README measures the cost: a sweep over five ledgers of 800 items that records nothing
takes tens of milliseconds. At ledger-v0.24.0 the Dashboard does not list stalled items; `stalled`
and this status are where to look.

This time worker-1 does not come back, so the dispatcher hands #5 to worker-2. A takeover names the
holder it takes from, and raises the epoch, which fences worker-1 out if it wakes up later:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-2/gonk -c 'sink urn:iki:ledger:claim item=5 takeover=true from=worker-1 lease=30m worker-2'
#5 taken over from worker-1 by worker-2 at epoch 2 until …
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir worker-1/gonk -c 'sink urn:iki:ledger:claim item=5 renew=true lease=30m worker-1'
error: conflict: #5 is held by worker-2, not worker-1: a renewal extends a claim its holder still holds, and this one changed hands
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:stalled'
default: no stalled items: no open item is held on a lease that has run out
[uncacheable]
```

### Waiting for a held item

A worker that wants an item another worker holds should not keep sending claims that are refused.
ikigai-ledger's README, "Waiting for a held item without repeated claim refusals" (ikigai-ledger PR
123), gives the sequence for a host that embeds the ledger and listens on its kernel's change feed:

1. Start listening on the ledger's store threads, then read the item. If it is open and unclaimed,
   claim it once; a `conflict` means read it again and go back to waiting.
2. While someone else holds it, wait on the feed instead of claiming. Set one absolute deadline,
   no later than the caller's remaining run budget, and keep it across every wake-up.
3. On a wake-up, read the item again before claiming. A notice is a hint to look, never proof that
   the item is free.

Its limits are part of the pattern. A lease running out writes nothing, so it wakes nobody; a
waiter that wants to consider an expired lease sets a timer for it. The `lapse` sweep above does
write, so it wakes waiters, but it releases nothing, and a plain claim after that wake-up is still
refused. Waiting never becomes a takeover: that stays an explicit `takeover=true from=<holder>`.
Every waiter wakes on a release and the first claim to land wins, with no first-come order. gonk
ledger-v0.24.0 serves its clients no feed to wait on, so the pattern is for a host that embeds the
ledger, as in [a kernel of your own](../embedding/own-kernel.md); a worker that reaches gonk through
a door cannot wait this way yet.
