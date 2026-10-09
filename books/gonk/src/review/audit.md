# The audit protocol, and why per-commit review is off

The last chapter showed the machinery: a review pass reads a file, a model proposes findings,
and a person declines them or publishes them with the evidence that they are real. gonk can
run that pass on every commit, from a `post-commit` hook. On the machine this book was written
on, that hook is installed in every repository and does nothing, because
`gonk.review.arm` is not `true`. This chapter is why, told with the numbers that decided it,
and what replaced it.

There is nothing to run on this page. It is the one chapter in the book whose claims are
measurements someone else made, so it says where they are and what they do and do not show.

## The question

A per-commit reviewer is cheap to start and expensive to keep. Every commit queues every file it
touched; a model reads each one and proposes findings; a person reads every proposal and decides.
By late September 2026 that queue held more than a thousand pending findings across the
repositories gonk browses, most of them minor, and the people reading it were spending their
attention on the reviewer's guesses.

So the question was put as an experiment, with the rule for deciding written down before any
data came in: **per serious bug actually found, which costs less — following the reviewer's
leads, or a strong model auditing the same code with no leads at all?** A bug counted only if
it came with a reproduction: a test or probe that fails, on the pinned commit, *because of* the
defect. "Serious" meant wrong output, a crash, a security hole or lost data. The cost was the
tokens each arm spent.

## Round 1: the local reviewer's leads (2026-10-03)

Three repositories, each at a pinned commit, read-only. The **led** arm was handed the
reviewer's findings that a second model had already judged credible, and set out to reproduce
them; the **unled** arm audited the same files blind. Same model in both arms, the same
tool-call budget.

| | serious bugs | tokens | serious per 100k tokens |
| --- | --- | --- | --- |
| led (follow the reviewer) | 6 | 756,357 | 0.79 |
| unled (audit blind) | 19 | 1,128,583 | 1.68 |

The unled audits found about **2.1 times** as many serious bugs per token, and the overlap was
small: in one repository the two arms found none of each other's bugs. Worse for the reviewer,
47 of its claims that had been judged credible were checked one by one, and **none held as
stated** — two were partly true, with the mechanism wrong. Where the leads helped, they helped
by pointing at a file, and the file list did not need them.

## Round 2: a frontier reviewer (2026-10-05)

Round 1's reviewer was a local model, so a fair objection was that a better model would review
better. Round 2 used the same frontier model in every arm, on two repositories (the ledger
itself and a web application), and added two arms:

| arm | what it did | serious per 100k tokens |
| --- | --- | --- |
| R | per-commit diff review in the style of a review bot, read-only, then a separate pass to reproduce its claims (both passes' tokens count) | 0.72 |
| U | an unled audit, as in round 1 | 1.49 |
| U2 | a second, independent unled audit, same protocol | 1.99 |
| D | an unled audit that starts from the window's commits, then follows the code | 1.85 |

Sixteen distinct serious bugs were found by some arm; any single audit found 10 to 13 of them.

What the round decided, by the rules written before it:

- **Per-commit review stays off.** The frontier reviewer was *precise* — 12 of its 15 claims
  reproduced, against 0 of 47 in round 1 — but it found less than half as much per token. A
  diff never shows the reviewer a defect that is older than the diff, and a quiet week of
  dependency bumps gives it almost nothing to read. A better model makes a reviewer's findings
  honest; it does not make them complete.
- **Two independent auditors per repository.** The second audit, identical in protocol, added
  5 serious bugs to the first's 10 — exactly the 50% the rule had named as the bar.
- **Starting from the commits is allowed, not required.** D beat U on paper, but U and U2 — the
  same protocol run twice — differed by more than D and U did, so the gap is inside the noise.
  D did find one class of bug (a reflected script injection) that no other arm saw, so one of
  the two auditors may start from the commits.

## What the evidence is, and what it is not

It **is** five pairings across five repositories, one model per round, every bug backed by a
reproduction that was re-run by someone other than the arm that wrote it, and decision rules
fixed before the data. It is enough to stop paying for something that was measurably worse.

It is **not** a controlled study. The samples are small; there is no variance estimate beyond
the one duplicated arm, which showed that run-to-run variance is large; severity classes were
harmonized across arms by hand, after the fact, by the person who scored them; and in round 1
the file lists were chosen *by* the reviewer's findings, which favored the led arm. It says
nothing about other models, other codebases, or review whose purpose is teaching rather than
finding bugs. And it was run by the people who built the reviewer, which cuts both ways.

The write-ups are in `claude/research/review-exp-2026-10-03/README.md` and
`claude/research/review-exp2-2026-10-05/README.md` of `ikigai-devtools`, a private repository;
the numbers above are copied from them, and the reproductions that back them are not public.

## The protocol that replaced it

Real bugs now come from **periodic unled audits**, one repository at a time, on a rotation
(oldest first; a repository with a large recent change or a security surface moves up):

- **Two independent auditors**, each in its own detached worktree at one pinned commit,
  blind to each other and to every earlier finding, annotation and review for that repository.
  Code, tests, documentation and history are fair game.
- **Read-only.** No fixes during an audit: the findings are about one known commit, and fixing
  is a separate piece of work scheduled from the report.
- **A focus and a budget** — a file list or an area, and a number of tool calls — and a report
  that says whether the budget ran out or the work did, and lists what was *not* read.
- **A bug counts only with a reproduction.** An unreproduced suspicion is reported as a
  suspicion, never as a bug.
- **A finding is closed only with evidence**: fixed (the regression test and the change),
  refuted (why the reproduction was wrong), or deferred (to what). The evidence is written on
  the ledger item before it is closed.

The two auditors' reports are merged by root cause into one ledger item per repository, and that
item is the work. Nobody reads a model's unreproduced guesses any more.

## What is still there

None of the machinery went away, only the trigger:

- **One-off review** is the previous chapter, unchanged: the Review button on a file page in
  gonk's browser, or a Source of `urn:repo:{root}:review:{path}`. It is useful exactly where the
  experiment says a reviewer is — pointed by a person at a file the person already suspects.
- **Findings stay pending until a person acts.** A review pass never publishes; an annotation
  exists only because someone published a finding, and `reproduced=` says how they knew.
- **The hook is still installed**, and `gonk.review.arm = true` in the config home turns it
  back on. Nothing about this decision is enforced by deleting code; it is a configuration
  that can be reversed by the same evidence that set it.
- **What an armed queue fires is gonk's decision, not the queue directory's.** The review
  space keeps a `handler` file naming what to fire for each request, and by default the
  reactor underneath would fire whatever that file names, under the reviewer's authority.
  gonk fires its own review pass and nothing else: a `handler` file retargeted at any other
  resource gets the request refused, dead-lettered into the space's `error/` directory with a
  note naming that target, rather than run. Dropping a
  request is open to anything that can write the directory, so the directory must not be able
  to choose what runs.
