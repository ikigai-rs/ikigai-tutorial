# A fresh machine, end to end

The three bridges in the order a new machine needs them: the repositories first, then gonk
serving them, then the work already tracked elsewhere brought in, then the hook that keeps new
review findings arriving. Every step is a command from the last three chapters, and this page
runs them in sequence in one scratch home, so the sequence itself is checked, not only its
parts.

Before it: gonk and `ikigai` installed at the versions this book pins
([Which gonk](../introduction.md#which-gonk)), and git.

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
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk checkout "file://$PWD/upstream/flight" "file://$PWD/upstream/notes" --write-config
cloned     flight at …  ~/.ikigai/checkouts/flight
cloned     notes at …  ~/.ikigai/checkouts/notes

browse roots:
  gonk.browse.root = "flight=~/.ikigai/checkouts/flight"
  gonk.browse.root = "notes=~/.ikigai/checkouts/notes"

…/.config/ikigai/config.toml:
  added      gonk.browse.root = "flight=~/.ikigai/checkouts/flight"
  added      gonk.browse.root = "notes=~/.ikigai/checkouts/notes"

gonk reads gonk.browse.root at startup only: restart it to serve the new root(s). Under launchd:
  launchctl kickstart -k gui/$(id -u)/dev.ikigai-rs.gonk
```

## 2. gonk, serving them

gonk reads its roots when it starts. If it is already running as a service, restart it the
way the checkout said:

<!-- transcript: manual — it restarts a launchd service, which a scratch check does not have -->
```console
$ launchctl kickstart -k gui/$(id -u)/dev.ikigai-rs.gonk
```

Here it starts for the first time, with this page as its home. This is the one page where the
server still gets its homes from `HOME` and `XDG_CONFIG_HOME` rather than from `--config-home`
and `--data-home`: `checkout` reads the process's homes and has no flag for either at the gonk
this book pins, and the roots it wrote are `~/.ikigai/checkouts/…`, which mean whatever `HOME`
says to the server that reads them. So the server has to see the same `HOME` the checkout did:

<!-- transcript: serve -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s)
  browse  urn:repo:{flight (watched), notes (watched)}:* — annotations and archive in <urn:iki:browse:graph:default>
…
```

Both roots, both watched. A read through the socket says they are what was cloned:

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

<!-- transcript: run -->
```console
$ ikigai-gonk kata import kata.jsonl --gonk http://127.0.0.1:1070
kata export version 27: 2 issue(s), 1 link(s)
filed     #1       kata 0001  Stop the loop on an empty queue
filed     #2       kata 0002  Write the empty-queue test
linked    #2 blocks #1

2 filed, 0 already there; 0 comment(s), 0 close(s), 1 link(s) added; 0 deleted issue(s) and 0 link(s) skipped
```

Import it again whenever the old tracker has moved on: a re-run converges, adding comments,
closes and links that are new and filing nothing twice ([Importing a kata
ledger](kata.md)).

## 4. The hook

The last piece is one `[[hooks]]` entry in roborev's config, so every completed review files
its findings here from now on ([Filing roborev findings](roborev.md)):

<!-- transcript: file .roborev/config.toml -->
```toml
[[hooks]]
event = "review.completed"
command = "ikigai-gonk roborev file --gonk http://127.0.0.1:1070 --ledger default --root {repo_name} --repo-path {repo} --job {job_id} --sha {sha} --agent {agent} --findings {findings}"
```

(On your machine the port is gonk's own, 1060.) The next review roborev completes on `notes`
runs that line with roborev's values in place; this is what it receives and runs:

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
$ ikigai-gonk roborev file --gonk http://127.0.0.1:1070 --ledger default --root 'notes' --repo-path "$PWD/.ikigai/checkouts/notes" --job '7' --sha 'abc1234' --agent 'claude-code' --findings "$(cat review.md)"
filed    high     #3 The README has a title and no instructions, so a new reader cannot start the project.
```

The finding is about `urn:repo:notes:file:README.md`, a file gonk now serves, so browse's page
for that file and the ledger item find each other.

## Where that leaves you

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

One ledger: the work imported from kata, with its order intact, and a review finding filed by
a hook, about a file in a repository gonk checked out and watches. What keeps it current:

| what changes | what brings it in | restart? |
| --- | --- | --- |
| a repository upstream | `ikigai-gonk checkout --all`, by hand or [on a timer](checkout.md#on-a-timer) | no: the watcher sees the fast-forward |
| a new repository | `ikigai-gonk checkout <url> --write-config` | yes: roots are read at startup |
| kata | `ikigai-gonk kata import <a new export>` | no |
| a roborev review | the hook, by itself | no |
