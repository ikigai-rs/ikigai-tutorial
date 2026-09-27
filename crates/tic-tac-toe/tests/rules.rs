//! The claims the third chapter makes about the rules and the move, each one run.
//!
//! ```text
//! cargo test -p tic-tac-toe --test rules -- --nocapture --test-threads 1
//! ```
//!
//! As in `atoms.rs` and `lines.rs`, every test builds its own kernel over a `CellStore` it
//! keeps. Cache probes use LINE names (`cells:…`), never alias names: `cache` does not
//! resolve, so a name only the game's space rewrites reads "not cached" while its entry is
//! served (ledger #561).

use std::sync::atomic::Ordering;
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Endpoint, Error, Iri, Kernel, Request, Result, Verb};
use tic_tac_toe::{
    cell_name, checkset_name, kernel_over, move_name, play_move, stored_name, CellStore, BOARD,
    DRAW, EMPTY, TURN, WINNER,
};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

fn source(kernel: &Kernel, name: &str) -> Result<String> {
    let repr = block_on(kernel.issue(Request::new(Verb::Source, iri(name)), &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn sink(kernel: &Kernel, name: &str, content: &str) -> Result<String> {
    let request = Request::new(Verb::Sink, iri(name))
        .with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()));
    let repr = block_on(kernel.issue(request, &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

/// A move with no mark named: the side to play plays.
fn play(kernel: &Kernel, x: i64, y: i64) -> Result<String> {
    sink(kernel, &move_name(x, y), "")
}

fn cached(kernel: &Kernel, name: &str) -> bool {
    kernel.is_cached(&Request::new(Verb::Source, iri(name)), &Capability::root())
}

/// The refusal a move met: the argument it names, and what it says.
fn refusal(result: Result<String>) -> (String, String) {
    match result {
        Err(Error::InvalidArgument { name, detail }) => (name, detail),
        other => panic!("expected a refusal, got {other:?}"),
    }
}

const ROW_0: &str = "urn:iki:tutorial:ttt:cells:0.0,1.0,2.0";
const ROW_1: &str = "urn:iki:tutorial:ttt:cells:0.1,1.1,2.1";
const ROW_2: &str = "urn:iki:tutorial:ttt:cells:0.2,1.2,2.2";
const COLUMN_0: &str = "urn:iki:tutorial:ttt:cells:0.0,0.1,0.2";
const COLUMN_1: &str = "urn:iki:tutorial:ttt:cells:1.0,1.1,1.2";
const COLUMN_2: &str = "urn:iki:tutorial:ttt:cells:2.0,2.1,2.2";
const DIAGONAL_0: &str = "urn:iki:tutorial:ttt:cells:0.0,1.1,2.2";
const DIAGONAL_1: &str = "urn:iki:tutorial:ttt:cells:2.0,1.1,0.2";

// ANCHOR: platonic_checkset
/// The CheckSet names the lines through a cell by their alias names, and is computed once:
/// no move — not even one on that very square — un-caches it.
#[test]
fn the_checkset_is_a_list_of_names_computed_once() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let center = checkset_name(1, 1);

    assert_eq!(
        source(&kernel, &center)?,
        "urn:iki:tutorial:ttt:column:1\n\
         urn:iki:tutorial:ttt:diagonal:0\n\
         urn:iki:tutorial:ttt:diagonal:1\n\
         urn:iki:tutorial:ttt:row:1"
    );
    assert!(cached(&kernel, &center));

    play(&kernel, 1, 1)?;
    play(&kernel, 0, 0)?;
    sink(&kernel, &stored_name(1, 1), "O")?; // straight into the store, past the rules
    assert!(cached(&kernel, &center), "nothing it depends on can change");
    Ok(())
}
// ANCHOR_END: platonic_checkset

#[test]
fn a_corner_is_on_three_lines_an_edge_on_two_and_off_the_board_on_none() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    assert_eq!(source(&kernel, &checkset_name(0, 0))?.lines().count(), 3);
    assert_eq!(source(&kernel, &checkset_name(1, 0))?.lines().count(), 2);
    assert_eq!(source(&kernel, &checkset_name(3, 1))?, "");
    assert_eq!(source(&kernel, &checkset_name(-7, 40))?, "");
    // Each name in the list resolves, through the alias, to the line it names.
    play(&kernel, 0, 0)?;
    for name in source(&kernel, &checkset_name(0, 0))?.lines() {
        assert!(source(&kernel, name)?.starts_with('X'), "{name}");
    }
    Ok(())
}

#[test]
fn x_opens_the_turn_alternates_and_a_line_of_three_wins() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    assert_eq!(source(&kernel, WINNER)?, EMPTY);
    assert_eq!(source(&kernel, TURN)?, "X");

    assert_eq!(play(&kernel, 1, 1)?, "X plays 1,1");
    assert_eq!(source(&kernel, TURN)?, "O");
    assert_eq!(sink(&kernel, &move_name(1, 0), "O")?, "O plays 1,0");
    play(&kernel, 0, 0)?;
    play(&kernel, 2, 1)?;
    assert_eq!(source(&kernel, WINNER)?, EMPTY);
    play(&kernel, 2, 2)?;

    assert_eq!(source(&kernel, WINNER)?, "X");
    assert_eq!(source(&kernel, TURN)?, EMPTY, "nobody moves after a win");
    assert_eq!(source(&kernel, BOARD)?, "XO-\n-XO\n--X");
    Ok(())
}

#[test]
fn a_full_board_with_no_line_is_a_draw() -> Result<()> {
    // X O X
    // X O O
    // O X X
    let kernel = kernel_over(Arc::default());
    for (x, y) in [
        (0, 0),
        (1, 0),
        (2, 0),
        (1, 1),
        (0, 1),
        (2, 1),
        (1, 2),
        (0, 2),
    ] {
        play(&kernel, x, y)?;
        assert_eq!(source(&kernel, WINNER)?, EMPTY, "after {x},{y}");
    }
    play(&kernel, 2, 2)?;
    assert_eq!(source(&kernel, BOARD)?, "XOX\nXOO\nOXX");
    assert_eq!(source(&kernel, WINNER)?, DRAW);
    assert_eq!(source(&kernel, TURN)?, EMPTY);
    let (_, detail) = refusal(play(&kernel, 1, 1));
    assert_eq!(detail, "the game is over — a draw");
    Ok(())
}

// ANCHOR: refusals
/// Each rule refuses at the edge, with a reason — and the stored cell underneath still
/// takes any mark at any square.
#[test]
fn the_move_refuses_what_the_rules_forbid_and_the_store_does_not() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let x_y = "x, y".to_string();

    assert_eq!(
        refusal(play(&kernel, 3, 1)),
        (
            x_y.clone(),
            "3,1 is off the board — no line passes through it".into()
        )
    );
    play(&kernel, 1, 1)?;
    assert_eq!(
        refusal(sink(&kernel, &move_name(0, 0), "X")),
        ("content".into(), "it is O's turn, not X's".into())
    );
    assert_eq!(
        refusal(play(&kernel, 1, 1)),
        (x_y.clone(), "1,1 is taken — X played there".into())
    );
    for (x, y) in [(1, 0), (0, 0), (2, 1), (2, 2)] {
        play(&kernel, x, y)?;
    }
    assert_eq!(
        refusal(play(&kernel, 0, 2)),
        (x_y, "the game is over — X has won".into())
    );

    // The atoms are unchanged: any mark, any square, no turn, no board.
    assert_eq!(sink(&kernel, &stored_name(7, -3), "R")?, "ok");
    assert_eq!(source(&kernel, &cell_name(7, -3))?, "R");
    Ok(())
}
// ANCHOR_END: refusals

#[test]
fn a_refused_move_changes_nothing() -> Result<()> {
    let store = Arc::new(CellStore::default());
    let kernel = kernel_over(store.clone());
    play(&kernel, 1, 1)?;
    let before = source(&kernel, BOARD)?;
    refusal(play(&kernel, 1, 1));
    refusal(sink(&kernel, &move_name(0, 0), "X"));
    refusal(play(&kernel, 5, 5));
    assert_eq!(source(&kernel, BOARD)?, before);
    assert_eq!(source(&kernel, TURN)?, "O");
    Ok(())
}

#[test]
fn a_move_is_written_not_read() {
    let kernel = kernel_over(Arc::default());
    assert!(matches!(
        source(&kernel, &move_name(1, 1)),
        Err(Error::Endpoint(_))
    ));
    // And declares so: Sink (and Meta), with `content` optional and one of the marks.
    let described = play_move().describe();
    let actions = described.action_specs();
    assert_eq!(
        actions.iter().map(|a| a.verb).collect::<Vec<_>>(),
        [Verb::Sink]
    );
    let content = actions[0]
        .inputs
        .iter()
        .find(|input| input.name == "content")
        .expect("the move declares content");
    assert!(!content.required);
    assert_eq!(content.one_of, ["X", "O"]);
}

// ANCHOR: inner_sink
/// The move's `Sink` to the stored cell is a sub-request, and the kernel cuts the stored
/// cell's thread on it exactly as it does on a `Sink` a player issues by hand. The whole
/// recompute story rests on this.
#[test]
fn a_sink_issued_inside_the_move_cuts_the_stored_cells_thread() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let (by_move, by_hand) = (cell_name(1, 1), cell_name(2, 2));
    assert_eq!(source(&kernel, &by_move)?, EMPTY);
    assert_eq!(source(&kernel, &by_hand)?, EMPTY);
    assert!(cached(&kernel, &by_move) && cached(&kernel, &by_hand));

    play(&kernel, 1, 1)?; // the Sink to stored:1:1 is issued by the move
    sink(&kernel, &stored_name(2, 2), "O")?; // issued by the caller

    assert!(!cached(&kernel, &by_move), "cut from inside the move");
    assert!(!cached(&kernel, &by_hand), "cut from outside, as in part I");
    assert_eq!(source(&kernel, &by_move)?, "X");
    assert_eq!(source(&kernel, &by_hand)?, "O");
    Ok(())
}
// ANCHOR_END: inner_sink

// ANCHOR: one_move
/// One move recomputes the lines through its square, the winner and the turn — and reads
/// exactly one stored cell to do it.
#[test]
fn one_move_recomputes_only_the_lines_through_its_square() -> Result<()> {
    let store = Arc::new(CellStore::default());
    let kernel = kernel_over(store.clone());
    play(&kernel, 1, 1)?;
    play(&kernel, 1, 0)?;
    // Everything read once, so everything is cached.
    source(&kernel, WINNER)?;
    source(&kernel, TURN)?;

    play(&kernel, 0, 0)?; // a corner: one row, one column, one diagonal
    for through in [ROW_0, COLUMN_0, DIAGONAL_0, WINNER, TURN] {
        assert!(!cached(&kernel, through), "{through} passes through (0,0)");
    }
    for past in [ROW_1, ROW_2, COLUMN_1, COLUMN_2, DIAGONAL_1] {
        assert!(cached(&kernel, past), "{past} does not");
    }
    assert!(cached(&kernel, &checkset_name(0, 0)));

    let reads = store.reads.load(Ordering::SeqCst);
    assert_eq!(source(&kernel, WINNER)?, EMPTY);
    assert_eq!(source(&kernel, TURN)?, "O");
    assert_eq!(
        store.reads.load(Ordering::SeqCst),
        reads + 1,
        "of nine stored cells, the corner alone is read again"
    );
    Ok(())
}
// ANCHOR_END: one_move
