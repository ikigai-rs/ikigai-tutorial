# A fresh machine, end to end

The three bridges in the order a new machine needs them: the repositories first, then gonk
serving them, then the work already tracked elsewhere brought in, then the hook that keeps new
review findings arriving. Every step is a command from the last three chapters, and this page
runs them in sequence in one scratch home, so the sequence itself is checked, not only its
parts.

Before it: gonk and `ikigai` installed at the versions this book pins
([Which gonk](../introduction.md#which-gonk)), and git. In a new empty directory, as in [every
chapter](../introduction.md#a-gonk-per-chapter): if the gonk from [Importing a kata ledger](kata.md)
is still running, stop it first, since this one takes the same port. (This page is a fresh machine:
none of the last three chapters' directories is reused.)

## 1. The repositories

On a real machine these are URLs on GitHub. Here, two repositories made on the spot stand in
for them, cloned by `file://` URL:

<!-- transcript: run -->
```console
$ git init -q -b main upstream/flight
$ printf '# Flight\n' > upstream/flight/README.md
$ git -C upstream/flight add README.md
$ git -C upstream/flight -c user.name='A Reader' -c user.email=reader@example.com commit -qm 'Start flight'
$ git init -q -b main upstream/notes
$ printf '# Notes\n' > upstream/notes/README.md
$ git -C upstream/notes add README.md
$ git -C upstream/notes -c user.name='A Reader' -c user.email=reader@example.com commit -qm 'Start the notes'
```

Clone them and write their roots in one command:

<!-- transcript: run -->
```console
$ ikigai-gonk checkout --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" "file://$PWD/upstream/flight" "file://$PWD/upstream/notes" --write-config
cloned     flight at …  …/.ikigai/checkouts/flight
cloned     notes at …  …/.ikigai/checkouts/notes

browse roots:
  gonk.browse.root = "flight=…/.ikigai/checkouts/flight"
  gonk.browse.root = "notes=…/.ikigai/checkouts/notes"

…/.config/ikigai/config.toml:
  added      gonk.browse.root = "flight=…/.ikigai/checkouts/flight"
  added      gonk.browse.root = "notes=…/.ikigai/checkouts/notes"

gonk reads gonk.browse.root at startup only: restart gonk to serve the new root(s).
```

## 2. gonk, serving them

The roborev hook will file unattended, every time a review completes, so it gets an identity of
its own before gonk starts: a client certificate enrolled to write the `default` ledger and
nothing else ([Filing roborev findings](roborev.md#an-identity-for-the-hook)):

<!-- transcript: run -->
```console
$ ikigai-gonk client add roborev --ledger default=write --port 1070 --config-home "$PWD/.config/ikigai"
client `roborev`  …/.config/ikigai/gonk/quic/clients/roborev
  fingerprint  …
  principal    urn:iki:host:gonk:client:…
  enrolled     grant `roborev` (4 scopes) in …/.config/ikigai/gonk/grants.json
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <the moved directory>
```

gonk reads its roots, and the certificates it trusts, when it starts, which is what the last lines
of both commands say. If it is already running as a service, restart it the way that service is
run; under launchd, with the label your plist gives it (`dev.ikigai-rs.gonk` here; the checkout
stopped printing one, because a label is one operator's choice):

<!-- transcript: manual — it restarts a launchd service, which a scratch check does not have -->
```console
$ launchctl kickstart -k gui/$(id -u)/dev.ikigai-rs.gonk
```

Here it starts for the first time, with the same two homes the checkout and the enrollment were
given, so it reads the `config.toml` the checkout wrote and the client `client add` enrolled:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read: default; 0 passkey(s), sessions 12 h or 60 min idle; anonymous SPARQL budget 1000 ms
  browse  urn:repo:{flight (watched), notes (watched)}:* — annotations and archive in <urn:iki:browse:graph:default>
…
  quic    udp 0.0.0.0:1070 — 1 trusted certificate(s), 1 enrolled
          client  roborev  …  trusted and enrolled
…
```

Both roots, both watched, and the hook's client trusted and enrolled at the QUIC door. The `http`
line also says what an anonymous caller at that door may do: read the `default` ledger, and
nothing more. A read through the socket says the roots are what was cloned:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:flight:file:README.md'
# Flight
[uncacheable]
```

## 3. The work already tracked

A `kata export` from the old tracker, here two issues, one blocking the other:

<!-- transcript: file kata.jsonl -->
```json
{"kind":"meta","data":{"key":"export_version","value":"27"}}
{"kind":"project","data":{"created_at":"2026-09-01T12:00:00.000Z","id":1,"metadata":{},"name":"flight","revision":1,"uid":"01K6Z00000000000000000PRJ1"}}
{"kind":"issue","data":{"author":"chris","body":"The loop should stop when the queue is empty.","closed_at":null,"closed_reason":null,"content_revision":0,"created_at":"2026-09-02T09:15:00.000Z","deleted_at":null,"id":1,"metadata":{},"owner":"chris","priority":1,"project_id":1,"revision":1,"short_id":"0001","status":"open","title":"Stop the loop on an empty queue","uid":"01K6Z000000000000000000001","updated_at":"2026-09-02T09:15:00.000Z"}}
{"kind":"issue","data":{"author":"agent-7","body":"","closed_at":null,"closed_reason":null,"content_revision":0,"created_at":"2026-09-02T11:00:00.000Z","deleted_at":null,"id":2,"metadata":{},"owner":null,"project_id":1,"revision":1,"short_id":"0002","status":"open","title":"Write the empty-queue test","uid":"01K6Z000000000000000000002","updated_at":"2026-09-02T11:00:00.000Z"}}
{"kind":"link","data":{"author":"agent-7","created_at":"2026-09-02T11:01:00.000Z","from_issue_id":2,"from_issue_uid":"01K6Z000000000000000000002","id":1,"to_issue_id":1,"to_issue_uid":"01K6Z000000000000000000001","type":"blocks"}}
```

The import runs once, by you, on this machine, so it can take the owner's socket, the shortcut
that needs no enrollment ([Importing a kata ledger](kata.md#the-owners-socket-a-shortcut)); kata's
authors stay the items' authors:

<!-- transcript: run -->
```console
$ ikigai-gonk kata import kata.jsonl --socket "$PWD/.ikigai/gonk.sock"
kata export version 27: 2 issue(s), 1 link(s)
filed     #1       kata 0001  Stop the loop on an empty queue
filed     #2       kata 0002  Write the empty-queue test
linked    #2 blocks #1

2 filed, 0 already there; 0 comment(s), 0 close(s), 1 link(s) added; 0 deleted issue(s) and 0 link(s) skipped
```

Import it again whenever the old tracker has moved on: a re-run converges, adding comments,
closes and links that are new and filing nothing twice. An import that runs unattended, or as
another user, takes a client of its own instead, `--quic` and `--cert-dir` as the hook below does
([Importing a kata ledger](kata.md)).

## 4. The hook

The last piece is one `[[hooks]]` entry in roborev's config, so every completed review files
its findings here from now on, as the client enrolled in step 2 ([Filing roborev
findings](roborev.md)):

<!-- transcript: file .roborev/config.toml -->
```toml
[[hooks]]
event = "review.completed"
command = "ikigai-gonk roborev file --quic 127.0.0.1:1062 --cert-dir ~/.config/ikigai/gonk/quic/clients/roborev --ledger default --root {repo_name} --repo-path {repo} --job {job_id} --sha {sha} --agent {agent} --findings {findings}"
```

That is the line for your machine: gonk's own port, 1062, and the bundle where `client add`
writes it in your config home. This page's gonk is on 1070 with a scratch config home, so the next
review roborev completes on `notes` runs that line with roborev's values in place, and this
page's port and bundle; this is what it receives and runs:

<!-- transcript: file review.md -->
```markdown
## Summary

The README names the project and says nothing about how to run it.

**Agent assessment:** Fail

## Findings

### 1. High

**Location:** README.md:1

**Problem:** The README has a title and no instructions, so a new reader cannot start the project.

**Fix:** Add a "Getting started" section with the one command that runs it.

**Reported by:** claude-code
```

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --quic 127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/roborev --ledger default --root 'notes' --repo-path "$PWD/.ikigai/checkouts/notes" --job '7' --sha 'abc1234' --agent 'claude-code' --findings "$(cat review.md)"
filed    high     #3 The README has a title and no instructions, so a new reader cannot start the project.
```

The finding is about `urn:repo:notes:file:README.md`, a file gonk now serves, so browse's page
for that file and the ledger item find each other. It is authored by the `roborev` client, not by
you.

<a id="where-that-leaves-you"></a>

## What you have now

<!-- transcript: run -->
```console
$ curl -s http://127.0.0.1:1070/iki/ledger/next
 1.    #3  open    p1  The README has a title and no instructions, so a new reader cannot start the project.  [high roborev]
    p1 — priority 1; last updated …
 2.    #2  open    p-  Write the empty-queue test
    unprioritized — no priority set, so it ranks below every item that has one; last updated …

policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 1
  not #1: blocked by #2
```

One ledger: the work imported from kata, with its order intact (`kata 0002` blocking `kata 0001`),
and a review finding filed by a hook, about a file in a repository gonk checked out and watches.
What keeps it current:

| what changes | what brings it in | restart? |
| --- | --- | --- |
| a repository upstream | `ikigai-gonk checkout --all`, by hand or [on a timer](checkout.md#on-a-timer) | no: the watcher sees the fast-forward |
| a new repository | `ikigai-gonk checkout <url> --write-config` | yes: roots are read at startup |
| kata | `ikigai-gonk kata import <a new export>` | no |
| a roborev review | the hook, by itself | no |
