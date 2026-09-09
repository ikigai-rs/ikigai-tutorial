//! A host of your own — about thirty lines.
//!
//! Run it:
//!
//! ```text
//! cargo run -p hello-camel -- "resource oriented computing"
//! cargo run -p hello-camel -- --catalog
//! ```

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();

    // A kernel is a root space plus the machinery around it. `kernel()` is `space()` —
    // the library's own bindings chained onto ikigai-fn's — under a kernel that has been
    // given a Meta renderer, which is what lets it describe itself. See the book's
    // "Binding, and a host of your own".
    let kernel = hello_camel::kernel();

    // `--catalog`: the host describing itself, as one Turtle graph.
    if args.iter().any(|a| a == "--catalog") {
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:kernel:catalog").expect("valid IRI"),
        );
        match block_on(kernel.issue(request, &Capability::root())) {
            Ok(repr) => print!("{}", String::from_utf8_lossy(&repr.bytes)),
            Err(e) => {
                eprintln!("could not resolve urn:kernel:catalog: {e}");
                std::process::exit(1);
            }
        }
        return;
    }

    let text = args.join(" ");
    let text = if text.is_empty() {
        "resource oriented computing".to_string()
    } else {
        text
    };

    let request = Request::new(
        Verb::Source,
        Iri::parse("urn:iki:tutorial:camel-case").expect("valid IRI"),
    )
    .with_arg("in", ArgRef::Inline(text.clone().into_bytes()));

    match block_on(kernel.issue(request, &Capability::root())) {
        Ok(repr) => {
            println!("in  {text}");
            println!("out {}", String::from_utf8_lossy(&repr.bytes));
        }
        Err(e) => {
            eprintln!("could not resolve urn:iki:tutorial:camel-case: {e}");
            std::process::exit(1);
        }
    }
}
