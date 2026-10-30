# Filing roborev findings

[roborev](https://github.com/kenn-io/roborev) reviews each commit with a coding agent and
reports **findings**: a problem, a fix, a severity, and usually a file and line. Its reports
are good at being read once. A ledger is good at what comes after: triage, a priority, a
claim, a link to the item it blocks, a close with a reason. This bridge files every finding
of every completed roborev review as a ledger item, without changing how roborev runs.

It is one command, `ikigai-gonk roborev file`, which roborev runs from a hook when a review
completes. This page runs it by hand, against a review saved to a file, which is exactly what
the hook hands it.

Filing is a write, so the command files with an identity: a **client certificate** enrolled to
write one ledger, at gonk's QUIC door, which is how this page files; or, as a shortcut for a hook
that runs as the same user as gonk, gonk's owner-only **socket**, shown at the end of the page.
(Through gonk 3edf830 it filed through the HTTP door as that door's anonymous caller. Since gonk
PR 186, ledger #1687, that caller reads and writes nothing, and the gonk this book pins files only
with an identity: gonk PR 212, ledger #2075.)

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

## An identity for the hook

Work in a new empty directory, as in [every chapter](../introduction.md#a-gonk-per-chapter). First
give the hook a client of its own, enrolled to write the `default` ledger and nothing else:

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ ikigai-gonk client add roborev --ledger default=write --port 1070 --config-home "$PWD/.config/ikigai"
client `roborev`  …/.config/ikigai/gonk/quic/clients/roborev
  fingerprint  …
  principal    urn:iki:host:gonk:client:…
  enrolled     grant `roborev` (4 scopes) in …/.config/ikigai/gonk/grants.json
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <the moved directory>
```

The bundle, `.config/ikigai/gonk/quic/clients/roborev/`, holds the client's certificate and key
and the server certificate to pin; the command names it with `--cert-dir`. When roborev runs as
another user, in a sandbox or on another machine, that directory is what you move to where it can
read it. The grant is narrow (one ledger's writes, as [Over MCP](../agent/mcp.md) reads the same
four scopes) and revocable (`ikigai-gonk client remove roborev`).

<a id="filing-it"></a>

## Filing a review

Start a scratch gonk in the same directory. The last chapter to start one was [Explain, with a
mounted model](../browse/explain.md) (the review and spec chapters run none); if that gonk is still
running, stop it first, since this one takes the same port. This one opens its QUIC door, on UDP
1070 beside the HTTP door, because a client is enrolled:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  quic    udp 0.0.0.0:1070 — 1 trusted certificate(s), 1 enrolled
          client  roborev  …  trusted and enrolled
…
```

`--dry-run` prints what it would file, each append's arguments with the key among them, and
touches nothing. It does not ask gonk anything, so it says `would file` for a finding that is
already there, and it does not check that the client may write the ledger; the real run is the
one that finds out both:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --quic 127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/roborev --ledger default --root demo --repo-path /work/demo --job 42 --sha 1a2b3c4 --agent codex --findings - --dry-run < review.md
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
- **`author=roborev` is the socket's author.** Over the QUIC door the command sends no author,
  and gonk fills in the client's own principal, as the item below shows. The dry run prints the
  plan the same way whichever door the real run would take.

Now for real, and the item it filed first, read back through the HTTP door, whose anonymous
caller may read `default`:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --quic 127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/roborev --ledger default --root demo --repo-path /work/demo --job 42 --sha 1a2b3c4 --agent codex --findings - < review.md
skipped  low      below --min-severity medium: The doc comment says "retries three times" and the constant is 4.
filed    critical #1 `write_with_retry` loops while the store answers `Busy` and has no attempt limit. Trigger: a writer…
filed    high     #2 On the final attempt the error is logged and `Ok(())` is returned.
filed    medium   #3 The retry delay is a fixed 100 ms, so two writers retrying together collide on every attempt.
$ curl -s http://127.0.0.1:1070/iki/ledger/item/1
   #1  open    p0  `write_with_retry` loops while the store answers `Busy` and has no attempt limit. Trigger: a writer…  [critical roborev]
  iri:      urn:iki:ledger:default:item:…
  key:      urn:roborev:finding:9b1fa066c174c6e500245e3a48d8eae8
  filed:    … by urn:iki:host:gonk:client:…
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
problem, the fix, and where it came from. The `filed` line names who filed it: the `roborev`
client's principal, `urn:iki:host:gonk:client:` and its certificate's fingerprint, not gonk's
owner. The item is an ordinary ledger item from here on: `next` ranks it, a person claims it, and
closing it is a decision with a reason rather than a report scrolling away.

<a id="the-hook"></a>

## The completion hook

What ran that by hand is one `[[hooks]]` entry in roborev's config (`~/.roborev/config.toml`,
or a repository's `.roborev.toml`):

```toml
[[hooks]]
event = "review.completed"
command = "ikigai-gonk roborev file --quic 127.0.0.1:1062 --cert-dir ~/.config/ikigai/gonk/quic/clients/roborev --ledger default --root {repo_name} --repo-path {repo} --job {job_id} --sha {sha} --agent {agent} --findings {findings}"
```

roborev replaces each `{…}` with a **single-quoted** value before `sh -c` runs the line, so
nothing in a finding can escape into the shell, and `{findings}` is the whole Markdown review
as one argument. `{repo_name}` is right for `--root` when the repository's name and its browse
root's name match; otherwise write the root's name in. (The port is a real gonk's, 1062, whose
QUIC door is on the same port number, and the bundle is where `client add roborev` writes it in
your own config home; the page uses 1070 and its scratch config home.)

So the hook's own invocation, for the review above, is this. It is the second time this page has
handed gonk the same review, and it files nothing: each finding is already there, and the command
prints `already` with the item's number and status:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --quic 127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/roborev --ledger default --root 'demo' --repo-path '/work/demo' --job '42' --sha '1a2b3c4' --agent 'codex' --findings "$(cat review.md)"
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
$ printf 'Applied the fix and ran the tests.\n' | ikigai-gonk roborev file --quic 127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/roborev --ledger default --root demo --findings -
not a structured review (a fix or task job?): nothing to file
```

## Whose ledger it files into

The client files under the grant it was enrolled with, and nothing more: `default`'s writes. A
ledger the grant does not name is refused at the first append, and the command stops with exit
status 1, naming the client it filed as:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --quic 127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/roborev --ledger reviews --root demo --findings - < review.md
skipped  low      below --min-severity medium: The doc comment says "retries three times" and the constant is 4.
ikigai-gonk: gonk refused Sink urn:iki:ledger:reviews:append over quic://127.0.0.1:1070 as the client in .config/ikigai/gonk/quic/clients/roborev: denied: this capability does not hold `urn:cap:ledger:write:reviews`. A ledger grant names exactly one ledger — holding a grant over another ledger satisfies the declared family `urn:cap:ledger:write:*` but not this resource, which is the point of naming them
```

To keep findings in a ledger of their own, widen the hook's grant to that ledger and name it in
the hook's `--ledger`: `client add roborev --ledger default=write --ledger reviews=write --force`.
An enrolled client keeps its identity, so the bundle does not change; `--force` replaces its grant
(without it, a change to an existing grant is refused, listing every scope it would add and
remove), and the running gonk holds the new list from the client's next request. A certificate gonk does not trust, or
one enrolled under no grant, is refused at the door before any of that, and the command says
what that likely means and prints the enrollment command.

## The owner's socket, a shortcut

Without `--quic`, the command files over gonk's owner-only socket (`--socket PATH`, or the
`gonk.socket` of the config it reads; `~/.ikigai/gonk.sock` by default), as the user who runs
gonk. That needs no enrollment, and it is a shortcut only for a hook that runs as that user, on
that machine, outside any sandbox, because the socket carries the owner's whole authority over
every ledger. Here it files the same review into `reviews`, which the client could not, and the
items are authored `roborev`, the author the command sends:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --socket "$PWD/.ikigai/gonk.sock" --ledger reviews --root demo --repo-path /work/demo --job 42 --sha 1a2b3c4 --agent codex --findings - < review.md
skipped  low      below --min-severity medium: The doc comment says "retries three times" and the constant is 4.
filed    critical reviews#1 `write_with_retry` loops while the store answers `Busy` and has no attempt limit. Trigger: a writer…
filed    high     reviews#2 On the final attempt the error is logged and `Ok(())` is returned.
filed    medium   reviews#3 The retry delay is a fixed 100 ms, so two writers retrying together collide on every attempt.
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:reviews:item:1' | grep 'filed:'
[uncacheable]
  filed:    … by roborev
```

(`[uncacheable]` is the cli's status line for a read through a mount; it goes to standard error,
which `grep` does not see.)

When a hook line names both, `--quic` wins and the socket is not dialed. A hook line written for
an older gonk names `--gonk`, the HTTP door, which the command now refuses with exit status 1,
naming both ways in:

<!-- transcript: run -->
```console
$ ikigai-gonk roborev file --gonk http://127.0.0.1:1070 --ledger default --root demo --findings - < review.md 2>&1 | head -n 1
ikigai-gonk: roborev file: --gonk is gone. It named gonk's HTTP door, whose anonymous caller only reads since ledger #1687, so filing through it answered 403. This command files with an identity now: drop --gonk, and pass --quic HOST:PORT --cert-dir DIR to file as a client enrolled with `ikigai-gonk client add <name> --ledger <ledger>=write`, or file as gonk's owner over its socket (--socket PATH, or --config the config.toml naming `gonk.socket`, when it is not ~/.ikigai/gonk.sock)
```

(It prints the command's usage after that line, which `head` leaves out.)

roborev logs a hook's failure and its output in its daemon log, and never retries, so a hook
that is refused is visible there and nowhere else. That is worth a look the first time a
review completes.

⚠ One more limit, from roborev's side. A **panel** review fires `review.completed` once per
member and once more for the synthesis, with the findings merged and reworded, and a hook cannot
tell which is which. So a panel files each member's findings and the synthesis's, deduplicated
only where the words match. Point the hook at single-agent reviews if that is noise.
