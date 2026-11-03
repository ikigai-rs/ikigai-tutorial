# Work, evidence, review, close

This chapter takes one task from claim to close: reproduce it, fix it, get a second opinion, report
with evidence, and let the dispatcher decide. You hold both roles, as in
[Solo on one machine](../ways/solo.md), and the process still keeps them apart: **a report is not a
close**, and a close says the evidence held.

## Catch up

This gonk also serves the repository as a browse root, so it can show the work and have it reviewed.
The model is this book's stub, as in [Review every commit](../ways/review-every-commit.md): it gives
the same review every time, so this page prints the same thing on every machine. In a new empty
directory, as in [Start a project](start.md). If the gonk from [the previous chapter](claim.md) is
still running, stop it first, since this one takes the same port.

<!-- transcript: serve -->
```console
$ model-stub "$PWD/model.sock" | tee model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
```

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ (cd starter && python3 tools/new.py ../library --process solo,spec > /dev/null)
$ git -C library init -q -b main
$ mkdir -p .config/ikigai
$ printf 'gonk.mount = "prefer urn:llm:=%s/model.sock"\ngonk.browse.root = "library=%s/library"\n' "$PWD" "$PWD" > .config/ikigai/config.toml
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py init --ledger library --socket ../.ikigai/gonk.sock --holder ada | tail -1)
ready.
$ git -C library add -A
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Generate the library sample'
$ (cd library && python3 tools/gonk.py seed | tail -1)
seed: 8 filed, 0 already filed, 0 checked off (not filed), 6 order link(s) added
```

## Claim, and branch

Take the small change's first task, and read what it asks. The lease is longer than the default,
because this is real work:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py claim 7 --lease 2h)
library#7 claimed by ada until …
$ git -C library switch -q -c cap-late-fees-1.1
$ grep -A1 'Requirement: A late return' library/openspec/changes/cap-late-fees/specs/overdue/spec.md
### Requirement: A late return owes a fee
```

## Reproduce before you fix

The delta spec says a return 101 days late owes 1000 cents. An item is a claim about the code, and it
may be wrong, so see it hold first. If it does not, the right change is none, and the evidence is the
reproduction. This small script drives the rules directly and prints each return:

<!-- transcript: file repro.py -->
```python
import sys
from datetime import date

sys.path.insert(0, "library/artifacts/service")
from lending.domain import Library

library = Library()
library.add_title("9780141439518", "Pride and Prejudice", "Jane Austen")
library.add_copy("pp-1", "9780141439518")
library.add_member("ada", "Ada Lovelace")
library.lend("ada", "pp-1", date(2026, 10, 1))
for returned in sys.argv[1:]:
    print(library.give_back("pp-1", date.fromisoformat(returned)))
    library.lend("ada", "pp-1", date(2026, 10, 1))
```

<!-- transcript: run -->
```console
$ PYTHONDONTWRITEBYTECODE=1 python3 repro.py 2027-01-31
{'copy': 'pp-1', 'member': 'ada', 'returned': '2027-01-31', 'days_late': 101, 'fee_cents': 2525}
```

2525 cents, where the spec says 1000. It holds.

## Fix it, and check the box in the same commit

<!-- transcript: file cap.patch -->
```text
--- a/artifacts/service/lending/domain.py
+++ b/artifacts/service/lending/domain.py
@@ -18,6 +18,7 @@
 MAX_LOANS = 3
 MAX_RENEWALS = 2
 FEE_CENTS_PER_DAY = 25
+MAX_FEE_CENTS = 1000
 
 
 class Refused(Exception):
@@ -137,7 +138,7 @@
             "member": loan.member,
             "returned": today.isoformat(),
             "days_late": late,
-            "fee_cents": late * FEE_CENTS_PER_DAY,
+            "fee_cents": min(late * FEE_CENTS_PER_DAY, MAX_FEE_CENTS),
         }
 
     def loans_of(self, member: str, today: date) -> dict:
```

The box goes in the same commit as the code, so the tree and the code never disagree about whether the
task is done:

<!-- transcript: run -->
```console
$ git -C library apply ../cap.patch
$ PYTHONDONTWRITEBYTECODE=1 python3 repro.py 2027-01-31
{'copy': 'pp-1', 'member': 'ada', 'returned': '2027-01-31', 'days_late': 101, 'fee_cents': 1000}
$ (cd library && python3 scripts/test.py 2>&1 | tail -1)
all 3 suites passed
$ sed -i.bak 's/- \[ \] 1.1 /- [x] 1.1 /' library/openspec/changes/cap-late-fees/tasks.md && rm library/openspec/changes/cap-late-fees/tasks.md.bak
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -am 'cap-late-fees 1.1: stop a late fee at 1000 cents'
```

The tests passed before the fix too, since no vector covers the cap yet. That vector is task 1.2, and
this is why a passing suite is not this fix's evidence on its own; the reproduction is.

## Review it

Ask the mounted model to read the file. A review mints **findings**, and a finding is pending until a
person decides it ([One-off review](../review/one-off.md)). Wait a moment first, because gonk watches
the root and reviews the file as it finds it:

<!-- transcript: run -->
```console
$ sleep 1
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:library:review:artifacts/service/lending/domain.py'
review by stub · review-v5@stub · 1 finding · reviewed the whole file (… bytes)
--- annotations (1) ---
L141 [review:stub] ""fee_cents": min(late * FEE_CENTS_PER_DAY, MAX_FEE_CENTS)," -- The cap is applied per day rather than to the total, so a return 41 days late still owes more than 1000 cents.
[uncacheable]
```

A finding is a claim too, and it is cheap to check. The cap is reached at 40 days, so try 40 and 41
(the loan was due on October 22, so that is December 1 and 2):

<!-- transcript: run -->
```console
$ PYTHONDONTWRITEBYTECODE=1 python3 repro.py 2026-12-01 2026-12-02
{'copy': 'pp-1', 'member': 'ada', 'returned': '2026-12-01', 'days_late': 40, 'fee_cents': 1000}
{'copy': 'pp-1', 'member': 'ada', 'returned': '2026-12-02', 'days_late': 41, 'fee_cents': 1000}
```

It does not hold, so decline it, with a reason from the closed set and what you ran. A published
finding would need a reproduction that does show the defect; both are records nobody overwrites.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:library:findings:artifacts/service/lending/domain.py' 2>/dev/null | jq -r '.[0].iri | sub("^urn:"; "urn:gk:")' > finding.txt
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c "sink $(cat finding.txt) decision=decline reason=misread 41 days late owes 1000 cents: the cap is on the total."
urn:iki:finding:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c "source $(cat finding.txt)" 2>/dev/null | grep -c 'declined'
1
```

With your own model, the words and the findings will be different, and some will be right. The steps
are the same.

## Report with evidence

The worker's end is `report`: the evidence goes on the item as a comment, the claim is handed back, and
the item waits for its dispatcher. Evidence is required:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py report 7)
gonk.py: --evidence is a URL (the pull request, or the reproduction that shows it is not a defect): a report without evidence is a claim nobody can check
```

The rules ask for the pull request, and this repository has no forge. A made-up URL is the easy wrong
answer. The honest one, until there is a forge, is gonk's own page for the file, with the commit and
the check in the note. It is local evidence and proves nothing to a reader on another machine, so a
repository that will be shared gets a forge before its first hand-back.

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py report 7 --evidence http://127.0.0.1:1070/browse/urn:repo:library:file:artifacts/service/lending/domain.py --note 'on the branch cap-late-fees-1.1; repro.py says 1000 at 101 days; scripts/test.py passes')
commented on library#7 (urn:iki:ledger:library:comment:…) awaiting-dispatcher
library#7 released
$ (cd library && python3 tools/gonk.py next | grep -E '^ [0-9]\.|^ready')
 1. library#1  open    p-  add-holds 1.1: Accept a hold on a title whose copies are all out  [artifact:cli artifact:service change:add-holds]
ready: 1   excluded: 7
```

`next` stops offering it. The item is neither done nor free: it is waiting for someone to check.

## The evidence does not hold

Now change hats. The dispatcher reads the evidence, and the report says the fix is on a branch: `main`
still owes 2525 cents. `reject` says why, and the item is ready again:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py reject 7 --note 'The fix is on a branch: merge it, then report again')
commented on library#7 (urn:iki:ledger:library:comment:…) cleared-awaiting
$ (cd library && python3 tools/gonk.py next | grep -E '^ 1\.')
 1. library#7  open    p-  cap-late-fees 1.1: Stop a late return's fee at 1000 cents  [artifact:service change:cap-late-fees]
```

So the worker claims it again, merges, and reports again. With a forge, this merge is the pull request
going green:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py claim 7)
library#7 claimed by ada until …
$ git -C library switch -q main
$ git -C library -c user.name=Ada -c user.email=ada@example.com merge -q --no-ff -m 'Merge cap-late-fees 1.1' cap-late-fees-1.1
$ PYTHONDONTWRITEBYTECODE=1 python3 repro.py 2027-01-31
{'copy': 'pp-1', 'member': 'ada', 'returned': '2027-01-31', 'days_late': 101, 'fee_cents': 1000}
$ (cd library && python3 tools/gonk.py report 7 --evidence http://127.0.0.1:1070/browse/urn:repo:library:file:artifacts/service/lending/domain.py --note 'merged to main; repro.py says 1000 at 101 days')
commented on library#7 (urn:iki:ledger:library:comment:…) awaiting-dispatcher
library#7 released
```

## Close

The dispatcher checks again, and this time it holds. `close` needs evidence as well, and closing makes
the next task ready:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py close 7 --evidence http://127.0.0.1:1070/browse/urn:repo:library:file:artifacts/service/lending/domain.py --note 'Checked: on main, 1000 cents at 101 days')
closed library#7 (done)
$ (cd library && python3 tools/gonk.py next | grep -E '^ [0-9]\.')
 1. library#1  open    p-  add-holds 1.1: Accept a hold on a title whose copies are all out  [artifact:cli artifact:service change:add-holds]
 2. library#8  open    p-  cap-late-fees 1.2: Add a vector for the capped return to openspec/vectors.json  [artifact:service change:cap-late-fees]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:library:item:7:history' 2>/dev/null | grep -E 'claim|release|closed'
… claim ada
… release ada (released)
… claim ada
… release ada (released)
… closed done
```

Task 1.2 is yours if you want it: write the vector for "Returning very late owes at most ten dollars"
in `openspec/vectors.json`. Run it with the fix reverted to see it fail, which is the test this chapter
did not have. When the last box is checked, the change is archived
([Spec first](../ways/spec-first.md#5-archive)).

| step | who | what it writes |
| --- | --- | --- |
| `claim` | the worker | the lease |
| `report --evidence` | the worker | a comment, `awaiting-dispatcher`, the claim released |
| `reject --note` | the dispatcher | a comment, and the item is ready again |
| `close --evidence` | the dispatcher | a comment, and the close |

<a id="what-you-built"></a>

## What you have now

One task done the whole way: a defect seen before it was fixed, a model's claim checked and declined,
and a report that was turned down once for a good reason. The [next chapter](two-agents.md) adds
someone working beside you.
