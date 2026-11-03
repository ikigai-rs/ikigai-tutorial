# A spec change, and the items it becomes

The sample arrives with two changes in flight. This chapter adds a third, then lets `seed` turn all
of them into ledger items and `next` say what to do first. [Spec first](../ways/spec-first.md) wrote
these requests out by hand; here the starter's `seed` sends them.

## Catch up

Each chapter of this part stands alone, so it starts from a new empty directory: the same steps as
[Start a project](start.md), run quietly. If the gonk from [that chapter](start.md) is still running,
stop it first, since this one takes the same port.

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
$ (cd library && python3 tools/gonk.py init --ledger library --socket ../.ikigai/gonk.sock --holder ada | tail -1)
ready.
$ git -C library add -A
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Generate the library sample'
```

## Add a change

A change is a folder under `openspec/changes/`: why, what the requirements become, and how, as tasks
([OpenSpec in one chapter](../spec/openspec.md)). The library should waive a member's first late fee,
because a first late return is usually a mistake. Start with why, and name the artifacts it touches on
the line `seed` reads:

<!-- transcript: file library/openspec/changes/waive-first-fee/proposal.md -->
```markdown
# Waive a member's first late fee

## Why

A first late return is usually a mistake, and a fee for it costs more goodwill than it earns.

## What Changes

- A member's first late return owes nothing, however late it is.

## Impact

- Artifacts: `service`
- Affected specs: `overdue`
```

The delta spec says what the requirements become. Its scenario will become a test:

<!-- transcript: file library/openspec/changes/waive-first-fee/specs/overdue/spec.md -->
```markdown
## ADDED Requirements

### Requirement: A member's first late fee is waived

A member's first late return SHALL owe 0 cents, however late it is.

#### Scenario: A first late return owes nothing

- **WHEN** a member who has never returned a copy late returns one three days late
- **THEN** the return is three days late and owes 0 cents
```

The tasks say how, in order. Each names the requirement it implements in a trailing parenthesis:

<!-- transcript: file library/openspec/changes/waive-first-fee/tasks.md -->
```markdown
# Tasks

## 1. Service

- [ ] 1.1 Remember which members have returned a copy late (A member's first late fee is waived)
- [ ] 1.2 Owe nothing on a member's first late return (A member's first late fee is waived)
```

Nothing is built yet, and the repository's own check already holds the change to its conventions: a
task must cite a requirement its change adds or modifies. The new scenario has no vector yet, which
the check allows for a change in flight.

<!-- transcript: run -->
```console
$ (cd library && python3 scripts/test.py 2>&1 | tail -1)
all 3 suites passed
```

A pull request that adds this folder is the agreement to build it, and it changes no code, so it is
cheap to argue about. Here there is nobody to argue with:

<!-- transcript: run -->
```console
$ git -C library add -A
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Propose waive-first-fee'
```

## Turn the changes into items

`seed --dry-run` reads the tree and asks gonk nothing:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py seed --dry-run | tail -5)
would link  add-holds 2.1 blocks 2.2
would link  cap-late-fees 1.1 blocks 1.2
would link  waive-first-fee 1.1 blocks 1.2

seed --dry-run: 10 open task(s) would be filed, in 7 order link(s); 0 checked off
```

Run for real, each open task becomes one item, and each task blocks the next in its change:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py seed)
filed       library#1 add-holds 1.1: Accept a hold on a title whose copies are all out
filed       library#2 add-holds 1.2: Keep a returned copy for the first holder
filed       library#3 add-holds 1.3: Refuse a renewal while a hold waits
filed       library#4 add-holds 1.4: Add a vector to openspec/vectors.json for every new scenario
filed       library#5 add-holds 2.1: Add `library.py hold MEMBER ISBN`
filed       library#6 add-holds 2.2: Show who a kept copy waits for in `library.py copy`
filed       library#7 cap-late-fees 1.1: Stop a late return's fee at 1000 cents
filed       library#8 cap-late-fees 1.2: Add a vector for the capped return to openspec/vectors.json
filed       library#9 waive-first-fee 1.1: Remember which members have returned a copy late
filed       library#10 waive-first-fee 1.2: Owe nothing on a member's first late return
ordered     library#1 blocks library#2 (add-holds 1.1 before 1.2)
ordered     library#2 blocks library#3 (add-holds 1.2 before 1.3)
ordered     library#3 blocks library#4 (add-holds 1.3 before 1.4)
ordered     library#4 blocks library#5 (add-holds 1.4 before 2.1)
ordered     library#5 blocks library#6 (add-holds 2.1 before 2.2)
ordered     library#7 blocks library#8 (cap-late-fees 1.1 before 1.2)
ordered     library#9 blocks library#10 (waive-first-fee 1.1 before 1.2)

seed: 10 filed, 0 already filed, 0 checked off (not filed), 7 order link(s) added
```

Each item is keyed by its task's IRI, and is *about* both the task and the requirement it cites:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:library:item:key:urn:openspec:change:waive-first-fee:task:1.1'
library#9  open    p-  waive-first-fee 1.1: Remember which members have returned a copy late  [artifact:service change:waive-first-fee]
  iri:      urn:iki:ledger:library:item:…
  key:      urn:openspec:change:waive-first-fee:task:1.1
  filed:    …
  updated:  …
  about:    urn:openspec:change:waive-first-fee:task:1.1
  about:    urn:openspec:spec:overdue:requirement:a-member-s-first-late-fee-is-waived
  blocks:   urn:iki:ledger:library:item:…

  Task 1.1 of the change `waive-first-fee` (Waive a member's first late fee), openspec/changes/waive-first-fee/tasks.md.
  It implements the requirement "A member's first late fee is waived" (urn:openspec:spec:overdue:requirement:a-member-s-first-late-fee-is-waived).
  Artifacts: `service`
[uncacheable]
```

The labels come from the proposal's `Artifacts:` line and the change's name, and later chapters
narrow by them. The `about` lets the ledger answer by requirement:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:library:count about=urn:openspec:spec:overdue:requirement:a-late-return-owes-a-fee'
2
[uncacheable]
```

## Edit a change

Specs change as work teaches you things, and `seed` is safe to run again. Task 1.2 turns out to need a
vector of its own, so add one more task:

<!-- transcript: run -->
```console
$ printf '%s\n' "- [ ] 1.3 Add a vector for the waived return to openspec/vectors.json (A member's first late fee is waived)" >> library/openspec/changes/waive-first-fee/tasks.md
$ (cd library && python3 tools/gonk.py seed | grep -E 'library#11|^seed')
filed       library#11 waive-first-fee 1.3: Add a vector for the waived return to openspec/vectors.json
ordered     library#10 blocks library#11 (waive-first-fee 1.2 before 1.3)
seed: 1 filed, 10 already filed, 0 checked off (not filed), 1 order link(s) added
$ (cd library && python3 tools/gonk.py seed | tail -1)
seed: 0 filed, 11 already filed, 0 checked off (not filed), 0 order link(s) added
```

One new item, and a second run writes nothing. `seed` only adds. Reword a task and its key stays, so
the old title stays too; remove a task and its item stays open until a person closes it with
`reason=wontfix` and why.

## Ask next

`next` is the question the rest of this part keeps asking. It answers with the ready items, ranked,
and says why everything else is not ready:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py next | grep -E '^ [0-9]\.|^policy|^ready')
 1. library#9  open    p-  waive-first-fee 1.1: Remember which members have returned a copy late  [artifact:service change:waive-first-fee]
 2. library#7  open    p-  cap-late-fees 1.1: Stop a late return's fee at 1000 cents  [artifact:service change:cap-late-fees]
 3. library#1  open    p-  add-holds 1.1: Accept a hold on a title whose copies are all out  [artifact:cli artifact:service change:add-holds]
policy: priority-recency (weighs priority, recency, number)
ready: 3   excluded: 8
$ (cd library && python3 tools/gonk.py next | grep 'not library#8:')
  not library#8: blocked by library#7
```

Eleven items, and three ready: the head of each change. The rest wait behind `blocks`, so an agent
asking `next` cannot start a change in the middle. Nothing has a priority yet, so recency ranks them.
`gonk.py file --priority` sets one on an item you file by hand, and an item with a priority outranks
every item without one.

<a id="what-you-built"></a>

## What you have now

A spec change agreed in the repository and a ledger that knows about it: the tree says *what*, the
ledger says *who, when and what next*, and the task's IRI joins them. The [next chapter](claim.md)
takes an item.
