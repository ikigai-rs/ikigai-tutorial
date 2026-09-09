//! Your host, from a shell.
//!
//! ```text
//! cargo run -p your-endpoints -- "a few words here"
//! cargo run -p your-endpoints -- --catalog
//! ```
//!
//! Point `NAME` at whatever you bound in `space()` to see it answer.

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

/// The name this binary resolves. Change it to yours.
const NAME: &str = "urn:iki:tutorial:yours:word-count";

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let kernel = your_endpoints::kernel();

    let (name, request) = if args.iter().any(|a| a == "--catalog") {
        (
            "urn:kernel:catalog",
            Request::new(
                Verb::Source,
                Iri::parse("urn:kernel:catalog").expect("valid IRI"),
            ),
        )
    } else {
        let text = args.join(" ");
        let text = if text.is_empty() {
            "a few words here".to_string()
        } else {
            text
        };
        println!("in  {text}");
        (
            NAME,
            Request::new(Verb::Source, Iri::parse(NAME).expect("valid IRI"))
                .with_arg("in", ArgRef::Inline(text.into_bytes())),
        )
    };

    match block_on(kernel.issue(request, &Capability::root())) {
        Ok(repr) => print!(
            "{}{}",
            if name == NAME { "out " } else { "" },
            String::from_utf8_lossy(&repr.bytes)
        ),
        Err(e) => {
            eprintln!("could not resolve {name}: {e}");
            std::process::exit(1);
        }
    }
    if name == NAME {
        println!();
    }
}
