//! **Multi-verb endpoints** — one endpoint, three verbs, three contracts, and a capability
//! that is declared *and* enforced.
//!
//! `camel-case` answers one verb, so its flat description *is* its contract. `counter`
//! answers three, and they differ: reading needs nothing, writing and resetting need
//! authority. Each verb therefore gets its own `ActionSpec`, and the `requires` on the
//! two mutating ones is checked by the kernel before the endpoint runs — a caller whose
//! capability does not grant the scope is refused at the door.

use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, ArgSpec, Description, Error, FnEndpoint, Invocation, Representation, Verb,
};

use crate::{text_plain_utf8, TEXT_PLAIN_UTF8};

/// The name `counter` is bound at — and the golden thread the kernel cuts on a write.
pub const COUNTER: &str = "urn:iki:tutorial:counter";

/// The scope a write to the counter requires.
pub const WRITE: &str = "urn:cap:tutorial:counter:write";

// ANCHOR: endpoint
/// `counter`: a number the host holds. `Source` reads it, `Sink` sets it, `Delete`
/// resets it to zero.
///
/// The implementation branches on the verb; the description declares each branch as an
/// action. The two halves have to agree, and the kernel enforces only the declared half:
/// remove `.requires(WRITE)` from the `Sink` action and the write is open to anyone,
/// regardless of what the code below would like — there is no check in it, on purpose,
/// because a check the catalog does not know about is a lie in the other direction.
pub fn counter() -> FnEndpoint {
    let count = Arc::new(Mutex::new(0u64));
    FnEndpoint::new("counter", move |inv: &Invocation<'_>| {
        let mut current = count.lock().expect("counter lock");
        match inv.request.verb {
            Verb::Sink => {
                let text = inv.inline_str("content")?;
                *current = text.trim().parse().map_err(|e| Error::InvalidArgument {
                    name: "content".into(),
                    detail: format!("not a number: {e}"),
                })?;
                Ok(Representation::new(text_plain_utf8(), b"ok".to_vec()))
            }
            Verb::Delete => {
                *current = 0;
                Ok(Representation::new(text_plain_utf8(), b"reset".to_vec()))
            }
            _ => Ok(
                Representation::new(text_plain_utf8(), current.to_string().into_bytes())
                    .cacheable()
                    .depends_on(COUNTER),
            ),
        }
    })
    .with_description(
        Description::new("counter")
            .title("Counter")
            .summary("A number the host holds: read it, set it, or reset it.")
            .verb(Verb::Meta)
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("the current count")
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("set the count")
                    .input(
                        ArgSpec::new("content")
                            .summary("the new count, a non-negative integer")
                            .class("http://www.w3.org/2001/XMLSchema#nonNegativeInteger"),
                    )
                    .requires(WRITE),
            )
            .action(
                ActionSpec::new(Verb::Delete)
                    .summary("reset the count to zero")
                    .requires(WRITE),
            ),
    )
}
// ANCHOR_END: endpoint

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_in;
    use futures::executor::block_on;
    use ikigai_core::{ArgRef, Capability, Endpoint, Iri, Kernel, Request, Result};

    fn issue(
        kernel: &Kernel,
        verb: Verb,
        content: Option<&str>,
        cap: &Capability,
    ) -> Result<String> {
        let mut request = Request::new(verb, Iri::parse(COUNTER).expect("iri"));
        if let Some(content) = content {
            request = request.with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()));
        }
        let repr = block_on(kernel.issue(request, cap))?;
        Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
    }

    // ANCHOR: enforced
    /// The declared capability is the enforced one — per verb.
    #[test]
    fn a_narrow_capability_can_read_but_not_write() {
        let kernel = kernel_in(None);
        // Holds something, but not the write scope.
        let reader = Capability::root().attenuate(["urn:cap:kernel:inspect"]);
        let writer = Capability::root().attenuate([WRITE]);

        assert_eq!(issue(&kernel, Verb::Source, None, &reader).unwrap(), "0");

        let denied = issue(&kernel, Verb::Sink, Some("7"), &reader).unwrap_err();
        assert!(matches!(denied, Error::Denied(_)), "{denied:?}");
        assert!(
            denied.to_string().contains(WRITE) && denied.to_string().contains(COUNTER),
            "the message names the scope and the declarer: {denied}"
        );

        // Same request, a capability that grants the scope: the write lands, and the
        // reader — still narrow — sees it, because the Sink cut the counter's thread.
        assert_eq!(
            issue(&kernel, Verb::Sink, Some("7"), &writer).unwrap(),
            "ok"
        );
        assert_eq!(issue(&kernel, Verb::Source, None, &reader).unwrap(), "7");

        let denied = issue(&kernel, Verb::Delete, None, &reader).unwrap_err();
        assert!(matches!(denied, Error::Denied(_)), "{denied:?}");
        assert_eq!(
            issue(&kernel, Verb::Delete, None, &writer).unwrap(),
            "reset"
        );
        assert_eq!(issue(&kernel, Verb::Source, None, &reader).unwrap(), "0");
    }
    // ANCHOR_END: enforced

    // ANCHOR: actions
    /// The per-verb view is what the catalog, selection and the agent tool list see.
    #[test]
    fn three_verbs_are_three_actions_and_only_two_require_anything() {
        let actions = counter().describe().action_specs();
        let verbs: Vec<Verb> = actions.iter().map(|a| a.verb).collect();
        assert_eq!(verbs, vec![Verb::Source, Verb::Sink, Verb::Delete]);

        let requiring: Vec<Verb> = actions
            .iter()
            .filter(|a| a.requires.contains(&WRITE.to_string()))
            .map(|a| a.verb)
            .collect();
        assert_eq!(requiring, vec![Verb::Sink, Verb::Delete]);
    }
    // ANCHOR_END: actions

    #[test]
    fn a_write_that_is_not_a_number_is_an_argument_error_not_a_denial() {
        let kernel = kernel_in(None);
        let writer = Capability::root().attenuate([WRITE]);
        let error = issue(&kernel, Verb::Sink, Some("seven"), &writer).unwrap_err();
        assert!(matches!(error, Error::InvalidArgument { .. }), "{error:?}");
    }
}
