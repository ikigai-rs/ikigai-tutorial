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

**Do not assume the host enforces what a module declares.** An endpoint reached through
`ModuleSpace` can declare `requires("urn:cap:…")` and be invoked by a caller who does not
hold it — try it, it takes one line. Authority still *attenuates* correctly across the
boundary (the module's callbacks run under the caller's capability, narrowed, and are
refused when it is too narrow — exercise 3 below), but the module's own declaration is not
a gate today. For a linked-in endpoint it is.

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

Four, and all four run against `crates/loadable-module/src/lib.rs` — the file the listings
in this part are pulled from. The loop is `cargo test -p loadable-module`; the test module
at the bottom of that file already has the `greet` helper that builds a kernel and resolves
a name, so a new question is a copy of it with something changed.

### 1. Break the prefix

Change `ModuleSpace::new(["urn:greet:"], …)` in `host_space()` to a prefix the request does
not match.

- **Right when** — `the_module_resolves_a_host_resource_mid_invocation` fails with
  `no endpoint resolved for urn:greet:hello`, and
  `a_name_outside_the_module_prefix_does_not_reach_it` still passes. Then put it back.

<details>
<summary>Hint</summary>

Notice what did *not* change when it broke: the module is still compiled in, still
constructed, still perfectly able to answer that name. It has an endpoint bound at
`urn:greet:hello` in its own space, and the host cannot reach it.

That is routing being a host decision rather than a module's claim on a name space —
the same separation as [Binding, and a host of your own](../getting-started/binding.md),
one level up. A module does not get names by asking for them.

</details>

### 2. Chain a callback

Bind a second endpoint in `module_space()` — one that takes no arguments and returns a
constant — and then resolve `urn:greet:hello` with `name` pointing at *that* name instead
of `urn:host:name`.

- **Right when** — the greeting names whatever your second module endpoint returns, and
  `hello` itself is untouched.

<details>
<summary>Hint</summary>

Follow the path before you write it: the host routes `urn:greet:hello` to the module, the
module calls back to the host for a name, and the host routes that straight back into the
module — while the first invocation is still open. Re-entrancy into the same module,
mid-invocation.

It works, and it works on the loopback transport too. The reason it works is the same
reason exercise 1 broke: the module has an IRI and an `Issuer`, and no way to ask where a
name will end up. It cannot tell that the answer came back from itself.

The second endpoint is the shorter half of the exercise: `host_name()` in the same file
already shows the shape of an endpoint that takes nothing and returns a constant.

</details>

### 3. Attenuate

Give the host's `urn:host:name` a declared capability — `.requires("urn:cap:demo:host")` on
its `Description` — then resolve `urn:greet:hello` under
`Capability::root().attenuate(["urn:cap:demo:host"])`, and again under an attenuation that
grants something else.

- **Right when** — with the right scope you get `Hello, Peter!`; with the wrong one the
  resolution fails with `denied: capability does not grant urn:cap:demo:host (declared by
  urn:host:name)`.

<details>
<summary>Hint</summary>

The interesting part is *where* that denial happens. It is not the module being refused at
the door — the module ran. It is the module's **callback** being refused, mid-invocation,
because the capability the invocation is carrying is the caller's, narrowed, and it does
not grant what the host resource demands.

That is the property this part has been claiming: a module does not get authority by being
a module, it borrows the caller's, and the borrowing can only narrow. Here it is failing
closed in front of you.

> ⚠ Try the mirror image too, because the answer is not the one you would guess: put
> `.requires(…)` on the module's own `hello` endpoint instead, and it is **not** enforced.
> See "Do not assume the host enforces what a module declares", above.

</details>

### 4. Take the loopback

Swap `InProcessTransport::new(module_space())` for `LoopbackTransport::new(module_space())`
in `host_space()`.

- **Right when** — every test passes unchanged, including the chained callback from
  exercise 2.

<details>
<summary>Hint</summary>

Nothing about your code changes, which is the observation. What changes underneath is that
every message in the session — the invocation, each `HostCall`, each `HostResult`, the
final representation — is now genuinely encoded and decoded, in one process, over an
in-memory channel.

So the two transports partition the failure modes. If a test passes in-process and fails on
the loopback, the bug is in the *encoding*: something in the request, the capability or the
representation does not survive a round trip. If it fails on both, it is the semantics.
That split is worth stealing for anything else you build with a wire format — it is much
cheaper than debugging a socket.

</details>
