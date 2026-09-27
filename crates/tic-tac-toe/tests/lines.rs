//! The claims the second chapter makes about lines, aliases and the board, each one run.
//!
//! ```text
//! cargo test -p tic-tac-toe --test lines -- --nocapture --test-threads 1
//! ```
//!
//! As in `atoms.rs`, every test builds its own kernel over a `CellStore` it keeps, and the
//! store's read counter is how a test tells a served answer from a recomputed one.

use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Kernel, Request, Result, Space, Verb};
use tic_tac_toe::{cells_name, kernel_over, row_name, space, stored_name, CellStore, BOARD};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

fn source(kernel: &Kernel, name: &str) -> Result<String> {
    let repr = block_on(kernel.issue(Request::new(Verb::Source, iri(name)), &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn play(kernel: &Kernel, x: i64, y: i64, mark: &str) -> Result<()> {
    let request = Request::new(Verb::Sink, iri(&stored_name(x, y)))
        .with_arg("content", ArgRef::Inline(mark.as_bytes().to_vec()));
    block_on(kernel.issue(request, &Capability::root()))?;
    Ok(())
}

fn cached(kernel: &Kernel, name: &str) -> bool {
    kernel.is_cached(&Request::new(Verb::Source, iri(name)), &Capability::root())
}

const ROW_0: &str = "urn:iki:tutorial:ttt:cells:0.0,1.0,2.0";
const ROW_1: &str = "urn:iki:tutorial:ttt:cells:0.1,1.1,2.1";
const ROW_2: &str = "urn:iki:tutorial:ttt:cells:0.2,1.2,2.2";
const COLUMN_0: &str = "urn:iki:tutorial:ttt:cells:0.0,0.1,0.2";
const COLUMN_1: &str = "urn:iki:tutorial:ttt:cells:1.0,1.1,1.2";
const COLUMN_2: &str = "urn:iki:tutorial:ttt:cells:2.0,2.1,2.2";
const DIAGONAL_0: &str = "urn:iki:tutorial:ttt:cells:0.0,1.1,2.2";
const DIAGONAL_1: &str = "urn:iki:tutorial:ttt:cells:2.0,1.1,0.2";

#[test]
fn a_line_reads_its_members_in_the_order_it_names_them() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    play(&kernel, 0, 0, "X")?;
    play(&kernel, 2, 0, "O")?;
    assert_eq!(
        source(&kernel, &cells_name(&[(0, 0), (1, 0), (2, 0)]))?,
        "X-O"
    );
    // A sequence, not a set: the same cells the other way round are another line.
    assert_eq!(
        source(&kernel, &cells_name(&[(2, 0), (1, 0), (0, 0)]))?,
        "O-X"
    );
    Ok(())
}

#[test]
fn a_line_is_any_cells_of_any_length() -> Result<()> {
    // Loose, like the cell: nothing here knows the board is three by three.
    let kernel = kernel_over(Arc::default());
    play(&kernel, -4, 9, "R")?;
    assert_eq!(source(&kernel, &cells_name(&[(-4, 9)]))?, "R");
    assert_eq!(
        source(&kernel, &cells_name(&[(1, 1), (-4, 9), (1, 1), (7, 7)]))?,
        "-R--"
    );
    Ok(())
}

#[test]
fn a_line_has_one_spelling() {
    // Two names for one line would be two cache entries and two threads over one answer.
    let kernel = kernel_over(Arc::default());
    for list in [
        "0.0,1.0,", // a trailing comma
        "0.0,,1.0", // an empty member
        ",0.0",     // a leading comma
        "0.0, 1.0", // a space
        "0.0,01.0", // a coordinate in a second spelling
        "0.0,+1.0", // and another
        "0.0.0",    // three numbers in one member
        "0",        // one number
        "0:0",      // the cell's own separator, not the line's
    ] {
        let name = format!("urn:iki:tutorial:ttt:cells:{list}");
        // A space does not parse as an IRI at all; everything else reaches the endpoint.
        let Ok(target) = Iri::parse(&name) else {
            continue;
        };
        let err = block_on(kernel.issue(Request::new(Verb::Source, target), &Capability::root()))
            .expect_err(&name);
        assert!(
            matches!(err, Error::InvalidArgument { .. }),
            "{name}: {err:?}"
        );
    }
}

// ANCHOR: one_entry
/// An alias is a second NAME, not a second resource: the row and the line it names are
/// one cache entry. The second read runs nothing at all.
#[test]
fn a_row_and_its_line_are_one_cache_entry() -> Result<()> {
    let store = Arc::new(CellStore::default());
    let kernel = kernel_over(store.clone());

    assert_eq!(source(&kernel, &row_name(1))?, "---");
    let reads = store.reads.load(Ordering::SeqCst);
    assert!(cached(&kernel, ROW_1), "cached under the line's own name");

    assert_eq!(source(&kernel, ROW_1)?, "---");
    assert_eq!(
        store.reads.load(Ordering::SeqCst),
        reads,
        "served, not recomputed"
    );
    Ok(())
}
// ANCHOR_END: one_entry

#[test]
fn every_alias_answers_what_its_line_answers() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    play(&kernel, 0, 0, "X")?;
    play(&kernel, 1, 1, "O")?;
    play(&kernel, 0, 2, "X")?;
    for (alias, line, marks) in [
        ("row:0", ROW_0, "X--"),
        ("row:1", ROW_1, "-O-"),
        ("row:2", ROW_2, "X--"),
        ("column:0", COLUMN_0, "X-X"),
        ("column:1", COLUMN_1, "-O-"),
        ("column:2", COLUMN_2, "---"),
        ("diagonal:0", DIAGONAL_0, "XO-"),
        ("diagonal:1", DIAGONAL_1, "-OX"),
    ] {
        let alias = format!("urn:iki:tutorial:ttt:{alias}");
        assert_eq!(source(&kernel, &alias)?, marks, "{alias}");
        assert_eq!(source(&kernel, line)?, marks, "{line}");
    }
    Ok(())
}

#[test]
fn the_board_draws_row_zero_at_the_top() -> Result<()> {
    // `y` runs down: (2,0) is the top-right corner and (0,2) the bottom-left.
    let kernel = kernel_over(Arc::default());
    play(&kernel, 2, 0, "X")?;
    play(&kernel, 0, 2, "O")?;
    assert_eq!(source(&kernel, BOARD)?, "--X\n---\nO--");
    Ok(())
}

// ANCHOR: normalized
/// One move recomputes the cell, the lines through it and the board — and nothing else.
#[test]
fn one_move_recomputes_only_what_passes_through_it() -> Result<()> {
    let store = Arc::new(CellStore::default());
    let kernel = kernel_over(store.clone());

    // Read the board and every line once: all of it computed, all of it cached.
    source(&kernel, BOARD)?;
    for line in [COLUMN_0, COLUMN_1, COLUMN_2, DIAGONAL_0, DIAGONAL_1] {
        source(&kernel, line)?;
    }

    // Play the top-left corner. It is on one row, one column and one diagonal.
    play(&kernel, 0, 0, "X")?;
    for through in [ROW_0, COLUMN_0, DIAGONAL_0, BOARD] {
        assert!(!cached(&kernel, through), "{through} passes through (0,0)");
    }
    for past in [ROW_1, ROW_2, COLUMN_1, COLUMN_2, DIAGONAL_1] {
        assert!(cached(&kernel, past), "{past} does not");
    }

    // Read the board again. Of the nine stored cells, exactly one is read again: the
    // corner. Row 0's other two cells, and rows 1 and 2 whole, are served from cache.
    let reads = store.reads.load(Ordering::SeqCst);
    assert_eq!(source(&kernel, BOARD)?, "X--\n---\n---");
    assert_eq!(store.reads.load(Ordering::SeqCst), reads + 1);
    Ok(())
}
// ANCHOR_END: normalized

#[test]
fn the_catalog_lists_the_lines_template_once_and_no_alias() {
    // `Alias` is transparent to enumeration (core's decision 4): the space lists what is
    // BOUND — `cells:{list}` and `board` — and the eight aliases are data in its table,
    // visible in the topology rather than as eight more catalog entries.
    let patterns: Vec<String> = space()
        .entries()
        .unwrap_or_default()
        .into_iter()
        .map(|entry| entry.pattern)
        .collect();
    assert!(
        patterns.iter().any(|p| p == tic_tac_toe::CELLS),
        "{patterns:?}"
    );
    assert!(patterns.iter().any(|p| p == BOARD), "{patterns:?}");
    assert!(
        !patterns.iter().any(|p| p.contains(":row:")),
        "{patterns:?}"
    );
}
