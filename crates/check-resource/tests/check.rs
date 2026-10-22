//! The chapter's claims, outside the book's listings, so `cargo test` holds them too.
//!
//! cargo test -p check-resource --test check

use futures::executor::block_on;
use ikigai_core::{Capability, Error};

use check_resource::{check, kernel, turtle_face, PLAYER_O, PLAYER_X, READ, SHAPES};

#[test]
fn a_reader_gets_the_turtle_face_and_a_stranger_is_refused() {
    let kernel = kernel();
    let reader = Capability::scoped([READ]);
    let face = block_on(turtle_face(&kernel, PLAYER_X, &reader)).unwrap();
    assert!(
        face.contains("@prefix ttt: <http://example.org/ttt#>"),
        "{face}"
    );
    assert!(face.contains("ttt:name \"Ada\""), "{face}");

    let stranger = Capability::scoped(["urn:cap:tutorial:counter:write"]);
    let refused = block_on(turtle_face(&kernel, PLAYER_X, &stranger)).unwrap_err();
    assert!(matches!(refused, Error::Denied(_)), "{refused:?}");
}

#[test]
fn x_conforms_and_o_breaks_the_closed_shape() {
    let kernel = kernel();
    let reader = Capability::scoped([READ]);
    let x = block_on(check(&kernel, PLAYER_X, SHAPES, &reader)).unwrap();
    assert!(x.conforms, "{x:?}");

    let o = block_on(check(&kernel, PLAYER_O, SHAPES, &reader)).unwrap();
    assert!(!o.conforms);
    let outcome = o.outcome();
    assert_eq!(outcome.violations.len(), 1, "{o:?}");
    let v = &outcome.violations[0];
    assert_eq!(v.focus_node, PLAYER_O);
    assert_eq!(v.path.as_deref(), Some("http://example.org/ttt#wins"));
    assert_eq!(
        v.component,
        "http://www.w3.org/ns/shacl#ClosedConstraintComponent"
    );
}
