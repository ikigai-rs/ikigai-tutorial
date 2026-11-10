# The wire session

In [A module, end to end](end-to-end.md), you ran a module with `InProcessTransport`
and saw it call back into the host. When the host and module communicate across a
boundary — between WebAssembly instances or over a socket, for example — their requests
and replies have to be encoded.

## Two message types, not one

The obvious design is request-in, response-out. That is not enough, because of the
callback: the module's reply might be *"I need something first."*

So the session has two enums:

- **`ModuleCall`** — what the host sends: the invocation, and later the *results* of host
  calls the module asked for.
- **`ModuleReply`** — what the module sends: the finished representation, an error, or
  **`HostCall`** — "resolve this for me and call me back."

Since `ikigai-module` 0.3.0, the finished representation can also carry its declared
golden threads. Errors keep their *type* in both directions: a denial is a denial on
the other side too.

A `Cards` call at connect time fetches every binding's description so the host can list
and gate the module's endpoints as its own.

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

`UdsTransport` — the same session over a Unix socket, the module in its own process — is
the third layer on top of those two, and it is why a bug there is a bug in the *transport*:
the other two have their own proofs.

## The browser host

The one host that loads modules today is the web demo, and it uses the wasm-facing side of
this: `WasmModuleSpace`, `ModuleSessionTransport`, and `serve_host_call` — the last being
the host-side handler that receives a `HostCall` reply and resolves it against the
host kernel. It sends back a `ModuleCall::HostResult` on success or for an endpoint
error, and a `ModuleCall::HostError` for other errors so their types survive the crossing.

If you want to read one real implementation, that is the one that exists.
