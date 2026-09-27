//! The claims the chapter makes about the two cells, each one run.
//!
//! ```text
//! cargo test -p tic-tac-toe --test atoms -- --nocapture --test-threads 1
//! ```
//!
//! Every test builds its own kernel over a `CellStore` it keeps: the cache and the golden
//! threads live in the kernel, and the store's read counter is how a test tells an answer
//! that was served from one that was computed again.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Kernel, Request, Result, Verb};
use tic_tac_toe::{cell_name, kernel_over, stored_name, CellStore, EMPTY};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

fn source(kernel: &Kernel, name: &str) -> Result<String> {
    let repr = block_on(kernel.issue(Request::new(Verb::Source, iri(name)), &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn play(kernel: &Kernel, name: &str, mark: &str) -> Result<()> {
    let request = Request::new(Verb::Sink, iri(name))
        .with_arg("content", ArgRef::Inline(mark.as_bytes().to_vec()));
    block_on(kernel.issue(request, &Capability::root()))?;
    Ok(())
}

fn clear(kernel: &Kernel, name: &str) -> Result<()> {
    block_on(kernel.issue(Request::new(Verb::Delete, iri(name)), &Capability::root()))?;
    Ok(())
}

fn cached(kernel: &Kernel, name: &str) -> bool {
    kernel.is_cached(&Request::new(Verb::Source, iri(name)), &Capability::root())
}

#[test]
fn an_unplayed_stored_cell_is_not_found_and_a_played_one_answers_its_mark() -> Result<()> {
    let store = Arc::new(CellStore::default());
    let kernel = kernel_over(store.clone());
    let stored = stored_name(1, 1);

    // Bound, and empty: the kernel resolved the name and the endpoint said "not here".
    // That is not `Unresolved`, which would mean no endpoint answers to the name at all.
    assert!(matches!(source(&kernel, &stored), Err(Error::NotFound(_))));

    play(&kernel, &stored, "X")?;
    assert_eq!(source(&kernel, &stored)?, "X");

    clear(&kernel, &stored)?;
    assert!(matches!(source(&kernel, &stored), Err(Error::NotFound(_))));
    Ok(())
}

// ANCHOR: platonic_cut
/// The chapter's first lesson. The platonic cell's empty answer is cached; playing the
/// cell invalidates it; clearing the cell invalidates the mark. Nothing in either
/// endpoint names a thread.
#[test]
fn the_cached_empty_cell_is_cut_by_the_move_that_fills_it() -> Result<()> {
    let store = Arc::new(CellStore::default());
    let kernel = kernel_over(store.clone());
    let (cell, stored) = (cell_name(1, 1), stored_name(1, 1));

    // Empty, computed once and then served: the second read runs no code at all.
    assert_eq!(source(&kernel, &cell)?, EMPTY);
    assert_eq!(source(&kernel, &cell)?, EMPTY);
    assert_eq!(store.reads.load(Ordering::SeqCst), 1);
    assert!(cached(&kernel, &cell));

    // The stored cell's read FAILED — and the failure is still a dependency. So the
    // Sink to the stored cell cuts the thread the cached empty answer hangs from.
    play(&kernel, &stored, "X")?;
    assert!(!cached(&kernel, &cell));
    assert_eq!(source(&kernel, &cell)?, "X");

    // And the other way: the Delete cuts the thread the cached mark hangs from.
    clear(&kernel, &stored)?;
    assert!(!cached(&kernel, &cell));
    assert_eq!(source(&kernel, &cell)?, EMPTY);
    println!("played and cleared: the cell recomputed each time, and only then");
    Ok(())
}
// ANCHOR_END: platonic_cut

#[test]
fn a_move_elsewhere_leaves_this_cells_cached_answer_alone() -> Result<()> {
    // The thread is per cell: playing (0,0) does not disturb a cached (1,1).
    let store = Arc::new(CellStore::default());
    let kernel = kernel_over(store.clone());
    assert_eq!(source(&kernel, &cell_name(1, 1))?, EMPTY);
    play(&kernel, &stored_name(0, 0), "O")?;
    assert!(cached(&kernel, &cell_name(1, 1)));
    assert_eq!(source(&kernel, &cell_name(1, 1))?, EMPTY);
    assert_eq!(store.reads.load(Ordering::SeqCst), 1);
    Ok(())
}

#[test]
fn the_model_is_loose_any_integer_is_a_cell() -> Result<()> {
    // No 0..=2 in the grammar: a board's edges are a rule, and rules come later.
    let kernel = kernel_over(Arc::default());
    for (x, y) in [(-1, 0), (3, 7), (i64::MAX, i64::MIN)] {
        assert_eq!(source(&kernel, &cell_name(x, y))?, EMPTY);
        play(&kernel, &stored_name(x, y), "R")?;
        assert_eq!(source(&kernel, &cell_name(x, y))?, "R");
    }
    Ok(())
}

#[test]
fn one_integer_has_one_spelling() {
    // Two names over one piece of state would be two threads, and a write through one
    // would leave the other's cached read stale — so the second spelling is refused.
    let kernel = kernel_over(Arc::default());
    for name in [
        "urn:iki:tutorial:ttt:cell:01:0",
        "urn:iki:tutorial:ttt:cell:+1:0",
        "urn:iki:tutorial:ttt:cell:-0:0",
        "urn:iki:tutorial:ttt:cell:a:0",
        "urn:iki:tutorial:ttt:stored:1:0:0",
    ] {
        let err = source(&kernel, name).expect_err(name);
        assert!(
            matches!(err, Error::InvalidArgument { .. }),
            "{name}: {err:?}"
        );
    }
}

#[test]
fn the_platonic_cell_swallows_only_not_found() {
    // Only "nothing has been played" means empty. Put a stored cell that fails some other
    // way under the platonic cell, and the failure comes through rather than a `-`.
    use ikigai_core::{EndpointSpace, FnEndpoint, UriTemplate};
    let broken = FnEndpoint::new("broken", |_inv: &ikigai_core::Invocation<'_>| {
        Err(Error::Unavailable("the store is down".to_string()))
    });
    let space = EndpointSpace::new()
        .bind(UriTemplate::parse(tic_tac_toe::STORED).unwrap(), broken)
        .bind(
            UriTemplate::parse(tic_tac_toe::CELL).unwrap(),
            tic_tac_toe::platonic_cell(),
        );
    let kernel = Kernel::new(Arc::new(space));
    let err = source(&kernel, &cell_name(0, 0)).expect_err("not answered empty");
    assert!(matches!(err, Error::Unavailable(_)), "{err:?}");
}

#[test]
fn a_mark_is_trimmed_and_an_empty_one_is_refused() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    // A piped value keeps its trailing newline; the stored mark does not.
    play(&kernel, &stored_name(0, 0), "X\n")?;
    assert_eq!(source(&kernel, &cell_name(0, 0))?, "X");
    let err = play(&kernel, &stored_name(0, 1), "  ").expect_err("refused");
    assert!(matches!(err, Error::InvalidArgument { .. }), "{err:?}");
    Ok(())
}

#[test]
fn both_cells_describe_themselves() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    for (name, id) in [
        (stored_name(0, 0), "ttt-stored"),
        (cell_name(0, 0), "ttt-cell"),
    ] {
        let meta =
            block_on(kernel.issue(Request::new(Verb::Meta, iri(&name)), &Capability::root()))?;
        let turtle = String::from_utf8_lossy(&meta.bytes);
        assert!(
            turtle.contains(&format!("<urn:ikigai:endpoint:{id}> a ik:Endpoint")),
            "{turtle}"
        );
    }
    Ok(())
}
