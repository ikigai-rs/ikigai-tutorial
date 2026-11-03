# A lifecycle for your ledger

So far an item has been open or closed. Real work stops at more places than that: filed and not
looked at, being worked on, waiting for a reviewer, shipped. A **lifecycle** names those places.
It is a small document you write, gonk loads it when it starts, and from then on the ledger holds
every item to it: which states exist, which moves between them are legal, and who is responsible
for an item parked in each one.

This chapter writes one, assigns it to a ledger, and moves an item through it. On the way it shows
the three things a lifecycle makes possible: a refusal that names the legal moves, a `next` that
answers for one step of the loop, and a history that says what happened and when. It also shows how
a close says what closed an item.

## Write the document

In a new empty directory, save this as `gated.ttl`:

<!-- transcript: file gated.ttl -->
```turtle
@prefix ledger: <https://ikigai-rs.dev/ns/ledger#> .
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<urn:iki:ledger:lifecycle:gated> a ledger:Lifecycle ;
    owl:versionInfo "1" ;
    ledger:hasState <urn:iki:ledger:lifecycle:gated:filed> ,
                    <urn:iki:ledger:lifecycle:gated:doing> ,
                    <urn:iki:ledger:lifecycle:gated:review> ,
                    <urn:iki:ledger:lifecycle:gated:shipped> .

<urn:iki:ledger:lifecycle:gated:filed> a ledger:LifecycleState ;
    rdfs:label "filed" ; ledger:order 0 ; ledger:inFlight false ; ledger:drain "triage" ;
    ledger:movesTo <urn:iki:ledger:lifecycle:gated:doing> .

<urn:iki:ledger:lifecycle:gated:doing> a ledger:LifecycleState ;
    rdfs:label "doing" ; ledger:order 1 ; ledger:inFlight true ;
    ledger:movesTo <urn:iki:ledger:lifecycle:gated:review> .

<urn:iki:ledger:lifecycle:gated:review> a ledger:LifecycleState ;
    rdfs:label "review" ; ledger:order 2 ; ledger:inFlight false ; ledger:drain "review-gate" ;
    ledger:movesTo <urn:iki:ledger:lifecycle:gated:doing> ,
                   <urn:iki:ledger:lifecycle:gated:shipped> .

<urn:iki:ledger:lifecycle:gated:shipped> a ledger:LifecycleState ;
    rdfs:label "shipped" ; ledger:order 3 ; ledger:inFlight false ; ledger:drain "archive" .
```

Each state says four things:

- **Its place.** `ledger:order` puts the states in order, and `filed` comes first, at 0. An item that
  holds no state is in `filed`, so a newly filed item needs no write to be in it.
- **Whether it is in flight.** A state in flight is worked under a claim. `doing` is the only one
  here.
- **Its drain**, for every state that is not in flight: the name of the step responsible for an item
  parked there. Nothing can be left waiting with nobody named to collect it.
- **Its moves.** `ledger:movesTo` lists where an item can go next, and a state may list several.
  `shipped` lists none, so an item that reaches it stays. These lines are optional: a lifecycle that
  declares no moves at all lets any state move to any other, as the built-in one below does.

The line near the top, `owl:versionInfo "1"`, is required, and the next section shows why.

Then assign the lifecycle to a ledger, with one line in gonk's config home. A ledger takes one
lifecycle, and the line is repeatable for other ledgers. A relative path is read from the directory
gonk starts in; on a real machine, write `~/` and a path that will not move.

<!-- transcript: file .config/ikigai/config.toml -->
```toml
gonk.ledger.lifecycle = "tasks gated.ttl"
```

A ledger with no such line keeps the built-in lifecycle, `kata-flight`, which is what every other
chapter's ledgers use. It has six states and declares no moves, so nothing in those chapters changes.

## A version, or gonk will not start

Every move an item makes records the version of the lifecycle it was checked against, so a document
that does not say its version cannot be loaded. See what happens without it: make a copy with that
one line removed, point a second config at the copy, and start gonk with it.

<!-- transcript: run -->
```console
$ mkdir refused
$ grep -v versionInfo gated.ttl > unversioned.ttl
$ printf 'gonk.ledger.lifecycle = "tasks unversioned.ttl"\n' > refused/config.toml
$ ikigai-gonk --config-home "$PWD/refused" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk: gonk.ledger.lifecycle: unversioned.ttl: invalid argument `lifecycle`: `gated` declares no version, and every transition records the version it was checked against: give `urn:iki:ledger:lifecycle:gated` an owl:versionInfo or, if the document has none, a ledger:lifecycleVersion
  (config read from …/refused/config.toml)
```

gonk read and checked the document before it opened its store or any door, and it exited. ⚠ **If you
already run gonk with a `gonk.ledger.lifecycle` line, this is what the upgrade to `ikigai-ledger`
0.18.0 does to it**, and the fix is that one line. gonk refuses any other malformed document at this
point too, such as one with a move to a state it does not have.

The version is whatever text the document says. Nothing compares or orders versions, and nothing
notices a document that changed and kept its number, so change it whenever you change the states or
the moves. gonk reads the document only when it starts, so a change also takes a restart.

## Start it, and read the lifecycle back

Now start gonk with the real config. If the gonk from [The ledger in a
browser](browser.md) is still running, stop it first, since this one takes the same port
([A gonk per chapter](../introduction.md#a-gonk-per-chapter)).

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
  socket  …/.ikigai/gonk.sock — owner only
…
```

A lifecycle is a resource like any other, so read it, and file three items into `tasks` over the
owner's socket. The second must wait for the first:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:lifecycle:gated'
gated
   0. filed    drain: triage  (no ledger:state)  moves to: doing
   1. doing    in flight  moves to: review
   2. review   drain: review-gate  moves to: doing, shipped
   3. shipped  drain: archive  moves to: nowhere
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:tasks:append priority=1 Add a retry to the fetcher' -c 'sink urn:gk:iki:ledger:tasks:append priority=2 Write the release note' -c 'sink urn:gk:iki:ledger:tasks:append priority=3 Add retries to the fetcher' -c 'sink urn:gk:iki:ledger:tasks:link item=1 type=blocks 2'
tasks#1 urn:iki:ledger:tasks:item:…
[uncacheable]
tasks#2 urn:iki:ledger:tasks:item:…
[uncacheable]
tasks#3 urn:iki:ledger:tasks:item:…
[uncacheable]
linked tasks#1 blocks tasks#2
[uncacheable]
— batch: 4 commands · 4 uncacheable
```

## Moving an item

An item's state is its own resource, `item:{id}:state`. A move names both ends: where the item is
now (`from=`) and where it goes (`to=`). Ask for a move the document does not declare:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:item:1:state'
filed
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:tasks:item:1:state from=filed to=shipped'
error: conflict: The lifecycle `gated` declares no move from `filed` to `shipped`, so nothing was written. From `filed` it allows moves to `doing` (see `urn:iki:ledger:lifecycle:gated`)
```

The refusal says what would have been legal, and points at the document that decides. A misspelled
state is a different mistake, with a different answer: the first line below is the ledger, naming the
states of this ledger's lifecycle. The second is the action's contract, which lists every state of
every lifecycle this gonk has loaded, because one contract describes all its ledgers. Trust the first.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:tasks:item:1:state from=filed to=parked'
error: invalid argument `to`: `parked` is not a state of the lifecycle `gated` this ledger uses; one of filed, doing, review, shipped (see `urn:iki:ledger:lifecycle:gated`)
       `to`: The state to move the item to. A state of the ledger's lifecycle (`urn:iki:ledger:lifecycle:{name}`); `filed` is the state of an item that holds none. (one of: filed, queued, reviewed, resolving, refining, shipping, doing, review, shipped)
```

Now the legal moves. `from=` is a compare-and-set: if another writer has already moved the item, your
move is refused and told where the item is now, so two writers cannot both win. A value, like the
note on the first move, is recorded as a comment.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:tasks:item:1:state from=filed to=doing Starting now.' -c 'sink urn:gk:iki:ledger:tasks:item:1:state from=doing to=review'
tasks#1 doing (was filed)
[uncacheable]
tasks#1 review (was doing)
[uncacheable]
— batch: 2 commands · 2 uncacheable
```

## Asking one step for its work

A plain `next` ignores state: `tasks#1` is ranked first even though it is waiting for a reviewer, not
for someone to start it. `drain=` and `state=` narrow the answer to one step of the loop, so a
reviewer can ask what is waiting for them without anyone labeling anything:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:next'
 1. tasks#1  open    p1  Add a retry to the fetcher
    p1 — priority 1; last updated …
 2. tasks#3  open    p3  Add retries to the fetcher
    p3 — priority 3; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 1
  not tasks#2: blocked by tasks#1
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:next drain=review-gate'
 1. tasks#1  open    p1  Add a retry to the fetcher
    p1 — priority 1; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 0
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:next drain=triage'
 1. tasks#3  open    p3  Add retries to the fetcher
    p3 — priority 3; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 1
  not tasks#2: blocked by tasks#1
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:next state=doing'
nothing is ready (0 open item(s) excluded: the ledger has no open items matching the filter)
policy: priority-recency
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:next drain=revue'
error: invalid argument `drain`: `revue` is not a drain of the lifecycle `gated` this ledger uses; one of triage, review-gate, archive (see `urn:iki:ledger:lifecycle:gated`)
       `drain`: Only candidates whose current state drains to this step, `filed` included when its drain matches. A drain of the ledger's lifecycle (`urn:iki:ledger:lifecycle:{name}`), which is what a value is judged against. Blocked items, cycles and leverage are still computed over the whole ledger. (one of: triage, review-gate, ship, archive)
```

Read the `drain=triage` answer closely. `tasks#2` is in `filed`, so the filter admits it, and it is
still excluded, because its blocker is in another state. ★ **A filter chooses which items you are
asking about and never changes what ranks them**: blocked items, cycles and leverage are worked out
over the whole ledger first, and the answer is the unfiltered ranking with the other items taken out.

`filed` counts as a state here, though the items in it hold none. A state in flight has no drain, so
only `state=` asks about one, and a misspelled name is refused rather than read as "no filter". The
filters also work on a ledger with the built-in lifecycle: `next drain=review-gate` on `default`
is the same question, about `kata-flight`'s `queued` state.

## Closing, and saying what closed it

The reviewer approves, the item ships, and it is closed. `evidence=` on a close says what closed it,
and may be repeated. A reason of `duplicate` or `superseded` names the surviving item the same way,
so the target no longer has to be written into a note:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:tasks:item:1:state from=review to=shipped'
tasks#1 shipped (was review)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:tasks:close item=1 evidence=https://github.com/acme/fetcher/pull/8 evidence=https://github.com/acme/fetcher/commit/abc123 Checked: merged, CI green.'
closed tasks#1 done (2 evidence)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:tasks:close item=3 reason=duplicate evidence=tasks#99'
error: not found: evidence `tasks#99`: no ledger item tasks#99. Nothing was changed
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:tasks:close item=3 reason=duplicate evidence=tasks#1'
closed tasks#3 duplicate of tasks#1
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:item:1:state'
shipped
[uncacheable]
```

Evidence takes two forms, and they are treated differently:

- **An item must exist, and the closer must be able to read it**, or the close is refused and nothing
  is written, as `tasks#99` was. Evidence in another ledger also needs a grant to link into that
  ledger.
- **Any other IRI is stored as given and never fetched**, so gonk does not know whether that pull
  request exists. Checking it is the closer's job, which is why the note above says what was checked.

Closing and moving are different facts, and neither touches the other: `tasks#1` is closed and still
`shipped`. A reopen keeps each close's evidence on that close's record, and a second close starts its
own. A closed item is refused a move into a state in flight until it is reopened.

## What happened to an item

Each edit, claim, release, close and reopen leaves an event, and since `ikigai-ledger` 0.18.0 an edit
event says what it changed. `item:{id}:history` lists them, oldest first, under the same grant that
reads the ledger:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:item:1:history'
tasks#1: 5 event(s)
…  link blocks urn:iki:ledger:tasks:item:…
…  state filed to doing  lifecycle version 1
…  state doing to review  lifecycle version 1
…  state review to shipped  lifecycle version 1
…  closed done, evidence https://github.com/acme/fetcher/commit/abc123 https://github.com/acme/fetcher/pull/8
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:item:3:history'
tasks#3: 1 event(s)
…  closed duplicate of urn:iki:ledger:tasks:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:item:1:history as=application/json' 2>/dev/null | jq -r '.events[] | select(.kind == "state") | "\(.state_to) version \(.lifecycle_version)"'
urn:iki:ledger:lifecycle:gated:doing version 1
urn:iki:ledger:lifecycle:gated:review version 1
urn:iki:ledger:lifecycle:gated:shipped version 1
```

So when the document changes, an old event can still say which version allowed it. The lines here name
no one, because the owner's socket has no name to give; [Remote workers](../ways/workers.md#5-the-dispatcher-checks-and-closes-or-sends-it-back),
later in the book, shows the same history with the worker's name on each line. An event written by a
ledger older than 0.18.0 has no object and no version, and cannot be given them: nothing kept them.

The doctor still has nothing to report, and would name an item stranded in a state the document no
longer has:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:tasks:doctor'
doctor: tasks (lifecycle gated): no problems
checked: orphaned, abandoned, lease-expired, unknown-state, two-states, awaiting-closed, awaiting-stale at …
[uncacheable]
```

That is why renaming a state takes two versions. Version 2 declares both names and a move from the old
to the new, every item moves across by an ordinary transition, and version 3 drops the old name.
Dropping it while an item still holds it leaves that item in a state nothing can name, so ask the
doctor first.

<a id="what-you-built"></a>

## What you have now

| you want | ask |
| --- | --- |
| the states, drains and moves | `source lifecycle:{name}` |
| where an item is | `source {ledger}:item:{id}:state` |
| to move it | `sink {ledger}:item:{id}:state from=… to=…` |
| what one step should do next | `source {ledger}:next drain=…` or `state=…` |
| to say what closed it | `sink {ledger}:close item=… evidence=…` |
| what happened to it | `source {ledger}:item:{id}:history` |

In resource terms: the lifecycle is a document gonk checks once, at the start, and serves as a resource;
an item's state is one value, moved by one compare-and-set; `next` is still a view, now with a question
about one step; and the history is a view of events that each write leaves behind. The next part
reaches the ledger from another process, the way an agent does.
