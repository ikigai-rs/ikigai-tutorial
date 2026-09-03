# The wire session

`InProcessTransport` proves the semantics. A *real* module is on the other side of
something — another WebAssembly instance, an embedded wasmtime, a socket — and then the
conversation has to be encoded.

## Two message types, not one

The obvious design is request-in, response-out. That is not enough, because of the
callback: the module's reply might be *"I need something first."*

So the session has two enums:

- **`ModuleCall`** — what the host sends: the invocation, and later the *results* of host
  calls the module asked for.
- **`ModuleReply`** — what the module sends: the finished representation, an error, or
  **`HostCall`** — "resolve this for me and call me back."

`ModuleReply::HostCall` is the interesting variant, and it is why this is a *session*
rather than a call. A single invocation can bounce back and forth several times before it
completes.

## The loopback

`LoopbackTransport` runs that session **through the codec** in one process: every message
is genuinely encoded and decoded, but nothing leaves the machine.

This is a nice piece of test design worth stealing. It separates two failure modes that
would otherwise be tangled:

- `InProcessTransport` — does the *re-entrancy* work? (no encoding involved)
- `LoopbackTransport` — does the *encoding* round-trip? (no transport involved)

When a real transport is added, a bug is in the transport, because the other two layers
have their own proofs.

## The browser host

The one host that loads modules today is the web demo, and it uses the wasm-facing side of
this: `WasmModuleSpace`, `ModuleSessionTransport`, and `serve_host_call` — the last being
the module-side loop that receives a `HostCall` reply, resolves it against the host, and
sends back a `ModuleCall::HostResult`.

If you want to read one real implementation, that is the one that exists.
