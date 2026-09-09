# Testing an endpoint hermetically

Exercise 4 in Part I asked you to write a clock and cache it wrongly, and the hint said
the time should come from `inv.now()` rather than `SystemTime::now()`. This chapter is
the why, and the how: an endpoint that reads the clock, and three tests that never touch
one.

The design note this follows is `docs/design/hermetic-endpoint-tests.md` in `ikigai-core`.
Its diagnosis is worth carrying around: a test written *around* an ambient read rather
than *of* it asserts only shapes — `is_string()`, `is_number()` — because it does not own
the value. It passes, it stays passing, and it is not evidence about the endpoint.

## The endpoint

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/hermetic.rs:impl}}
```

Two decisions in eight lines.

`inv.now()` is an `Option`, and the `None` arm is an **error, not a fallback**. Reading the
wall clock behind the caller's back would make the resolution unrepeatable — the same
request, a different answer, and nothing in the request to say why — and `std::time` is
not there on every target this kernel runs on. Core reads the system clock in exactly one
place, `SystemClock`, which a host installs by name. An endpoint that wants time asks the
kernel; a kernel that has none says so.

`cacheable_until` names a deadline. That is only meaningful to a kernel that can read a
clock to compare it against — so a kernel *without* one declines to store the entry
rather than storing something it could never judge stale.

## Three tests, no wall clock

The full one: a real kernel, a `FixedClock`, and an exact answer the test chose.

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/hermetic.rs:fixed}}
```

The cheap one: no kernel at all. `Invocation::detached` builds a context with nothing
behind it, `with_clock` attaches the instant, and the implementation is called as a
function.

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/hermetic.rs:detached}}
```

> ⚠ A detached invocation skips argument routing and the kernel's capability floor, so an
> endpoint tested *only* this way is tested only in part. The kernel test above is the
> fuller one, and `FixedClock` exists so that the fuller one is cheap too. Use the detached
> form for the arithmetic; keep one kernel test so the contract is exercised.

And the one nobody writes: the branch with no clock anywhere. It is a branch, so it gets a
test, and the test pins the honest answer.

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/hermetic.rs:none}}
```

The design note has a number for why that last test matters: one module in the ecosystem
had five endpoints reading `inv.now()` and twenty-three test kernels, none of which
installed a clock — so every one of those five had only ever taken its `None` branch under
test. A branch no test has entered is behavior nobody has verified.

## `FixedClock` does not move

Deliberately. A test that needs two instants builds two kernels, each with its own
`FixedClock` — `FixedClock::from_time(last.plus_millis(..))` advances a scenario by
handing the next kernel a clock derived from the last one's. A test that needs time to
*move past a deadline* inside one kernel wants a settable clock, and core does not ship
one: three crates carry their own `AtomicU64`-backed version, all correct, none drifted,
and that is the bar for pushing a shape down — a count *plus* observed drift — not met.
The note records what would change it.

## Try it

```rust
# extern crate building_endpoints;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{Capability, FixedClock, Iri, Kernel, Request, Verb};

let kernel = Kernel::new(Arc::new(building_endpoints::space(None)))
    .with_clock(Arc::new(FixedClock::at(1_000)));

let request = Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:stamp").unwrap());
let repr = block_on(kernel.issue(request, &Capability::root())).unwrap();
assert_eq!(String::from_utf8_lossy(&repr.bytes), "1000");
```
