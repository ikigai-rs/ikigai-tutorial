# The Python track

<!-- urn-gate: illustration urn:py:hello — served by examples/python/hello.py, mounted at
     the reader's own `--mount urn:py:=`; not a name the CLI or this workspace binds. -->
<!-- urn-gate: illustration urn:py:shout — likewise. -->

Part I, chapter for chapter, from a language with no kernel in it. Every listing is in
`examples/python/` and was run against a served `ikigai-cli` 0.1.18 as it was written.

```bash
pip install rdflib /path/to/ikigai-python     # a checkout; not on PyPI yet
ikigai serve /tmp/ikbook-kernel.sock &         # a kernel to talk to
```

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
description a Rust host sees is exactly the one it would see from a Rust endpoint:

```text
$ ikigai --plain --mount urn:py:=/tmp/py-hello.sock -c 'describe urn:py:hello text/plain'
hello —
Greet someone
verbs: Source, Meta
  input who [argument]: the name to greet
outputs: text/plain;charset=utf-8
```

And from the client side, the same card as data — the JSON face of `Meta`, parsed:

```python
{{#include ../../../../examples/python/client.py:describe}}
```

## Binding: a mount is a binding

```bash
python3 examples/python/hello.py /tmp/py-hello.sock &
ikigai --plain --mount urn:py:=/tmp/py-hello.sock -c 'source urn:py:hello who=Ada'
```

```text
Hello, Ada!
[computed]
```

The Python process bound nothing under `urn:py:`; the **host** did, with `--mount`,
exactly as [Binding, and a host of your own](../getting-started/binding.md) said a host
decides the name. `list` shows where the name is served from:

```text
$ ikigai --plain --mount urn:py:=/tmp/py-hello.sock -c list
urn:py:hello                        → hello   [/tmp/py-hello.sock]
urn:py:shout                        → shout   [/tmp/py-hello.sock]
```

## What resolution buys you: the kernel's cache and trace, from outside

```python
{{#include ../../../../examples/python/client.py:cached}}
```

```python
{{#include ../../../../examples/python/client.py:traced}}
```

The cache is the **kernel's**. `cacheable=True` on the decorated function only marks the
representation; served straight from Python nothing is cached, and mounted through a Rust
kernel the same function's answers are — the cache verdict flips to `HIT` on the second
call. The trace events come back over the wire and are the same `TraceEvent`s a Rust
tracer receives.

## Capabilities: a scoped connect

```python
{{#include ../../../../examples/python/client.py:scoped}}
```

```text
denied: capability does not grant `urn:cap:fs:write:*` (declared by `urn:file:notes.txt`)
```

The client carried a narrowed capability; the served kernel clamped it to what the
channel is entitled to and enforced it. The refusal crossed the wire **typed** —
`DeniedError`, not a string to parse — which is what lets a REST face over this client
answer 403 rather than 500 ([Who is asking](../beyond/identity.md), from a client).

## What an L0 peer cannot do

`camel-title` in [What resolution buys you](../getting-started/payoff.md) resolved
`title` from *inside* its own invocation, and inherited its golden thread. A Python
endpoint cannot: there is no `inv.source` on this side of the wire, no back-channel from a
served peer into the kernel that is invoking it. A Python endpoint is a leaf — it takes
arguments and returns bytes — and composition happens in the kernel, above it. That is L0,
stated plainly; the ladder's next rung is the module protocol's host callback, which
[Part II](../modules/the-callback.md) showed and which no polyglot peer speaks yet.
