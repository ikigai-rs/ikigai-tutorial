# A kernel behind a socket

The last chapter of Part II ended on a sentence this part has to answer. A real module is
on the other side of *something* — another WebAssembly instance, an embedded wasmtime, a
socket — and then the conversation has to be encoded.

Take the socket. It is the smallest thing that is genuinely on the other side: another
process, its own address space, its own memory, gone when it crashes. Everything in this
part is downstream of that one step, because once a resolution can leave the process, four
questions arrive at once — who is asking, what may they do, what happens when the far side
is not there, and who benefits. This chapter is the first step and the easiest of the
four. The rest of the part is the other three.

## Serving

A server is a kernel and a path:

```rust,ignore
{{#include ../../../../crates/two-hosts/src/socket.rs:serve}}
```

That is the whole server. `ikigai_ipc::serve` binds the socket, replaces a stale one left
by a previous run, and hands each connection to a thread that reads framed messages and
answers them against the kernel.

## Connecting

```rust,ignore
{{#include ../../../../crates/two-hosts/src/socket.rs:connect}}
```

And this is the part worth stopping on. `connect` returns something that implements the
**same `Resolver` trait an embedded kernel implements**. Not a similar one, not a client
API that mirrors it — the same trait, with the same `issue`.

That is why there is no "remote resolution" concept anywhere in this book. A caller that
holds a `Resolver` cannot tell whether the answer came from a function call in this
process or a round trip to another one, and neither can the engine, which is what makes
[Preferential resolution](preference.md) possible at all: a thing that answers requests is
a thing a local kernel can compose.

The book's test says it flatly. Serve `hello_camel::space()` — the space Part I built —
on a socket, connect to it, and resolve `urn:iki:tutorial:camel-case`. The answer is
`resourceOrientedComputing`, the same string the in-process test gets, and the two calls
differ only in which constructor built the resolver.

## Failure arrives typed

The second test is the one that matters more than it looks. Ask the served kernel for a
name it does not bind, and the error that comes back is `Unresolved` — the server's own
typed error, not a generic "the call failed".

Keep that in view for two chapters. A client that can tell *"your kernel has no such
resource"* from *"your kernel is not there"* from *"your kernel says you may not"* can
make a policy decision about each. A client that gets one opaque failure for all three
cannot, and every degradation rule in this part is built on the distinction.

## The transport does not authenticate, and that is not an oversight

There is no credential in either listing. `ikigai-ipc` puts the socket in a `0700`
directory, makes it `0600`, and then — belt and braces — refuses any connection whose
kernel-verified UID is not the server's own.

That is the whole authentication story, and it is complete: **on one machine, between one
user's processes, the operating system already knows who is asking.** Adding a certificate
here would re-implement something the kernel underneath already does better. The next
chapter is what happens when that sentence stops being true.

## A second boundary, underneath the capability model

The one thing in the ecosystem that exists purely to be a server on a socket is gonk
(`ikigai-gonk`), the work-ledger and repository-browser host: a durable RDF store, the work
ledger, repositories browsed as resources and SPARQL over all of it, behind a socket, QUIC and
HTTP. It took that job over from `ikigai-dev-server`, which this chapter used to show here and
which is now retired. [The gonk Book](../gonk/index.html) teaches it door by door; from this
chapter's side it is one more socket:

```bash
ikigai-gonk                      # its socket: ~/.ikigai/gonk.sock
ikigai --connect ~/.ikigai/gonk.sock
ikigai --connect ~/.ikigai/gonk.sock -c 'source urn:kernel:catalog'
```

Its interesting property is not in any of its code. It is in its `Cargo.toml`, which says
in its first comment that it is the module manifest: **the binary links only what it
serves.** There is no calendar in it, no contacts, no EventKit, not even an HTTP client —
not disabled, not hidden behind a flag, *absent*. A flaw in code that was never compiled in
cannot be reached from this process whatever any capability says. The catalog that last
line reads is the proof you can run: the store, the ledger, compression, backup and render
rules, the kernel's own resources, and (once repositories are configured) the browse and
repository families. Nothing else — and gonk's tests pin that list id by id, so a
dependency bump that widens it fails the build.

That is a second boundary, and it sits *underneath* the capability model rather than
inside it. The contrast is with the omnibus `ikigai` binary run as
`ikigai mcp --grant dev`, which **config-gates**: it hides the calendar tools from the tool
list, and the calendar code is still linked, still in the address space, still one bug away
from being reachable. Linkage gating removes the option.

Two honest qualifications, because a boundary you overestimate is worse than one you do
not have:

- **It is coarse.** The unit is a dependency, not a resource and certainly not a caller. It
  can say "this binary has no calendar in it"; it cannot say "this client may read
  free/busy and nothing else". That is what capabilities are for, and neither replaces the
  other.
- **It is a judgment, not a rule.** The dev server gonk replaced *did* link `ikigai-llm`,
  deliberately, and its manifest argued the case: an outbound HTTP client to local inference
  is not the platform-authority class the gating exists to exclude. gonk made the other
  call. The model it explains with is *mounted* — one `gonk.mount` line, one prefix,
  `urn:llm:` — so the only outbound connection it can make is to one configured peer, over
  ikigai's own transports, at an address no caller can choose. Two defensible answers to
  the same question; what matters is that each manifest argues its own.

## What a broken connection is allowed to do

One last thing that will matter in two chapters. A long-lived client — a daemon holding a
standing mount, an interactive `--connect` session — outlives server restarts, so
`IpcResolver` **heals on use**: a connection that broke is dropped and redialed, version
handshake and all, rather than failing every subsequent call forever.

Healing is not the same as retrying, and the crate is careful about the difference. If the
*write* failed, the request was never fully delivered — the server reads whole frames and
cannot dispatch half of one — so nothing executed and replaying anything is safe. If the
*read* failed, the request was delivered and may well have run, with only the answer lost.
In that case only read-only calls are replayed; a `Sink` or a `Delete` surfaces its
transient error and lets the caller decide.

Notice the shape of that rule, because it is about to recur one level up with a different
answer: **what may be re-issued is not a property of the failure, it is a property of the
verb.**
