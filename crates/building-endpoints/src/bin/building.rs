//! The Building endpoints host, from a shell.
//!
//! ```text
//! cargo run -p building-endpoints -- --markdown    # Meta on camel-case, as Markdown
//! cargo run -p building-endpoints -- --strings     # SPARQL: which endpoints take a string?
//! cargo run -p building-endpoints -- --banner      # the configured banner text
//! ```

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let mode = args.first().map(String::as_str).unwrap_or("--markdown");
    let kernel = building_endpoints::kernel();
    let root = Capability::root();

    let outcome = match mode {
        "--markdown" => {
            let request = Request::new(
                Verb::Meta,
                Iri::parse("urn:iki:tutorial:camel-case").expect("valid IRI"),
            )
            .with_arg("as", ArgRef::Inline(b"text/markdown".to_vec()));
            block_on(kernel.issue(request, &root))
                .map(|repr| String::from_utf8_lossy(&repr.bytes).into_owned())
        }
        "--strings" => block_on(building_endpoints::graph::endpoints_taking(
            &kernel,
            "http://www.w3.org/2001/XMLSchema#string",
        ))
        .map(|ids| ids.join("\n") + "\n"),
        "--banner" => {
            let request = Request::new(
                Verb::Source,
                Iri::parse("urn:iki:tutorial:banner").expect("valid IRI"),
            );
            block_on(kernel.issue(request, &root))
                .map(|repr| String::from_utf8_lossy(&repr.bytes).into_owned() + "\n")
        }
        other => {
            eprintln!("unknown mode {other}; try --markdown, --strings or --banner");
            std::process::exit(2);
        }
    };

    match outcome {
        Ok(text) => print!("{text}"),
        Err(e) => {
            eprintln!("could not resolve: {e}");
            std::process::exit(1);
        }
    }
}
