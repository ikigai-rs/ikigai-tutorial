# Importing a kata ledger

[kata](https://github.com/kenn-io/kata) keeps issues in its own database, for people and for
agents. `ikigai-ledger` took its model from kata — issues with authors, priorities, labels,
comments, closes with a reason, and `parent`, `blocks` and `related` links — so most of an
import is one to one. This chapter plans an import of one, shows what the gonk this book pins
answers when the import tries to file it, and says plainly what an import keeps and what does
not survive the trip.

## An export

In a new empty directory, as in [every chapter](../introduction.md#a-gonk-per-chapter): if the gonk
from [Filing roborev findings](roborev.md) is still running, stop it first, since this one takes the
same port.

`kata export` writes the database out as JSON Lines, one record per line, each with a `kind`
(stop kata's daemon first, or pass `--allow-running-daemon`):

<!-- transcript: manual — it needs kata and a kata database; the file below is what it writes -->
```console
$ kata export --output kata.jsonl
```

Here is a small one: a project called `flight`, four issues (one closed `wontfix`, one deleted),
three comments, three labels, four links, and an event. Save it as `kata.jsonl`:

<!-- transcript: file kata.jsonl -->
```json
{"kind":"meta","data":{"key":"export_version","value":"27"}}
{"kind":"meta","data":{"key":"instance_uid","value":"01K6Z0000000000000000INST1"}}
{"kind":"meta","data":{"key":"schema_version","value":"27"}}
{"kind":"project","data":{"created_at":"2026-09-01T12:00:00.000Z","id":1,"metadata":{},"name":"flight","revision":1,"uid":"01K6Z00000000000000000PRJ1"}}
{"kind":"issue","data":{"author":"chris","body":"The loop should stop when the queue is empty.\n\nSee the flight log.","closed_at":null,"closed_reason":null,"content_revision":1,"created_at":"2026-09-02T09:15:00.000Z","deleted_at":null,"id":1,"metadata":{},"owner":"chris","priority":1,"project_id":1,"revision":3,"short_id":"0001","status":"open","title":"Stop the loop on an empty queue","uid":"01K6Z000000000000000000001","updated_at":"2026-09-03T10:00:00.000Z"}}
{"kind":"issue","data":{"author":"chris","body":"","closed_at":"2026-09-04T08:00:00.000Z","closed_reason":"wontfix","content_revision":0,"created_at":"2026-09-02T09:20:00.000Z","deleted_at":null,"id":2,"metadata":{},"owner":null,"priority":3,"project_id":1,"revision":2,"short_id":"0002","status":"closed","title":"Rename the flight binary","uid":"01K6Z000000000000000000002","updated_at":"2026-09-04T08:00:00.000Z"}}
{"kind":"issue","data":{"author":"agent-7","body":"Child of 0001.","closed_at":null,"closed_reason":null,"content_revision":0,"created_at":"2026-09-02T11:00:00.000Z","deleted_at":null,"id":3,"metadata":{},"owner":null,"project_id":1,"revision":1,"short_id":"0003","status":"open","title":"Write the empty-queue test","uid":"01K6Z000000000000000000003","updated_at":"2026-09-02T11:00:00.000Z"}}
{"kind":"issue","data":{"author":"chris","body":"Filed by mistake.","closed_at":null,"closed_reason":null,"content_revision":0,"created_at":"2026-09-02T12:00:00.000Z","deleted_at":"2026-09-02T12:05:00.000Z","id":4,"metadata":{},"owner":null,"priority":2,"project_id":1,"revision":2,"short_id":"0004","status":"open","title":"A mistake","uid":"01K6Z000000000000000000004","updated_at":"2026-09-02T12:05:00.000Z"}}
{"kind":"comment","data":{"author":"chris","body":"Seen twice this week.","created_at":"2026-09-02T10:00:00.000Z","id":1,"issue_id":1,"uid":"01K6Z0000000000000000000C1"}}
{"kind":"comment","data":{"author":"chris","body":"Agent 7 is on it.","created_at":"2026-09-03T10:00:00.000Z","id":2,"issue_id":1,"teammate":"agent-7","uid":"01K6Z0000000000000000000C2"}}
{"kind":"comment","data":{"author":"chris","body":"Never mind.","created_at":"2026-09-02T12:04:00.000Z","id":3,"issue_id":4,"uid":"01K6Z0000000000000000000C3"}}
{"kind":"issue_label","data":{"author":"chris","created_at":"2026-09-02T09:15:00.000Z","issue_id":1,"label":"bug"}}
{"kind":"issue_label","data":{"author":"chris","created_at":"2026-09-02T09:15:00.000Z","issue_id":1,"label":"area:loop"}}
{"kind":"issue_label","data":{"author":"chris","created_at":"2026-09-02T09:20:00.000Z","issue_id":2,"label":"chore"}}
{"kind":"link","data":{"author":"agent-7","created_at":"2026-09-02T11:00:00.000Z","from_issue_id":3,"from_issue_uid":"01K6Z000000000000000000003","id":1,"to_issue_id":1,"to_issue_uid":"01K6Z000000000000000000001","type":"parent"}}
{"kind":"link","data":{"author":"agent-7","created_at":"2026-09-02T11:01:00.000Z","from_issue_id":3,"from_issue_uid":"01K6Z000000000000000000003","id":2,"to_issue_id":1,"to_issue_uid":"01K6Z000000000000000000001","type":"blocks"}}
{"kind":"link","data":{"author":"chris","created_at":"2026-09-02T09:21:00.000Z","from_issue_id":1,"from_issue_uid":"01K6Z000000000000000000001","id":3,"to_issue_id":2,"to_issue_uid":"01K6Z000000000000000000002","type":"related"}}
{"kind":"link","data":{"author":"chris","created_at":"2026-09-02T12:01:00.000Z","from_issue_id":1,"from_issue_uid":"01K6Z000000000000000000001","id":4,"to_issue_id":4,"to_issue_uid":"01K6Z000000000000000000004","type":"related"}}
{"kind":"event","data":{"actor":"chris","created_at":"2026-09-02T09:15:00.000Z","id":1,"issue_id":1,"payload":{},"project_id":1,"type":"issue.created","uid":"01K6Z0000000000000000000E1"}}
{"kind":"sqlite_sequence","data":{"name":"issues","seq":4}}
```

## Into a ledger of its own

The importer speaks to gonk's HTTP door, like `roborev file`, as its anonymous loopback caller,
so the ledger it imports into must be one that door grants. Start gonk granting `kata` as well as
`default` (`--http-ledger` replaces the configured list, so name both; in a config file it is
one `gonk.http.ledger` line each):

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup --http-ledger default --http-ledger kata
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read: default, kata; 0 passkey(s), sessions 12 h or 60 min idle; anonymous SPARQL budget 1000 ms
…
```

Read the `http` line closely: `anonymous read: default, kata`. Since gonk PR 186 (ledger #1687)
the door's anonymous caller **reads** the ledgers it lists and writes nothing, because any process
on this machine, or on any machine a reverse tunnel puts the door on, could otherwise file an item
that an unattended worker would read as instructions. The importer has no other way in yet: it
files through that door alone, with no passkey and no socket. So against this gonk its dry run
works and its import is refused, and the transcripts below show both. (Through gonk 3edf830 the
same door wrote too, and this chapter's import filed the issues.)

`--dry-run` reads the export, asks the ledger for each issue's key exactly as the real run
does, and prints the plan; it writes nothing. So it says `already` for an issue that is there,
reports the comments, closes and links a real run would add, and is refused, like the real run,
on a ledger the door does not grant. (`roborev file --dry-run` is the other way round: it asks
gonk nothing.) ⚠ The one thing a read cannot tell: an issue whose item was *deleted* in the
ledger answers like one never filed, so the dry run says `would file` where the real run leaves it
alone. Into an empty ledger, the plan is every issue:

<!-- transcript: run -->
```console
$ ikigai-gonk kata import kata.jsonl --gonk http://127.0.0.1:1070 --ledger kata --dry-run
kata export version 27: 4 issue(s), 4 link(s) (dry run: reads only)
would file  kata 0001  Stop the loop on an empty queue
    key=urn:kata:issue:01K6Z000000000000000000001
    about=urn:kata:issue:01K6Z000000000000000000001
    author=chris
    priority=1
    labels=bug,area:loop
    then 2 comment(s)
would file  kata 0002  Rename the flight binary
    key=urn:kata:issue:01K6Z000000000000000000002
    about=urn:kata:issue:01K6Z000000000000000000002
    author=chris
    priority=3
    labels=chore
    then 0 comment(s), close
would file  kata 0003  Write the empty-queue test
    key=urn:kata:issue:01K6Z000000000000000000003
    about=urn:kata:issue:01K6Z000000000000000000003
    author=agent-7
    then 0 comment(s)
deleted   kata 0004  not imported
would linked    01K6Z000000000000000000003 parent 01K6Z000000000000000000001
would linked    01K6Z000000000000000000003 blocks 01K6Z000000000000000000001
would linked    01K6Z000000000000000000001 related 01K6Z000000000000000000002
skipped   link 01K6Z000000000000000000001 related 01K6Z000000000000000000004: an end was not imported, or the type is not one the ledger has
dry run, nothing written: 3 would be filed, 0 already there; 2 comment(s), 1 close(s), 3 link(s) would be added; 1 deleted issue(s) and 1 link(s) skipped; not read: event 1, sqlite_sequence 1
```

and without it, the import, which this gonk refuses at its first write:

<!-- transcript: run -->
```console
$ ikigai-gonk kata import kata.jsonl --gonk http://127.0.0.1:1070 --ledger kata
kata export version 27: 4 issue(s), 4 link(s)
ikigai-gonk: gonk refused POST /iki/ledger/kata/append (403): denied: capability does not grant `urn:cap:ledger:write:*` (declared by `urn:iki:ledger:kata:append`; any grant under `urn:cap:ledger:write:` satisfies it). The HTTP door grants an anonymous loopback caller the READ tokens of the ledgers in `gonk.http.ledger` and nothing else; since ledger #1687 it writes nothing, and this command files through that door alone, so it can read (a dry run) but not file into a gonk from that release on
```

The refusal names the scope the append needed, says that the door's anonymous caller holds the
read tokens and nothing more, and says why the command cannot file. It comes at the first append,
so nothing is filed, not even the first issue.

Where an import can file, it does what the dry run planned, in that order: issues first, so that
every link has two items to join, then the links. The soft-deleted issue is not imported, and
neither is anything that hangs from it: its comment, and the link to it, which is reported as
skipped. The last line counts everything the importer did not read, by kind, so nothing in the
export is silently dropped. The links come across as links, so `next` would know the order of the
work: `kata 0003` blocks `kata 0001`. Here the ledger is still empty, and `next` says so:

<!-- transcript: run -->
```console
$ curl -s http://127.0.0.1:1070/iki/ledger/kata/next
nothing is ready (0 open item(s) excluded: the ledger has no open items matching the filter)
policy: priority-recency
```

## Again, and later

Every issue is filed with **one** request, a keyed append whose key is `urn:kata:issue:<uid>`
(also filed as an `about`): the ledger checks the key and files the item in the same store
update, and a key already taken answers the item that holds it and files nothing. So the same
export imported twice files nothing the second time: each issue already there is reported as
`already`. Here, with nothing filed, a second run is refused just as the first was:

<!-- transcript: run -->
```console
$ ikigai-gonk kata import kata.jsonl --gonk http://127.0.0.1:1070 --ledger kata
kata export version 27: 4 issue(s), 4 link(s)
ikigai-gonk: gonk refused POST /iki/ledger/kata/append (403): denied: capability does not grant `urn:cap:ledger:write:*` (declared by `urn:iki:ledger:kata:append`; any grant under `urn:cap:ledger:write:` satisfies it). The HTTP door grants an anonymous loopback caller the READ tokens of the ledgers in `gonk.http.ledger` and nothing else; since ledger #1687 it writes nothing, and this command files through that door alone, so it can read (a dry run) but not file into a gonk from that release on
```

It does more than skip. A run **converges** on the items already there: a comment whose kata
id is not on its item is added, an issue closed in kata whose item is open is closed, and a
missing link is made. So a later export, with work done in kata since, brings exactly that work
across. Here a comment is added to the export, as a later `kata export` would have it, and the
dry run asks the ledger about each issue. None is there, because nothing was filed, so its plan
is the first plan with the new comment in it: `then 1 comment(s)` on `kata 0003`, and three
comments in all. Over items already filed, it would say `already` for each issue and name only
the one new comment. The import is refused, as before:

<!-- transcript: run -->
```console
$ cp kata.jsonl later.jsonl
$ printf '%s\n' '{"kind":"comment","data":{"author":"agent-7","body":"The test fails on an empty queue, as it should.","created_at":"2026-09-05T08:30:00.000Z","id":4,"issue_id":3,"uid":"01K6Z0000000000000000000C4"}}' >> later.jsonl
$ ikigai-gonk kata import later.jsonl --gonk http://127.0.0.1:1070 --ledger kata --dry-run
kata export version 27: 4 issue(s), 4 link(s) (dry run: reads only)
would file  kata 0001  Stop the loop on an empty queue
    key=urn:kata:issue:01K6Z000000000000000000001
    about=urn:kata:issue:01K6Z000000000000000000001
    author=chris
    priority=1
    labels=bug,area:loop
    then 2 comment(s)
would file  kata 0002  Rename the flight binary
    key=urn:kata:issue:01K6Z000000000000000000002
    about=urn:kata:issue:01K6Z000000000000000000002
    author=chris
    priority=3
    labels=chore
    then 0 comment(s), close
would file  kata 0003  Write the empty-queue test
    key=urn:kata:issue:01K6Z000000000000000000003
    about=urn:kata:issue:01K6Z000000000000000000003
    author=agent-7
    then 1 comment(s)
deleted   kata 0004  not imported
would linked    01K6Z000000000000000000003 parent 01K6Z000000000000000000001
would linked    01K6Z000000000000000000003 blocks 01K6Z000000000000000000001
would linked    01K6Z000000000000000000001 related 01K6Z000000000000000000002
skipped   link 01K6Z000000000000000000001 related 01K6Z000000000000000000004: an end was not imported, or the type is not one the ledger has
dry run, nothing written: 3 would be filed, 0 already there; 3 comment(s), 1 close(s), 3 link(s) would be added; 1 deleted issue(s) and 1 link(s) skipped; not read: event 1, sqlite_sequence 1
$ ikigai-gonk kata import later.jsonl --gonk http://127.0.0.1:1070 --ledger kata
kata export version 27: 4 issue(s), 4 link(s)
ikigai-gonk: gonk refused POST /iki/ledger/kata/append (403): denied: capability does not grant `urn:cap:ledger:write:*` (declared by `urn:iki:ledger:kata:append`; any grant under `urn:cap:ledger:write:` satisfies it). The HTTP door grants an anonymous loopback caller the READ tokens of the ledgers in `gonk.http.ledger` and nothing else; since ledger #1687 it writes nothing, and this command files through that door alone, so it can read (a dry run) but not file into a gonk from that release on
```

⚠ What does **not** converge: an issue's title, body, labels and priority are written once,
when it is filed. An edit made in kata later does not reach the item. Two imports of one export
at the same instant file each issue once, because the filing is the keyed append; but only the
filing is atomic. The convergence reads the item and then writes, and the ledger has no keyed
comment, so two imports racing on the same new comment can both add it.

## What is lost, and why

An imported item's `filed` line and the provenance line its body ends with are where the loss
shows. Against this gonk there is no item to read, so the same `grep` finds nothing:

<!-- transcript: run -->
```console
$ curl -s http://127.0.0.1:1070/iki/ledger/kata/item/1 | grep -E 'filed:|Imported from'
```

On an item the import filed, the two `filed` times disagree, and that is the loss in one line:
the `filed` line says when the import ran, by `chris`, and the provenance line says `Imported from
kata flight#0001 (01K6Z000000000000000000001), filed 2026-09-02T09:15:00.000Z by chris; owner
chris.` The ledger stamps every item with the time **it** was filed and mints its own number, because those are facts the ledger
vouches for, and it has no way today to record an item as filed at some other time by some
other system. So kata's `created_at`, its `short_id` (`0001`) and its `uid` survive only as
text, in the provenance line every imported body ends with, and in each comment's closing
note. The `uid` is also the item's key (and an `about`), which is what makes a re-run find it.

Carried as text only, or not at all:

| kata | in the ledger |
| --- | --- |
| `created_at`, `short_id`, `uid` | the provenance line; the `uid` is also the item's key and an `about` |
| a comment's time | its closing note `(kata comment <uid>, <created_at>)` |
| `owner` | the provenance line, not a claim |
| `updated_at`, `metadata`, who added a label or link and when | not carried |
| the event history | not carried; counted on the last line (`event 1`) |
| sync bindings, federation state, claims, purge logs | not carried; counted on the last line |

A faithful import, keeping kata's times and ids as data rather than text and carrying the
history, needs the ledger to accept an item's original times and a foreign id. That is tracked
as ledger item 774. Until then the import is lossy in exactly the ways above, and says so.
