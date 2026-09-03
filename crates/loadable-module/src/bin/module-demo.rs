//! Book 2's worked example: a host resolving into a module, and the module resolving
//! back into the host.
//!
//! ```text
//! cargo run -p loadable-module
//! ```

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request, Verb};

fn main() {
    let kernel = Kernel::new(Arc::new(loadable_module::host_space()));

    // `urn:greet:hello` is bound by nobody in the host. The ModuleSpace matches the
    // `urn:greet:` prefix and hands the request to the module.
    let request = Request::new(
        Verb::Source,
        Iri::parse("urn:greet:hello").expect("valid IRI"),
    )
    .with_arg("name", ArgRef::Inline(b"urn:host:name".to_vec()));

    match block_on(kernel.issue(request, &Capability::root())) {
        Ok(repr) => {
            println!("host    resolves urn:greet:hello name=urn:host:name");
            println!("module  asks the host for urn:host:name");
            println!("out     {}", String::from_utf8_lossy(&repr.bytes));
        }
        Err(e) => {
            eprintln!("could not resolve: {e}");
            std::process::exit(1);
        }
    }
}
