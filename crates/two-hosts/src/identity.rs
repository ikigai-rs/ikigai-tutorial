//! Who is asking — minting one connection's authority from the certificate that
//! authenticated it.
//!
//! Nothing here dials anything. A [`Minter`] is a decision function, and the two claims
//! the chapter makes about it — an unenrolled certificate is *refused* rather than given
//! a default, and a capability carried over the wire can only narrow — are both
//! decidable with no network at all. That is why they are tested here and not described.

use std::collections::BTreeMap;
use std::sync::Arc;

use ikigai_core::Capability;
use ikigai_quic::{Minter, PeerIdentity, Session};

// ANCHOR: minter
/// A minter over an enrolment: certificate fingerprint → the capability scopes that
/// certificate is granted.
///
/// The `?` is the whole security posture. An unenrolled fingerprint makes the closure
/// return `None`, and `None` **refuses the connection** — the server closes it with a
/// distinct `UNAUTHORIZED` code rather than serving the request under some fallback.
/// There is deliberately no `unwrap_or(everyone)` here: a host that cannot decide what a
/// certificate may do must not fall back to a shared ceiling or to root.
pub fn minter(enrolled: BTreeMap<String, Vec<String>>) -> Minter {
    Arc::new(move |peer: &PeerIdentity| {
        let scopes = enrolled.get(&peer.fingerprint)?;
        Some(Session {
            capability: Capability::root().attenuate(scopes.iter().cloned()),
            // The peer's own namespace segment: its `urn:file:` names land inside it, so
            // one client cannot address another's workspace even by guessing the path.
            file_segment: peer.segment_id.clone(),
        })
    })
}
// ANCHOR_END: minter

/// A `PeerIdentity` as the handshake would hand one over: the stable fingerprint an
/// operator enrols, and the namespace segment this connection's files live under.
///
/// Both ids are public and this is a plain struct, which is what lets an authority
/// decision be tested without a certificate, a socket or a running peer.
pub fn peer(fingerprint: &str) -> PeerIdentity {
    PeerIdentity {
        segment_id: format!("segment-{fingerprint}"),
        fingerprint: fingerprint.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enrolment() -> BTreeMap<String, Vec<String>> {
        BTreeMap::from([(
            "aa01".to_string(),
            vec!["urn:cap:kernel:inspect".to_string()],
        )])
    }

    #[test]
    fn an_enrolled_certificate_gets_exactly_the_scopes_it_was_granted() {
        let session = minter(enrolment())(&peer("aa01")).expect("enrolled");
        assert!(session.capability.allows("urn:cap:kernel:inspect"));
        assert!(!session.capability.allows("urn:cap:fs:write:/"));
    }

    #[test]
    fn an_unenrolled_certificate_is_refused_rather_than_given_a_default() {
        // It authenticated — mutual TLS pinned it, or it would not have reached the
        // minter at all. It still gets nothing, because nobody said what it may do.
        assert!(minter(enrolment())(&peer("bb02")).is_none());
    }

    #[test]
    fn a_capability_carried_over_the_wire_can_only_narrow() {
        let session = minter(enrolment())(&peer("aa01")).expect("enrolled");

        // The server resolves under `session.capability.clamp(&carried)`. A peer that
        // asks for more than it holds gets what it holds, not what it asked for.
        let greedy = Capability::root();
        assert!(session
            .capability
            .clamp(&greedy)
            .allows("urn:cap:kernel:inspect"));
        let overreaching = Capability::scoped(["urn:cap:fs:write:/"]);
        assert!(!session
            .capability
            .clamp(&overreaching)
            .allows("urn:cap:fs:write:/"));

        // Narrowing works, which is the point of carrying one at all: a client handing
        // work to an agent can spend less than it holds.
        let narrowed = Capability::scoped(["urn:cap:kernel:inspect"]);
        assert!(session
            .capability
            .clamp(&narrowed)
            .allows("urn:cap:kernel:inspect"));
    }
}
