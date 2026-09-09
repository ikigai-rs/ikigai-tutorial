//! **Testing an endpoint hermetically** — an endpoint that reads the clock, tested
//! without one.
//!
//! "Now" is an ambient input: read it from the wall clock and the endpoint cannot be
//! tested for a value, only for a shape. The kernel models the clock instead — a `Clock`
//! injected into the `Kernel`, reachable from an invocation as `inv.now()` — so a test
//! hands the kernel a `FixedClock` and asserts the exact answer. The design note this
//! follows is `docs/design/hermetic-endpoint-tests.md` in `ikigai-core`.

use ikigai_core::{Description, Error, FnEndpoint, Invocation, Representation, Result, Verb};

use crate::{text_plain_utf8, TEXT_PLAIN_UTF8};

// ANCHOR: impl
/// The current time in milliseconds since the epoch, per the kernel's clock.
///
/// `inv.now()` is `None` when nothing injected a clock. That is a legal state and it is
/// **not** papered over with `SystemTime::now()`: reading the wall clock behind the
/// caller's back is what would make this resolution unrepeatable, and `std::time` is not
/// there on every target the kernel runs on. A host that wants time installs a clock;
/// one that did not gets told so.
///
/// The answer is cacheable *for one second* — `cacheable_until` names a deadline, and a
/// deadline is only meaningful to a kernel that can read a clock to compare it against.
pub fn stamp_impl(inv: &Invocation<'_>) -> Result<Representation> {
    let now = inv
        .now()
        .ok_or_else(|| Error::Endpoint("this host has no clock".to_string()))?;
    Ok(
        Representation::new(text_plain_utf8(), now.as_millis().to_string().into_bytes())
            .cacheable_until(now.plus_millis(1_000)),
    )
}
// ANCHOR_END: impl

/// `stamp`: the kernel's idea of now.
pub fn stamp() -> FnEndpoint {
    FnEndpoint::new("stamp", stamp_impl).with_description(
        Description::new("stamp")
            .title("Timestamp")
            .summary("The current time in milliseconds since the epoch, per the host's clock.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::space;
    use futures::executor::block_on;
    use ikigai_core::{Bindings, Capability, FixedClock, Iri, Kernel, Request};
    use std::sync::Arc;

    fn request() -> Request {
        Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:stamp").expect("iri"),
        )
    }

    // ANCHOR: fixed
    /// A kernel with a fixed clock: the endpoint answers an exact value, and the value
    /// is one the test chose.
    #[test]
    fn under_a_fixed_clock_the_stamp_is_exactly_that_instant() {
        let clock = FixedClock::at(1_700_000_000_000);
        let kernel = Kernel::new(Arc::new(space(None))).with_clock(Arc::new(clock));

        let repr = block_on(kernel.issue(request(), &Capability::root())).expect("resolves");
        assert_eq!(String::from_utf8_lossy(&repr.bytes), "1700000000000");

        // Cacheable until a deadline, and the kernel has a clock to judge it by — so the
        // second read is served, not computed.
        assert!(kernel.is_cached(&request(), &Capability::root()));
    }
    // ANCHOR_END: fixed

    // ANCHOR: detached
    /// No kernel at all: the implementation, called directly, with the clock attached to
    /// the invocation. Cheapest possible test — and a partial one, which the chapter
    /// says out loud.
    #[test]
    fn a_detached_invocation_can_carry_its_own_clock() {
        let request = request();
        let bindings = Bindings::new();
        let root = Capability::root();
        let inv = Invocation::detached(&request, &bindings, &root)
            .with_clock(Arc::new(FixedClock::at(42)));

        let repr = stamp_impl(&inv).expect("a clock was attached");
        assert_eq!(String::from_utf8_lossy(&repr.bytes), "42");
    }
    // ANCHOR_END: detached

    // ANCHOR: none
    /// The branch a test usually never takes: no clock anywhere. It is a branch, so it
    /// gets a test, and the test pins the honest answer — an error that says why.
    #[test]
    fn without_a_clock_the_stamp_refuses_rather_than_guessing() {
        let kernel = Kernel::new(Arc::new(space(None)));
        let error = block_on(kernel.issue(request(), &Capability::root())).unwrap_err();
        assert!(error.to_string().contains("no clock"), "{error}");
    }
    // ANCHOR_END: none
}
