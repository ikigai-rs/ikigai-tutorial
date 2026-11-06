# Two agents in one repository

[Work, evidence, review, close](work.md) took one task through the worker and dispatcher roles.
This chapter adds a second worker to the same repository, shows what claiming prevents, and says
what it leaves to you.

## Catch up

Start in a new empty directory, as in [Start a project](start.md); this chapter regenerates the
sample and seeds a fresh ledger. If the gonk (and the `model-stub`) from
[the previous chapter](work.md) is still running, stop it first, since this one takes the same port.
Keep gonk running in one terminal and run the remaining commands in another, from the same directory.

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ (cd starter && python3 tools/new.py ../library --process solo,spec > /dev/null)
$ git -C library init -q -b main
$ (cd library && python3 tools/gonk.py init --ledger library --socket ../.ikigai/gonk.sock --holder claude-1 | tail -1)
ready.
$ git -C library add -A
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Generate the library sample'
$ (cd library && python3 tools/gonk.py seed | tail -1)
seed: 8 filed, 0 already filed, 0 checked off (not filed), 6 order link(s) added
```

## One working tree each

Two agents in one directory overwrite each other's files, whatever the ledger says. So each gets its
own working tree and its own holder name. The holder name is stored in the git-ignored
`.gonk/local.toml`, so each working tree sets its own. The ledger name is stored in the committed
`.gonk/gonk.toml`, so both working trees use the same ledger.

<!-- transcript: run -->
```console
$ git -C library worktree add -q ../second -b second
$ (cd second && python3 tools/gonk.py init --socket ../.ikigai/gonk.sock --holder claude-2 | tail -1)
ready.
$ git -C library worktree list | sed -E 's#^.*/([^/ ]+) +[0-9a-f]+ #\1 #'
library [main]
second [second]
```

```text
  library/          holder claude-1    claim 7    branch main
  second/           holder claude-2    claim 1    branch second
       │                                   │
       └──────── one gonk, one ledger ─────┘
```

Both are the socket's owner here, so what tells them apart is the holder name, and that name is a
convention between processes of yours. Agents that must be told apart by gonk itself use QUIC, one
certificate each ([Remote workers](../ways/workers.md)).

## What claiming prevents

**Two agents on one task.** Both ask `next`, and both are told the same thing. The first claim wins.
The second agent is refused and asks again, and now `next` has moved on:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py next | grep -E '^ 1\.')
 1. library#7  open    p-  cap-late-fees 1.1: Stop a late return's fee at 1000 cents  [artifact:service change:cap-late-fees]
$ (cd second && python3 tools/gonk.py next | grep -E '^ 1\.')
 1. library#7  open    p-  cap-late-fees 1.1: Stop a late return's fee at 1000 cents  [artifact:service change:cap-late-fees]
$ (cd library && python3 tools/gonk.py claim 7)
library#7 claimed by claude-1 until …
$ (cd second && python3 tools/gonk.py claim 7)
error: conflict: library#7 is already claimed by claude-1. Release it first (`delete urn:iki:ledger:library:claim item=7`), or take it up with them — a claim is a fence, and stealing one silently is how two workers end up in the same tree
$ (cd second && python3 tools/gonk.py next | grep -E '^ 1\.|not library#7')
 1. library#1  open    p-  add-holds 1.1: Accept a hold on a title whose copies are all out  [artifact:cli artifact:service change:add-holds]
  not library#7: claimed by claude-1
$ (cd second && python3 tools/gonk.py claim 1)
library#1 claimed by claude-2 until …
```

**One agent juggling.** A tree holds one claim, so its work stays on one question:

<!-- transcript: run -->
```console
$ (cd second && python3 tools/gonk.py claim 3)
gonk.py: this working tree already holds item 1 on `library`: one item per working tree. Release or close it first
```

**Blocked tasks can still be claimed.** `next` will not offer a blocked item, but that is
advice rather than a refusal. This tree hands its item back and claims the blocked `library#8`, and the
claim goes through, so an agent has to ask `next` and not pick for itself:

<!-- transcript: run -->
```console
$ (cd second && python3 tools/gonk.py release 1 --note 'Back to the pool')
library#1 released
$ (cd second && python3 tools/gonk.py claim 8)
library#8 claimed by claude-2 until …
$ (cd second && python3 tools/gonk.py release 8 --note 'Blocked by library#7: claimed by mistake')
library#8 released
```

**Dropping someone else's claim by accident.** `report` and `release` from a tree are
compare-and-set on that tree's holder, so a slow agent cannot hand back a claim that has changed hands
([Claiming](claim.md#take-it-over)).

**A race.** Two agents can also claim at the same moment. Here they do, as background processes, and
there is one winner and one refusal. Which one wins can differ from run to run:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py release 7 --note 'To race for it') > /dev/null
$ (cd library && python3 tools/gonk.py claim 1 > ../first.txt 2>&1) & (cd second && python3 tools/gonk.py claim 1 > ../second.txt 2>&1) & wait
$ cat first.txt second.txt | grep -c '^library#1 claimed by'
1
$ cat first.txt second.txt | grep -c '^error: conflict'
1
```

## The hook: making it a refusal for one agent

The rules are in `AGENTS.md`, and every agent keeps them by reading them. For Claude Code the starter
can also turn the first rule into a refusal, with an optional hook that runs before each file edit.
Merge `hooks/claude/settings.sample.json` into `.claude/settings.json` to turn it on. Here it is fed
what Claude Code would send it, for a tree with no claim and then the same tree with one:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py release 1 --note 'Done racing') > /dev/null 2>&1; (cd second && python3 tools/gonk.py release 1 --note 'Done racing') > /dev/null 2>&1; true
$ printf '{"tool_name":"Edit","tool_input":{"file_path":"%s/second/README.md"}}' "$PWD" | python3 second/hooks/claude/require-claim.py; echo "exit $?"
require-claim: refused an edit to README.md: this working tree holds no claim. Claim the item first: `python3 tools/gonk.py claim <number>` (or `just claim <number>`)
exit 2
$ (cd second && python3 tools/gonk.py claim 7)
library#7 claimed by claude-2 until …
$ printf '{"tool_name":"Edit","tool_input":{"file_path":"%s/second/README.md"}}' "$PWD" | python3 second/hooks/claude/require-claim.py; echo "exit $?"
exit 0
```

It cannot see a file written by a shell command, it means nothing to other agents, and it refuses when
gonk is down, because it cannot check a claim it cannot read. It is a second layer, never the only one.

## What claiming does not prevent

A claim fences an *item*. The ledger does not know which files an item will touch, so:

| what can still go wrong | what catches it |
| --- | --- |
| two items change the same file | `git`, at the merge; `blocks`, if you knew beforehand |
| a claim on a blocked item | only the rule: ask `next`, and take what it offers |
| two agents share one working tree | only the rule, which is why each tree has one holder |
| an agent works without claiming | only the rule, or the hook, for an agent that has one |
| an agent says it finished and did not | the dispatcher, checking the evidence |
| an agent keeps writing after losing its claim | `status`, a renewal's exit 3, and the merge |

So the claim is one control among several, and the rest are the ones [Harness
engineering](../ways/harness.md) lists. With two agents you also become their dispatcher, and the part
that grows is checking evidence. [Remote workers](../ways/workers.md) and [Hub and
spoke](../ways/hub-and-spoke.md) show how this works when two agents become many.

<a id="what-you-built"></a>

## What you have now

Two agents working the same ledger without colliding on a task, and a clear list of what they can
still collide on. The [next chapter](query.md) asks the project questions nobody built a command for.
