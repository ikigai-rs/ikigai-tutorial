# A ledger of your own

A **ledger** here is a list of work: items you file, comment on, link, claim and close, and
one question it answers as a resource rather than as a sort order — *what should I do next?*
It is the smallest useful thing gonk serves, and everything later in this book files into it,
so it comes first.

You will use the same ledger three times in this chapter, and they are three views of one
thing rather than three things:

1. **in this page**, where a kernel compiled to wasm holds it and every Run is real;
2. **in a kernel of your own**, a few lines of Rust over two published modules; and
3. **served by gonk**, a binary you start, reached with `curl`.

## In the page

Each cell below sends its line to this page's kernel: `ikigai-ledger` over an in-memory
`ikigai-store`, the same composition the Rust half of this chapter builds. The cells share one
ledger, so run them in order; reload the page for an empty one. Where the ledger mints
something — an item's id, a time — the expected output says `…`.

File three items. The first line of a value is the title; `priority` and `labels` are named
arguments.

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:append Write the gonk book'>
<pre class="ikigai-run-expected">#1 urn:iki:ledger:default:item:…
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:append priority=0 labels=book Pin gonk to a git rev'>
<pre class="ikigai-run-expected">#2 urn:iki:ledger:default:item:…
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:append priority=1 Check every transcript'>
<pre class="ikigai-run-expected">#3 urn:iki:ledger:default:item:…
[uncacheable]</pre>
</div>

An item has two names. `#2` is a **label** the ledger gives it for people, numbered in filing
order; `urn:iki:ledger:default:item:…` is its **name**, minted once and never reused, and it is
what a link, a comment or another graph refers to. A write is `[uncacheable]`: it changes the
ledger, so there is nothing to keep.

List them:

<div class="ikigai-run" data-cmd='source urn:iki:ledger:items'>
<pre class="ikigai-run-expected">   #3  open    p1  Check every transcript
   #2  open    p0  Pin gonk to a git rev  [book]
   #1  open    p-  Write the gonk book
3 item(s)
[computed]</pre>
</div>

`p-` is no priority at all, which is different from a low one. The list is the most recently
updated first; it is a listing, not a recommendation. A comment and a link make the next answer
more interesting: `#2` must happen before `#1` can.

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:comment item=2 The rev lives in books/gonk/gonk.rev.'>
<pre class="ikigai-run-expected">commented on #2 (urn:iki:ledger:default:comment:…)
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:link item=2 type=blocks 1'>
<pre class="ikigai-run-expected">linked #2 blocks #1
[uncacheable]</pre>
</div>

Now ask:

<div class="ikigai-run" data-cmd='source urn:iki:ledger:next'>
<pre class="ikigai-run-expected"> 1.    #2  open    p0  Pin gonk to a git rev  [book]
    p0 — priority 0; last updated …
 2.    #3  open    p1  Check every transcript
    p1 — priority 1; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 1
  not #1: blocked by #2
[computed]</pre>
</div>

That answer is three things a sort order cannot give you. It is a **ready set**: `#1` is not
ranked low, it is *excluded*, and the reason is printed. Each ranked item says **why** it is
where it is. And the policy is **named** — `priority-recency` is one of the ranking policies
the host registered, and `urn:iki:ledger:policy:priority-recency` describes what it weighs.
`next` is a view over the ledger, not a column in it.

Claim `#3`, and close `#2`. A claim needs a holder, because an unattributed claim is
indistinguishable from none; a close takes its reason as the value, recorded as a comment.

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:claim item=3 you'>
<pre class="ikigai-run-expected">#3 claimed by you
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:close item=2 Pinned.'>
<pre class="ikigai-run-expected">closed #2 (done)
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='source urn:iki:ledger:next'>
<pre class="ikigai-run-expected"> 1.    #1  open    p-  Write the gonk book
    unprioritized — no priority set, so it ranks below every item that has one; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 1
  not #3: claimed by you
[computed]</pre>
</div>

Closing `#2` unblocked `#1` without anybody touching `#1`, and `#3` left the ready set because
somebody has it. Nothing told `next` that anything changed; it was **derived again**, because
what it was derived from was written. That is a golden thread, the kernel's dependency
tracking — the [ikigai Book's chapter on it](../../getting-started/golden-threads.html) is where it is taught.

## In a kernel of your own

That page's kernel is a crate in this repository,
[`crates/ledger-host`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates/ledger-host),
and it is short because it writes no endpoint. The ledger owns no bytes: every read it makes
is a SPARQL query at the store's narrow door for one graph, and every write is a SPARQL update
at another. So a host's whole job is to put the two side by side:

```rust,ignore
{{#include ../../../../crates/ledger-host/src/lib.rs:space}}
```

and to give the kernel a clock, because every item, comment and claim is stamped and the
ledger refuses a write it cannot stamp:

```rust,ignore
{{#include ../../../../crates/ledger-host/src/lib.rs:kernel}}
```

The story clock starts at 2026-10-06T09:00:00Z and advances a second each time it is read, so
two items are never filed "at the same instant" and the times a run prints are the times the
next run prints. A real host passes its own clock to `kernel_with_clock`.

Using a resource from Rust is one request, issued: a verb, a name, named arguments. The body of
a Sink is the argument called `content` — the name the REPL and gonk's HTTP door both put a
value or a request body under.

```rust,ignore
{{#include ../../../../crates/ledger-host/src/lib.rs:ask}}
```

Here is the page's walk again, compiled and run by `mdbook test`:

```rust
# extern crate ikigai_core;
# extern crate ledger_host;
use ikigai_core::Verb;
use ledger_host::{ask, kernel};

let ledger = kernel();
let file = |text: &str, args: &[(&str, &str)]| {
    let mut all = vec![("content", text)];
    all.extend_from_slice(args);
    ask(&ledger, Verb::Sink, "urn:iki:ledger:append", &all).unwrap()
};

assert!(file("Write the gonk book", &[]).starts_with("#1 urn:iki:ledger:default:item:"));
file("Pin gonk to a git rev", &[("priority", "0"), ("labels", "book")]);
file("Check every transcript", &[("priority", "1")]);

ask(&ledger, Verb::Sink, "urn:iki:ledger:link",
    &[("item", "2"), ("type", "blocks"), ("content", "1")]).unwrap();

let next = ask(&ledger, Verb::Source, "urn:iki:ledger:next", &[]).unwrap();
assert!(next.contains("not #1: blocked by #2"), "{next}");

ask(&ledger, Verb::Sink, "urn:iki:ledger:claim",
    &[("item", "3"), ("content", "you")]).unwrap();
ask(&ledger, Verb::Sink, "urn:iki:ledger:close",
    &[("item", "2"), ("content", "Pinned.")]).unwrap();

let next = ask(&ledger, Verb::Source, "urn:iki:ledger:next", &[]).unwrap();
assert!(next.contains("ready: 1"), "{next}");
assert!(next.contains("not #3: claimed by you"), "{next}");
```

### Whose ledger

Every request above ran as **root**, which a host may do for itself. A caller is given less.
The ledger gates each action by the ledger's name, and — because it asks the store under the
*caller's* capability, never its own — the caller also needs the store's grant for that
ledger's graph. Four tokens make a writer of one ledger:

```rust,ignore
{{#include ../../../../crates/ledger-host/src/lib.rs:grant}}
```

and those four reach that ledger and no other. A ledger is named in the resource's IRI
(`urn:iki:ledger:book:append`), not passed as an argument, precisely so that a capability can
bind to it:

```rust
# extern crate ikigai_core;
# extern crate ledger_host;
use ikigai_core::{Error, Verb};
use ledger_host::{ask_as, kernel, write_grant};

let ledger = kernel();
let book = write_grant("book");

let filed = ask_as(&ledger, &book, Verb::Sink, "urn:iki:ledger:book:append",
                   &[("content", "Mine")]).unwrap();
assert!(filed.starts_with("book#1 "), "{filed}");

let refused = ask_as(&ledger, &book, Verb::Sink, "urn:iki:ledger:append",
                     &[("content", "Not mine")]);
assert!(matches!(refused, Err(Error::Denied(_))));
```

A named ledger spells its labels with its name (`book#1`), so two ledgers' `#1`s cannot be
confused in one sentence. The ledger called `default` is the one whose name may be left out.

## Served

gonk is that same composition in a process that keeps it: a durable store on disk instead of
an in-memory one, and doors for other processes to reach it by. Install it at the revision
this book pins (see [Which gonk](../introduction.md#which-gonk)), then start one **of your
own** in an empty directory, with its two homes inside that directory:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s)
…
  socket  …/.ikigai/gonk.sock — owner only
  quic    off (--no-quic)
…
```

⚠ **The two homes are the point of that line.** gonk keeps its files in two places. The
**config home** (`~/.config/ikigai` unless told otherwise) holds `config.toml` and gonk's
`gonk/` directory of clients, grants and certificates; the **data home** (`~/.ikigai`) holds the
store, the socket, backups and the review queue. `--config-home` and `--data-home` move them,
so this gonk keeps everything in the directory you started it in. Started without them, it
would open — or, if one is running, be refused by — the store a real gonk on your machine holds.
(`--store` names the dataset's directory outright, and `--socket` the socket, when only one of
them should move.) The paths are written with `$PWD` because gonk prints and hands on the
paths it was given: a relative one would reach the banner's `mount` line as relative too.
Port 1070 keeps it off a real gonk's 1060; `--no-quic` and `--no-backup` keep it from opening a
network door or writing archives this chapter does not need.

The banner is the server describing itself, a line per door and per family it did or did not
compose. Leave it running and use another terminal. Every ledger resource is reachable over
HTTP by a mechanical mapping: the path's segments become the name, the method the verb, query
parameters the arguments, and the body the content.

<!-- transcript: run -->
```console
$ curl -s -X POST --data-binary 'Write the gonk book' http://127.0.0.1:1070/iki/ledger/append
#1 urn:iki:ledger:default:item:…
$ curl -s -X POST --data-binary 'Pin gonk to a git rev' 'http://127.0.0.1:1070/iki/ledger/append?priority=0&labels=book'
#2 urn:iki:ledger:default:item:…
$ curl -s -X POST --data-binary 'Check every transcript' 'http://127.0.0.1:1070/iki/ledger/append?priority=1'
#3 urn:iki:ledger:default:item:…
$ curl -s http://127.0.0.1:1070/iki/ledger/items
   #3  open    p1  Check every transcript
   #2  open    p0  Pin gonk to a git rev  [book]
   #1  open    p-  Write the gonk book

3 item(s)
```

The same answers as the page's, from a different host. Link, claim, close, and ask:

<!-- transcript: run -->
```console
$ curl -s -X POST --data-binary '1' 'http://127.0.0.1:1070/iki/ledger/link?item=2&type=blocks'
linked #2 blocks #1
$ curl -s -X POST --data-binary 'you' 'http://127.0.0.1:1070/iki/ledger/claim?item=3'
#3 claimed by you
$ curl -s -X POST --data-binary 'Pinned.' 'http://127.0.0.1:1070/iki/ledger/close?item=2'
closed #2 (done)
$ curl -s http://127.0.0.1:1070/iki/ledger/next
 1.    #1  open    p-  Write the gonk book
    unprioritized — no priority set, so it ranks below every item that has one; last updated …

policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 1
  not #3: claimed by you
```

What the HTTP door will **not** do is part of the lesson. An anonymous caller on loopback holds
the read and write grants of the ledgers gonk is configured to serve that way — `default`,
unless told otherwise — and nothing else. Deleting is a separate grant:

<!-- transcript: run -->
```console
$ curl -s -X DELETE http://127.0.0.1:1070/iki/ledger/item/1
denied: capability does not grant `urn:cap:ledger:delete:*` (declared by `urn:iki:ledger:item:1`)
$ ikigai-gonk grants default delete
[
  "urn:cap:ledger:read:default",
  "urn:cap:store:read:graph:urn:iki:ledger:graph:default",
  "urn:cap:ledger:write:default",
  "urn:cap:store:write:graph:urn:iki:ledger:graph:default",
  "urn:cap:ledger:delete:default",
  "urn:cap:store:write:graph:urn:iki:ledger:graph:default:deleted"
]
```

`grants` prints the token list for one ledger at one authority, for writing a grant by hand;
the next chapter gives one to a person, with a passkey. Read it as the four-token writer from
the Rust half plus two: the ledger's delete grant, and a write grant for a SECOND graph,
`…:default:deleted`. A delete is a move into that ledger's graveyard, recoverable by hand, so
it is a write to two graphs, and the caller must be able to make both. The refusal names the token it wanted,
because a capability that is declared is a capability that is enforced, and the other way
round.

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open http://localhost:1070/
```

opens the same ledger as a page: the three items, `next`, a form to file another. That page is
the next chapter.

## What you built

| | the page | your kernel | gonk |
| --- | --- | --- | --- |
| store | in memory, per page load | in memory, per process | RocksDB, in the data home (`~/.ikigai/store` by default) |
| clock | the story clock | the story clock, or yours | the system's |
| caller | root | root, or a grant | per door: loopback HTTP holds `default`'s read and write |
| reached by | Run | `ask` | HTTP, the socket, QUIC |

In resource terms: the **sets** are the ledgers and each one's items; the only **atoms** that
hold state are one named graph per ledger, which the store holds; every action is a
**composition** over that graph; `items`, `item:{id}` and `next` are **views**. None of that is
code in `ledger-host`, which is a composition of two published spaces and a clock.
