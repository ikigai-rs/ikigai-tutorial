//! The Turtle faces "Checking a resource" publishes beside itself, for the one case the page's
//! own kernel cannot serve: a reader whose browser did not load it. Then the chapter's "check
//! this resource" link opens the RDF Toolbox in a new tab on these files
//! (`books/ikigai/src/building/check/*.ttl`, which mdbook copies into the built book), so they
//! must be exactly what the kernel answers a reader who holds the scope. This test is what
//! keeps them so.
//!
//! cargo test -p check-resource --test published_faces
//! UPDATE_FACES=1 cargo test -p check-resource --test published_faces   # rewrite them

use std::path::PathBuf;

use futures::executor::block_on;
use ikigai_core::Capability;

use check_resource::{kernel, turtle_face, PLAYER_O, PLAYER_X, READ, SHAPES};

fn published(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../books/ikigai/src/building/check")
        .join(name)
}

#[test]
fn the_published_faces_are_what_a_reader_with_the_scope_reads() {
    let kernel = kernel();
    let reader = Capability::scoped([READ]);
    let update = std::env::var_os("UPDATE_FACES").is_some();
    for (iri, file) in [
        (PLAYER_X, "player-x.ttl"),
        (PLAYER_O, "player-o.ttl"),
        (SHAPES, "shapes.ttl"),
    ] {
        let face =
            block_on(turtle_face(&kernel, iri, &reader)).unwrap_or_else(|e| panic!("{iri}: {e}"));
        let path = published(file);
        if update {
            std::fs::write(&path, &face).unwrap_or_else(|e| panic!("{}: {e}", path.display()));
            continue;
        }
        let on_disk = std::fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!(
                "{}: {e} (UPDATE_FACES=1 cargo test -p check-resource --test published_faces)",
                path.display()
            )
        });
        assert_eq!(
            on_disk,
            face,
            "{} is not {iri}'s Turtle face any more; rewrite it with UPDATE_FACES=1",
            path.display()
        );
    }
}
