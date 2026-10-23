//! Print a view's graph (RDF/XML) and its fragment: `cargo run -p inspector --example dump -- cache`.
use futures::executor::block_on;
use ikigai_core::{Capability, Iri, Request, Verb};
fn main() {
    let kernel = inspector::kernel(Vec::new());
    let root = Capability::root();
    let read = |iri: &str| {
        block_on(kernel.issue(Request::new(Verb::Source, Iri::parse(iri).unwrap()), &root))
    };
    read("urn:iki:tutorial:inspect:demo:card").unwrap();
    for arg in std::env::args().skip(1) {
        for iri in [
            format!("urn:iki:tutorial:inspect:graph:{arg}"),
            format!("urn:iki:tutorial:inspect:{arg}"),
        ] {
            match read(&iri) {
                Ok(r) => println!(
                    "=== {iri} ({})\n{}",
                    r.repr_type.media_type,
                    String::from_utf8_lossy(&r.bytes)
                ),
                Err(e) => println!("=== {iri} ERR {e}"),
            }
        }
    }
}
