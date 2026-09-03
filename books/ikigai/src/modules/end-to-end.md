# A module, end to end

Everything from the previous two chapters, running.

## Run it

```bash
cargo run -p loadable-module
```

```text
host    resolves urn:greet:hello name=urn:host:name
module  asks the host for urn:host:name
out     Hello, Peter!
```

Three resolutions happened, in this order:

1. The host resolved `urn:greet:hello`. Nothing in the host's own space matched, so the
   `Fallback` reached the `ModuleSpace`, whose prefix `urn:greet:` did.
2. The module's endpoint ran, and resolved `urn:host:name` — **back through the host**.
3. The host answered `Peter` from a space the module cannot see.

`"Hello, Peter!"` is a string that neither side could have produced alone.

## Wiring it yourself

```rust,no_run
# extern crate loadable_module;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request, Verb};

let kernel = Kernel::new(Arc::new(loadable_module::host_space()));

let request = Request::new(Verb::Source, Iri::parse("urn:greet:hello").unwrap())
    .with_arg("name", ArgRef::Inline(b"urn:host:name".to_vec()));

let repr = block_on(kernel.issue(request, &Capability::root())).unwrap();
assert_eq!(String::from_utf8_lossy(&repr.bytes), "Hello, Peter!");
```

Note what the caller did *not* do: it never mentioned a module. From the outside, resolving
a module-backed name is indistinguishable from resolving any other name — which is the
point of routing being a host concern.

## `InProcessTransport`

The demo uses `InProcessTransport`, which runs the module in the same process: it resolves
the request in the module's space and invokes the endpoint **with the host as its issuer**.

That sounds like it is skipping the hard part, and in one sense it is — there is no
marshalling. But it exercises the part that actually carries risk: the re-entrancy. The
host is inside a resolution, calls the module, and the module calls back into the host
before the first resolution has returned. Deadlocks, borrow problems and lock inversions
live there, not in the byte format.

## Three tests worth reading

The crate's tests pin the three properties this book claims, and they are short enough to
be worth opening:

- `the_module_resolves_a_host_resource_mid_invocation` — the callback works. This single
  assertion is the difference between a module and a peer.
- `the_host_routes_by_prefix_not_by_knowing_the_endpoint` — the host bound no
  `urn:greet:hello`.
- `a_name_outside_the_module_prefix_does_not_reach_it` — routing is bounded; the module
  does not become a catch-all.
