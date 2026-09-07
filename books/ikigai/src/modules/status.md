# Where this actually stands

This chapter exists because a tutorial that leaves you to discover the maturity of a
feature by hitting its edges has wasted your afternoon.

## Phase 1

`ikigai-module`'s own first line calls it **"the dynamically-loadable module format
(Phase 1: in-process proof)"**. That line undersells the crate a little — it has grown a
socket transport since somebody wrote it — which is worth knowing as a habit: a crate's
one-line self-description is the thing least likely to be updated when the crate changes.

What is actually there:

| | |
|---|---|
| the callback machinery | **proven** — `InProcessTransport`, exercised end to end |
| the wire session | **proven through the codec** — `LoopbackTransport` encodes and decodes every message |
| an out-of-process transport | **built, Unix only** — `UdsTransport` + `serve`: the module runs in its own process behind a `0600` socket, and `serve` refuses peers belonging to another user |
| an embedded wasm runtime | **not built** — a host embedding wasmtime is Phase 2 |
| isolation | **not there** — a process boundary is not a resource budget |
| hosts that load modules | **one**, the browser demo |

## Read that table the right way

The order is deliberate and it is not the order most projects build in. The *semantics*
came first — re-entrancy, capability inheritance, the session shape — and every transport
since has been that same session with a different pipe under it: a direct call, then the
codec over in-memory channels, then a socket.

That is the harder half done first. Marshalling bytes over a socket is well-understood
work; a host and a module calling into each other mid-invocation without deadlocking, with
authority attenuating correctly across the boundary, is where the design risk lives.

## What you should not assume

**Do not assume a module is a security boundary.** The Unix-socket transport buys you a
*process* boundary — a separate address space, and a socket that refuses peers belonging to
another user — and that is worth something. It is not a sandbox: a module reached that way
is a native binary with all the ambient filesystem and network access its own process has,
the callback is a deliberate hole in whatever isolation you would put around it, and the
transport this book demonstrates (`InProcessTransport`) runs the module inside your own
process anyway. A module today is a *packaging* and *lazy-loading* mechanism.

**Do not assume you can ship a module to a running host.** Nothing loads one at runtime
outside the browser demo.

**Do not assume the ABI is stable.** Phase 2 exists precisely to change how these messages
travel.

## What is genuinely usable now

- The dual-mode pattern, as a way to keep a heavy dependency out of hosts that do not need
  it ([Dual-mode crates](dual-mode.md)).
- The browser case, which is real and running.
- The `InProcessTransport`, as a way to develop and test a module's *semantics* long before
  its transport exists — which is what this book's demo does.

## Where isolation would come from (direction, not shipped)

Capabilities constrain **authority**: which resources a resolution may reach, narrowing as
they pass down a call chain. That is what exists today, it is enforced, and it is most of
what an agent-facing system needs. What a capability says nothing about is **resource
consumption** — how much memory an endpoint allocates, how long it runs, which syscalls it
makes. A module that resolves nothing it was not granted can still allocate until the host
dies.

The direction is containment *beside* the authority model rather than inside it: run the
module as WebAssembly under an embedded runtime, so a host can hand it a memory limit, an
execution budget and a closed syscall surface as well as a capability. Part of that already
exists for an unrelated reason — the browser demo runs modules as wasm, `ikigai-xslt-module`
builds one, and a wasm module has no ambient filesystem or network to begin with. What does
*not* exist is any of the native half: no host in the ecosystem embeds wasmtime, and no
crate sets an execution or memory budget — the word "fuel" appears in none of them. Treat
this section as the direction and the table above as the state, and do not plan a deployment
on the difference.

## Exercises

1. **Break the prefix.** Change `ModuleSpace::new(["urn:greet:"], …)` to a prefix the
   request does not match and watch it become `Unresolved`. Routing is bounded on purpose.
2. **Chain a callback.** Point `name` at another module-backed name rather than a host one,
   and satisfy yourself the module never learns the difference.
3. **Attenuate.** Resolve the demo under a scoped capability instead of `Capability::root()`
   and confirm the module inherits the narrowing rather than escaping it.
4. **Take the loopback.** Swap `InProcessTransport` for `LoopbackTransport` and watch the
   same test pass with every message encoded. If it fails, you have found an encoding bug
   rather than a semantics bug — which is exactly what that split is for.
