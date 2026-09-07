# Preferential resolution

A connected peer is a `Resolver`, and a `Resolver` is a thing a local kernel can compose.
So the interesting question was never *how do I call another machine* — that was the last
two chapters, and the answer is a function call. It is: **what do that machine's answers
mean here?**

There are exactly three answers, they are three different intentions rather than three
settings, and the difference between them is entirely about what happens when the peer is
not there.

## Alias — a local name for somebody else's namespace

```rust,ignore
{{#include ../../../../crates/two-hosts/src/mounts.rs:alias}}
```

<!-- urn-gate: illustration urn:rest — the tail of a name in the rewrite rule, not a resource: `<prefix>rest` and `urn:rest` are the two halves of one substitution. -->

An alias mount is composed **after** every local space, so it only ever catches what this
kernel lacks. `<prefix>rest` is rewritten to `urn:rest` on the way out, so the peer — which
serves `urn:*` like everyone else — recognizes the name, and its catalog comes back
re-prefixed and tagged with where it came from, so a federated listing shows which machine
each resource lives on.

The failure mode is worth naming because it is silent: point an alias at a prefix the
local kernel already serves and the local binding wins every time, so the mount is never
used and nothing says so. The host warns about exactly this at startup, and the fix is
either an alias prefix the kernel does not serve or one of the next two kinds.

## Override — the same namespace, served elsewhere

```rust,ignore
{{#include ../../../../crates/two-hosts/src/mounts.rs:overriding}}
```

The IRI is forwarded unchanged and the mount is composed **before** the local spaces, so
nothing at the call site changes: `urn:llm:ask` is still `urn:llm:ask`, and it now happens
somewhere else even though this kernel binds it too. Precedence is not an implementation
detail here — it *is* the semantics. A mount composed after a local binding of the same
name is not an override of anything.

And if the peer is down, the resolution fails. That is not a missing feature. **You asked
for that machine.**

## Prefer — an override that degrades

```rust,ignore
{{#include ../../../../crates/two-hosts/src/mounts.rs:prefer}}
```

The peer when it answers, this machine when it does not. It is built out of the parts
already in play: an override, paired with the local space under a `Failover`, and the pair
confined to the mount's prefix.

That confinement is the least obvious line in the file and the one worth reading twice.
`Failover` resolves *every* target, so an unguarded `[peer, local]` pair would answer for
every IRI the local spaces bind — including names this mount never claimed, hitting before
a more specific mount behind it and silently defeating it. The guard is small:

```rust,ignore
{{#include ../../../../crates/two-hosts/src/mounts.rs:prefix_guard}}
```

Its `entries` deserve a glance too: the catalog comes from the peer alone, so listing a
prefer-mount shows what is mounted there rather than re-listing the whole local kernel
under the peer's name.

Composition then sorts by prefix **length**, so the most specific mount wins regardless of
the order the flags were written in:

```rust,ignore
{{#include ../../../../crates/two-hosts/src/mounts.rs:compose}}
```

A whole IRI is simply the most specific prefix there is, which is what makes overriding a
*single resource* work: send `urn:llm:ask` to one peer and the rest of `urn:llm:*` to
another, and the sort puts them in the right order for you.

## What makes a degrading mount honest

A mount that falls back is a mount that can answer a different question than the one you
asked, and the three rules below are what keep that from happening. They are the heart of
this chapter, they are each a test in this book's own crate, and the source states the
principle in one line:

> "graceful" never means "silently ignored the answer the peer actually gave".

### Only a transient failure falls through

`a_prefer_mount_falls_back_to_the_local_binding_when_the_peer_is_down` — the peer is tried
first, it is unreachable, and this machine answers.

"Transient" is a typed property, not a guess about the message: exactly `Timeout` and
`Unavailable`. Everything else — a bad argument, an endpoint error, a resource that is not
there — is permanent, and re-issuing it somewhere else would only produce a second wrong
answer. Notice that this is the payoff for the chapter about the socket: the taxonomy
survives the wire, so the fallback rule can be about *what kind of failure it was* rather
than about the fact that something failed.

### A denial still propagates

`a_prefer_mount_does_not_swallow_a_denial` — the peer answered `Denied`, and the local
binding does **not** quietly answer instead.

This is the one that would be easy to get wrong and hard to notice. A denial is a real
answer, delivered by an authority that considered the request; treating it as a failure to
route around turns a capability boundary into a suggestion. Every argument in this book
about authority attenuating correctly would be worth nothing if any client could get the
answer anyway by having a local copy.

### A mutating verb is never replayed

`a_prefer_mount_never_replays_a_sink` — the peer is transiently down, the verb is `Sink`,
and the error surfaces rather than the write landing locally. The peer may have applied it
before the connection broke, and writing twice is not writing once.

But `Delete` **does** fall through — `a_prefer_mount_does_replay_a_delete_because_deleting_twice_is_deleting_once`,
which is a long name for a real distinction. The line is drawn at *idempotence*, not at
mutation: `Source`, `Exists`, `Meta` and `Delete` may be re-issued to the next target, and
`Sink` may not.

Worth comparing with the same question one layer down, where the answer is different. The
IPC client redialing a *broken connection to the same kernel* replays only read-only calls
and lets a `Delete` surface its error. Two different questions: "may I ask a **different**
machine this?" versus "may I re-ask the machine that might already have done it?" Two
defensible lines. When you build something like this, the useful discipline is not picking
the right rule once — it is knowing which of the two questions you are answering.

## What it looks like on a real machine

None of this is hypothetical; it is how the ecosystem's own topology is written. Mounts are
usually not flags at all but lines in a host's config, so every kernel-building mode on
that machine — the REPL, one-shot commands, the daemon, the MCP server — composes the same
topology:

```toml
mount = "prefer urn:repo:=/Users/brian/.ikigai/dev.sock"
```

That line points at the development server from
[A kernel behind a socket](socket.md), which owns an archive held under an exclusive lock —
exactly one process may have it open, so every other process on the machine reaches it
through the socket. `prefer` is what keeps the machine working when that server is stopped
for an upgrade, and it is also how a topology can name a peer that is normally *not*
running: one started on demand is absent most of the time, and a prefer-mount is not
broken while it is.

Across machines the target is an address or a discovered name rather than a path —
`--prefer urn:llm:=peer:plasma` puts a heavier machine's inference behind the same resource
names, and falls back to this one when it is asleep.

One trap that config lines walk into, and it is the string matching again: if a family also
**mints** under its bare name — a `Sink` that creates a new resource and hands back its IRI
— then a prefix written with the trailing colon covers every read and covers no write at
all. Reads keep working, so nothing looks wrong; new resources are silently created on the
local kernel instead of the one holding the archive. Write the prefix that covers both.

## Putting it together

All three chapters in one wiring: dial the peer, decide what its answers mean, compose it
in front of what this machine serves itself.

```rust,no_run
# extern crate two_hosts;
# extern crate ikigai_core;
# extern crate ikigai_resolve;
# extern crate hello_camel;
use std::path::Path;
use std::sync::Arc;

use ikigai_core::{Kernel, Space};
use ikigai_resolve::Resolver;
use two_hosts::mounts::{compose, prefer};
use two_hosts::socket::connect;

// A peer is a Resolver — this one over a Unix socket, but the type is what matters.
let peer: Arc<dyn Resolver> = Arc::new(connect(Path::new("/tmp/dev.sock"))?);

// Everything this machine serves itself.
let local: Arc<dyn Space> = Arc::new(hello_camel::space());

// `urn:repo:` there when it answers, here when it does not.
let root = compose(
    vec![(
        "urn:repo:".to_string(),
        prefer("urn:repo:", "/tmp/dev.sock", peer, Arc::clone(&local)),
    )],
    local,
);

let kernel = Kernel::new(Arc::new(root));
# let _ = kernel;
# Ok::<(), std::io::Error>(())
```

`no_run`, because it dials a socket nothing has bound — but it *compiles*, every time this
book is built, which is the part that catches an API moving underneath the prose. The
listings above it are `ignore`d for a different reason and lose nothing by it: each one is
included from `crates/two-hosts`, which the same CI job compiles and runs the tests of.

## Two cautions

**Write the canonical name.** A kernel may carry an alias table that rewrites names before
resolution, and mounts are composed *inside* it — so a mount matches the canonical spelling,
not the one you type. A mount prefix written in a spelling the alias table rewrites stops
matching the moment that table ships, and no alias can save it.

**A prefer-mount is not replication.** The two sides are different kernels with different
state; falling back means getting *this machine's* answer, which may legitimately differ
from the peer's. That is fine when the resources are equivalent — a function library, a
model behind a facade — and wrong when they are not. Prefer is for resources you would be
equally happy to get from either side. When you would not be, you wanted an override, and
you wanted it to fail.
