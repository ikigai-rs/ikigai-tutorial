# Start a project

Every chapter so far taught one piece of gonk. This part puts them together over a real project: a
small library-lending system, with specs, two artifacts, tests and two changes in flight, generated
by the [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter). You will start it, change
its spec, turn the change into ledger items, claim one, reproduce and fix it, close it with evidence,
run a second agent beside the first, ask the project questions in SPARQL, and let a model draft one
of those questions.

[A spec in the repository](../spec/openspec.md) taught the `openspec/` tree and the items it becomes.
[Solo on one machine](../ways/solo.md) and [Spec first](../ways/spec-first.md) showed the starter's
packaging in passing. Here the starter is the subject, and `tools/gonk.py` does what those pages
wrote out by hand.

## What you need

`ikigai-gonk` and `ikigai`, as [Which gonk](../introduction.md#which-gonk) installs them, plus `git`,
`jq`, and Python 3.11 or later. The sample's tests and `tools/gonk.py` use only Python's standard
library.

⚠ Each `$` line below runs from this chapter's directory, so a command that works inside the project
is written `(cd library && …)`. The parentheses keep your shell where it is.

## Generate the sample

In a new empty directory, as in [every chapter](../introduction.md#a-gonk-per-chapter). Clone the
starter at the revision this book checks and generate from it. `--process solo,spec` says how you
work: you are your own dispatcher, and a change in behavior starts as a spec change.

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ (cd starter && python3 tools/new.py ../library --process solo,spec)
generated the library-lending sample (python, process solo, spec) in …/library: 94 files
next, in that directory:
  git init -b main, then commit everything it wrote (the one commit where every file is yours; -b main because the worker loop branches from origin/main, and git's own default is still master where init.defaultBranch is unset)
  python3 scripts/test.py                      every test, as CI runs them
  python3 tools/gonk.py init --ledger <name> --socket ~/.ikigai/gonk.sock --holder <you>
  python3 tools/gonk.py seed                   file the open changes' tasks as items
  python3 tools/gonk.py next                   what to do first, and why
AGENTS.md holds this process's rules; README.md walks through the sample.
```

Do what it says: make it a repository and run its tests. `--target rust`, `deno` or `go` generates the
same sample in another language, against the same spec.

<!-- transcript: run -->
```console
$ git -C library init -q -b main
$ git -C library add -A
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Generate the library sample'
$ (cd library && python3 scripts/test.py 2>&1 | tail -1)
all 3 suites passed
```

## What it gave you

| path | holds |
| --- | --- |
| `AGENTS.md` | the rules, for every agent and every person; `CLAUDE.md` imports it |
| `openspec/specs/` | what the library does now, one capability per directory |
| `openspec/changes/` | what is proposed: `cap-late-fees` (small) and `add-holds` (both artifacts) |
| `openspec/vectors.json` | every scenario of the specs, as steps any language's tests run |
| `artifacts/` | one directory per deployable thing: `service` and `cli` |
| `.gonk/` | which ledger, the process, the saved queries and the workflow scripts |
| `tools/gonk.py` | this repository's one way to reach gonk |

<!-- transcript: run -->
```console
$ LC_ALL=C ls library/openspec/specs library/openspec/changes library/artifacts
library/artifacts:
README.md
cli
service

library/openspec/changes:
README.md
add-holds
cap-late-fees

library/openspec/specs:
README.md
catalog
loans
members
overdue
renewals
```

Nothing connects it to a gonk yet, and `status` says exactly what is missing:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py status)
gonk.py: no ledger is named in .gonk/gonk.toml: `python3 tools/gonk.py init --ledger <name>`
this working tree does not say how to reach gonk (.gonk/local.toml): `python3 tools/gonk.py init --socket <path>` or `init --quic quic://host:port --cert-dir <bundle>`
process: solo, spec
```

## A gonk on this machine

If the gonk from [Harness engineering](../ways/harness.md) is still running, stop it first, since
this one takes the same port. The page starts a scratch gonk; on your own machine yours is probably
already running, at `~/.ikigai/gonk.sock`.

Run the next command in a second terminal, from the same chapter directory. Leave it running and
return to the first terminal for the commands that follow.

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

`init` writes what its flags say, then checks everything it can reach. The **ledger** name goes in a
committed file, so every clone works in the same ledger. The **socket** and the **holder**, the name
this working tree claims under, go in a git-ignored one, because they belong to this machine and
this tree.

Name the ledger after the project, as this one does. One gonk can hold many projects, and each
repository's saved queries are published under its ledger's name.

⚠ Do not use `default` as a project's ledger at the starter revision this book pins. gonk answers an
append in `default` with `#3 …` and in any other ledger with `library#3 …`, and `seed` reads only the
second form, so it files every task, calls each one failed, and links none of them.

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py init --ledger library --socket ../.ikigai/gonk.sock --holder ada)
gonk init: …/library
  ok       layout    openspec at openspec/; artifacts at artifacts/; queries at .gonk/queries/; scripts at .gonk/scripts/
  note     harness   4 harness question(s) in .gonk/queries/ are asked by `python3 tools/gonk.py harness` through gonk's store, not published: gonk runs a script under a ceiling of ledger reads, and they read the harness graph
  note     openspec  8 open task(s) in 2 change(s) in flight: `python3 tools/gonk.py seed` files them as ledger items (again is a no-op)
  ok       process   solo, spec (.gonk/process.toml)
  note     solo      one person and their agent: you are your own dispatcher, and close after checking your own evidence
  note     role      you: both the dispatcher and whoever does the work
  note     spec      spec first: every change in behavior starts as an openspec change, seed files its tasks, next orders them
  note     hook      the Claude Code claim hook is optional and off: merge hooks/claude/settings.sample.json into .claude/settings.json to turn it on
  ok       ikigai    ikigai … at …/ikigai
  ok       ledger    `library` (.gonk/gonk.toml)
  ok       holder    `ada` (.gonk/local.toml, this working tree only)
  ok       identity  the socket's owner: every request over it is `owner`, so the log cannot tell this tree from any other process of yours. A machine that must be named uses QUIC
  ok       gonk      socket …/.ikigai/gonk.sock answers
  note     items     `library` holds nothing yet, or this identity may not read it. A ledger has no create: it exists once something is filed in it (`python3 tools/gonk.py file "<title>"`)
  ok       scripts   gonk offers `urn:script:catalog`: saved queries and workflow scripts (ikigai-script)
  ok       scripts   published 4: urn:script:library-open-items-for-artifact, urn:script:library-open-work-by-change, urn:script:library-requirements-with-open-work, urn:script:library-next-for-artifact
  absent   nl        gonk does not offer `urn:nl:sparql` yet, so a query is written by hand rather than drafted from a sentence (ikigai-nl); nothing to do until it does

agents over MCP (configure the agent with this command):
  ikigai mcp --grant agent --override urn:iki:ledger:=…/.ikigai/gonk.sock

ready.
$ git -C library status --short
 M .gonk/gonk.toml
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -am 'Work in the library ledger'
```

Read the rows as a checklist. `ok` is checked, `note` is something to know, and `missing` would stop
you, with the command that fixes it. `absent` means gonk mounts no model, so it offers no drafting;
[A query from a sentence](sentence.md), later in this part, changes that. Only `.gonk/gonk.toml`
changed in the repository. That is the one line every clone shares.

## A gonk on another machine

A remote gonk is reached over **QUIC with a client certificate per machine**, so the same page can
show it by enrolling a second working tree against this gonk's QUIC door. On the machine that will do
the work, make an identity. Its private key never leaves that machine.

<!-- transcript: run -->
```console
$ ikigai cert generate --dir laptop-cert
wrote server.{crt,key} and client.{crt,key} to laptop-cert
to attach a client on another machine, copy client.crt, client.key, and server.crt there.
```

Then, in a clone there, `init` with `--quic` and `--cert-dir`. This gonk does not know the
certificate yet, so `init` reports what is missing and prints the two lines the operator chooses
between: `--worker` for an unattended machine, which claims and comments and cannot close, and
`=write` for a person's, which also files, closes and relabels. `git worktree` stands in for that
clone:

<!-- transcript: run -->
```console
$ git -C library worktree add -q ../laptop -b laptop
$ (cd laptop && python3 tools/gonk.py init --holder laptop --quic quic://127.0.0.1:1070 --cert-dir ../laptop-cert | grep -E ' identity  |missing  |client add laptop|thing\(s\)')
  ok       identity  urn:iki:host:gonk:client:… (from …/laptop-cert/client.crt)
  missing  gonk      quic quic://127.0.0.1:1070 (certificates in …/laptop-cert) did not answer: …
    ikigai-gonk client add laptop --cert client.crt --worker library
    ikigai-gonk client add laptop --cert client.crt --ledger library=write
1 thing(s) missing.
```

The operator runs one of those on gonk's machine, restarts gonk, because it reads the certificates it
trusts at startup, and sends back the `server.crt` from the bundle it wrote. That copy replaces the
one `cert generate` made, which is not gonk's. Here gonk's machine is this one:

<!-- transcript: manual — it restarts gonk, and one page holds one gonk; "The three doors" and "A QUIC client" run this exchange against a real door -->
```console
$ ikigai-gonk client add laptop --cert laptop-cert/client.crt --ledger library=write --port 1070 --config-home "$PWD/.config/ikigai"
$ cp .config/ikigai/gonk/quic/clients/laptop/server.crt laptop-cert/server.crt
```

After the restart, `init` in that tree ends `ready.`, and every command in the rest of this part works
the same. The differences are who gonk says you are, which matters in
[Claiming](claim.md#a-person-or-a-machine), and what the grant lets you do:

| | the socket | QUIC |
| --- | --- | --- |
| reaches | gonk on this machine | gonk on any machine |
| gonk knows you as | `owner`, with every other process of yours | this machine's certificate |
| may do | everything | what the operator granted: `--worker` claims and comments, `=write` also closes |
| choose it for | yourself, on your own machine | a laptop, a build box, an unattended worker |

[The three doors](../machines/doors.md) and [A QUIC client](../machines/quic-client.md) teach that
door properly.

## What you have now

A repository whose tests pass, a ledger named in it, a working tree that knows how to reach gonk and
what to call itself, and four saved queries already published. The ledger is still empty. The
[next chapter](change.md) fills it from the spec.
