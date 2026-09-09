# Where this actually stands

This chapter exists because a tutorial that leaves you to discover the maturity of a
feature by hitting its edges has wasted your afternoon.

Every claim below about a repository other than this one carries **the version it was
true at**, because those repositories move on their own schedules and a sentence about
them is a claim, not a fact. This book builds against `ikigai-module` **0.2.0** and its
shell examples were probed against `ikigai-cli` **0.1.18**, both on 2026-09-08; a test
(`crates/book-urns/tests/book_claims.rs`) can re-check each stamped claim against the
published crate it names, so that when one of them stops being true the fix is a diff,
not a discovery.

## Phase 1

`ikigai-module` 0.2.0's own first line still calls it **"the dynamically-loadable module
format (Phase 1: in-process proof)"**. That line undersells the crate — it has grown a
socket transport and, at 0.2.0, capability enforcement across the boundary since somebody
wrote it — which is worth knowing as a habit: a crate's one-line self-description is the
thing least likely to be updated when the crate changes.

What is actually there, at `ikigai-module` 0.2.0:

| piece | state |
|---|---|
| the callback machinery | **proven** — `InProcessTransport`, exercised end to end |
| the wire session | **proven through the codec** — `LoopbackTransport` encodes and decodes every message |
| an out-of-process transport | **built, Unix only** — `UdsTransport` + `serve`: the module runs in its own process behind a `0600` socket, and `serve` refuses peers belonging to another user |
| a module's declared capability | **enforced** since 0.2.0 — `ModuleFloor` on the mount, and an endpoint's own `requires` honored across the boundary (the crate's test `a_module_endpoints_declared_requirement_is_enforced_like_a_linked_ones`) |
| an embedded wasm runtime | **not built** — no `wasmtime` in its manifest; a host embedding one is Phase 2 |
| isolation | **not there** — a process boundary is not a resource budget |
| hosts that load modules | **one**, the browser demo (`ikigai-web-demo`, `WasmModuleSpace`, as of 2026-09-08) |

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

**Do assume the host enforces what a module declares — since `ikigai-module` 0.2.0, and
not before.** At 0.1.10 an endpoint reached through `ModuleSpace` could declare
`requires("urn:cap:…")` and be invoked by a caller who did not hold it; this book said so,
in this paragraph. 0.2.0 closed that: `ModuleSpace::new` takes a third argument, the
mount's `ModuleFloor` (every request through the mount must satisfy it), and an endpoint's
own declaration is enforced across the boundary exactly as it is for a linked-in one.
Authority *attenuates* correctly across the boundary as it always did (the module's
callbacks run under the caller's capability, narrowed — exercise 3 below), and now the
module's own card is a gate too. The test that keeps this sentence true:

```rust,ignore
{{#include ../../../../crates/loadable-module/src/lib.rs:enforced}}
```

**Do not assume you can ship a module to a running host.** Nothing loads one at runtime
outside the browser demo (as of 2026-09-08).

**Do not assume the ABI is stable.** Phase 2 exists precisely to change how these messages
travel — and 0.2.0 already changed `ModuleSpace::new`'s signature once.

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
crate sets an execution or memory budget — the word "fuel" appears in none of them
(checked across the organization's public repositories on 2026-09-08; the probe re-checks
`ikigai-module` 0.2.0 and `ikigai-cli` 0.1.18 by manifest). Treat this section as the
direction and the table above as the state, and do not plan a deployment on the
difference.

## Exercises

Four, and all four run against **your** module — `crates/your-endpoints/src/module.rs`,
the same shape as the `loadable-module` crate this part quotes, with a `greeting` endpoint
under `urn:iki:tutorial:yours:module:` that calls back to the host's
`urn:iki:tutorial:yours:name`. The loop is `cargo test -p your-endpoints`; the test module
at the bottom of that file already has the `greet` helper that builds a kernel and resolves
a name under a capability, so a new question is a copy of it with something changed.

### 1. Break the prefix

Change `ModuleSpace::new([MODULE_PREFIX], …)` in `host_space()` to a prefix the request
does not match.

- **Right when** — `the_module_resolves_a_host_resource_mid_invocation` fails with
  `no endpoint resolved for urn:iki:tutorial:yours:module:greeting`, and
  `a_name_outside_the_module_prefix_does_not_reach_it` still passes. Then put it back.

<details>
<summary>Hint</summary>

Notice what did *not* change when it broke: the module is still compiled in, still
constructed, still perfectly able to answer that name. It has an endpoint bound at
`urn:iki:tutorial:yours:module:greeting` in its own space, and the host cannot reach it.

That is routing being a host decision rather than a module's claim on a name space —
the same separation as [Binding, and a host of your own](../getting-started/binding.md),
one level up. A module does not get names by asking for them.

</details>

### 2. Chain a callback

Bind a second endpoint in `module_space()` — one that takes no arguments and returns a
constant — and then resolve `urn:iki:tutorial:yours:module:greeting` with `name` pointing
at *that* name instead of `urn:iki:tutorial:yours:name`.

- **Right when** — the greeting names whatever your second module endpoint returns, and
  `greeting` itself is untouched.

<details>
<summary>Hint</summary>

Follow the path before you write it: the host routes `urn:greet:hello` to the module, the
module calls back to the host for a name, and the host routes that straight back into the
module — while the first invocation is still open. Re-entrancy into the same module,
mid-invocation.

It works, and it works on the loopback transport too. The reason it works is the same
reason exercise 1 broke: the module has an IRI and an `Issuer`, and no way to ask where a
name will end up. It cannot tell that the answer came back from itself.

The second endpoint is the shorter half of the exercise: `name()` in the same file
already shows the shape of an endpoint that takes nothing and returns a constant — and
remember the prefix: a name the host does not route to the module never reaches it, which
is exercise 1 arriving from the other side.

</details>

### 3. Attenuate

Give the host's `urn:iki:tutorial:yours:name` a declared capability —
`.requires("urn:cap:yours:name")` on its `Description` — then resolve the greeting under
`Capability::root().attenuate(["urn:cap:yours:name"])`, and again under an attenuation
that grants something else. The `greet` helper already takes the capability.

- **Right when** — with the right scope you get `Hello, Ada!`; with the wrong one the
  resolution fails with ``denied: capability does not grant `urn:cap:yours:name` (declared
  by `urn:iki:tutorial:yours:name`)``.

<details>
<summary>Hint</summary>

The interesting part is *where* that denial happens. It is not the module being refused at
the door — the module ran. It is the module's **callback** being refused, mid-invocation,
because the capability the invocation is carrying is the caller's, narrowed, and it does
not grant what the host resource demands.

That is the property this part has been claiming: a module does not get authority by being
a module, it borrows the caller's, and the borrowing can only narrow. Here it is failing
closed in front of you.

> ⚠ Try the mirror image too: put `.requires(…)` on the module's own `greeting` endpoint
> instead. Since `ikigai-module` 0.2.0 that **is** enforced — the caller is refused before
> the module runs, with the same message shape. It was not at 0.1.10, and the first
> edition of this exercise said so; see "Do assume the host enforces what a module
> declares", above, for the version and the test.

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
