# Harness engineering

Six of the chapters before this one are processes (the seventh, [Routing work to
agents](routing.md), runs one of them unattended), and each ended with a section called "What holds
it": the rules the work follows and the checks that catch it when it does not. This chapter is
about those rules and checks themselves, around any of the six: what they are, where each one
acts, which of them ever does anything, and how they get better.

The words are Birgitta Böckeler's, from [Harness engineering for coding agent
users](https://martinfowler.com/articles/harness-engineering.html) (martinfowler.com, 2 April
2026). In our words, and briefly:

- A coding agent's **harness** is the set of **controls** around it that make what it produces
  more trustworthy. A control is a **guide** or a **sensor**.
- A **guide** acts *before* the agent does (feedforward): a rule in `AGENTS.md`, a brief, a
  template it starts from. A **sensor** acts *after* (feedback): it observes what was done and
  refuses, warns or reports, ideally in words the agent can act on. Guides alone are rules
  nobody checks; sensors alone let an agent make the same mistake every time.
- Each control is **computational** (a program decides: a test, a linter, a hook that refuses)
  or **inferential** (a model or a person decides: a review, a judge, a dispatcher reading the
  evidence). Computational controls are cheap and deterministic, so they can run on every change;
  inferential ones see what a program cannot, at a cost and without a guarantee.
- Controls act at **stages** of a change's life: as the agent calls a tool, at commit, in CI, at
  merge, and outside any one change (a scheduled drift scan, a periodic audit). The earlier a
  control acts, the cheaper the mistake it stops.
- Controls regulate **categories**: maintainability, architecture fitness and behavior. The gonk
  starter adds a fourth, authority and safety, for the rules about who may do what: claim, merge,
  close.
- The **steering loop** is the person's part. When a failure recurs, it becomes a new control or
  a better one, so the human's job is to "steer the agent by iterating on the harness".
- A **harness template** packages guides and sensors for one kind of project, so a new
  repository starts with them rather than growing them one incident at a time.

Her article ends on open questions, and two of them are what this chapter is about. How would you
measure a harness's **coverage**, the way code coverage measures tests? And when a sensor never
fires, is that a sign the work is good, or that the sensor cannot see?

The worked example is the [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter), which
is a harness template: every repository it generates carries its harness **as a graph**, logs
every time a sensor fires, and asks gonk four coverage questions of both. Its loop is this
chapter's loop, so where the six process chapters before it end with "With the starter", this one is the
starter from the beginning.

## Who

| role | who | does |
| --- | --- | --- |
| the person who steers | whoever owns the process | reads the coverage answers; turns a recurring failure into a new or better control |
| the agent | any agent working in the repository | reads the guides; trips the sensors; is told why |
| the sensors | the repository's own tools and hooks | refuse or warn, and record one line each time |
| the dispatcher | the owner of the gonk the work lives in | loads the harness into gonk and asks the coverage questions |
| gonk | the same gonk as the ledger | holds the harness graph and the firings beside the ledger, and answers SPARQL over them |

## What runs where

```text
  the repository (committed)                      this working tree (git-ignored)
  ┌─────────────────────────────────────┐         ┌──────────────────────────────────┐
  │ AGENTS.md ───────────── the guides  │         │ .gonk/state/harness/firings.log  │
  │ .gonk/harness/harness.ttl           │         │  one line per refusal or warning │
  │   every guide and sensor, as RDF    │         └─────────────────▲────────────────┘
  │ .gonk/harness/vocabulary.ttl        │                           │
  │ tools/, hooks/ ──────── the sensors ├───────── each firing ─────┘
  │ .gonk/queries/harness-*.rq          │
  └──────────────────┬──────────────────┘
                     │ python3 tools/gonk.py harness  (the owner's socket)
                     ▼
  gonk: the named graph urn:iki:harness:graph:<ledger>, beside the ledger's own,
        holding the description and every tree's firings; the four questions asked of it
```

Everything but gonk is files in the repository. The graph is committed, so a change to a control is
a reviewed change like any other; the firing log is per working tree and never committed.

## The loop, step by step

This page's gonk, as in [every chapter](../introduction.md#a-gonk-per-chapter) before it: if the
one from [Periodic unled audits](audits.md) is still running, stop it first, since this one takes
the same port.

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
```

### 1. Generate a repository with a profile

The starter's `tools/new.py` generates a repository for one process, here [Solo on one
machine](solo.md)'s. The book checks the starter at the revision in `books/gonk/starter.rev`, which
carries the harness and the vocabulary's complete definition:

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
$ LC_ALL=C ls library/.gonk/harness library/.gonk/queries library/tools
library/.gonk/harness:
harness.ttl
vocabulary.ttl

library/.gonk/queries:
README.md
harness-firings-by-control.rq
harness-guides-without-sensor.rq
harness-sensors-never-fired.rq
harness-sensors-without-guide.rq
open-items-for-artifact.rq
open-work-by-change.rq
requirements-with-open-work.rq

library/tools:
gonk.py
harness.py
```

`harness.ttl` is assembled the way `AGENTS.md` is, from one fragment per section of the rules, so
a repository describes the controls of the process it was generated for and no others. Another
profile, or several, gives a different graph.

### 2. The harness as a graph

Each rule of `AGENTS.md` is a **guide**, and each place the tools enforce one is a **sensor**, both
with a stable IRI whose last segment never changes, so that every firing recorded under it stays
joined to it. (`guide:` and `ctl:` abbreviate `urn:iki:harness:starter:guide:` and
`urn:iki:harness:starter:control:`.) One rule, from `AGENTS.md`'s fifth section:

<!-- transcript: run -->
```console
$ awk -v RS= '/^guide:report-with-evidence /' library/.gonk/harness/harness.ttl
guide:report-with-evidence a harness:Guide ;
    rdfs:label "report with evidence" ;
    rdfs:comment "Finished work is reported with the evidence that it is done, a pull request's URL or the reproduction that shows it is not a defect, recorded as a comment a later reader can check." ;
    harness:statedIn "AGENTS.md" ; harness:section "5. Report with evidence; the dispatcher closes" ;
    harness:direction harness:feedforward ; harness:execution harness:inferential ;
    harness:stage harness:toolCall ; harness:category harness:behavior .
```

A guide is always feedforward and inferential: an agent reads it and applies it, or does not. The
sensor that checks this one:

<!-- transcript: run -->
```console
$ awk -v RS= '/^ctl:evidence-required /' library/.gonk/harness/harness.ttl
ctl:evidence-required a harness:Control ;
    rdfs:label "report and close need evidence" ;
    rdfs:comment "`tools/gonk.py report` and `close` refuse without `--evidence`, or with one that is not a URL, before anything reaches the ledger." ;
    harness:direction harness:feedback ; harness:execution harness:computational ;
    harness:stage harness:toolCall ; harness:category harness:behavior ;
    harness:outcome harness:refuse ;
    harness:enforcedAt "tools/gonk.py#evidence_note" ;
    harness:statedBy guide:report-with-evidence ;
    harness:firedBy test:test_harness:SensorsFire.test_evidence_required .
```

Feedback and computational, acting as the tool is called, and it refuses. `enforcedAt` says where
its code is, `statedBy` names the rule it mechanizes, and `firedBy` names the test that drives it
and sees the firing recorded, which is the proof that it *can* fire. A graph like this drifts from
the code it describes unless something holds the two together, and `tools/harness.py check` is that
something: it reads the control ids the code names, holds them to the graph in both directions,
the graph to its vocabulary, and each guide's section to a heading of `AGENTS.md`:

<!-- transcript: run -->
```console
$ cd library && python3 tools/harness.py check
harness check: 18 guide(s) and 14 other control(s) agree with the code
```

### 3. A rule fires

The repository is wired to this page's gonk as in [Solo](solo.md#with-the-starter). Two rows of
`init` matter here: `harness`, which says the coverage questions are asked rather than published
(a script runs under a ceiling of ledger reads, and these read the harness graph), and `hook`,
which says the Claude Code claim hook is off. Step 5 comes back to that one.

<!-- transcript: run -->
```console
$ cd library && git init -q && python3 tools/gonk.py init --ledger default --socket ../.ikigai/gonk.sock --holder ada
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
  ok       holder    `ada` (.gonk/local.toml, this working tree only)
  ok       identity  the socket's owner: every request over it is `owner`, so the log cannot tell this tree from any other process of yours. A machine that must be named uses QUIC
  ok       gonk      socket …/.ikigai/gonk.sock answers
  note     items     `default` holds nothing yet, or this identity may not read it. A ledger has no create: it exists once something is filed in it (`python3 tools/gonk.py file "<title>"`)
  ok       scripts   gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  ok       scripts   published 4: urn:script:default-open-items-for-artifact, urn:script:default-open-work-by-change, urn:script:default-requirements-with-open-work, urn:script:default-next-for-artifact
  absent   nl        gonk does not offer `urn:nl:sparql` yet, so a query is written by hand rather than drafted from a sentence (ikigai-nl); nothing to do until it does

agents over MCP (configure the agent with this command):
  ikigai mcp --override urn:iki:ledger:=…/.ikigai/gonk.sock

ready.
```

(The `absent` row is this page's gonk mounting no model, as in [Periodic unled
audits](audits.md#with-the-starter).) Now an item, a claim, and a hand-back that forgets its
evidence:

<!-- transcript: run -->
```console
$ cd library && python3 tools/gonk.py file 'Renew a loan before it is due'
#1 urn:iki:ledger:default:item:…
$ cd library && python3 tools/gonk.py claim 1
#1 claimed by ada until …
$ cd library && python3 tools/gonk.py report 1 --note 'renewal works'
gonk.py: --evidence is a URL (the pull request, or the reproduction that shows it is not a defect): a report without evidence is a claim nobody can check
```

That refusal is the sensor working, and its message says what to do instead, which is the part
an agent reads. It also wrote one line to this working tree's firing log:

<!-- transcript: run -->
```console
$ cat library/.gonk/state/harness/firings.log
# ikigai-log v1
@prefix harness: <https://ikigai-rs.dev/ns/harness#> .
@prefix log:     <https://ikigai-rs.dev/ns/log#> .
@name     urn:log:harness:tree-…
@instance urn:iki:harness:tree:…
@level    log:info
@started  …
@prev     genesis
# This working tree's harness firings: one line each time a sensor in this
# repository's tools refused or warned. UNSEALED: no key, and no last line is
# known. Written by tools/harness.py; `python3 tools/gonk.py harness` loads it.

… harness:Refusal urn:iki:harness:starter:control:evidence-required seq=1 cmd=sha256:…
```

The file is one [ikigai-log](https://github.com/ikigai-rs/ikigai-log) segment, so ikigai-log can
read it as well as the starter can. The line is a time, the kind of firing, the control's IRI, a
sequence number, and the first twelve hex digits of a digest of the command, enough to tie
repeated firings of one command together. Never the command's text, a file's contents, a path or
a name. Recording is best effort and the starter's tests prove it: an unwritable directory or a
corrupt segment leaves the refusal and its message exactly as they were. Read back, by control:

<!-- transcript: run -->
```console
$ cd library && python3 tools/harness.py firings
    1  refusal  evidence-required                    last …
1 firing(s) in urn:log:harness:tree-…
```

### 4. Ask the four coverage questions

`gonk.py harness` loads the graph and this tree's firings into gonk's named graph
`urn:iki:harness:graph:default` with one SPARQL update, then asks the four questions in
`.gonk/queries/harness-*.rq` and prints the answers. Writing that graph needs a store grant no
ledger grant includes, and gonk's owner-only socket holds it, so it is the dispatcher's command,
from gonk's machine; here this page's gonk is the owner's. Before it loads, it decides from this
working tree's own files which sensors are turned on (the same checks `init` reports), names each
one that is not, and loads that beside the graph:

<!-- transcript: run -->
```console
$ cd library && python3 tools/gonk.py harness
harness: 18 rule(s), 14 sensor(s) and 0 other control(s) in .gonk/harness/harness.ttl; 1 firing(s) in this tree
  not wired: ctl:edit-hook-not-wired
  not wired: ctl:edit-hook-unreadable-input
  not wired: ctl:edit-without-live-claim
loaded:  updated <urn:iki:harness:graph:default>: +341 -0 quads

harness-firings-by-control: How often has each control fired, refusing or warning, and when first and last?
  control                label                           outcome  firings  first                     last
  ctl:evidence-required  report and close need evidence  Refusal  1        …  …

harness-guides-without-sensor: Which rules does no computational control check?
  guide                           label                                                  section
  guide:hand-back-when-you-stop   hand it back when you stop                             1. Claim the item before you write
  guide:reproduce-before-fix      reproduce before you fix                               3. Reproduce before you fix
  guide:last-command-exit-status  a pipeline's status is its last command's              4. Every change through a pull request with green CI
  guide:stage-files-by-name       stage files by name                                    4. Every change through a pull request with green CI
  guide:no-bare-numbers           never a bare #N                                        6. Never a bare `#N`
  guide:read-accessibility-first  read ACCESSIBILITY.md before changing what people see  7. Read ACCESSIBILITY.md before changing what people see

harness-sensors-never-fired: Which sensors of this repository's tools have never fired, and has a test shown each one can?
  control                         label                                             wired  proof
  ctl:claim-held-by-another       claim: held by another holder                     -      fired by a test
  ctl:claim-lease-expired         claim: an expired lease is not free               -      fired by a test
  ctl:one-claim-per-tree          a working tree holds one claim                    -      fired by a test
  ctl:reject-needs-reason         a rejection says why                              -      fired by a test
  ctl:takeover-refused            takeover: only an expired lease, from its holder  -      fired by a test
  ctl:edit-hook-not-wired         claim hook: no claim can be checked               no     not wired
  ctl:edit-hook-unreadable-input  claim hook: input it cannot read                  no     not wired
  ctl:edit-without-live-claim     claim hook: an edit needs a live claim            no     not wired

harness-sensors-without-guide: Which sensors state no rule, so an agent meets them only by tripping them?
  none. Every sensor cites the rule of AGENTS.md it mechanizes.
```

Four answers:

- **Firings by control.** What has fired, how often, first and last (times at a fixed width, to
  the millisecond, in UTC). Over a real month this is the
  list of rules that are not landing: a sensor that fires often is doing its job, and saying that
  agents keep making the move its rule warns against.
- **Rules with no sensor.** Six of the eighteen rules are text an agent has to apply, with
  nothing that notices when it does not. Each row is a candidate for a sensor, not a defect, and
  some can only ever be applied by a reader: the starter's `profiles/README.md` records why several
  stay prose on purpose (`reproduce-before-fix`, for one, is a judgment about how an agent worked,
  with nothing in a tool call to observe).
- **Sensors never fired.** Eight of the fourteen. Five are on wherever their code is (`-` under
  `wired`), each with a test that shows it can fire. The other three are the Claude Code claim
  hook's, and they read `no` and `not wired`: the hook is off in this tree, so their silence says
  nothing about the rule. `evidence-required` is not on the list, because step 3 fired it.
- **Sensors with no rule.** None: every sensor cites the rule it mechanizes. That is the answer
  this question exists to reach. `reject-needs-reason`, which refuses a rejection without a
  reason, used to be its one row, because no section of `AGENTS.md` said that a rejection needs
  one, so an agent first heard of it from the refusal. The fix was a sentence in the rules, not
  another sensor: section 5 now says it, and the sensor cites that rule.

The load is idempotent: a firing's IRI is its log and its sequence number, so loading again adds
nothing, and every working tree's firings land in the same graph beside each other's:

<!-- transcript: run -->
```console
$ cd library && python3 tools/gonk.py harness | grep '^loaded'
loaded:  updated <urn:iki:harness:graph:default>: +0 -0 quads
```

### 5. Steer: a rule with no sensor, and a sensor that never fired

The answers are where the steering loop starts. Take one rule from the second answer:

<!-- transcript: run -->
```console
$ awk -v RS= '/^guide:stage-files-by-name /' library/.gonk/harness/harness.ttl
guide:stage-files-by-name a harness:Guide ;
    rdfs:label "stage files by name" ;
    rdfs:comment "Stage the files you changed by name: `git add -A` picks up files that are not yours." ;
    harness:statedIn "AGENTS.md" ; harness:section "4. Every change through a pull request with green CI" ;
    harness:direction harness:feedforward ; harness:execution harness:inferential ;
    harness:stage harness:preCommit ; harness:category harness:authoritySafety .
```

A sensor for it is easy to describe and cheap to run: a hook on the agent's shell commands (the
same kind of hook as the claim hook) that refuses `git add -A`, `git add .`, `git add -u` and
`git commit -a` before the shell runs them, and says why: name the files. Computational, outcome
`refuse`, and acting at the tool call, one stage earlier than the commit the rule is about. Adding
it is one pull request with three parts: the hook, which records each refusal under a new control
id; that control in the graph, with `statedBy guide:stage-files-by-name`; and a test that fires it,
named in `firedBy`. `harness.py check` fails if the code names an id the graph does not describe,
or the graph describes a sensor nothing shows can fire, so a part left out is caught, and the next
coverage answer has one row fewer. Whether the sensor is worth its friction is the steering
question, and the starter has answered it for now: not yet, because it would need a hook on every
`git add`, and the repository's hooks are optional (its `profiles/README.md` says so, and to revisit
the rule when a cheap, exact signal for it appears). Before it exists the evidence comes from outside the graph (how often review or the
dispatcher catches a stray file in a commit); once it exists, the first answer counts it.

Now take one sensor from the third answer, `edit-without-live-claim`: the claim hook refusing an
edit while the working tree holds no live claim. It has never fired. Is that because nobody edits
without a claim? The answer says not: it is **not wired**, as `init` said the hook is **off**,
because it is optional and nothing merged its settings. A sensor that is not installed never
fires, and without that column its silence would look exactly like health. The hook is a program that reads a Claude Code tool call as JSON, so it can be run
by hand the way Claude Code would run it, after the claim is released:

<!-- transcript: run -->
```console
$ cd library && python3 tools/gonk.py release 1 --note 'stopping here'
#1 released
$ printf '{"tool_name": "Edit", "tool_input": {"file_path": "%s/library/README.md"}}' "$PWD" | python3 library/hooks/claude/require-claim.py
require-claim: refused an edit to README.md: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
$ cd library && python3 tools/harness.py firings
    1  refusal  evidence-required                    last …
    1  refusal  edit-without-live-claim              last …
2 firing(s) in urn:log:harness:tree-…
```

So the sensor works, and was silent because nothing ran it. The `firedBy` proof answers a
narrower question than it seems to: it shows the code *can* fire, not that it is wired into
anyone's work, nor that its trigger still matches what it watches. The claim refusals in the same
answer show the third gap: `tools/gonk.py` recognizes the ledger's refusal by its wording, so a
change of wording in the ledger would silence the firing while the refusal goes on (the graph's
own comment on `claim-held-by-another` says so, and an integration test holds the wording).

That is as far as the starter answers the question Böckeler leaves open. "Never fired" splits
into three: **cannot fire** (no test shows it can; `harness.py check` refuses that state), **is not
wired** (it can fire, and nothing runs it), and **is quiet** (it runs and nothing trips it). Only
the last is evidence of good work. The graph now tells the second from the third where it can: a
profile's fragment says what turns its sensor on (`harness:wiredBy`: the Claude Code hook, the
review hook, a worker's configuration), and `gonk.py harness` loads `harness:wired` for the working
tree that ran it. Two gaps remain. Wiring is known for that one tree, not for every tree whose
firings land in the same graph. And a sensor that is on wherever its code is (`-`) can still be
silent because its trigger no longer matches what it watches, as the claim refusals' wording
could; for that only a test against the real thing helps. Real gaps, stated rather than solved.

## What holds it

The harness has a harness:

- **`tools/harness.py check`**, the guide-and-code agreement. A control the code names and the
  graph does not describe, a control the graph describes and no code names, a term the vocabulary
  does not define, a guide whose section is not a heading of `AGENTS.md`, or a `wiredBy` that names
  no part the process carries, each fails it by name.
- **One test per sensor**, named in the graph's `firedBy`, that drives the sensor and sees the
  firing recorded; the starter's own tests hold `firedBy` to the tests that exist.
- **Stable ids.** A control's id never changes, so its firings stay joined to it across renames,
  and a firing outlives the control it names (the first answer shows it with an empty label).
- **Recording that cannot change a decision.** A firing is written best effort, never to standard
  output, and the tests prove a failed write leaves the refusal unchanged, so turning the log on
  cannot break the tools it watches.
- **The questions as files**, `.gonk/queries/harness-*.rq`, each with its question and what an
  empty answer means in its header, so a reader can run them and a reviewer can change them.

## What it costs, and when to choose it

Very little to keep: the graph is a few hundred lines of Turtle the starter generated, the log is
a line per firing, and the questions are four queries. What costs is the discipline it asks for
when the harness changes: a new sensor is a control in the graph and a test that fires it as well
as the code, in the same pull request. That is the point. A control nobody has described is a
control nobody can ask about.

Choose it whenever agents do real work in a repository, because the alternative is the harness
everyone already has: rules in prose, checks in scripts, and no way to ask which rules are
checked, which checks have ever done anything, or which refusals the rules never warned of.
Choose the starter's template to begin with it rather than retrofit it.

The authors run a larger harness the same way, in the authors' private tooling: a graph of the
controls, a firing log, and coverage questions asked of both.

## The vocabulary

The terms are in the namespace `https://ikigai-rs.dev/ns/harness#` (`harness:`), whose definition
is the starter's `.gonk/harness/vocabulary.ttl`: every term, including those a generated
repository does not use. It is not served at that address yet, so the starter's file is the
definition.
