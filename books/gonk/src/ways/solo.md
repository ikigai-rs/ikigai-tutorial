# Solo on one machine

One person and one agent, one machine, one gonk. The agent takes an item, does the work on a
branch, gets a pull request through CI, and reports with the evidence; the person, who dispatched
the work to begin with, checks the evidence and closes. Working alone you hold **both** roles,
and the process still keeps them apart: a report is not a close, and a close is a statement that
the evidence holds.

## Who

| role | who | how it reaches gonk |
| --- | --- | --- |
| the dispatcher | you | the socket, as the owner |
| the worker | you, or an agent you started | the socket, as the owner, with its own holder name |
| the model (optional) | a local model gonk mounts, for explanations and one-off review | gonk dials it; nobody else does |

Both of you are the **owner** over the socket ([Over the socket](../agent/socket.md)): the socket
cannot tell your terminal from an agent's, and the access log says `principal=owner` for both. What
tells you apart in the ledger is the holder name on a claim. An agent that should be told apart by
gonk itself, or held to less than everything, is the next chapter's.

## What runs where

```text
  one machine
  ┌──────────────────────────────────────────────────────────────────────────┐
  │  you ──┐                                                                 │
  │        ├── ikigai --mount urn:gk:=…/gonk.sock ──▶ gonk ──▶ store         │
  │  agent ┘          (the owner-only socket)          │                     │
  │                                                    └─▶ model (optional)  │
  │                                                                          │
  │  git: a branch per item ────────────┐                                    │
  └─────────────────────────────────────┼────────────────────────────────────┘
                                        └──push──▶ the forge: pull request + CI
```

The forge (GitHub, say) is the one thing not on the machine; this page cannot reach it, and the
steps that need it are marked.

## The loop, step by step

### 0. Once: a gonk, a repository, and a model

A model is optional, and this page mounts the book's stub, so that the one explanation below is
answered without a network: in practice it is a local model host
([Explain, with a mounted model](../browse/explain.md#a-real-model)). In a new empty directory, as
in [every chapter](../introduction.md#a-gonk-per-chapter): if the gonk and the `model-stub` from
[Two teams, one gonk](../teams/two-teams.md) are still running, stop them first, since this gonk
takes the same port.

<!-- transcript: serve -->
```console
$ model-stub "$PWD/model.sock" | tee model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
```

<!-- transcript: run -->
```console
$ git init -q -b main demo
$ mkdir -p .config/ikigai
$ printf 'gonk.browse.root = "demo=%s/demo"\ngonk.mount = "prefer urn:llm:=%s/model.sock"\n' "$PWD" "$PWD" > .config/ikigai/config.toml
```

<!-- transcript: file demo/src/lib.rs -->
```rust,ignore
/// A name's initials: "Ada Lovelace" is "AL".
pub fn initials(name: &str) -> String {
    name.split(' ').map(|word| &word[..1]).collect()
}
```

<!-- transcript: run -->
```console
$ git -C demo add src/lib.rs
$ git -C demo -c user.name=Ada -c user.email=ada@example.com commit -q -m 'initials'
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  socket  …/.ikigai/gonk.sock — owner only
…
```

### 1. File

The work starts as an item, with what it is for in its body. That body is the brief: it is what
the worker reads, and what the evidence will be checked against.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=bug initials panics on an empty word'
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:comment item=1 author=ada Done when initials("Ada  Lovelace") is "AL" and a test says so.'
commented on #1 (urn:iki:ledger:default:comment:…)
[uncacheable]
```

### 2. Ask what is next, and claim it

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
 1.    #1  open    p1  initials panics on an empty word  [bug]
    p1 — priority 1; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 0
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:claim item=1 lease=30m agent'
#1 claimed by agent until …
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
nothing is ready (1 open item(s) excluded: #1 claimed by agent)
policy: priority-recency
[uncacheable]
```

A claim is a **lease**: it says who holds the item and until when. `next` stops offering a claimed
item, so nothing else picks it up, and a lease that runs out does not quietly free it: `next`
names the holder whose lease expired, and only a deliberate takeover moves it
([the ledger's claims](../ledger/your-own.md)). A worker that is still at it renews:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:claim item=1 renew=true lease=30m agent'
#1 renewed by agent until …
[uncacheable]
```

`renew=true` renews and never takes: if the claim had changed hands, this would be refused rather
than quietly taking the item back.

### 3. Read before you change

The model is optional and read-only: it explains a file once per version of its content, and every
later read is the archive's ([Explain, with a mounted model](../browse/explain.md)).

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:demo:explain:src/lib.rs'
Defines `initials`, which joins the first letter of each space-separated word of a name into one string.
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:demo:explain:src/lib.rs'
Defines `initials`, which joins the first letter of each space-separated word of a name into one string.
[uncacheable]
$ cat model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
model-stub: ask 1: explain
```

### 4. Branch, fix, commit

One branch per item, named for it. The fix comes with the test the item's body asked for.

<!-- transcript: run -->
```console
$ git -C demo switch -q -c item-1-empty-word
$ printf '/// A name'"'"'s initials: "Ada Lovelace" is "AL", and so is "Ada  Lovelace".\npub fn initials(name: &str) -> String {\n    name.split_whitespace().filter_map(|word| word.get(..1)).collect()\n}\n' > demo/src/lib.rs
$ git -C demo -c user.name=Agent -c user.email=agent@example.com commit -q -am 'initials: skip empty words (ledger item 1)'
$ git -C demo log --oneline
… initials: skip empty words (ledger item 1)
… initials
```

### 5. A pull request, and CI

<!-- transcript: manual — it needs a forge (GitHub here) and its CI, which this book's checks do not reach -->
```console
$ git -C demo push -u origin item-1-empty-word
$ gh pr create --fill
https://github.com/you/demo/pull/7
$ gh pr checks 7 --watch
```

The pull request is where the change is reviewed and where CI says whether it holds. Green CI is
the evidence a machine can check; nothing in this loop accepts "it works on my machine".

### 6. Report, with the evidence

The worker hands the item back with the evidence as a comment, marked as a hand-back, so the
ledger knows the item now waits for its dispatcher, and then releases its claim. The URL is the one
step 5 printed (this page cannot open a real pull request, so it is the shape of one). `author=` is
the worker's name for itself, a claim the socket records and does not prove:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:comment item=1 author=agent handback=true https://github.com/you/demo/pull/7 adds the test, CI green'
commented on #1 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'delete urn:gk:iki:ledger:claim item=1 from=agent'
#1 released
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:next'
nothing is ready (1 open item(s) excluded: #1 handed back, awaiting the dispatcher)
policy: priority-recency
[uncacheable]
```

The release names its holder with `from=`, which makes it a compare-and-set: had someone taken
the item over meanwhile, it would release nothing. And `next` still does not offer the item to
anyone, now for the other reason: it is the dispatcher's.

### 7. Close, as your own dispatcher

The dispatcher reads the evidence (the pull request merged, CI green, the test in the diff), and
only then closes, with a reason and a note that says what was checked. (Had the worker kept its
claim, the close would have released it in the same write and said so.)

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=1 reason=done author=ada Checked: PR 7 merged with CI green; the test covers two spaces.'
closed #1 (done)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:1'
   #1  closed  p1  initials panics on an empty word  [bug]  (done)
  iri:      urn:iki:ledger:default:item:…
  filed:    …
  updated:  …
3 comment(s):
  … ada
    Done when initials("Ada  Lovelace") is "AL" and a test says so.
  … agent
    https://github.com/you/demo/pull/7 adds the test, CI green
  … ada
    Checked: PR 7 merged with CI green; the test covers two spaces.
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:doctor'
doctor: default (lifecycle kata-flight): no problems
checked: orphaned, abandoned, lease-expired, unknown-state, two-states, awaiting-closed at …
[uncacheable]
```

## What holds it

The **guides** are the rules both of you work to, written where an agent reads them (an
`AGENTS.md` in the repository, say): no write without a claim; reproduce a reported bug before
fixing it; every change through a pull request with green CI; report with evidence; the
dispatcher closes. And the item's body, which says what done means.

The **sensors**:

- **The claim and its lease.** A second claim on a held item is refused, naming the holder; a
  lease that runs out shows up in `next` rather than freeing the work silently.
- **CI** on the pull request, the one check that does not depend on anyone's word.
- **The hand-back**, which takes the item out of `next` until a dispatcher answers it.
- **Your own evidence check** at close: the weakest sensor here, because the checker is the
  worker. Make it a habit to open the pull request before closing, not after.
- **The doctor**, which reports claims nobody is working (a closed item still held, a lease long
  expired). Clean here: the worker let go of its claim when it handed the item back.

## What it costs, and when to choose it

It costs nearly nothing: one gonk, one socket, no identities to mint, no second machine. What it
does not give you is separation. The agent holds the owner's authority unless its session seals
itself to less ([Over the socket](../agent/socket.md#whose-authority)), the log cannot tell the
two of you apart, and the evidence check is self-review.

Choose it when one person is responsible for everything that merges and the agent works where
that person can see it. Move to [remote workers](workers.md) when an agent should run unattended
or on another machine, hold less than everything, and be unable to close its own work.

## With the starter

The [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter) packages this process as a
**profile**, `solo`: its `tools/new.py --process solo` generates a repository (a library-lending
sample, with its specs) whose `AGENTS.md` holds the rules above and no other profile's, and whose
`.gonk/process.toml` records the process for `tools/gonk.py` to read back. `new.py` says to
`git init -b main`, because the starter's worker loop branches from `origin/main`. The book checks the
starter at one revision, held in one file (`books/gonk/starter.rev`) the way [the gonk it
pins](../introduction.md#which-gonk) is. (`LC_ALL=C`, so the listing's order is the same on every
machine.)

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ cd starter && python3 tools/new.py ../library --process solo
generated the library-lending sample (python, process solo) in …/library: 60 files
next, in that directory:
  git init -b main, then commit everything it wrote (the one commit where every file is yours; -b main because the worker loop branches from origin/main, and git's own default is still master where init.defaultBranch is unset)
  python3 scripts/test.py                      every test, as CI runs them
  python3 tools/gonk.py init --ledger <name> --socket ~/.ikigai/gonk.sock --holder <you>
  python3 tools/gonk.py seed                   file the open changes' tasks as items
  python3 tools/gonk.py next                   what to do first, and why

AGENTS.md holds this process's rules; README.md walks through the sample.
$ LC_ALL=C ls -A library
.gitattributes
.github
.gitignore
.gonk
ACCESSIBILITY.md
AGENTS.md
CLAUDE.md
README.md
artifacts
docs
hooks
justfile
openspec
scripts
tests
tools
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
## Optional: a hook for Claude Code
$ cat library/.gonk/process.toml
# How this repository's team works: the process profiles it was generated with, and what
# each one asks of it. Committed. Written by ikigai-gonk-starter's tools/new.py from its
# profiles/; `python3 tools/gonk.py init` and `status` report it and read nothing else to
# decide it. AGENTS.md holds the rules of exactly these profiles, so change both together.
version = 1
process = ["solo"]
hooks = ["claude:require-claim"]
checks = []

[profile.solo]
summary = "one person and their agent: you are your own dispatcher, and close after checking your own evidence"
roles = ["you: both the dispatcher and whoever does the work"]
```

`AGENTS.md`'s numbered sections 1 to 6 are this chapter's guides, and `CLAUDE.md` is a link to it.
Section 7 is one every generated repository carries, whatever its profile: read the repository's
`ACCESSIBILITY.md` (what the project shows people, and how that is tested) before changing a page,
terminal text or anything else people see, and never claim testing that did not happen. The one
hook every profile carries, `claude:require-claim`, is an optional Claude Code hook that refuses an
edit while the working tree holds no claim; it is off until its settings are merged. `init` names
the ledger, the socket and the holder this working tree claims under, then asks this page's gonk:

<!-- transcript: run -->
```console
$ cd library && git init -q && python3 tools/gonk.py init --ledger default --socket ../.ikigai/gonk.sock --holder agent
gonk init: …/library
  ok       layout    openspec at openspec/; artifacts at artifacts/; queries at .gonk/queries/; scripts at .gonk/scripts/
  note     harness   4 harness question(s) in .gonk/queries/ are asked by `python3 tools/gonk.py harness` through gonk's store, not published: gonk runs a script under a ceiling of ledger reads, and they read the harness graph
  note     openspec  8 open task(s) in 2 change(s) in flight: `python3 tools/gonk.py seed` files them as ledger items (again is a no-op)
  ok       process   solo (.gonk/process.toml)
  note     solo      one person and their agent: you are your own dispatcher, and close after checking your own evidence
  note     role      you: both the dispatcher and whoever does the work
  note     hook      the Claude Code claim hook is optional and off: merge hooks/claude/settings.sample.json into .claude/settings.json to turn it on
  ok       ikigai    ikigai 0.1.67 at …/ikigai
  ok       ledger    `default` (.gonk/gonk.toml)
  ok       holder    `agent` (.gonk/local.toml, this working tree only)
  ok       identity  the socket's owner: every request over it is `owner`, so the log cannot tell this tree from any other process of yours. A machine that must be named uses QUIC
  ok       gonk      socket …/.ikigai/gonk.sock answers
  ok       items     `default` holds 1 item(s), 0 open
  ok       scripts   gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  ok       scripts   published 4: urn:script:default-open-items-for-artifact, urn:script:default-open-work-by-change, urn:script:default-requirements-with-open-work, urn:script:default-next-for-artifact
  ok       nl        gonk offers `urn:nl:sparql`: drafting a query from a sentence (ikigai-nl)

agents over MCP (configure the agent with this command):
  ikigai mcp --override urn:iki:ledger:=…/.ikigai/gonk.sock

ready.
$ cd library && python3 tools/gonk.py status
process: solo
no live claim: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
```

`init` also published the sample's four saved queries to gonk ([Scripts and
sentences](../scripts/in-gonk.md)), as the owner may. `status` is the question the claim hook asks
before every edit; here it says to claim first, which is step 2 above as `python3 tools/gonk.py
claim <number>`. The `harness` row is about the repository's harness, its rules and checks as a
graph, which [Harness engineering](harness.md) walks through.

Profiles **combine**: `--process` takes a list, because they answer different questions (who
dispatches, how the work is defined, how bugs are found), so `--process solo,spec` is this chapter
with [spec first](spec-first.md)'s rules added. With no `--process` at all a repository gets
`solo,workers,spec`. The one pair refused is [unled audits](audits.md) with [review every
commit](review-every-commit.md), two answers to the same question.
