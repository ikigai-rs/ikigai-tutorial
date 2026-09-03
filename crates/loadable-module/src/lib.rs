//! The code taught by **Book 2 — Loadable Modules**.
//!
//! A module and a host, and the callback that distinguishes a module from a remote peer.
//! The book pulls these snippets in by anchor; this is the code that compiles.
//!
//! Read the book: `mdbook serve books/loadable-modules --open`

use std::sync::Arc;

use ikigai_core::{
    AsyncFnEndpoint, Description, Endpoint, EndpointSpace, Error, Exact, Fallback, Invocation,
    InvokeFuture, Iri, ReprType, Representation, Space, Verb,
};
use ikigai_module::{InProcessTransport, ModuleSpace};

fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

// ANCHOR: module_endpoint
/// `urn:greet:hello` — the module's one endpoint.
///
/// It takes a `name` argument that is *an IRI naming another resource*, and resolves it.
/// That resolution is the whole point of this book: the module does not own the resource,
/// cannot see it, and has no catalog of its own to find it in. `inv.source(…)` crosses
/// back into the **host's** kernel to get it.
pub fn hello() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("hello", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let target = inv.inline_str("name")?;
            let iri = Iri::parse(target).map_err(|e| Error::InvalidArgument {
                name: "name".into(),
                detail: format!("not a valid IRI: {e}"),
            })?;

            // ← THE CALLBACK. This is a resolution against the host, from inside the
            //   module, in the middle of the module's own invocation.
            let resolved = inv.source(&iri).await?;
            let who = String::from_utf8_lossy(&resolved.bytes).trim().to_string();

            let greeting = format!("Hello, {who}!");
            Ok(Representation::new(
                text_plain_utf8(),
                greeting.into_bytes(),
            ))
        })
    })
    .with_description(
        Description::new("hello")
            .title("Greet")
            .summary("Greets whoever the `name` resource resolves to.")
            .verb(Verb::Source)
            .verb(Verb::Meta),
    )
}

/// The module's space — everything it offers, independent of any host.
pub fn module_space() -> EndpointSpace {
    EndpointSpace::new().bind(Exact::new("urn:greet:hello"), hello())
}
// ANCHOR_END: module_endpoint

// ANCHOR: host
/// A resource the **host** owns. The module cannot reach this except by asking.
pub fn host_name() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("host-name", |_inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move { Ok(Representation::new(text_plain_utf8(), b"Peter".to_vec())) })
    })
    .with_description(
        Description::new("host-name")
            .title("Who the host is")
            .verb(Verb::Source)
            .verb(Verb::Meta),
    )
}

/// The host's root space: its own resources, plus a [`ModuleSpace`] that routes
/// everything under `urn:greet:` to the module.
///
/// `ModuleSpace` implements `Space`, so it sits in the `Fallback` exactly where a
/// statically linked `space()` would — the host does not have a special case for
/// "modules", it has one more space.
pub fn host_space() -> Fallback {
    let module = InProcessTransport::new(module_space());

    Fallback::new(vec![
        Arc::new(EndpointSpace::new().bind(Exact::new("urn:host:name"), host_name()))
            as Arc<dyn Space>,
        Arc::new(ModuleSpace::new(["urn:greet:"], Arc::new(module))) as Arc<dyn Space>,
    ])
}
// ANCHOR_END: host

/// The module's endpoint, described — proof it is reachable through the host.
pub fn module_endpoint_id() -> String {
    hello().describe().id.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use ikigai_core::{ArgRef, Capability, Kernel, Request};

    fn greet(name_iri: &str) -> String {
        let kernel = Kernel::new(Arc::new(host_space()));
        let request = Request::new(Verb::Source, Iri::parse("urn:greet:hello").expect("iri"))
            .with_arg("name", ArgRef::Inline(name_iri.as_bytes().to_vec()));
        let repr = block_on(kernel.issue(request, &Capability::root())).expect("resolves");
        String::from_utf8(repr.bytes).expect("utf-8")
    }

    #[test]
    fn the_module_resolves_a_host_resource_mid_invocation() {
        // The module returned a greeting it could only build by asking the host who it
        // is. This single assertion is the difference between a module and a peer.
        assert_eq!(greet("urn:host:name"), "Hello, Peter!");
    }

    #[test]
    fn the_host_routes_by_prefix_not_by_knowing_the_endpoint() {
        // The host bound no `urn:greet:hello`. ModuleSpace matched the prefix.
        let kernel = Kernel::new(Arc::new(host_space()));
        let request = Request::new(Verb::Source, Iri::parse("urn:host:name").expect("iri"));
        let repr = block_on(kernel.issue(request, &Capability::root())).expect("resolves");
        assert_eq!(String::from_utf8_lossy(&repr.bytes), "Peter");
    }

    #[test]
    fn a_name_outside_the_module_prefix_does_not_reach_it() {
        let kernel = Kernel::new(Arc::new(host_space()));
        let request = Request::new(Verb::Source, Iri::parse("urn:nowhere:x").expect("iri"));
        assert!(block_on(kernel.issue(request, &Capability::root())).is_err());
    }
}
