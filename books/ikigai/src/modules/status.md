# Where this actually stands

This chapter exists because a tutorial that leaves you to discover the maturity of a
feature by hitting its edges has wasted your afternoon.

Every claim below about a repository other than this one carries **the version it was
true at**, because those repositories move on their own schedules and a sentence about
them is a claim, not a fact. This book builds against `ikigai-core` **0.1.93** and
`ikigai-module` **0.3.2**, stamped on 2026-10-10 (core 0.1.95 is on crates.io as of that
day; the book has not moved to it), and its shell examples are replayed against
`ikigai-cli` **0.1.44**, the one release pinned in `books/ikigai/ikigai-cli.version`
(0.1.45 is on crates.io as of 2026-10-10). A test (`crates/book-urns/tests/book_claims.rs`)
holds those three numbers to the workspace's lock and that pin, and re-checks each stamped
claim below against the published crate it names, so that when one of them stops being
true the fix is a diff, not a discovery.

## Phase 1

`ikigai-module` 0.3.2's own first line still calls it **"the dynamically-loadable module
format (Phase 1: in-process proof)"**. That line undersells the crate — it has grown a
socket transport, at 0.2.0 capability enforcement across the boundary, and at 0.3.0 typed
errors, golden threads and endpoint cards crossing the wire, since somebody wrote it —
which is worth knowing as a habit: a crate's one-line self-description is the
thing least likely to be updated when the crate changes.

What is actually there, at `ikigai-module` 0.3.2 (0.3.2 over 0.3.1 records a failed callback, below; 0.3.1 over 0.3.0 was a constructor form, no new surface):

| piece | state |
|---|---|
| the callback machinery | **proven** — `InProcessTransport`, exercised end to end |
| the wire session | **proven through the codec** — `LoopbackTransport` encodes and decodes every message |
| an out-of-process transport | **built, Unix only** — `UdsTransport` + `serve`: the module runs in its own process behind a `0600` socket, and `serve` refuses peers belonging to another user |
| a module's declared capability | **enforced** since 0.2.0 — `ModuleFloor` on the mount, and an endpoint's own `requires` honored across the boundary (the crate's test `a_module_endpoints_declared_requirement_is_enforced_like_a_linked_ones`); since 0.3.0 the module's **cards** cross at `connect` (`ModuleCall::Cards`), so the host enforces against the module's own description and the manifold through a `ModuleSpace` lists the module's actions rather than one endpoint named `module` |
| the error type across the wire | **since 0.3.0** — `ModuleReply::ErrorTyped` and `ModuleCall::HostError`: a module-side denial arrives at the host as a `Denied`, a host resource's `NotFound` arrives in the module as a `NotFound`; before that the wire flattened both to a string |
| golden threads across the wire | **since 0.3.0** — `ModuleReply::ResolvedThreaded` carries a result's declared threads; before that a module result through the loopback or wasm session was cached forever with nothing to cut it, and a `Sink` through the mount left it stale. Since 0.3.2 a callback that **failed** is a dependency on the wasm path too, by core's rule for a failed sub-request (`record_failure`): a `NotFound` or `Unresolved` hangs the module's answer from the missing name, any other failure makes it uncacheable; the native transports (in-process, loopback, Unix socket) already had that, through the host's own issuer |
| an embedded wasm runtime | **not built** — no `wasmtime` in its manifest; a host embedding one is Phase 2 |
| isolation | **not there** — a process boundary is not a resource budget |
| hosts that load modules | **one**, the browser demo (`ikigai-web-demo`, `WasmModuleSpace`, as of 2026-10-10) |

## Read that table the right way

The order is deliberate and it is not the order most projects build in. The *semantics*
came first — re-entrancy, capability inheritance, the session shape — and every transport
since has been that same session with a different pipe under it: a direct call, then the
codec over in-memory channels, then a socket.

That is the harder half done first. Marshaling bytes over a socket is well-understood
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
outside the browser demo (as of 2026-10-10).

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
*not* exist is any of the native half: no host in the ecosystem embeds wasmtime, no crate
sets a memory budget, and none can stop a computation it has started:
the word "fuel" appears in none of them (checked across the organization's repositories
on 2026-10-10; the probe re-checks `ikigai-module` 0.3.2 and
`ikigai-cli` 0.1.45 by manifest). The nearest thing is a deadline on the *answer*, not on
the work: since `ikigai-xslt` 0.2.1 every transform is answered
within a time budget, five seconds, with a typed `Timeout`, and the crate says plainly that
the work past the deadline is **abandoned, never stopped** (checked at
`ikigai-xslt` 0.3.0) — the caller is released and the thread keeps running. Treat this
section as the direction and the table above as the state, and do not plan a deployment on
the difference.

## The kernel since this chapter's last stamp

The chapters above build against `ikigai-core` 0.1.93. This chapter was last stamped at
0.1.78, on 2026-09-27, and fifteen releases landed between the two in thirteen days. Each is
one line here, and taught where a chapter teaches it:

- **0.1.79 — documents and tests only.** The check of what a wall keeps out answers only for
  what the graph shows in full, and a second query lists the paths it cannot answer for.
- **0.1.80 — `Conflict`.** A typed refusal for a request the resource's *current state* says
  no to, the 409 of the taxonomy, found by this book's game: a move to a taken square
  ([Tic-tac-toe, VI](../applied/tic-tac-toe-6.md)). The spreadsheet's market and its
  read-only past refuse with it ([A spreadsheet, V](../applied/spreadsheet-5.md),
  [VI](../applied/spreadsheet-6.md)).
- **0.1.81 — levels and sealed names.** `Level`, the resolved scope a hit reports, and a
  host's seals on names, checked when a space is built ([Tic-tac-toe,
  VII](../applied/tic-tac-toe-7.md), "What a declaration may and may not do").
- **0.1.82 — corridors stack, and a read says why it was not cached.** `Scope::stack`
  ([A spreadsheet, VI](../applied/spreadsheet-6.md)); every uncacheable read names its
  reason ([Tracing a resolution](../beyond/tracing.md)); a fallback over a composite's
  `NotFound` is cut when the atom changes, and the cache readout marks each row live, cut or
  expired ([A spreadsheet, III](../applied/spreadsheet-3.md)); and cut listeners.
- **0.1.83 — a space built from its declaration.** `Registry`, `build` and
  `Topology::from_turtle` (the `declare` feature), the inverse of `urn:kernel:topology`, with
  every door naming the endpoint that answers it ([Tic-tac-toe,
  VII](../applied/tic-tac-toe-7.md)).
- **0.1.84 — declarations are bounded.** A declaration past a depth, node or text bound is
  refused rather than overflowing the stack; and `Kernel::root_topology`, the root's own
  arrangement, which Part VII's test compares the coded game with the declared one through.
- **0.1.85 — the audit release.** The capability floor covers every verb an endpoint can be
  asked for, not only the ones it declares; nothing but the kernel answers inside the
  kernel's own namespace; and a seal names a level by the object registered, not by its name.
- **0.1.86 — exclusions are sticky.** A deny-shaped scope survives `attenuate` and `clamp`:
  grants only shrink, exclusions only accumulate.
- **0.1.87 — the "why" tools.** `urn:kernel:explain`, a dry run of resolution ([Tracing a
  resolution](../beyond/tracing.md)); `urn:kernel:dependents`, what a cut would recompute
  ([Golden threads in practice](../getting-started/golden-threads.md)); and
  `Description::unsatisfied_scopes`, the floor's own predicate ([Who is
  asking](../beyond/identity.md)).
- **0.1.88 and 0.1.90 — vocabulary only.** Natural-language drafting terms move into `ik:`,
  and `ikigai_vocab::space()` names itself.
- **0.1.89 — a space's name survives only what keeps its doors.** Binding onto a named space
  drops its name, and a module space that needs no configuration names itself. This book
  binds every such space behind doors of its own, so no page printed differently.
- **0.1.91 — an action match has its own IRI, and a contract is content-addressed.** A match
  is named by its verb and door, and the contract behind it by a digest of what it declares,
  which every `Meta` and catalog graph in this book now prints ([What resolution buys
  you](../getting-started/payoff.md)).
- **0.1.92 — a failed invocation is a trace node.** An invocation that ran and failed
  records an event with the error's kind, so the trace and the cache agree about what a
  resolution touched. No chapter teaches it yet.
- **0.1.93 — who a request is for, and the thread a failure hangs from.** A door puts the
  principal it authenticated into the capability (`Capability::with_principal`, read back
  with `Capability::principal`), and `Invocation::depends_on` lets an atom whose absence is
  another name's state hang its `NotFound` from that name's thread. No chapter teaches either
  yet.

Two more are on crates.io as of 2026-10-10 and this book does not build against them yet:
0.1.94 (a percent escape in a match or contract IRI is exactly two hex digits) and 0.1.95
(those IRIs are parsed leniently and always emitted in one canonical spelling, and a write
that ran and failed cuts its target).

The stamp before this one, 0.1.78, covered the six releases after 0.1.72: the two
cache-soundness holes closed and a depth budget (0.1.73), one thread for "the binding set
changed" (0.1.74; both in [Golden threads in
practice](../getting-started/golden-threads.md)), the first temporal corridor (0.1.75) and
`Limit` (0.1.76; both in [Scope and alias](../getting-started/scope-and-alias.md)), lossless
by declaration ([Transreption](../building/transreption.md), 0.1.77), and the arrangement as
a resource, `urn:kernel:topology` ([Tic-tac-toe, VII](../applied/tic-tac-toe-7.md), 0.1.78).

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
