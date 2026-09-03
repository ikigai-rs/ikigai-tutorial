//! A host of your own — about thirty lines.
//!
//! Run it:
//!
//! ```text
//! cargo run -p ikigai-book-getting-started -- "resource oriented computing"
//! ```

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request, Verb};

fn main() {
    let text = std::env::args().skip(1).collect::<Vec<_>>().join(" ");
    let text = if text.is_empty() {
        "resource oriented computing".to_string()
    } else {
        text
    };

    // A kernel is a root space plus the machinery around it. `space()` here is the
    // library's own bindings chained onto ikigai-fn's — see chapter 5.
    let kernel = Kernel::new(Arc::new(getting_started::space()));

    let request = Request::new(
        Verb::Source,
        Iri::parse("urn:fn:toCamel").expect("valid IRI"),
    )
    .with_arg("in", ArgRef::Inline(text.clone().into_bytes()));

    match block_on(kernel.issue(request, &Capability::root())) {
        Ok(repr) => {
            println!("in  {text}");
            println!("out {}", String::from_utf8_lossy(&repr.bytes));
        }
        Err(e) => {
            eprintln!("could not resolve urn:fn:toCamel: {e}");
            std::process::exit(1);
        }
    }
}
