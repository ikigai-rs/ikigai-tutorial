# Two ways to reach a kernel

## Linked in

The default, and the right default. Your crate exposes a `space()`; the host takes a Cargo
dependency on it and chains its bindings onto its own:

```rust,ignore
ikigai_fn::space().bind(Exact::new("urn:fn:toCamel"), to_camel())
```

The endpoints are in the binary. Resolution is a function call away. There is no
marshalling, no versioning question at runtime, and nothing to go wrong at load time
because there is no load time.

**Every endpoint in the `ikigai` CLI arrives this way.** The CLI does not depend on
`ikigai-module` at all.

## Routed to a module

A **module** is compiled separately. The host does not name its endpoints and does not
know what they are; it routes a *prefix* of the name space to a transport and lets the
module answer:

```rust,ignore
{{#include ../../../crates/loadable-module/src/lib.rs:host}}
```

Two things to notice, because they are the reason this composes at all.

**`ModuleSpace` implements `Space`.** It goes into the `Fallback` exactly where a
statically linked `space()` would go. The host has no special case for "modules" — it has
one more space, tried in order like the others.

**The host binds nothing under `urn:greet:`.** It has no `Exact` for `urn:greet:hello`,
and could not write one without knowing what the module offers. Prefix routing is what
lets a host delegate a region of the name space to code it has never seen.

## When each is right

Link it in when you can. It is simpler, faster, and has fewer failure modes.

Reach for a module when one of these is true:

- **The dependency is expensive and rarely used.** `ikigai-xslt` exists as a module
  precisely so that its XSLT engine is not linked into every host that will never
  transform an XML document.
- **The host cannot link it.** A kernel running as WebAssembly in a browser page cannot
  grow a new statically linked space; it can fetch one.
- **The code arrives after the host was built.** Which is the whole point, and also the
  part that is not finished — see [chapter 6](ch06-status.md).
