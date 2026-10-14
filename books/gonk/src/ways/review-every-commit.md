# Review every commit

A read-only reviewer reads every file every commit touches. Each claim it makes is a **pending
finding**; a second model, the judge, checks the serious ones; a person declines or publishes
each finding with a reason or a reproduction; and a real one becomes a ledger item that goes
through the ordinary loop. gonk does all of that from a `post-commit` hook and one setting.

This is the process gonk's review machinery was first built for, and the one its own authors
turned off: [The audit protocol](../review/audit.md) has the measurement. This chapter shows how
it works end to end, so you can decide for yourself; the last section says what it costs and when
it is still the right choice. The pieces are the ones [One-off review](../review/one-off.md)
taught one at a time, and this page links there rather than teaching them again.

## Who

| role | who | holds |
| --- | --- | --- |
| the author | whoever commits: a person, or an agent | nothing of the review's; their hook drops a request, which spends nothing and needs no grant |
| the reviewer | a model, run by gonk on each request, unattended | read every browse root, the browse graph, the model's host; **never** `urn:cap:annotate` |
| the judge | the same mount, asked once more for each serious finding | the reviewer's grant: it runs inside the pass |
| the triager | a person, at gonk's Queue page or over the socket | `urn:cap:annotate`, the one token that publishes |
| whoever fixes it | the [ordinary loop](solo.md) on a ledger item | the ledger's writer |

## What runs where

One machine. gonk holds the store, the queue and the reviewer; the model is a peer gonk mounts at
`urn:llm:`, here a stub on a socket, in practice a model host on the same machine or on a GPU box
over QUIC.

```text
  git commit ──post-commit hook──▶ ikigai-gonk review request <repo> <path>
                                     one file into the queue; no store, no door, no network
                                              │
  gonk ─── urn:space:reviews ──▶ armed reactor: one pass at a time, under grant `reviewer`
                                              │  urn:llm:coder:ask (the mount)
                                              ▼
            pending findings ◀── the judge, once per serious finding
                    │
  a person ─ decline with a reason │ publish with a reproduction ─▶ annotation
                    │
            a ledger item, keyed by the finding
```

## The loop, step by step

### 1. A model, a repository, and the queue

The model is this book's stub, the one [Scripts and sentences in gonk](../scripts/in-gonk.md)
mounts. It also answers what a browse root asks a model: a fixed review of the file below, a
judge's check, and an explanation, told apart by the system prompt each ask carries. It logs a
line for every ask, which is how this page shows what a commit cost.

<!-- transcript: serve -->
```console
$ model-stub "$PWD/model.sock" | tee model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
```

A repository with one file, the one [One-off review](../review/one-off.md) reviewed:

<!-- transcript: run -->
```console
$ git init -q -b main demo
```

<!-- transcript: file demo/src/lib.rs -->
```rust,ignore
/// A name's initials: "Ada Lovelace" is "AL".
pub fn initials(name: &str) -> String {
    name.split(' ').map(|word| &word[..1]).collect()
}
```

Arming the reviewer takes **two facts, and neither is implicit**: a grant written into
`grants.json`, and the word that says to use it. The first line binds the queue at
`urn:space:reviews`; on its own it arms nothing.

<!-- transcript: file .config/ikigai/config.toml -->
```toml
gonk.review.space = "reviews"
gonk.review.grant = "reviewer"
gonk.review.arm = true
```

The grant is exactly this, and gonk checks it against the review's own contract at startup:

<!-- transcript: file .config/ikigai/gonk/grants.json -->
```json
{
  "reviewer": [
    "urn:cap:browse:read:*",
    "urn:cap:net:localhost",
    "urn:cap:store:read:graph:urn:iki:browse:graph:default",
    "urn:cap:store:write:graph:urn:iki:browse:graph:default",
    "urn:cap:exec:gh"
  ]
}
```

Read every root; spend the model, whose host is `localhost` for a socket (for a QUIC peer it is
that peer's host, exactly); read and write browse's own graph, where the archive and the findings
live; and run `gh`, which a review of a pull request uses. What is missing is the point:
**`urn:cap:annotate`**. A pass writes pending findings and nothing else, and publishing a finding
is the only way into the annotations everyone reads, so a reviewer without that token *cannot*
publish. gonk refuses to start rather than arm a reviewer whose grant holds it.

### 2. The hook

<!-- transcript: file demo/.git/hooks/post-commit -->
```sh
#!/bin/sh
for f in $(git diff-tree --root --no-commit-id --name-only -r HEAD); do
  ikigai-gonk review request demo "$f" --config-home ../.config/ikigai --data-home ../.ikigai
done
```

`review request` opens no store, binds no door and dials nothing: it writes one request into the
queue's directory and exits, so a commit never waits on gonk, let alone on a model, and the hook
works while gonk is down. The two flags are this page's scratch homes (git runs a hook in the
repository's top directory, so `..` is the page's); with gonk's default homes, leave them off. A
request is just `(repository, path)`, with no commit in it, so the same file committed three times
while the queue waits is one request, and the pass reviews the file as it is when the pass runs.

<!-- transcript: run -->
```console
$ chmod +x demo/.git/hooks/post-commit
$ printf 'gonk.browse.root = "demo=%s/demo"\ngonk.mount = "prefer urn:llm:=%s/model.sock"\n' "$PWD" "$PWD" >> .config/ikigai/config.toml
```

The root and the mount are written with `printf` and `$PWD` because a `~` in `config.toml` means
the home of whoever runs gonk ([Browse roots](../browse/roots.md)).

### 3. gonk, armed

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  review  urn:space:reviews — ARMED: a drop fires urn:iki:gonk:review:pass under grant `reviewer` (5 scopes, net localhost, no urn:cap:annotate — a pass CANNOT publish). 0 waiting now. …
…
```

The `review` line says what an operator needs the moment this is on: what runs, under which
grant, that it cannot publish, and (in the part the page elides) that passes run one at a time
and nothing bounds the wall clock.

### 4. Commit

<!-- transcript: run -->
```console
$ git -C demo add src/lib.rs
$ git -C demo -c user.name=Ada -c user.email=ada@example.com commit -q -m 'initials'
5d0b19b5150eb39a67cffadd66d6191e3180dbd627c3aa2b5b8a61e39832165f
```

The one line is the hook's: the request's id, a hash of its content, which is why a repeat
collapses into it. The commit is done; the review happens in gonk, in its own time. Wait for it
the way an operator does, on the queue's depth, then see what it cost:

<!-- transcript: run -->
```console
$ until ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:gonk:review:depth' 2>/dev/null | grep -q '1 handled'; do sleep 0.2; done
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:gonk:review:depth'
The review queue is empty: no request is waiting to be reviewed, 1 handled. 1 pass(es) this run, 0 failed; the last ended …s ago. 2 findings minted this run, 1 serious (50%).
[uncacheable]
$ cat model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
model-stub: ask 1: review src/lib.rs
model-stub: ask 2: judge one finding
```

Two asks for one file: the review, and the judge on the one finding the review called serious.
The minor one is minted and counted, and nobody is asked about it unless they look. The depth's
last figure, the **serious share**, is a sensor in its own right: severity is the model's own word,
so a share that jumps with no change of model or prompt is the label inflating, not the code
getting worse.

### 5. Triage

The findings, as anyone holding a browse read sees them:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:demo:findings:src/lib.rs as=text/plain'
--- findings (2) ---
Findings wait for a person. Nothing here blocks a commit, a push or a merge, and nothing reaches the annotation family until someone publishes it.
src/lib.rs L3 [pending] major "name.split(' ').map(|word| &word[..1]).collect()" -- Slicing the first byte panics on an empty word, so a name with two spaces in a row, or an empty name, crashes the caller.
src/lib.rs L2 [pending] minor "pub fn initials(name: &str) -> String {" -- Returning a String allocates on every call; return a &str instead.
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:demo:findings:src/lib.rs' 2>/dev/null | jq -r '.[] | "\(.id) \(.severity) judge: \(.judge.verdict // "none")"'
645d521aab16830f076e0857 major judge: confirmed
1fa92632a54c273633bc10fe minor judge: none
```

The second line of the listing is the rule this whole process rests on: **a finding is a
question, not a gate**. The judge's verdict orders the Queue (confirmed first, refuted folded
last) and decides nothing. Its id is a hash of the pass and the quote, so the same review of the
same bytes names the same findings on any machine.

A person decides. In gonk that is the **Queue** page, which asks only about the serious findings
and draws the judge's order; it needs a signed-in person, so this page cannot drive it:

<!-- transcript: manual — a signed-in browser session (a passkey ceremony), which no check can perform -->
```console
$ open http://localhost:1070/queue
```

Over the socket the owner holds `urn:cap:annotate` too, so the same two decisions are Sinks to
the findings. The minor one is declined with a reason from the closed set; the major one is
published, marked reproduced, with the reproduction as its note. The reproduction is the
three-line test in [One-off review](../review/one-off.md#deciding), and making it is the one step
of this loop no machine does for you.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:finding:1fa92632a54c273633bc10fe decision=decline reason=no-issue A borrowed str cannot hold letters taken from several words.'
urn:iki:finding:1fa92632a54c273633bc10fe
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:finding:645d521aab16830f076e0857 decision=publish reproduced=yes initials("Ada  Lovelace") panics: the empty middle word has no byte 0.'
urn:iki:annotation:645d521aab16830f076e0857
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:demo:findings:src/lib.rs as=text/plain'
--- findings (0) ---
Findings wait for a person. Nothing here blocks a commit, a push or a merge, and nothing reaches the annotation family until someone publishes it.
1 declined finding on this file, on 1 distinct quote (no-issue 1)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:demo:annotations:src/lib.rs' 2>/dev/null | jq -r '.[] | "\(.iri) derived from \(.derived_from)"'
urn:iki:annotation:645d521aab16830f076e0857 derived from urn:iki:finding:645d521aab16830f076e0857
```

Nothing is pending on the file now. The declined finding is summarized by its reason, which is what
marks the same claim when a later pass makes it again; the published one is an annotation that
everyone who reads the file sees, one hop from the model's claim it came from.

### 6. File it

A published finding is a fact about the code, and fixing it is work, so it goes on the ledger.
Filing is a person's act here too, keyed by the finding so that filing it twice files once:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append key=finding:645d521aab16830f076e0857 about=urn:iki:finding:645d521aab16830f076e0857 priority=1 labels=review initials panics on an empty word'
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append key=finding:645d521aab16830f076e0857 about=urn:iki:finding:645d521aab16830f076e0857 priority=1 labels=review initials panics on an empty word'
#1 urn:iki:ledger:default:item:… existing open
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:item:1'
   #1  open    p1  initials panics on an empty word  [review]
  iri:      urn:iki:ledger:default:item:…
  key:      finding:645d521aab16830f076e0857
  filed:    …
  updated:  …
  about:    urn:iki:finding:645d521aab16830f076e0857
[uncacheable]
```

`about` joins the item to the finding, and through it to the file and the annotation, in one
graph: "what is open against this file" is a query, not a search. From here the item is ordinary
work: claimed, fixed on a branch with the reproduction as its regression test, and closed with the
pull request as evidence, as in [Solo on one machine](solo.md).

⚠ **gonk files nothing from its own review by itself.** The keyed append above is the whole of the
step, and something has to send it: a person after triage, as here, or a script of your own over
the socket. (For another tool's findings gonk ships a command, below; for its own there is none
at the revision this book pins.)

### The same process with another reviewer

Nothing above depends on gonk being the reviewer. A dedicated review tool that runs on every
commit, such as [roborev](https://github.com/kenn-io/roborev), implements the same process
with its own reviewer, its own queue and its own report; what gonk adds is the ledger the
findings land in. [Filing roborev findings](../bridges/roborev.md) is that bridge: the tool's
completion hook files each finding as a ledger item, keyed so a re-run files nothing new, and from
there triage, the claim and the close are the steps above. A team that already keeps its work in
another tracker, such as [kata](https://github.com/kenn-io/kata), brings that ledger across once with
[Importing a kata ledger](../bridges/kata.md). Either way the process is the one this chapter
walks: a reviewer that cannot publish, a person who decides with a reason or a reproduction, and
a ledger item for what is real.

## What holds it

The **guides**, the rules written down before anything runs:

- *Nothing is published except by a person.* Not a policy but a grant: the reviewer's lacks the
  token, and gonk will not arm one that has it.
- *Decline with a reason* from a closed set (`misread`, `restates`, `no-issue`, `wont-fix`,
  `duplicate`), because "why not" is what the next review can learn from.
- *Publish with a reproduction*: `reproduced=yes` records that a person made the defect happen.
- *The prompt is not yours to steer.* The review and judge prompts are `ikigai-browse`'s, and
  nothing gonk renders reaches a pass, so no banner or form can tell a model which severities
  get read.

The **sensors**, which say when it is going wrong:

- **The judge**, a second reading with the context the reviewer lacked, on every serious finding.
- **The depth**, `urn:iki:gonk:review:depth`, and the Queue badge that polls it: a queue that is
  armed and not empty with nothing in flight is stuck, and says so.
- **The serious share** of what a run minted, for label inflation.
- **The keyed append**, so a re-run or a second person files nothing twice.

And one thing that is deliberately **not** a sensor: the queue blocks nothing. A finding never
fails a commit, a push or CI. If you want a gate, it is a test, and the reproduction is where it
comes from.

## What it costs, and when to choose it

Every changed file is a model call, plus one per serious finding for the judge. Passes run one at
a time, so a commit touching forty files occupies the reviewer for forty passes. And every serious
finding costs a person's attention, which is the expensive part: on the authors' own
repositories the queue passed a thousand pending findings, and measured against unled audits a
per-commit reviewer found less than half as many serious bugs per token, because a diff never shows
it a defect older than the diff ([The audit protocol](../review/audit.md) has the numbers and
their limits). So on the machine this book was written on the hook is installed everywhere and
`gonk.review.arm` is unset: requests are dropped and nothing drains them.

Choose it anyway when the arithmetic is different:

- **A small team**, where few commits land and someone reads every change regardless; the
  findings are a second reader, not a queue.
- **A fast, cheap model**, local and idle, where a pass costs nothing worth counting and the queue
  drains in seconds.
- **A codebase new to the team**, where a second reading of each change teaches what the code
  does and precision matters less than attention.

Turning it off is one line: remove `gonk.review.arm = true`. The hook keeps dropping requests and
nothing drains them; a person can still run one by hand over the socket (the `review` line of an
unarmed banner prints how), and [One-off review](../review/one-off.md) is always there for a file
someone already suspects.

## With the starter

The [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter)'s `review-every-commit`
profile packages step 2: `tools/new.py --process review-every-commit` adds a "Review on every
commit" section to `AGENTS.md` and a committed `hooks/git/post-commit`, whose
`review-on-commit.py` runs the same `ikigai-gonk review request <root> <path>` for each file a
commit touched, detached, so a commit never waits and never fails because of it. Its settings (the
root's name, gonk's config file, paths never queued) are a `[review]` table in
`.gonk/process.toml`. The book checks the starter at the revision in `books/gonk/starter.rev`
([Solo](solo.md#with-the-starter) says more):

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ cd starter && python3 tools/new.py ../library --process review-every-commit
generated the library-lending sample (python, process review-every-commit) in …/library: 61 files
next, in that directory:
  git init, then commit everything it wrote (the one commit where every file is yours)
  python3 scripts/test.py                      every test, as CI runs them
  python3 tools/gonk.py init --ledger <name> --socket ~/.ikigai/gonk.sock --holder <you>
  git config core.hooksPath hooks/git     the post-commit review hook
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
## Review on every commit
## Optional: a hook for Claude Code
$ ls library/hooks/git
post-commit
review-on-commit.py
$ cat library/.gonk/process.toml
# How this repository's team works: the process profiles it was generated with, and what
# each one asks of it. Committed. Written by ikigai-gonk-starter's tools/new.py from its
# profiles/; `python3 tools/gonk.py init` and `status` report it and read nothing else to
# decide it. AGENTS.md holds the rules of exactly these profiles, so change both together.
version = 1
process = ["review-every-commit"]
hooks = ["claude:require-claim", "git:post-commit"]
checks = ["review"]

[profile.review-every-commit]
summary = "a read-only reviewer on every commit: a post-commit hook asks gonk to review the touched files, and a person triages the findings"
roles = ["reviewer: gonk's review pass, read-only; it cannot publish a finding", "triager: a person who decides each finding and files the ones to act on"]

# The review-every-commit profile: what hooks/git/post-commit asks gonk for. Committed, since
# every clone commits to the same repository on the same gonk.
[review]
# The name gonk serves this repository under, its `gonk.browse.root = "<name>=<path>"` line.
# Unset, it is this working tree's directory name.
# root = "my-repo"
# gonk's config.toml, when it is not in gonk's default config home: `ikigai-gonk review
# request` reads the queue's location from it.
# config = "~/.config/ikigai/config.toml"
# Paths never queued: machine-written files a review could only restate. Shell-style
# patterns matched against the whole path as git spells it, where `*` also matches `/`.
skip = ["*.lock", "package-lock.json", "*/package-lock.json"]
```

The hook is installed by pointing git at `hooks/git` (`just hooks` does the same), and `init`, the
profile's `review` check, then asks this page's gonk whether it can review this repository:

<!-- transcript: run -->
```console
$ cd library && git init -q && git config core.hooksPath hooks/git && python3 tools/gonk.py init --ledger default --socket ../.ikigai/gonk.sock --holder triager
gonk init: …/library
  ok       layout               openspec at openspec/; artifacts at artifacts/; queries at .gonk/queries/; scripts at .gonk/scripts/
  note     harness              4 harness question(s) in .gonk/queries/ are asked by `python3 tools/gonk.py harness` through gonk's store, not published: gonk runs a script under a ceiling of ledger reads, and they read the harness graph
  note     openspec             8 open task(s) in 2 change(s) in flight: `python3 tools/gonk.py seed` files them as ledger items (again is a no-op)
  ok       process              review-every-commit (.gonk/process.toml)
  note     review-every-commit  a read-only reviewer on every commit: a post-commit hook asks gonk to review the touched files, and a person triages the findings
  note     role                 reviewer: gonk's review pass, read-only; it cannot publish a finding
  note     role                 triager: a person who decides each finding and files the ones to act on
  note     hook                 the Claude Code claim hook is optional and off: merge hooks/claude/settings.sample.json into .claude/settings.json to turn it on
  ok       hook                 post-commit asks gonk to review each commit (core.hooksPath)
  ok       ikigai               ikigai 0.1.56 at …/ikigai
  ok       ledger               `default` (.gonk/gonk.toml)
  ok       holder               `triager` (.gonk/local.toml, this working tree only)
  ok       identity             the socket's owner: every request over it is `owner`, so the log cannot tell this tree from any other process of yours. A machine that must be named uses QUIC
  ok       gonk                 socket …/.ikigai/gonk.sock answers
  ok       items                `default` holds 1 item(s), 1 open
  ok       scripts              gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  ok       scripts              published 4: urn:script:default-open-items-for-artifact, urn:script:default-open-work-by-change, urn:script:default-requirements-with-open-work, urn:script:default-next-for-artifact
  ok       nl                   gonk offers `urn:nl:sparql`: drafting a query from a sentence (ikigai-nl)
  missing  review               gonk serves no browse root named `library`: its operator adds `gonk.browse.root = "library=<path to this repository on gonk's machine>"`, or name gonk's root for it in .gonk/process.toml's `[review] root`

agents over MCP (configure the agent with this command):
  ikigai mcp --override urn:iki:ledger:=…/.ikigai/gonk.sock

1 thing(s) missing.
$ cd library && python3 tools/gonk.py status
process: review-every-commit
no live claim: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
```

The one thing missing is true: this page's gonk serves `demo`, not the repository just generated.
With its root added to gonk's `config.toml` (step 2's `printf`) and gonk restarted, `init` goes on
to ask whether gonk binds a review pass (a model is mounted) and whether its queue is armed, the
two things step 3's banner shows. A tree that reaches gonk over QUIC is told instead that its
commits queue nothing: the queue is a directory on gonk's machine, which no network door reaches.

Profiles combine (`--process solo,review-every-commit` is [Solo](solo.md)'s roles with this
review), with one exception: this profile and [unled audits](audits.md) answer the same question,
how bugs are found, and `tools/new.py` refuses the pair.
