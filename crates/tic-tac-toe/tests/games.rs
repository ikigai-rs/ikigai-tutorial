//! The claims the fourth chapter makes about many games, each one run.
//!
//! ```text
//! cargo test -p tic-tac-toe --test games -- --nocapture --test-threads 1
//! ```
//!
//! One kernel per test, as in the other three files, and now several games on it: each game
//! is a `Scope` built ONCE by [`tic_tac_toe::game`] over a store the test keeps, and every
//! request in that game is issued in that scope. The root keeps its own store — the game a
//! request plays when it names no game — so "the other game" always includes that one too.
//! Cache probes use LINE names, as before (ledger #561).

use std::collections::BTreeMap;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, EndpointSpace, Error, FnEndpoint, Invocation, Iri, Kernel, ReprType,
    Representation, Request, Result, Scope, Space, UriTemplate, Verb,
};
use tic_tac_toe::{
    check_store, checkset_name, game, game_name, kernel_over, move_name, space_with_store,
    stored_space, CellStore, BOARD, CELL, EMPTY, STORED, WINNER,
};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

fn source_in(kernel: &Kernel, scope: &Scope, name: &str) -> Result<String> {
    let request = Request::new(Verb::Source, iri(name));
    let repr = block_on(kernel.issue_in(request, &Capability::root(), scope.clone()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

/// A move with no mark named, in `scope`: the side to play there plays.
fn play_in(kernel: &Kernel, scope: &Scope, x: i64, y: i64) -> Result<String> {
    let request = Request::new(Verb::Sink, iri(&move_name(x, y)))
        .with_arg("content", ArgRef::Inline(Vec::new()));
    let repr = block_on(kernel.issue_in(request, &Capability::root(), scope.clone()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn cached_in(kernel: &Kernel, scope: &Scope, name: &str) -> bool {
    kernel.is_cached_in(
        &Request::new(Verb::Source, iri(name)),
        &Capability::root(),
        scope,
    )
}

/// A game over a fresh in-memory store the caller keeps.
fn new_game(id: &str) -> (Scope, Arc<CellStore>) {
    let store = Arc::new(CellStore::default());
    let scope = game(id, Arc::new(stored_space(store.clone()))).expect("a valid game id");
    (scope, store)
}

/// X takes diagonal 0 in five moves: X 0,0 · O 1,0 · X 1,1 · O 2,0 · X 2,2.
fn x_wins(kernel: &Kernel, scope: &Scope) -> Result<()> {
    for (x, y) in [(0, 0), (1, 0), (1, 1), (2, 0), (2, 2)] {
        play_in(kernel, scope, x, y)?;
    }
    Ok(())
}

const FRESH: &str = "---\n---\n---";
const ROW_0: &str = "urn:iki:tutorial:ttt:cells:0.0,1.0,2.0";

// ANCHOR: two_games
/// Two games on one kernel: the same names, the same code, two boards. A move in one
/// appears in that game and in no other — including the root's, the game a request plays
/// when it names none.
#[test]
fn two_games_are_two_boards_under_the_same_names() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let (a, _) = new_game("a");
    let (b, _) = new_game("b");

    assert_eq!(play_in(&kernel, &a, 1, 1)?, "X plays 1,1");
    assert_eq!(play_in(&kernel, &b, 0, 0)?, "X plays 0,0");
    assert_eq!(play_in(&kernel, &b, 2, 2)?, "O plays 2,2");

    assert_eq!(source_in(&kernel, &a, BOARD)?, "---\n-X-\n---");
    assert_eq!(source_in(&kernel, &b, BOARD)?, "X--\n---\n--O");
    assert_eq!(source_in(&kernel, &Scope::empty(), BOARD)?, FRESH);
    Ok(())
}
// ANCHOR_END: two_games

// ANCHOR: stays
/// A move reads four resources before it writes — the CheckSet, the winner, the cell and
/// the turn — and every one of them, and the write, is in the move's own game. Game A is
/// over; that refuses a move in A, and nowhere else. Reading B's board touches B's store
/// and never A's.
#[test]
fn a_move_stays_in_its_game() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let (a, a_store) = new_game("a");
    let (b, b_store) = new_game("b");

    x_wins(&kernel, &a)?;
    assert_eq!(source_in(&kernel, &a, WINNER)?, "X");
    match play_in(&kernel, &a, 0, 2) {
        Err(Error::InvalidArgument { detail, .. }) => {
            assert_eq!(detail, "the game is over — X has won")
        }
        other => panic!("a move in a finished game must be refused, got {other:?}"),
    }

    let a_reads = a_store.reads.load(Ordering::SeqCst);
    assert_eq!(play_in(&kernel, &b, 0, 2)?, "X plays 0,2");
    assert_eq!(source_in(&kernel, &b, WINNER)?, EMPTY);
    assert_eq!(source_in(&kernel, &b, BOARD)?, "---\n---\nX--");
    assert_eq!(
        a_store.reads.load(Ordering::SeqCst),
        a_reads,
        "B read A's store"
    );
    assert!(b_store.reads.load(Ordering::SeqCst) > 0);

    // And the root's game, which nobody named, is untouched by either.
    assert_eq!(source_in(&kernel, &Scope::empty(), BOARD)?, FRESH);
    assert_eq!(play_in(&kernel, &Scope::empty(), 1, 1)?, "X plays 1,1");
    Ok(())
}
// ANCHOR_END: stays

/// The same with the roles turned round: the ROOT's game is the one that is over, and a
/// game in a corridor still plays. A corridor that forgot to carry itself into the move's
/// sub-requests would read the root's winner here and refuse.
#[test]
fn a_finished_root_game_does_not_end_a_game_in_a_corridor() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let (b, _) = new_game("b");
    x_wins(&kernel, &Scope::empty())?;
    assert_eq!(source_in(&kernel, &Scope::empty(), WINNER)?, "X");
    assert_eq!(play_in(&kernel, &b, 1, 1)?, "X plays 1,1");
    Ok(())
}

// ANCHOR: partitions
/// Two games are two cache partitions: the key carries the chain's fingerprint, so an
/// answer computed in one game is never served in another. Right for the board. For the
/// CheckSet — which depends on nothing a game changes — it means one copy per game.
#[test]
fn every_game_computes_its_own_checkset() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let games: Vec<Scope> = ["a", "b", "c"].iter().map(|id| new_game(id).0).collect();
    let centre = checkset_name(1, 1);

    source_in(&kernel, &games[0], &centre)?;
    assert!(cached_in(&kernel, &games[0], &centre));
    assert!(!cached_in(&kernel, &games[1], &centre));
    assert!(!cached_in(&kernel, &Scope::empty(), &centre));

    // Every CheckSet of the board, in the root and in each of three games: nine answers,
    // the same nine every time, filed four times over.
    let before = kernel.cache_len();
    let mut answers = BTreeMap::new();
    for scope in std::iter::once(&Scope::empty()).chain(&games) {
        for x in 0..3 {
            for y in 0..3 {
                let name = checkset_name(x, y);
                let answer = source_in(&kernel, scope, &name)?;
                assert_eq!(*answers.entry(name).or_insert(answer.clone()), answer);
            }
        }
    }
    assert_eq!(answers.len(), 9);
    assert_eq!(kernel.cache_len() - before, 4 * 9 - 1); // the centre in `a` was already there
    Ok(())
}
// ANCHOR_END: partitions

// ANCHOR: cut
/// A golden thread is a NAME, not a name in a game. A move in A cuts `stored:0:0`, and every
/// entry that hangs from that thread goes, in every game: B's corner cell is recomputed,
/// although nothing in B changed. Sound — a cut can only over-invalidate — and a cost.
#[test]
fn a_move_in_one_game_uncaches_the_same_square_in_every_game() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let (a, _) = new_game("a");
    let (b, b_store) = new_game("b");
    let corner = CELL.replace("{x}", "0").replace("{y}", "0");

    source_in(&kernel, &a, &corner)?;
    source_in(&kernel, &b, &corner)?;
    source_in(&kernel, &b, ROW_0)?;
    assert!(cached_in(&kernel, &b, &corner) && cached_in(&kernel, &b, ROW_0));

    play_in(&kernel, &a, 0, 0)?;
    assert!(!cached_in(&kernel, &b, &corner));
    assert!(!cached_in(&kernel, &b, ROW_0));

    // Recomputed, and still right: B's corner is still empty.
    let reads = b_store.reads.load(Ordering::SeqCst);
    assert_eq!(source_in(&kernel, &b, &corner)?, EMPTY);
    assert_eq!(b_store.reads.load(Ordering::SeqCst), reads + 1);
    Ok(())
}
// ANCHOR_END: cut

#[test]
fn a_game_has_one_name_in_one_spelling() {
    assert_eq!(
        game_name("a").expect("valid").as_str(),
        "urn:iki:tutorial:ttt:game:a"
    );
    for bad in ["", "a b", "a:b", "é", "a/b"] {
        assert!(
            matches!(game_name(bad), Err(Error::InvalidArgument { ref name, .. }) if name == "game"),
            "{bad:?}"
        );
    }
    // A store that names itself cannot be injected under another game's name.
    let named = EndpointSpace::new().named(iri("urn:example:somebody-else"));
    assert!(matches!(
        game("a", Arc::new(named)),
        Err(Error::InvalidArgument { .. })
    ));
}

// ANCHOR: contract_ok
/// The in-memory store keeps the contract — and so does a store written differently, which
/// is then a game's store like any other. The game cannot tell them apart.
#[test]
fn any_space_that_keeps_the_contract_is_a_store() -> Result<()> {
    block_on(check_store(Arc::new(stored_space(Arc::default())))).expect("in-memory store");

    let other = Arc::new(by_name_store());
    block_on(check_store(other.clone())).expect("a store written differently");

    let kernel = kernel_over(Arc::default());
    let elsewhere = game("elsewhere", other)?;
    assert_eq!(play_in(&kernel, &elsewhere, 1, 1)?, "X plays 1,1");
    assert_eq!(source_in(&kernel, &elsewhere, BOARD)?, "---\n-X-\n---");
    Ok(())
}
// ANCHOR_END: contract_ok

/// A store that breaks the contract is refused by the check — here, one that answers the
/// empty string for an unplayed cell, which would make every empty square read as nothing
/// at all (a line of `X` and two empties would read `X`, not `X--`).
#[test]
fn a_store_that_answers_empty_for_unplayed_breaks_the_contract() {
    let broken = EndpointSpace::new().bind(
        UriTemplate::parse(STORED).expect("parses"),
        FnEndpoint::new("broken-store", |_: &Invocation<'_>| {
            Ok(Representation::new(ReprType::new("text/plain"), Vec::new()).cacheable())
        }),
    );
    let refusal = block_on(check_store(Arc::new(broken))).expect_err("must be refused");
    assert!(refusal.contains("NotFound"), "{refusal}");
}

/// The whole game over a store that is not a `CellStore` at all: `space_with_store` over
/// the differently-written store, as a kernel's ROOT rather than a corridor.
#[test]
fn the_whole_game_runs_over_a_store_it_did_not_build() -> Result<()> {
    let kernel = Kernel::new(Arc::new(space_with_store(Arc::new(by_name_store()))));
    let root = Scope::empty();
    x_wins(&kernel, &root)?;
    assert_eq!(source_in(&kernel, &root, WINNER)?, "X");
    assert_eq!(source_in(&kernel, &root, BOARD)?, "XOO\n-X-\n--X");
    Ok(())
}

/// A second store, written another way: marks kept by the cell's NAME in a map, the
/// coordinates checked by parsing and re-spelling. Same contract, different internals —
/// the shape a Python or TypeScript store would have.
fn by_name_store() -> EndpointSpace {
    let marks: Arc<Mutex<BTreeMap<String, String>>> = Arc::default();
    let endpoint = FnEndpoint::new("by-name-store", move |inv: &Invocation<'_>| {
        for axis in ["x", "y"] {
            let text = inv.bindings.get(axis).unwrap_or("");
            if text.parse::<i64>().map(|n| n.to_string()).as_deref() != Ok(text) {
                return Err(Error::InvalidArgument {
                    name: axis.to_string(),
                    detail: format!("`{text}` is not in its plain form"),
                });
            }
        }
        let key = inv.request.target.as_str().to_string();
        let mut marks = marks.lock().expect("lock");
        let text = |bytes: &str| {
            Representation::new(ReprType::new("text/plain"), bytes.as_bytes().to_vec())
        };
        match inv.request.verb {
            Verb::Sink => {
                let mark = inv.inline_str("content")?.trim();
                if mark.is_empty() {
                    return Err(Error::InvalidArgument {
                        name: "content".into(),
                        detail: "empty".into(),
                    });
                }
                marks.insert(key, mark.to_string());
                Ok(text("ok"))
            }
            Verb::Delete => {
                marks.remove(&key);
                Ok(text("ok"))
            }
            _ => match marks.get(&key) {
                Some(mark) => Ok(text(mark).cacheable()),
                None => Err(Error::NotFound(key)),
            },
        }
    });
    EndpointSpace::new().bind(UriTemplate::parse(STORED).expect("parses"), endpoint)
}

/// The space type the game takes a store as — a compile-time check that the seam is
/// `Arc<dyn Space>`, so a `ModuleSpace` over a peer socket fits it.
#[allow(dead_code)] // the check is that it compiles
fn the_seam_is_a_space(store: Arc<dyn Space>) -> impl Space {
    space_with_store(store)
}
