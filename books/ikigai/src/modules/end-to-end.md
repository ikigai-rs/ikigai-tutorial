# A module, end to end

In [Two ways to reach a kernel](two-ways.md), you saw the host route a prefix to a
module. In [The callback](the-callback.md), you saw the module ask that host to resolve
`urn:host:name`. Here you will run them together.

## Run it

Run this command in a terminal, from the root of your `ikigai-tutorial` clone. If you
need a clone, [Running it](../getting-started/running-it.md#the-tutorial-binary-this-repo-nothing-else)
shows how to get one.

```bash
cargo run -p loadable-module
```

```text
{{#include ../../../../crates/loadable-module/module-demo.stdout}}
```

Three resolutions happened, in this order:

1. The host resolved `urn:iki:tutorial:module:greeting`. Nothing in the host's own space
   matched, so the `Fallback` reached the `ModuleSpace`, whose prefix
   `urn:iki:tutorial:module:` did.
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

let request = Request::new(
    Verb::Source,
    Iri::parse("urn:iki:tutorial:module:greeting").unwrap(),
)
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
marshaling. But it exercises the part that actually carries risk: the re-entrancy. The
host is inside a resolution, calls the module, and the module calls back into the host
before the first resolution has returned. Deadlocks, borrow problems and lock inversions
live there, not in the byte format.

## Three tests worth reading

The [crate's tests](https://github.com/ikigai-rs/ikigai-tutorial/blob/main/crates/loadable-module/src/lib.rs)
exercise the callback, the host's own resource, and the routing boundary:

- `the_module_resolves_a_host_resource_mid_invocation` — the callback works. This single
  assertion is the difference between a module and a peer.
- `the_host_routes_by_prefix_not_by_knowing_the_endpoint` — despite its name, this
  test checks that the host still answers `urn:host:name` with `Peter` while the module
  is mounted. The demo above exercises the route to the module.
- `a_name_outside_the_module_prefix_does_not_reach_it` — routing is bounded; the module
  does not become a catch-all.
