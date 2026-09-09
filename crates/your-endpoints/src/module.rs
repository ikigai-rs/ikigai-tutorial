//! Your module, and the host that loads it — Part II's exercises are done here.
//!
//! The same shape as `crates/loadable-module`, which the book quotes: a module space
//! with one endpoint that calls back into the host mid-invocation, and a host whose root
//! space routes a prefix to that module through an in-process transport. The four
//! exercises in "Where this actually stands" each change one thing in this file and
//! watch which test moves.

use std::sync::Arc;

use ikigai_core::{
    AsyncFnEndpoint, Description, EndpointSpace, Error, Exact, Fallback, Invocation, InvokeFuture,
    Iri, Representation, Space, Verb,
};
use ikigai_module::{InProcessTransport, ModuleFloor, ModuleSpace};

use crate::text_plain_utf8;

/// The prefix the host routes to your module. Everything under it is the module's.
pub const MODULE_PREFIX: &str = "urn:iki:tutorial:yours:module:";

/// `urn:iki:tutorial:yours:module:greeting` — the module's one endpoint.
///
/// `name` is an IRI naming a resource the module does not own. `inv.source(..)` resolves
/// it through the **host's** kernel — the callback — while this invocation is still open.
pub fn greeting() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("greeting", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let target = inv.inline_str("name")?;
            let iri = Iri::parse(target).map_err(|e| Error::InvalidArgument {
                name: "name".into(),
                detail: format!("not a valid IRI: {e}"),
            })?;
            let resolved = inv.source(&iri).await?;
            let who = String::from_utf8_lossy(&resolved.bytes).trim().to_string();
            Ok(Representation::new(
                text_plain_utf8(),
                format!("Hello, {who}!").into_bytes(),
            ))
        })
    })
    .with_description(
        Description::new("greeting")
            .title("Greeting")
            .summary("Greets whoever the `name` resource resolves to.")
            .verb(Verb::Source)
            .verb(Verb::Meta),
    )
}

/// The module's space — what it offers, independent of any host.
pub fn module_space() -> EndpointSpace {
    EndpointSpace::new().bind(
        Exact::new("urn:iki:tutorial:yours:module:greeting"),
        greeting(),
    )
}

/// `urn:iki:tutorial:yours:name` — a resource the **host** owns. The module can reach it
/// only by asking.
pub fn name() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("name", |_inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move { Ok(Representation::new(text_plain_utf8(), b"Ada".to_vec())) })
    })
    .with_description(
        Description::new("name")
            .title("Who the host is")
            .verb(Verb::Source)
            .verb(Verb::Meta),
    )
}

/// The host's root space: your Part I endpoints, the host's own `name`, and a
/// [`ModuleSpace`] routing everything under [`MODULE_PREFIX`] to the module.
pub fn host_space() -> Fallback {
    let module = InProcessTransport::new(module_space());

    Fallback::new(vec![
        Arc::new(crate::space()) as Arc<dyn Space>,
        Arc::new(EndpointSpace::new().bind(Exact::new("urn:iki:tutorial:yours:name"), name()))
            as Arc<dyn Space>,
        // `ModuleFloor::public()`: no mount-wide floor. Your endpoints' own `requires`
        // are still enforced across the boundary — Part II's exercise 3 shows both.
        Arc::new(ModuleSpace::new(
            [MODULE_PREFIX],
            Arc::new(module),
            ModuleFloor::public(),
        )) as Arc<dyn Space>,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel;
    use futures::executor::block_on;
    use ikigai_core::{ArgRef, Capability, Error, Request};

    /// Resolve the module's greeting with `name` pointing at `name_iri`, under `capability`.
    fn greet(name_iri: &str, capability: &Capability) -> Result<String, Error> {
        let kernel = kernel();
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:yours:module:greeting").expect("iri"),
        )
        .with_arg("name", ArgRef::Inline(name_iri.as_bytes().to_vec()));
        let repr = block_on(kernel.issue(request, capability))?;
        Ok(String::from_utf8(repr.bytes).expect("utf-8"))
    }

    #[test]
    fn the_module_resolves_a_host_resource_mid_invocation() {
        assert_eq!(
            greet("urn:iki:tutorial:yours:name", &Capability::root()).expect("resolves"),
            "Hello, Ada!"
        );
    }

    #[test]
    fn a_name_outside_the_module_prefix_does_not_reach_it() {
        let kernel = kernel();
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:yours:nowhere").expect("iri"),
        );
        assert!(block_on(kernel.issue(request, &Capability::root())).is_err());
    }
}
