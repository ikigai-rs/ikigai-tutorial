# The Python track

<!-- urn-gate: illustration urn:py:hello — served by examples/python/hello.py, mounted at
     the reader's own `--mount urn:py:=`; not a name the CLI or this workspace binds. -->
<!-- urn-gate: illustration urn:py:shout — likewise. -->

Part I, chapter for chapter, from a language with no kernel in it. Every listing is in
`examples/python/`. The blocks that start a peer and mount it
([Binding](#binding-a-mount-is-a-binding) and the family below) are replayed against the
book's pinned `ikigai-cli` on every change to this repository, with ikigai-python at its
`main`, by `scripts/polyglot-tracks.sh`: the peers are started from these very lines, and what
each `ikigai` command prints, cache verdicts included, is compared with the page. The client
listings and the `describe` card were run against a served `ikigai-cli` 0.1.18 as they were
written and are not replayed (the refusal text shown in the capabilities section is a later
kernel's: see the note there).

If the `ttt-host` from [Tic-tac-toe, VI](../applied/tic-tac-toe-6.md) or its apps are still
running, they can stay: this kernel is on a socket of its own.

<!-- transcript: serve -->
```bash
pip install rdflib /path/to/ikigai-python     # a checkout; not on PyPI yet
ikigai serve /tmp/ikbook-kernel.sock &         # a kernel to talk to
until [ -S /tmp/ikbook-kernel.sock ]; do sleep 0.2; done   # wait for its socket
```

The `pip install` brings ikigai-python's one dependency with it: `blake3`, for the digests in
the contract IRIs a served door publishes.

## Resolution: a name, over the wire

```python
{{#include ../../../../examples/python/client.py:resolve}}
```

`connect` speaks the wire protocol — a versioned hello each way, then framed requests —
and `source` is the verb. The answer is a **representation**: text, a media type, and how
the kernel's cache answered. Nothing about it says the endpoint was Rust, or in another
process, which is [Resolution](../getting-started/resolution.md)'s point made from the
other side.

## Hello, resource: a decorated function

```python
{{#include ../../../../examples/python/hello.py:endpoint}}
```

```python
{{#include ../../../../examples/python/hello.py:serve}}
```

Compare [Hello, resource](../getting-started/hello-resource.md): a function, a
description, a binding. Here the decorator is the description and the socket path is the
binding — this process *is* a space, and `serve` speaks the same protocol a Rust host
speaks to any other peer.

## Why an endpoint describes itself: the signature is the contract

In Rust you wrote the `ArgSpec`s by hand. Here they are **derived from the signature**:
`who: str` is a required input of class `xsd:string`, the `Annotated` text is its
summary, a default would make it optional, `Literal[...]` would be `one_of`. The
description a Rust host sees is exactly the one it would see from a Rust endpoint (the peer it
asks is started in [Binding](#binding-a-mount-is-a-binding), below; start it first to run this):

```text
$ ikigai --plain --mount urn:py:=/tmp/py-hello.sock -c 'describe urn:py:hello text/plain'
hello —
Greet someone
verbs: Source, Meta
  input who [argument]: the name to greet
outputs: text/plain;charset=utf-8

[computed]
```

And from the client side, the same card as data — the JSON face of `Meta`, parsed:

```python
{{#include ../../../../examples/python/client.py:describe}}
```

## Binding: a mount is a binding

Start the peer, and wait for its socket before you mount it (the `rm -f` clears a socket an
earlier run left behind, which would end the wait before this peer is listening):

<!-- demo: start -->
```bash
rm -f /tmp/py-hello.sock
python3 examples/python/hello.py /tmp/py-hello.sock &
until [ -S /tmp/py-hello.sock ]; do sleep 0.2; done
```

<!-- demo: run -->
```text
$ ikigai --plain --mount urn:py:=/tmp/py-hello.sock -c 'source urn:py:hello who=Ada'
Hello, Ada!
[uncacheable]
```

The Python process bound nothing under `urn:py:`; the **host** did, with `--mount`,
exactly as [Binding, and a host of your own](../getting-started/binding.md) said a host
decides the name. `list` shows where the name is served from, after every name the host
binds itself and before its names layer, which `ikigai-cli` 0.2.6 binds last:

<!-- demo: run -->
```text
$ ikigai --plain --mount urn:py:=/tmp/py-hello.sock -c list
…
urn:py:hello                        → hello   [/tmp/py-hello.sock]
urn:py:shout                        → shout   [/tmp/py-hello.sock]
urn:iki:host:names                  → host-names
urn:iki:space:dynamic:names:status  → dynamic-space-status
```

`hello` declares `cacheable=True`, and the answer above still says `[uncacheable]`. Read it
twice and ask the host why:

<!-- demo: run -->
```text
$ ikigai --plain --mount urn:py:=/tmp/py-hello.sock -c 'source urn:py:hello who=Ada' -c 'source urn:py:hello who=Ada' -c 'source urn:kernel:uncached'
Hello, Ada!
[uncacheable]
Hello, Ada!
[uncacheable]
uncached (why a computed read was not cached; the last 64, most recent first)
  urn:py:hello      ×2  declared  [root]

[uncacheable]
— batch: 3 commands · 3 uncacheable
```

`ikigai-cli` 0.2.6, the version this page is replayed against, caches nothing a mounted peer
answers: both reads reached the Python process, and `urn:kernel:uncached` gives `declared` as
the reason. That is the mount's rule, on purpose: a kernel that mounts a peer stores no copy of
what the peer answers, because a change on the peer's side (a file it watches, a write
that reaches it by another route) never crosses back to the mounting kernel, so a stored
copy could be served stale for as long as the host runs. The `declared` reason is the
mount's own declaration, not the peer's. So `cacheable=True` on a served peer describes its answers;
through `--mount`, the host does not keep them.

## What resolution buys you: the kernel's cache and trace, from outside

```python
{{#include ../../../../examples/python/client.py:cached}}
```

```python
{{#include ../../../../examples/python/client.py:traced}}
```

The cache is the **kernel's**: what these two listings probe is a name the served kernel
binds itself, and its answer is cached there, for every client of that kernel. `cacheable=True`
on a decorated function only marks the representation. Served straight from Python nothing is
cached, and mounted through `ikigai-cli` 0.2.6 nothing is either, as
[Binding](#binding-a-mount-is-a-binding) showed. The trace events come back over the wire and are the same `TraceEvent`s a Rust
tracer receives.

## Capabilities: a scoped connect

```python
{{#include ../../../../examples/python/client.py:scoped}}
```

```text
denied: capability does not grant `urn:cap:fs:write:*` (declared by `urn:file:notes.txt`; any grant under `urn:cap:fs:write:` satisfies it)
```

That refusal line is `ikigai-core` 0.1.97's wording (its "any grant under … satisfies it"
clause); the 0.1.18 host these listings were first run against said only that the capability
did not grant the name. The typed refusal is the same either way.

The client carried a narrowed capability; the served kernel clamped it to what the
channel is entitled to and enforced it. The refusal crossed the wire **typed** —
`DeniedError`, not a string to parse — which is what lets a REST face over this client
answer 403 rather than 500 ([Who is asking](../beyond/identity.md), from a client).

## Families: one door, many names, four verbs

<!-- urn-gate: illustration urn:py:note:{name} — the family examples/python/notes.py serves,
     mounted at the reader's own `--mount urn:py:=`; not a name the CLI or this workspace
     binds. -->
<!-- urn-gate: illustration urn:py:note:milk — a note in that family; likewise. -->
<!-- urn-gate: illustration urn:py:note:bread — a note nobody wrote; likewise. -->
<!-- urn-gate: illustration urn:py:note:index — the family's index door; likewise. -->
<!-- urn-gate: illustration urn:note:{name} — the alias-stripped form of the family, what a
     `--mount urn:py:=` host forwards; a placeholder, not a resource. -->
<!-- urn-gate: illustration urn:note:milk — likewise, for one note. -->
<!-- urn-gate: illustration urn:py:note: — the prefix a deeper mount of that family would
     take; a placeholder, not a resource. -->

Every name so far was one resource. Most interesting resources come in families, such as
a note for every name or a cell for every square, and a Rust endpoint binds a family with a
`UriTemplate`. In Python a family is `family(pattern)`, and each verb is a decorated function whose parameters include the template's variables:

```python
{{#include ../../../../examples/python/notes.py:family}}
```

`{name}` is a Level 1 variable, and it matches the way `ikigai_core::UriTemplate` matches,
line for line. A variable followed by a literal captures up to the leftmost occurrence of
that literal, a trailing variable takes the rest, and every capture is non-empty. The host
and the peer have to agree on which names a pattern covers, because the host reads the
peer's catalog to decide what to forward, so the face mirrors core's rule and does not
approximate it. The catalog lists the template, so `list` shows one family, not every note
anyone has written.

Each verb gets its own contract, which is core's per-verb `ActionSpec` form: a Source that
is cacheable and answers `NotFound` for a missing note, a Sink whose body arrives as
`content`, and a Delete. Exists is not written at all. It defaults to "would Source
succeed", so a `NotFound` answers `false`.

<!-- demo: start -->
```bash
rm -f /tmp/py-notes.sock
python3 examples/python/notes.py /tmp/py-notes.sock &
until [ -S /tmp/py-notes.sock ]; do sleep 0.2; done
```

<!-- demo: run -->
```text
$ ikigai --plain --mount urn:py:=/tmp/py-notes.sock -c 'sink urn:py:note:milk two liters' \
  -c 'source urn:py:note:milk' -c 'source urn:py:note:milk' \
  -c 'sink urn:py:note:milk three liters' -c 'source urn:py:note:milk' \
  -c 'source urn:py:note:index' -c 'delete urn:py:note:milk' -c 'source urn:py:note:milk'
wrote milk
[uncacheable]
two liters
[uncacheable]
two liters
[uncacheable]
wrote milk
[uncacheable]
three liters
[uncacheable]
milk
[uncacheable]
forgot milk
[uncacheable]
error: not found: no note called milk
— batch: 8 commands · 7 uncacheable
```

Read the verdicts. The note's Source declares itself cacheable, and, as in
[Binding](#binding-a-mount-is-a-binding), `ikigai-cli` 0.2.6 caches nothing a mounted peer
answers, so every read here reached the peer. The peer keeps no cache of its own either, and
that is why the read after the second write saw the new text. The writes and the delete are
uncacheable whatever the host does, because a Sink or Delete answer always is, and the index
is uncacheable on purpose: it reads every note, and a write is about only the note it wrote.
Exists, from a client (the block above ended by deleting `milk`, so sink it again
first):

```python
k = ikigai.connect("/tmp/py-notes.sock")   # the peer itself; Exists is a read
print(k.exists("urn:py:note:milk").text)    # true, after `sink urn:py:note:milk two liters`
print(k.exists("urn:py:note:bread").text)   # false
```

### Which door answers

Two rules, and the second one is the reason the first can be trusted.

**The first declared door that matches wins**, as in core's `EndpointSpace`. The family
would match `urn:py:note:index` too, with `{name}` capturing `index`. The index works
because it is declared first:

```python
{{#include ../../../../examples/python/notes.py:index}}
```

**Every door has two forms, and every door's declared form is tried before any door's
stripped form.** `--mount urn:py:=` is an *alias* mount: the host rewrites
`urn:py:note:milk` to `urn:note:milk` before forwarding it, the way
[Preferential resolution](../beyond/preference.md) described. A peer cannot know the prefix it was
mounted at, only that it was aliased, so it answers each door at its declared pattern and at
that pattern with the first segment stripped: `urn:py:note:{name}` and `urn:note:{name}`.
If the forms were tried door by door, one door's stripped form could shadow the next door's
declared name. Trying every declared form first means a stripped form can only answer a
name that no door declares. A connection whose hello says it comes from an alias mount is
sending stripped names, so for that connection the order reverses, and stripped forms go
first. Because only the first segment can be stripped, a family mounted deeper,
at `urn:py:note:` for example, needs `--override`, which forwards names unchanged.

A family is how a store in another language holds a game: the sixth part of the tic-tac-toe
arc, [Tic-tac-toe, VI](../applied/tic-tac-toe-6.md), which you have already read, serves the game's stored cell as one,
from Python, under a Rust host that keeps everything else.

## What an L0 peer cannot do

`camel-title` in [What resolution buys you](../getting-started/payoff.md) resolved
`title` from *inside* its own invocation, and inherited its golden thread. A Python
endpoint cannot: there is no `inv.source` on this side of the wire, no back-channel from a
served peer into the kernel that is invoking it. A Python endpoint is a leaf — it takes
arguments and returns bytes — and composition happens in the kernel, above it. That is L0,
stated plainly; the ladder's next rung is the module protocol's host callback, which
[Part II](../modules/the-callback.md) showed and which no polyglot peer speaks yet.
