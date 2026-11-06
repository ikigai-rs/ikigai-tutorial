# The callback

## The problem a module has

Give a module an endpoint that transforms a document with a stylesheet. The caller passes
`stylesheet=urn:file:report.xsl`.

The module cannot resolve that. It has no file space, no catalog, no cache, and no
capability of its own. The name means something only in the **host's** kernel.

So either the host resolves every argument eagerly before dispatching — which would defeat
laziness, break `Exists`, and force it to know which arguments are resource references —
or the module asks. It asks.

## Where the seam already was

An endpoint reaches the kernel through its `Invocation`, which holds an **`Issuer`**:
the interface for issuing sub-requests. That indirection already lets sub-requests inherit
the caller's capability and be traced.

So a module is simply handed *the host* as its issuer. From inside the endpoint, the call
is the same one you would write in a linked-in endpoint:

```rust,ignore
{{#include ../../../../crates/loadable-module/src/lib.rs:module_endpoint}}
```

`inv.source(&iri)` is the callback. The module is in the middle of its own invocation, and
that line resolves a name in the host's kernel — with the host's spaces, the host's cache,
and the capability the invocation is already carrying.

That last clause matters: the module does not get authority by being a module. It borrows
the caller's, which can only narrow on the way down.

## Why this is the interesting property

A remote peer is *autonomous*: it answers with its own resources. A module is
*parasitic* — deliberately — it contributes endpoints while continuing to live in the
host's world.

That is what makes a module composable in a way a peer is not. The module's endpoint can
take any name the host can resolve, including endpoints from other modules, without
knowing any of them exist.

It is also what makes a module harder to isolate than a peer, since the callback is a hole
in whatever boundary you put around it. That tension is the subject of
[Where this actually stands](status.md).
