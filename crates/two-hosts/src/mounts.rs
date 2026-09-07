//! The three relationships a mounted peer can have with the local namespace.
//!
//! A connected peer is a `Resolver`. Mounting it is deciding *what its answers mean here*
//! — a name the local kernel does not have, a name it has but should not serve, or a name
//! it can serve if the peer cannot. Those are the three, and the third is the one with
//! rules, because it is the only one that can quietly answer a different question than
//! the one asked.

use std::sync::Arc;

use ikigai_core::{Fallback, Request, Resolution, Scope, Space, SpaceEntry};
use ikigai_resolve::{MountedRemote, Resolver};
use ikigai_throttle::Failover;

// ANCHOR: alias
/// **Alias** — `prefix` is a *local name* for the peer's namespace. `<prefix>rest` is
/// rewritten to `urn:rest` before it is forwarded, and the peer's catalog comes back
/// re-prefixed and tagged with `origin`.
///
/// Compose this **after** the local spaces: an alias only ever catches what this kernel
/// lacks. A prefix the local kernel already serves would win, and the mount would
/// silently never be used.
pub fn alias(prefix: &str, origin: &str, peer: Arc<dyn Resolver>) -> Arc<dyn Space> {
    Arc::new(MountedRemote::new(peer, prefix, origin))
}
// ANCHOR_END: alias

// ANCHOR: overriding
/// **Override** — the *same* namespace, served remotely. The IRI is forwarded unchanged,
/// so nothing at the call site changes.
///
/// Compose this **before** the local spaces; precedence is the other half of the
/// semantics, and a mount composed after a local binding of the same name is not an
/// override of anything. If the peer is down the resolution *fails*, and that is the
/// point — you asked for that machine.
pub fn overriding(prefix: &str, origin: &str, peer: Arc<dyn Resolver>) -> Arc<dyn Space> {
    Arc::new(MountedRemote::overriding(peer, prefix, origin))
}
// ANCHOR_END: overriding

// ANCHOR: prefer
/// **Prefer** — an override wrapped in a [`Failover`] over the local spaces: the peer
/// when it answers, this machine when it does not.
///
/// The `PrefixGuard` is not decoration. `Failover` resolves *every* target, so an
/// unguarded `[peer, local]` pair would also answer for IRIs the local spaces bind and
/// this mount never claimed — hitting before a less-specific override behind it and
/// silently defeating it.
///
/// The catalog comes from the peer alone, so listing a prefer-mount shows what is
/// mounted there rather than re-listing the whole local kernel under the peer's name.
pub fn prefer(
    prefix: &str,
    origin: &str,
    peer: Arc<dyn Resolver>,
    local: Arc<dyn Space>,
) -> Arc<dyn Space> {
    let remote: Arc<dyn Space> = Arc::new(MountedRemote::overriding(peer, prefix, origin));
    Arc::new(PrefixGuard {
        prefix: prefix.to_string(),
        inner: Arc::new(Failover::new(vec![Arc::clone(&remote), local])),
        catalog: remote,
    })
}
// ANCHOR_END: prefer

// ANCHOR: prefix_guard
/// Confines a composed space to one IRI prefix.
struct PrefixGuard {
    prefix: String,
    inner: Arc<dyn Space>,
    catalog: Arc<dyn Space>,
}

impl Space for PrefixGuard {
    fn resolve(&self, request: &Request, scope: &Scope) -> Resolution {
        if !request.target.as_str().starts_with(&self.prefix) {
            return Resolution::Miss;
        }
        self.inner.resolve(request, scope)
    }

    fn entries(&self) -> Option<Vec<SpaceEntry>> {
        self.catalog.entries()
    }
}
// ANCHOR_END: prefix_guard

// ANCHOR: compose
/// Compose one kernel's root space: the mounts that front the local namespace, most
/// specific prefix first, then everything this machine serves itself.
///
/// Sorting by prefix *length* is what makes a single-resource override work: a whole IRI
/// is simply the most specific prefix there is, so `urn:llm:ask` mounted at one peer and
/// `urn:llm:` at another route the way an operator would expect regardless of the order
/// they wrote the flags in.
pub fn compose(mut fronting: Vec<(String, Arc<dyn Space>)>, local: Arc<dyn Space>) -> Fallback {
    fronting.sort_by_key(|(prefix, _)| std::cmp::Reverse(prefix.len()));
    let mut ordered: Vec<Arc<dyn Space>> = fronting.into_iter().map(|(_, space)| space).collect();
    ordered.push(local);
    Fallback::new(ordered)
}
// ANCHOR_END: compose

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use ikigai_core::{
        ArgRef, AsyncFnEndpoint, Capability, Description, EndpointSpace, Error, Exact, Invocation,
        InvokeFuture, Iri, Kernel, ReprType, Representation, Verb,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};

    /// A peer that is always down, or always refuses. Every mount rule in this file is
    /// about what happens when the answer is *not* a representation, so the only peer
    /// these tests need is one that never gives one.
    struct DeadPeer {
        error: fn(String) -> Error,
        calls: Arc<AtomicUsize>,
    }

    #[async_trait::async_trait]
    impl Resolver for DeadPeer {
        fn issue(
            &self,
            _request: Request,
        ) -> Result<(Representation, ikigai_resolve::CacheStatus), Error> {
            self.calls.fetch_add(1, Ordering::SeqCst);
            Err((self.error)("the peer".to_string()))
        }

        fn is_cached(&self, _request: &Request, _capability: &Capability) -> bool {
            false
        }

        fn entries(&self) -> Option<Vec<SpaceEntry>> {
            None
        }
    }

    fn dead(error: fn(String) -> Error) -> (Arc<dyn Resolver>, Arc<AtomicUsize>) {
        let calls = Arc::new(AtomicUsize::new(0));
        (
            Arc::new(DeadPeer {
                error,
                calls: Arc::clone(&calls),
            }),
            calls,
        )
    }

    /// A local binding that answers every verb and counts its own invocations.
    /// `urn:iki:tutorial:camel-case` is the name Part I taught, and it is bound here so a
    /// fallback is visible as a different *answer*, not merely as an absence of an error.
    ///
    /// The counter is handed in rather than kept in a `static`: these tests run in
    /// parallel, and a shared counter would make each one's assertion depend on the
    /// others — the first version of this file failed exactly that way.
    fn local_space(calls: Arc<AtomicUsize>) -> Arc<dyn Space> {
        let endpoint =
            AsyncFnEndpoint::new("local", move |_inv: &Invocation<'_>| -> InvokeFuture<'_> {
                let calls = Arc::clone(&calls);
                Box::pin(async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok(Representation::new(
                        ReprType::new("text/plain"),
                        b"answered locally".to_vec(),
                    ))
                })
            })
            .with_description(
                Description::new("local")
                    .verb(Verb::Source)
                    .verb(Verb::Sink)
                    .verb(Verb::Delete),
            );
        Arc::new(EndpointSpace::new().bind(Exact::new("urn:iki:tutorial:camel-case"), endpoint))
    }

    fn issue(space: Arc<dyn Space>, verb: Verb, iri: &str) -> Result<String, Error> {
        let kernel = Kernel::new(space);
        let request = Request::new(verb, Iri::parse(iri).expect("iri"))
            .with_arg("in", ArgRef::Inline(b"hi".to_vec()));
        let representation = block_on(kernel.issue(request, &Capability::root()))?;
        Ok(String::from_utf8_lossy(&representation.bytes).to_string())
    }

    /// The point of a prefer-mount: the peer is unreachable, so this machine answers.
    #[test]
    fn a_prefer_mount_falls_back_to_the_local_binding_when_the_peer_is_down() {
        let (peer, calls) = dead(Error::Unavailable);
        let space = prefer(
            "urn:iki:tutorial:",
            "test://peer",
            peer,
            local_space(Arc::new(AtomicUsize::new(0))),
        );

        let answer = issue(space, Verb::Source, "urn:iki:tutorial:camel-case")
            .expect("local must answer for a dead peer");

        assert_eq!(answer, "answered locally");
        assert!(
            calls.load(Ordering::SeqCst) > 0,
            "the peer must be TRIED first — preferring it is the whole point"
        );
    }

    /// The same mount as an override fails instead. You named that machine; a silent
    /// local substitution would answer a question you did not ask.
    #[test]
    fn an_override_mount_fails_when_the_peer_is_down() {
        let (peer, _) = dead(Error::Unavailable);
        let space = compose(
            vec![(
                "urn:iki:tutorial:".to_string(),
                overriding("urn:iki:tutorial:", "test://peer", peer),
            )],
            local_space(Arc::new(AtomicUsize::new(0))),
        );

        let error = issue(Arc::new(space), Verb::Source, "urn:iki:tutorial:camel-case")
            .expect_err("an override must not fall back");

        assert!(matches!(error, Error::Unavailable(_)), "{error:?}");
    }

    /// A denial is not a transient failure. If the peer answered "you may not", the
    /// local binding must not quietly answer instead — that would turn a capability
    /// boundary into a suggestion.
    #[test]
    fn a_prefer_mount_does_not_swallow_a_denial() {
        let (peer, _) = dead(Error::Denied);
        let space = prefer(
            "urn:iki:tutorial:",
            "test://peer",
            peer,
            local_space(Arc::new(AtomicUsize::new(0))),
        );

        let error = issue(space, Verb::Source, "urn:iki:tutorial:camel-case")
            .expect_err("a denial must propagate");

        assert!(matches!(error, Error::Denied(_)), "{error:?}");
    }

    /// A `Sink` is not replayed on another target, even for a transient failure: the
    /// write may already have happened on the far side of a broken connection, and
    /// doing it again here is not the same as doing it once.
    #[test]
    fn a_prefer_mount_never_replays_a_sink() {
        let written = Arc::new(AtomicUsize::new(0));
        let (peer, _) = dead(Error::Unavailable);
        let space = prefer(
            "urn:iki:tutorial:",
            "test://peer",
            peer,
            local_space(Arc::clone(&written)),
        );

        let error = issue(space, Verb::Sink, "urn:iki:tutorial:camel-case")
            .expect_err("a mutating verb must not fall through");

        assert!(matches!(error, Error::Unavailable(_)), "{error:?}");
        assert_eq!(
            written.load(Ordering::SeqCst),
            0,
            "the local binding must not have been written to"
        );
    }

    /// ...and `Delete` *is*, which is not a contradiction: the rule is idempotence, not
    /// mutation. Deleting twice is deleting once, so a delete may safely be re-issued to
    /// the next target; writing twice is not writing once, so a `Sink` may not.
    #[test]
    fn a_prefer_mount_does_replay_a_delete_because_deleting_twice_is_deleting_once() {
        let deleted = Arc::new(AtomicUsize::new(0));
        let (peer, _) = dead(Error::Unavailable);
        let space = prefer(
            "urn:iki:tutorial:",
            "test://peer",
            peer,
            local_space(Arc::clone(&deleted)),
        );

        issue(space, Verb::Delete, "urn:iki:tutorial:camel-case").expect("delete falls through");

        assert_eq!(deleted.load(Ordering::SeqCst), 1);
    }

    /// A prefer-mount pairs its peer with the WHOLE local space, so without the prefix
    /// guard it would answer for every IRI — hitting before a mount behind it and
    /// silently defeating it.
    #[test]
    fn a_prefer_mount_does_not_claim_iris_outside_its_prefix() {
        let (preferred, prefer_calls) = dead(Error::Unavailable);
        let (overridden, override_calls) = dead(Error::Unavailable);
        let space = compose(
            vec![
                (
                    "urn:llm:".to_string(),
                    prefer(
                        "urn:llm:",
                        "test://a",
                        preferred,
                        local_space(Arc::new(AtomicUsize::new(0))),
                    ),
                ),
                (
                    "urn:iki:tutorial:".to_string(),
                    overriding("urn:iki:tutorial:", "test://b", overridden),
                ),
            ],
            local_space(Arc::new(AtomicUsize::new(0))),
        );

        let error = issue(Arc::new(space), Verb::Source, "urn:iki:tutorial:camel-case")
            .expect_err("the override still owns urn:iki:tutorial:");

        assert!(matches!(error, Error::Unavailable(_)), "{error:?}");
        assert_eq!(
            prefer_calls.load(Ordering::SeqCst),
            0,
            "the urn:llm: mount must not see a urn:iki:tutorial: request"
        );
        assert!(override_calls.load(Ordering::SeqCst) > 0);
    }
}
