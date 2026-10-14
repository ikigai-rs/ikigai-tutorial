# Periodic unled audits

Every so often, one repository is audited: two auditors (agents, usually), each in its own copy of
the repository at one pinned commit, each blind to the other and to every earlier review, with a
list of files and a budget, and one rule above the rest: **a bug counts only with a
reproduction**. Their reports are merged by root cause, and each real finding is filed as its own
ledger item, under one item for the audit.

[The audit protocol](../review/audit.md) is why: measured against following a reviewer's leads,
unled audits found about twice as many serious bugs per token, and a second independent auditor
added half again. That chapter has the numbers and what they do and do not show. This one is the
how-to.

## Who

| role | who | does |
| --- | --- | --- |
| the coordinator | a person, or a hub | picks the repository, pins the commit, writes the scope and budget, merges the reports, files the findings |
| auditor A, auditor B | two agents (or people), independent | read, reproduce, report; never fix, never see each other's report |
| whoever fixes | the [ordinary loop](solo.md), later | one finding at a time, each closed with its evidence |

## What runs where

```text
  repository at commit C
     ├── worktree audit-a (detached at C) ── auditor A ── report A ─┐
     └── worktree audit-b (detached at C) ── auditor B ── report B ─┤
                                                                    ▼
                         coordinator: merge by root cause, file into gonk
                         ┌──────────────────────────────────────────────┐
                         │ audit item (scope, budget, what was not read)│
                         │   ├─ finding 1 (reproduction, both auditors) │
                         │   └─ finding 2 (reproduction, auditor B)     │
                         └──────────────────────────────────────────────┘
```

Read-only on the code; everything written goes to the ledger. One machine is enough.

## The loop, step by step

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
```

### 1. Pick, pin and scope

The repository to audit is chosen on a rotation, oldest audit first; one with a large recent change
or a security surface moves up. Here it is a small one, two shell tools:

<!-- transcript: file tools/bin/initials.sh -->
```sh
#!/bin/sh
# initials NAME: the first letter of each word of NAME. "Ada Lovelace" is "AL".
for word in $1; do printf '%s' "$(printf '%s' "$word" | cut -c1)"; done
echo
```

<!-- transcript: file tools/bin/slug.sh -->
```sh
#!/bin/sh
# slug TITLE: a title as a URL slug. "Ada Lovelace" is "ada-lovelace".
printf '%s\n' "$1" | tr 'A-Z ' 'a-z-'
```

<!-- transcript: run -->
```console
$ git -C tools init -q -b main
$ git -C tools add bin
$ git -C tools -c user.name=Ada -c user.email=ada@example.com commit -q -m 'initials and slug'
$ sh tools/bin/initials.sh 'Ada Lovelace'
AL
$ sh tools/bin/slug.sh 'Ada Lovelace'
ada-lovelace
```

The audit's item is its brief: the repository, the commit, the files, the budget and the rules. The
commit is pinned so that every finding is about one known state of the code:

<!-- transcript: run -->
```console
$ printf 'audit: tools at %s\n\nFiles: bin/initials.sh, bin/slug.sh. Budget: 40 tool calls per auditor.\nRead-only. A bug counts only with a reproduction that fails at this commit because of it.\nReport what was not read, and whether the budget or the work ran out.\n' "$(git -C tools rev-parse --short HEAD)" | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append labels=audit key=audit:tools:2026-10-13'
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:claim item=1 coordinator'
#1 claimed by coordinator
[uncacheable]
```

### 2. Two auditors, two trees, blind

Each auditor gets a detached worktree at the pinned commit, so neither can change what the other
reads, and neither is shown the other's report, the repository's earlier findings, or any review's:

<!-- transcript: run -->
```console
$ git -C tools worktree add -q --detach ../audit-a HEAD
$ git -C tools worktree add -q --detach ../audit-b HEAD
$ git -C audit-a log --oneline
… initials and slug
```

What the auditors do with their budget is reading, thinking and probing, which this page cannot
show (an agent does it). What it can show is what they must bring back: reproductions. Auditor A
found that `initials.sh` leaves its argument unquoted, so the shell expands a name as a pattern. The
reproduction runs the tool in a directory with one file in it:

<!-- transcript: run -->
```console
$ mkdir -p probe-a && touch probe-a/zebra
$ cd probe-a && sh ../audit-a/bin/initials.sh '*'
z
```

The initials of the name `*` came out as the first letter of a file name. Auditor B found the same
thing on its own, and one more: `slug.sh` turns every space into a dash, so two spaces make two
dashes:

<!-- transcript: run -->
```console
$ sh audit-b/bin/slug.sh 'Ada  Lovelace'
ada--lovelace
```

Each auditor's report lists its findings with their reproductions, its suspicions as suspicions,
what it did not read, and whether the budget or the work ran out first. Auditor A also suspected
that a name passed as two arguments loses its second word; it is the documented usage that the
name is one argument, so it stays a suspicion and is not filed.

### 3. Merge by root cause, file one item per finding

The coordinator merges the two reports by root cause, so that the glob bug, found twice, is one
finding, credited to both, and files each finding as its own item: keyed so that filing it again
files nothing, a child of the audit, about the file it is in, with its reproduction in the body.
([The audit protocol](../review/audit.md#the-protocol-that-replaced-it) describes the merged report
as one item per repository; that is the audit's item here, and each root cause goes under it as
its own item so that each can be claimed, fixed and closed with its own evidence.)

<!-- transcript: file finding-glob.txt -->
```text
initials.sh expands a name as a glob pattern

Found by: auditor A, auditor B. Severity: wrong output.
Reproduction: in a directory holding only `zebra`, `sh bin/initials.sh '*'` prints `z`; it should print `*`.
Cause: `for word in $1` leaves $1 unquoted.
```

<!-- transcript: file finding-slug.txt -->
```text
slug.sh turns two spaces into two dashes

Found by: auditor B. Severity: wrong output.
Reproduction: `sh bin/slug.sh 'Ada  Lovelace'` prints `ada--lovelace`; it should print `ada-lovelace`.
Cause: `tr` maps each space, and nothing squeezes runs of them.
```

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=audit,bug key=audit:tools:2026-10-13:glob about=urn:repo:tools:file:bin/initials.sh' < finding-glob.txt
#2 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=2 labels=audit,bug key=audit:tools:2026-10-13:slug about=urn:repo:tools:file:bin/slug.sh' < finding-slug.txt
#3 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:link item=2 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=3 type=parent 1'
linked #2 parent #1
[uncacheable]
linked #3 parent #1
[uncacheable]
— batch: 2 commands · 2 uncacheable
```

What is not a finding goes on the audit's own item, so the next audit of this repository starts
from it: the budget, what was not read, and the suspicion.

<!-- transcript: file audit-notes.txt -->
```text
A: 23 of 40 tool calls; the work ran out. Read both files. Suspicion, not reproduced as a defect: a name given as two arguments keeps only the first (the usage says one argument).
B: 31 of 40 tool calls; the work ran out. Read both files.
Not read by either: nothing in scope. Out of scope: how the tools are installed.
```

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:comment item=1 author=coordinator' < audit-notes.txt
commented on #1 (urn:iki:ledger:default:comment:…)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
 1.    #2  open    p1  initials.sh expands a name as a glob pattern  [audit bug]
    p1 — priority 1; last updated …
 2.    #3  open    p2  slug.sh turns two spaces into two dashes  [audit bug]
    p2 — priority 2; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 1
  not #1: claimed by coordinator
[uncacheable]
```

### 4. Close each finding with evidence

The fixing is ordinary work, scheduled from the ledger and done in a separate pull request (an
audit never fixes what it reads). Each finding closes with one of three kinds of evidence, written
on the item before the close: **fixed** (the reproduction as a regression test, and the pull
request), **refuted** (why the reproduction was wrong), or **deferred** (to what).

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=2 reason=done author=coordinator Fixed: https://github.com/you/tools/pull/3 quotes the argument; the reproduction is its regression test.'
closed #2 (done)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:1'
   #1  open    p-  audit: tools at …  [audit]  claimed:coordinator
  iri:      urn:iki:ledger:default:item:…
  key:      audit:tools:2026-10-13
  filed:    …
  updated:  …
  Files: bin/initials.sh, bin/slug.sh. Budget: 40 tool calls per auditor.
  Read-only. A bug counts only with a reproduction that fails at this commit because of it.
  Report what was not read, and whether the budget or the work ran out.
1 comment(s):
  … coordinator
    A: 23 of 40 tool calls; the work ran out. Read both files. Suspicion, not reproduced as a defect: a name given as two arguments keeps only the first (the usage says one argument).
    B: 31 of 40 tool calls; the work ran out. Read both files.
    Not read by either: nothing in scope. Out of scope: how the tools are installed.
[uncacheable]
```

The audit's item stays with the coordinator until its last finding is closed, and then closes with
a line saying what the audit found and what came of it. Its scope, budget and notes are what the
next audit of this repository starts from, and its key names the date, so a later audit is a new
item, never this one reopened.

## What holds it

The **guides**: the audit's brief (the repository, the pinned commit, the file list, the budget);
the rules (read-only, blind, a reproduction for every bug, suspicions reported as suspicions); and
the three kinds of closing evidence.

The **sensors**:

- **The reproduction.** It is the whole filter: a claim that cannot be made to fail at the pinned
  commit is not filed as a bug, however plausible it reads. It also becomes the regression test.
- **The second auditor.** Independent, same protocol, and the check that the first did not miss
  half of what was there; where the two overlap, the overlap is a confidence, and where they do
  not, it is the reason to run two.
- **The pinned commit and detached trees**, so every finding is about one known state, and no
  auditor can change what another reads.
- **The keys**, so the coordinator filing again (or a second coordinator) files each finding once.
- **The report's own account**: what was not read, and whether the budget ran out, which turns "we
  audited it" into a statement someone can check.

## What it costs, and when to choose it

Two strong-model audits of one repository is real money in tokens, and the coordinator's merge is
a person's hour. What it buys is measured, for the authors' repositories and models, in
[The audit protocol](../review/audit.md): more serious bugs per token than following a per-commit
reviewer, with every one of them reproduced. It finds bugs older than any diff, which is what a
per-commit reviewer cannot. It finds nothing between audits, which is what CI and tests are for.

Choose it on a rotation for every repository that matters, and out of turn for one with a large
recent change or a security surface. Choose [Review every commit](review-every-commit.md) instead
or as well where its arithmetic is different (a small team, a cheap local model, a codebase new to
its readers).

## With the starter

The [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter)'s `audits` profile packages
step 1's brief: `tools/audit.py` (and `just audit`) files an unled audit of the files it is given
as one item, its body `.gonk/templates/audit.md` with the repository, the budget and the file list
filled in, and refuses a file that does not exist. `AGENTS.md` gains a "Periodic audits" section.
It replaces review on every commit rather than adding to it, so asking for both is refused, with
the reason. The book checks the starter at the revision in `books/gonk/starter.rev`
([Solo](solo.md#with-the-starter) says more):

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ cd starter && python3 tools/new.py ../library --process audits,review-every-commit
new.py: --process audits,review-every-commit: per-commit review and periodic unled audits are two answers to one question, how bugs are found; the audits profile replaces the other
$ cd starter && python3 tools/new.py ../library --process audits
generated the library-lending sample (python, process audits) in …/library: 61 files
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
## Periodic audits
## Optional: a hook for Claude Code
$ ls library/tools library/.gonk/templates
library/.gonk/templates:
audit.md

library/tools:
audit.py
gonk.py
harness.py
$ cat library/.gonk/process.toml
# How this repository's team works: the process profiles it was generated with, and what
# each one asks of it. Committed. Written by ikigai-gonk-starter's tools/new.py from its
# profiles/; `python3 tools/gonk.py init` and `status` report it and read nothing else to
# decide it. AGENTS.md holds the rules of exactly these profiles, so change both together.
version = 1
process = ["audits"]
hooks = ["claude:require-claim"]
checks = []

[profile.audits]
summary = "periodic unled audits instead of per-commit review: one repository, a file list, a budget, a reproduction per bug, one item per finding"
roles = ["auditor: claims an audit item, reads only what it lists, files one item per reproduced bug"]
```

<!-- transcript: run -->
```console
$ cd library && git init -q && python3 tools/gonk.py init --ledger default --socket ../.ikigai/gonk.sock --holder coordinator
gonk init: …/library
  ok       layout    openspec at openspec/; artifacts at artifacts/; queries at .gonk/queries/; scripts at .gonk/scripts/
  note     harness   4 harness question(s) in .gonk/queries/ are asked by `python3 tools/gonk.py harness` through gonk's store, not published: gonk runs a script under a ceiling of ledger reads, and they read the harness graph
  note     openspec  8 open task(s) in 2 change(s) in flight: `python3 tools/gonk.py seed` files them as ledger items (again is a no-op)
  ok       process   audits (.gonk/process.toml)
  note     audits    periodic unled audits instead of per-commit review: one repository, a file list, a budget, a reproduction per bug, one item per finding
  note     role      auditor: claims an audit item, reads only what it lists, files one item per reproduced bug
  note     hook      the Claude Code claim hook is optional and off: merge hooks/claude/settings.sample.json into .claude/settings.json to turn it on
  ok       ikigai    ikigai 0.1.61 at …/ikigai
  ok       ledger    `default` (.gonk/gonk.toml)
  ok       holder    `coordinator` (.gonk/local.toml, this working tree only)
  ok       identity  the socket's owner: every request over it is `owner`, so the log cannot tell this tree from any other process of yours. A machine that must be named uses QUIC
  ok       gonk      socket …/.ikigai/gonk.sock answers
  ok       items     `default` holds 3 item(s), 2 open
  ok       scripts   gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  ok       scripts   published 4: urn:script:default-open-items-for-artifact, urn:script:default-open-work-by-change, urn:script:default-requirements-with-open-work, urn:script:default-next-for-artifact
  unknown  nl        could not ask gonk about `urn:nl:sparql`

agents over MCP (configure the agent with this command):
  ikigai mcp --override urn:iki:ledger:=…/.ikigai/gonk.sock

ready.
$ cd library && python3 tools/audit.py artifacts/service/lending/domain.py artifacts/cli/library.py --dry-run
Audit library: 2 file(s), 60 tool calls

Repository: `library`. Budget: 60 tool calls. Files, and nothing else:

- `artifacts/service/lending/domain.py`
- `artifacts/cli/library.py`

An **unled audit**: read the files above and find the bugs in them. No earlier findings, no
hints and no suspicions come with it, on purpose: a lead narrows what gets looked at to what
someone already suspected.

- **A reproduction for every bug**: a failing test, or a command and its output. A suspicion
  without one is not reported as a bug.
- **One item per finding**: `python3 tools/gonk.py file "<title>" --labels audit-finding
  --body "<the reproduction, the file and the line>"`, so each is claimed, fixed and closed on
  its own.
- **Stop at the budget.** Say where you stopped and what you did not read.
- **Report** with `--note` listing the items you filed; close `--reason audit-no-change` when
  you found nothing.
$ cd library && python3 tools/gonk.py status
process: audits
no live claim: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
```

`--dry-run` prints the item and files nothing; without it the audit is an item labeled `audit`,
and a second audit of the same files is a second item, on purpose: an audit is periodic. The
template's rules are this chapter's in the starter's own commands: `gonk.py file` for each
finding (step 3), then the audit's `report` naming the items filed, or a close with
`audit-no-change` when nothing was found. The `unknown` row is this
page's gonk mounting no model, so it binds no `urn:nl:sparql` (its banner says so); the starter
reads that refusal as a gonk it could not ask.

Every other pair of profiles combines: `--process solo,audits` is one person auditing their own
repository on a schedule, and `--process solo,spec` is [Solo](solo.md) with [spec
first](spec-first.md)'s rules.
