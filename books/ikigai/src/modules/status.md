# Where this actually stands

This chapter exists because a tutorial that leaves you to discover the maturity of a
feature by hitting its edges has wasted your afternoon.

Every claim below about a repository other than this one carries **the version it was
true at**, because those repositories move on their own schedules and a sentence about
them is a claim, not a fact. This book builds against `ikigai-core` **0.1.78** and
`ikigai-module` **0.3.1**, stamped on 2026-09-27, and its shell examples were last probed
against `ikigai-cli` **0.1.26** on 2026-09-26 (0.1.28 is on crates.io as of 2026-09-27; the
probe runs whatever binary is installed); a test (`crates/book-urns/tests/book_claims.rs`)
can re-check each stamped claim against the published crate it names, so that when one of
them stops being true the fix is a diff, not a discovery.

## Phase 1

`ikigai-module` 0.3.1's own first line still calls it **"the dynamically-loadable module
format (Phase 1: in-process proof)"**. That line undersells the crate — it has grown a
socket transport, at 0.2.0 capability enforcement across the boundary, and at 0.3.0 typed
errors, golden threads and endpoint cards crossing the wire, since somebody wrote it —
which is worth knowing as a habit: a crate's one-line self-description is the
thing least likely to be updated when the crate changes.

What is actually there, at `ikigai-module` 0.3.1 (0.3.1 over 0.3.0 is a constructor form, no new surface):

| piece | state |
|---|---|
| the callback machinery | **proven** — `InProcessTransport`, exercised end to end |
| the wire session | **proven through the codec** — `LoopbackTransport` encodes and decodes every message |
| an out-of-process transport | **built, Unix only** — `UdsTransport` + `serve`: the module runs in its own process behind a `0600` socket, and `serve` refuses peers belonging to another user |
| a module's declared capability | **enforced** since 0.2.0 — `ModuleFloor` on the mount, and an endpoint's own `requires` honored across the boundary (the crate's test `a_module_endpoints_declared_requirement_is_enforced_like_a_linked_ones`); since 0.3.0 the module's **cards** cross at `connect` (`ModuleCall::Cards`), so the host enforces against the module's own description and the manifold through a `ModuleSpace` lists the module's actions rather than one endpoint named `module` |
| the error type across the wire | **since 0.3.0** — `ModuleReply::ErrorTyped` and `ModuleCall::HostError`: a module-side denial arrives at the host as a `Denied`, a host resource's `NotFound` arrives in the module as a `NotFound`; before that the wire flattened both to a string |
| golden threads across the wire | **since 0.3.0** — `ModuleReply::ResolvedThreaded` carries a result's declared threads; before that a module result through the loopback or wasm session was cached forever with nothing to cut it, and a `Sink` through the mount left it stale |
| an embedded wasm runtime | **not built** — no `wasmtime` in its manifest; a host embedding one is Phase 2 |
| isolation | **not there** — a process boundary is not a resource budget |
| hosts that load modules | **one**, the browser demo (`ikigai-web-demo`, `WasmModuleSpace`, as of 2026-09-26) |

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
module's own card is a gate too. At 0.3.0 the card itself crosses the wire when the host
connects, so the enforcement is host-side against the module's own description rather
than against a floor the host wrote for it. The test that keeps this sentence true:

```rust,ignore
{{#include ../../../../crates/loadable-module/src/lib.rs:enforced}}
```

**Do not assume you can ship a module to a running host.** Nothing loads one at runtime
outside the browser demo (as of 2026-09-26).

**Do not assume the ABI is stable.** Phase 2 exists precisely to change how these messages
travel — 0.2.0 changed `ModuleSpace::new`'s signature once, and 0.3.0 added variants to
`ModuleCall` and `ModuleReply`, which are exhaustive public enums, so anyone matching on
them had a source break while `ModuleSpace::new` did not move. (The 0.2 bytes on the wire
are unchanged; the new messages are appended.)

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
(checked across the organization's repositories on 2026-09-26; the probe re-checks
`ikigai-module` 0.3.1 and `ikigai-cli` 0.1.28 by manifest). Treat this section as the
direction and the table above as the state, and do not plan a deployment on the
difference.

## The kernel since this book's last bump

The chapters above build against `ikigai-core` 0.1.78. Between the previous edition's
0.1.72 and it, six releases landed in nine days, each one sentence here and taught where a
chapter teaches it:

- **0.1.73 — the two cache-soundness holes closed, and a depth budget.** A cacheable read
  hangs from its own name whether or not it declares it, and a failed sub-request is a
  dependency — a thread on the missing name, or no caching at all for a result built on a
  refusal ([Golden threads in practice](../getting-started/golden-threads.md)); and a
  sub-request has a nesting budget, `Kernel::with_max_depth`, default 64, refused as
  `DepthExceeded` rather than overflowed.
<!-- urn-gate: illustration urn:kernel:bindings — a golden thread's name, not a resource;
     see "Golden threads in practice". -->
- **0.1.74 — `urn:kernel:bindings`.** One thread meaning "the binding set changed", which
  every face derived from the bindings hangs from and the party that rebinds a running
  kernel cuts ([Golden threads in practice](../getting-started/golden-threads.md), "Not
  yet"); and the capability floor memoizes an endpoint's description instead of rebuilding
  it per request.
- **0.1.75 — the first temporal corridor.** The chain reaches selection, the cache probe
  and the pipe, and `Scope::with_named_at` carries a clock derived from the corridor that
  pins time ([Scope and alias](../getting-started/scope-and-alias.md), "As of").
- **0.1.76 — `Limit`.** A door onto a wall: a family of names carved out of a chain by
  structure, `Unresolved` rather than `Denied`, and subtracted from the catalog ([Scope
  and alias](../getting-started/scope-and-alias.md), "Not yet").
- **0.1.77 — lossless by declaration.** `.lossy()` on a transreptor's description, and a
  planner that refuses a lossy hop without `lossy=allow`
  ([Transreption](../building/transreption.md)).
- **0.1.78 — the arrangement is a resource.** Spaces can carry a name, a hit reports which
  space answered, and `urn:kernel:topology` renders the chain you are standing in as
  Turtle, every node an IRI. No chapter teaches it yet.

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

Follow the path before you write it: the host routes `urn:iki:tutorial:yours:module:greeting`
to the module, the
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
