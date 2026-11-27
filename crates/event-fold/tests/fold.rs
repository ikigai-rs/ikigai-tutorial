//! The claims the fold makes, each one run against a kernel and a log the test holds.
//!
//! ```text
//! cargo test -p event-fold --test fold
//! ```
//!
//! The fold's step counts itself ([`Log::steps`]), so "answered from the cache" is a fact
//! these tests check: a read that computed nothing ran no step.

use std::sync::Arc;

use event_fold::{checkpoint_name, entry_name, kernel, state_name, Log, EVERY, LOG, STATE};
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Expiry, Iri, Kernel, Request, Result, Verb};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

fn source(kernel: &Kernel, name: &str) -> Result<String> {
    let repr = block_on(kernel.issue(Request::new(Verb::Source, iri(name)), &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn expiry(kernel: &Kernel, name: &str) -> Expiry {
    block_on(kernel.issue(Request::new(Verb::Source, iri(name)), &Capability::root()))
        .expect("an answer")
        .expiry
}

fn cached(kernel: &Kernel, name: &str) -> bool {
    kernel.is_cached(&Request::new(Verb::Source, iri(name)), &Capability::root())
}

fn play(kernel: &Kernel, cell: &str) -> Result<String> {
    let request = Request::new(Verb::Sink, iri(LOG))
        .with_arg("content", ArgRef::Inline(cell.as_bytes().to_vec()));
    let repr = block_on(kernel.issue(request, &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn host() -> (Kernel, Arc<Log>) {
    let log = Arc::new(Log::default());
    (kernel(Arc::clone(&log)), log)
}

// ANCHOR: a_game
#[test]
fn the_board_is_a_fold_over_the_moves() {
    let (kernel, _) = host();
    assert_eq!(source(&kernel, STATE).unwrap(), "...\n...\n...\nnext: X");

    for (n, cell) in ["5", "1", "9", "3", "2"].iter().enumerate() {
        assert_eq!(play(&kernel, cell).unwrap(), (n + 1).to_string());
    }
    assert_eq!(source(&kernel, LOG).unwrap(), "5");
    assert_eq!(source(&kernel, &entry_name(1)).unwrap(), "X5");
    assert_eq!(source(&kernel, &entry_name(4)).unwrap(), "O3");
    assert_eq!(source(&kernel, STATE).unwrap(), "OXO\n.X.\n..X\nnext: O");

    // Any position of the game can still be asked for, by name.
    assert_eq!(
        source(&kernel, &state_name(0)).unwrap(),
        "...\n...\n...\nnext: X"
    );
    assert_eq!(
        source(&kernel, &state_name(2)).unwrap(),
        "O..\n.X.\n...\nnext: X"
    );
    assert_eq!(
        source(&kernel, &state_name(3)).unwrap(),
        "O..\n.X.\n..X\nnext: O"
    );

    // X completes the middle column on the seventh move, and the log refuses an eighth.
    play(&kernel, "7").unwrap();
    play(&kernel, "8").unwrap();
    assert_eq!(source(&kernel, STATE).unwrap(), "OXO\n.X.\nOXX\nwinner: X");
    assert!(matches!(play(&kernel, "4"), Err(Error::Conflict(_))));
    assert_eq!(source(&kernel, LOG).unwrap(), "7");
}
// ANCHOR_END: a_game

#[test]
fn the_log_only_holds_a_game_that_can_be_played() {
    let (kernel, _) = host();
    play(&kernel, "5").unwrap();
    match play(&kernel, "5") {
        Err(Error::Conflict(why)) => assert_eq!(why, "cell 5 is taken: X played there"),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        play(&kernel, "0"),
        Err(Error::InvalidArgument { .. })
    ));
    assert!(matches!(
        play(&kernel, "10"),
        Err(Error::InvalidArgument { .. })
    ));
    // A refused move appended nothing.
    assert_eq!(source(&kernel, LOG).unwrap(), "1");
    assert!(matches!(
        source(&kernel, &entry_name(2)),
        Err(Error::NotFound(_))
    ));
    assert!(matches!(
        source(&kernel, &state_name(2)),
        Err(Error::NotFound(_))
    ));
}

// ANCHOR: forever
#[test]
fn a_position_is_cached_forever_and_only_the_head_moves() {
    let (kernel, log) = host();
    for cell in ["5", "1", "9", "3"] {
        play(&kernel, cell).unwrap();
    }
    assert_eq!(expiry(&kernel, &entry_name(2)), Expiry::Never);
    assert_eq!(expiry(&kernel, &state_name(4)), Expiry::Never);
    assert_eq!(expiry(&kernel, &checkpoint_name(3)), Expiry::Never);

    let before = source(&kernel, STATE).unwrap();
    assert!(cached(&kernel, STATE));
    assert!(cached(&kernel, &state_name(4)));

    // An append cuts the log, and so the head's board, and nothing that rests on a prefix.
    play(&kernel, "2").unwrap();
    assert!(!cached(&kernel, LOG));
    assert!(!cached(&kernel, STATE));
    for kept in [state_name(4), checkpoint_name(3), entry_name(4)] {
        assert!(cached(&kernel, &kept), "{kept} was let go by an append");
    }

    // The new head costs one step: the board at 4 is read from the cache, and one entry
    // is applied to it. Asked again, it costs none.
    let steps = log.steps();
    assert_ne!(source(&kernel, STATE).unwrap(), before);
    assert_eq!(log.steps() - steps, 1);
    source(&kernel, STATE).unwrap();
    source(&kernel, &state_name(4)).unwrap();
    assert_eq!(log.steps() - steps, 1);
}
// ANCHOR_END: forever

// ANCHOR: checkpoints
#[test]
fn a_cold_read_recurses_through_checkpoints_not_through_every_entry() {
    let (warm, log) = host();
    for cell in ["1", "2", "3", "5", "4", "6", "8", "7"] {
        play(&warm, cell).unwrap();
    }
    // A second kernel over the same log: the same entries, and an empty cache.
    let cold = kernel(Arc::clone(&log));
    let cold_log = log;
    let steps = cold_log.steps();
    let cold_answer = source(&cold, &state_name(8)).unwrap();

    // The board after 8 moves, read cold: state 8 rests on state 7, which rests on the
    // checkpoint at 6, which rests on the checkpoint at 3. Every entry is applied once
    // (8 steps), and the recursion is 2 positions and 2 checkpoints deep, not 8.
    let ran = cold_log.steps() - steps;
    assert_eq!(cold_answer, source(&warm, &state_name(8)).unwrap());
    assert_eq!(ran, 8, "a cold read of 8 moves ran {ran} steps");
    assert!(cached(&cold, &checkpoint_name(3)));
    assert!(cached(&cold, &checkpoint_name(6)));
    assert_eq!(EVERY, 3);

    // A checkpoint falls only where it says.
    assert!(matches!(
        source(&cold, &checkpoint_name(4)),
        Err(Error::InvalidArgument { .. })
    ));
}
// ANCHOR_END: checkpoints

#[test]
fn a_position_is_spelled_one_way() {
    let (kernel, _) = host();
    for refused in ["01", "x", "-1"] {
        let name = format!("urn:iki:tutorial:fold:state:seq:{refused}");
        assert!(
            matches!(source(&kernel, &name), Err(Error::InvalidArgument { .. })),
            "{name}"
        );
    }
}
