# Filing roborev findings

[roborev](https://github.com/kenn-io/roborev) reviews each commit with a coding agent and
reports **findings**: a problem, a fix, a severity, and usually a file and line. Its reports
are good at being read once. A ledger is good at what comes after: triage, a priority, a
claim, a link to the item it blocks, a close with a reason. This bridge files every finding
of every completed roborev review as a ledger item, without changing how roborev runs.

It is one command, `ikigai-gonk roborev file`, which roborev runs from a hook when a review
completes. This page runs it by hand, against a review saved to a file, which is exactly what
the hook hands it.

## A review, as roborev hands it over

roborev's agent returns a structured document, and what a hook receives is that document
**rendered as Markdown**, in this shape (save it as `review.md`):

<!-- transcript: file review.md -->
```markdown
## Summary

The change adds a retry loop around the store write, but one path retries without a bound and a second swallows the error it retries on.

**Agent assessment:** Fail

## Findings

### 1. Critical

**Location:** /work/demo/src/retry.rs:41-58

**Problem:** `write_with_retry` loops while the store answers `Busy` and has no attempt limit. Trigger: a writer that holds the RocksDB lock for longer than the caller waits (a second `ikigai-gonk` on the same store) spins this loop forever at 100% CPU.

**Fix:** Bound the loop (five attempts with backoff) and return the last `Busy` to the caller.

**Reported by:** codex, claude-code

### 2. High

**Location:** src/retry.rs:72

**Problem:** On the final attempt the error is logged and `Ok(())` is returned.

The caller therefore records a write that never happened. Trigger: any store refusal on the third attempt.

**Fix:** Return the error from the final attempt instead of `Ok(())`.

**Reported by:** codex

### 3. Medium

**Problem:** The retry delay is a fixed 100 ms, so two writers retrying together collide on every attempt.

**Fix:** Add jitter to the delay.

**Reported by:** claude-code

### 4. Low

**Location:** `src/retry.rs:12`

**Problem:** The doc comment says "retries three times" and the constant is 4.

**Fix:** Make the comment name the constant.

**Reported by:** codex
```

`roborev file` parses that rendering back, and accepts a finding only when rendering it again
reproduces the bytes exactly. A review that has drifted from the format (a new roborev that
renders differently, say) is refused, loudly, with nothing filed, rather than filed as a guess.

## Filing it

Start a scratch gonk, as in every chapter:

<!-- transcript: serve -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
```

`--dry-run` prints what it would file, each append's arguments with the key among them, and
touches nothing. It does not ask gonk anything, so it says `would file` for a finding that is
already there, and it does not check that the door grants the ledger; the real run is the one
that finds out both:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --gonk http://127.0.0.1:1070 --ledger default --root demo --repo-path /work/demo --job 42 --sha 1a2b3c4 --agent codex --findings - --dry-run < review.md
skipped  low      below --min-severity medium: The doc comment says "retries three times" and the constant is 4.
would file critical: `write_with_retry` loops while the store answers `Busy` and has no attempt limit. Trigger: a writer…
    key=urn:roborev:finding:9b1fa066c174c6e500245e3a48d8eae8
    labels=roborev,critical
    priority=0
    about=urn:repo:demo:file:src/retry.rs urn:roborev:finding:9b1fa066c174c6e500245e3a48d8eae8
    author=roborev
    revision=1a2b3c4
would file high: On the final attempt the error is logged and `Ok(())` is returned.
    key=urn:roborev:finding:f8ac8944363de4badd880f8a593b1ccc
    labels=roborev,high
    priority=1
    about=urn:repo:demo:file:src/retry.rs urn:roborev:finding:f8ac8944363de4badd880f8a593b1ccc
    author=roborev
    revision=1a2b3c4
would file medium: The retry delay is a fixed 100 ms, so two writers retrying together collide on every attempt.
    key=urn:roborev:finding:9f1fdc5febfc449e1de32552a5365e54
    labels=roborev,medium
    priority=2
    about=urn:roborev:finding:9f1fdc5febfc449e1de32552a5365e54
    author=roborev
    revision=1a2b3c4
```

Each line of that is a decision the command made, and each is worth reading once:

- **The low finding is skipped.** roborev renders low findings even on a passing review, and
  one ledger item per nit buries the findings worth a person's time. `--min-severity low` files
  them; `high` files less.
- **Severity becomes a label and a priority**: critical 0, high 1, medium 2, low 3.
- **A location becomes an `about`.** `/work/demo/src/retry.rs:41-58` is under `--repo-path`, so
  it is made relative and named as the browse resource for that file,
  `urn:repo:demo:file:src/retry.rs`, where `demo` is the **browse root** name gonk serves the
  repository under (`--root`). That is the join browse's file page makes, so the item shows up
  beside the file. The medium finding named no file, so it gets no file `about` rather than a
  guessed one.
- **Every finding gets a key**, `urn:roborev:finding:{hash}`, over the root, the file and the
  problem's words with their whitespace collapsed. It is filed as the append's `key` and again
  as an `about`, and it is how the ledger knows a finding is already filed.

Now for real:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --gonk http://127.0.0.1:1070 --ledger default --root demo --repo-path /work/demo --job 42 --sha 1a2b3c4 --agent codex --findings - < review.md
skipped  low      below --min-severity medium: The doc comment says "retries three times" and the constant is 4.
filed    critical #1 `write_with_retry` loops while the store answers `Busy` and has no attempt limit. Trigger: a writer…
filed    high     #2 On the final attempt the error is logged and `Ok(())` is returned.
filed    medium   #3 The retry delay is a fixed 100 ms, so two writers retrying together collide on every attempt.
$ curl -s http://127.0.0.1:1070/iki/ledger/item/1
   #1  open    p0  `write_with_retry` loops while the store answers `Busy` and has no attempt limit. Trigger: a writer…  [critical roborev]
  iri:      urn:iki:ledger:default:item:…
  key:      urn:roborev:finding:9b1fa066c174c6e500245e3a48d8eae8
  filed:    … by roborev
  updated:  …
  revision: 1a2b3c4
  about:    urn:repo:demo:file:src/retry.rs
  about:    urn:roborev:finding:9b1fa066c174c6e500245e3a48d8eae8

  `write_with_retry` loops while the store answers `Busy` and has no attempt limit. Trigger: a writer that holds the RocksDB lock for longer than the caller waits (a second `ikigai-gonk` on the same store) spins this loop forever at 100% CPU.

  Fix: Bound the loop (five attempts with backoff) and return the last `Busy` to the caller.

  - severity: critical
  - location: /work/demo/src/retry.rs:41-58
  - commit: 1a2b3c4
  - roborev job: 42 (`roborev show 42`)
  - agent: codex
  - reported by: codex, claude-code
  - key: urn:roborev:finding:9b1fa066c174c6e500245e3a48d8eae8
```

The title is the problem's first line, cut near a hundred characters; the body keeps the whole
problem, the fix, and where it came from. The item is an ordinary ledger item from here on:
`next` ranks it, a person claims it, and closing it is a decision with a reason rather than a
report scrolling away.

## The hook

What ran that by hand is one `[[hooks]]` entry in roborev's config (`~/.roborev/config.toml`,
or a repository's `.roborev.toml`):

```toml
[[hooks]]
event = "review.completed"
command = "ikigai-gonk roborev file --gonk http://127.0.0.1:1060 --ledger default --root {repo_name} --repo-path {repo} --job {job_id} --sha {sha} --agent {agent} --findings {findings}"
```

roborev replaces each `{…}` with a **single-quoted** value before `sh -c` runs the line, so
nothing in a finding can escape into the shell, and `{findings}` is the whole Markdown review
as one argument. `{repo_name}` is right for `--root` when the repository's name and its browse
root's name match; otherwise write the root's name in. (The URL is a real gonk's port, 1060,
because that is what your hook will name. The page uses 1070.)

So the hook's own invocation, for the review above, is this. It is the third time this page has
handed gonk the same review, and it files nothing:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --gonk http://127.0.0.1:1070 --ledger default --root 'demo' --repo-path '/work/demo' --job '42' --sha '1a2b3c4' --agent 'codex' --findings "$(cat review.md)"
skipped  low      below --min-severity medium: The doc comment says "retries three times" and the constant is 4.
already  critical #1 (open) urn:roborev:finding:9b1fa066c174c6e500245e3a48d8eae8
already  high     #2 (open) urn:roborev:finding:f8ac8944363de4badd880f8a593b1ccc
already  medium   #3 (open) urn:roborev:finding:9f1fdc5febfc449e1de32552a5365e54
```

Each finding is **one** request, a keyed append (`append key=…`): the ledger checks the key and
files the item in the same store update, and a key that is already taken, by an open, closed or
deleted item, answers that item and files nothing. The command reads that answer through the
ledger's JSON face, not its plain text, and prints `already` with the item's status. So a payload
run twice files once, a later review that carries a finding forward word for word files nothing
new, and two hooks filing the same finding at the same instant file it once. roborev runs every
hook in its own goroutine, so that last one is not hypothetical; gonk's own tests race eight
hooks on one finding and get one item. One limit, stated rather than discovered: the same defect
in **different words** is a new key and a second item.

A roborev `fix` or `task` job fires the same event, and its output is prose, not a review. The
command says so and files nothing, with exit status 0, so roborev's log stays quiet:

<!-- transcript: run -->
```console
$ printf 'Applied the fix and ran the tests.\n' | ikigai-gonk roborev file --gonk http://127.0.0.1:1070 --ledger default --root demo --findings -
not a structured review (a fix or task job?): nothing to file
```

## Whose ledger it files into

The command speaks to gonk's HTTP door as its anonymous loopback caller, which holds the read
and write tokens of the ledgers in `gonk.http.ledger` and nothing else. The keyed append is a
write, and the door grants a listed ledger's read and write together. A ledger the door does not
grant is refused at that first append, and the command stops with exit status 1 and names the
setting:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --gonk http://127.0.0.1:1070 --ledger reviews --root demo --findings - < review.md
skipped  low      below --min-severity medium: The doc comment says "retries three times" and the constant is 4.
ikigai-gonk: gonk refused POST /iki/ledger/reviews/append (403): denied: this capability does not hold `urn:cap:ledger:write:reviews`. A ledger grant names exactly one ledger — holding a grant over another ledger satisfies the declared family `urn:cap:ledger:write:*` but not this resource, which is the point of naming them. The HTTP door grants an anonymous loopback caller the read and write tokens of the ledgers in `gonk.http.ledger` and nothing else; filing needs both (`ikigai-gonk grants <ledger> write` prints them, and listing the ledger there grants them)
```

To keep findings in a ledger of their own, list it beside `default` in the config home's
`config.toml` and restart gonk:

```toml
gonk.http.ledger = "default"
gonk.http.ledger = "reviews"
```

roborev logs a hook's failure and its output in its daemon log, and never retries, so a hook
that is refused is visible there and nowhere else. That is worth a look the first time a
review completes.

⚠ One more limit, from roborev's side. A **panel** review fires `review.completed` once per
member and once more for the synthesis, with the findings merged and reworded, and a hook cannot
tell which is which. So a panel files each member's findings and the synthesis's, deduplicated
only where the words match. Point the hook at single-agent reviews if that is noise.
