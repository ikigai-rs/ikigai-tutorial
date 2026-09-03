# Where this actually stands

This chapter exists because a tutorial that leaves you to discover the maturity of a
feature by hitting its edges has wasted your afternoon.

## Phase 1

`ikigai-module`'s own first line calls it **"the dynamically-loadable module format
(Phase 1: in-process proof)"**.

What that means concretely:

| | |
|---|---|
| the callback machinery | **proven** — `InProcessTransport`, exercised end to end |
| the wire session | **proven through the codec** — `LoopbackTransport` encodes and decodes every message |
| a real out-of-process transport | **not built** — "a second wasm instance, or an embedded wasmtime, or a socket" is Phase 2 |
| isolation | **not there** |
| hosts that load modules | **one**, the browser demo |

## Read that table the right way

The order is deliberate and it is not the order most projects build in. The *semantics*
came first — re-entrancy, capability inheritance, the session shape — and the transport is
the part still missing.

That is the harder half done first. Marshalling bytes over a socket is well-understood
work; a host and a module calling into each other mid-invocation without deadlocking, with
authority attenuating correctly across the boundary, is where the design risk lives.

## What you should not assume

**Do not assume a module is a security boundary.** It is not one yet. The callback is a
deliberate hole in whatever isolation you would put around it, and Phase 1 runs the module
in your own process anyway. A module today is a *packaging* and *lazy-loading* mechanism,
not a sandbox.

**Do not assume you can ship a module to a running host.** Nothing loads one at runtime
outside the browser demo.

**Do not assume the ABI is stable.** Phase 2 exists precisely to change how these messages
travel.

## What is genuinely usable now

- The dual-mode pattern, as a way to keep a heavy dependency out of hosts that do not need
  it ([chapter 5](dual-mode.md)).
- The browser case, which is real and running.
- The `InProcessTransport`, as a way to develop and test a module's *semantics* long before
  its transport exists — which is what this book's demo does.

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
