//! The claims the fifth chapter makes about the game's face, each one run.
//!
//! ```text
//! cargo test -p tic-tac-toe --test views -- --nocapture --test-threads 1
//! ```
//!
//! The views are resources like the rest: templates bound at names, whose markers name the
//! cells, the winner and the turn, all sourced through the kernel. So each test here is a kernel, a few requests, and
//! a look at the HTML — and at the cache, by the VIEW's name, which no alias rewrites, so
//! `is_cached` answers truly for it (ledger #561 bites only alias names).

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Error, Iri, Kernel, Request, Result, Scope, Verb};
use tic_tac_toe::{
    game, kernel_over, lines, move_name, stored_name, stored_space, template_name, view_play_name,
    view_square_name, CellStore, CELLS, RESET, TEMPLATES, VIEW_BOARD, VIEW_RESET, VIEW_STATUS,
};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

fn issue_in(kernel: &Kernel, scope: &Scope, request: Request) -> Result<String> {
    let repr = block_on(kernel.issue_in(request, &Capability::root(), scope.clone()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn source(kernel: &Kernel, name: &str) -> Result<String> {
    issue_in(
        kernel,
        &Scope::empty(),
        Request::new(Verb::Source, iri(name)),
    )
}

fn sink(kernel: &Kernel, name: &str) -> Result<String> {
    issue_in(kernel, &Scope::empty(), Request::new(Verb::Sink, iri(name)))
}

fn cached(kernel: &Kernel, name: &str) -> bool {
    kernel.is_cached(&Request::new(Verb::Source, iri(name)), &Capability::root())
}

/// The squares the board view shows, as `(x, y, what)`: `open`, `closed`, or the mark.
fn squares(html: &str) -> Vec<(String, String)> {
    html.split("<button")
        .skip(1)
        .map(|button| {
            let label = button
                .split("aria-label=\"")
                .nth(1)
                .and_then(|rest| rest.split('"').next())
                .expect("every square has a label")
                .to_string();
            let open = button.contains("hx-post=");
            (label, if open { "open" } else { "not open" }.to_string())
        })
        .collect()
}

#[test]
fn every_template_is_a_resource_computed_once() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    for (name, file) in TEMPLATES {
        let served = source(&kernel, &template_name(name))?;
        assert_eq!(served, file.strip_suffix('\n').unwrap_or(file), "{name}");
        assert!(cached(&kernel, &template_name(name)), "{name}");
    }
    assert!(matches!(
        source(&kernel, &template_name("nope")),
        Err(Error::NotFound(_))
    ));
    Ok(())
}

// ANCHOR: fresh_board
/// A fresh board is nine open squares, each a move by its HTTP path; the status says X opens.
#[test]
fn a_fresh_board_is_nine_moves() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let board = source(&kernel, VIEW_BOARD)?;
    assert_eq!(board.matches("<button").count(), 9);
    assert_eq!(board.matches("hx-post=").count(), 9);
    assert!(board.contains(
        "hx-post=\"iki/tutorial/ttt/view/play/2/0\" hx-target=\"previous .ttt-status\" \
         aria-label=\"2,0: empty, play here\""
    ));
    assert_eq!(source(&kernel, VIEW_STATUS)?, "X to play.");
    Ok(())
}
// ANCHOR_END: fresh_board

// ANCHOR: play_view
/// A play through the view is a move through the kernel: the reply says what happened and
/// whose turn it is, the square is taken, and a refusal is ANSWERED in the kernel's words
/// rather than failed — whatever kind of error the move raised.
#[test]
fn a_play_answers_what_happened_and_whose_turn_it_is() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    assert_eq!(
        sink(&kernel, &view_play_name(1, 1))?,
        "X plays 1,1. O to play."
    );
    let board = source(&kernel, VIEW_BOARD)?;
    assert!(board.contains("aria-disabled=\"true\" aria-label=\"X at 1,1\">X</button>"));
    assert_eq!(board.matches("hx-post=").count(), 8);

    assert_eq!(
        sink(&kernel, &view_play_name(1, 1))?,
        "invalid argument `x, y`: 1,1 is taken — X played there. O to play."
    );
    assert!(sink(&kernel, &view_play_name(3, 1))?.contains("3,1 is off the board"));
    Ok(())
}
// ANCHOR_END: play_view

/// Once the game is over every empty square is closed — present, labeled, and not a move —
/// and the status says who won.
#[test]
fn a_won_game_closes_the_empty_squares() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    for (x, y) in [(0, 0), (1, 0), (1, 1), (2, 0), (2, 2)] {
        sink(&kernel, &view_play_name(x, y))?;
    }
    assert_eq!(source(&kernel, VIEW_STATUS)?, "X has won.");
    let board = source(&kernel, VIEW_BOARD)?;
    assert_eq!(board.matches("hx-post=").count(), 0);
    let labels = squares(&board);
    assert_eq!(labels.len(), 9);
    assert!(labels.contains(&("0,1: empty, the game is over".into(), "not open".into())));
    assert!(labels.contains(&("O at 1,0".into(), "not open".into())));
    assert_eq!(
        sink(&kernel, &view_play_name(0, 1))?,
        "invalid argument `x, y`: the game is over — X has won. X has won."
    );
    Ok(())
}

#[test]
fn a_draw_says_so() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    // X O X / X O O / O X X — nine moves, no line.
    for (x, y) in [
        (0, 0),
        (1, 0),
        (2, 0),
        (1, 1),
        (0, 1),
        (2, 1),
        (1, 2),
        (0, 2),
        (2, 2),
    ] {
        sink(&kernel, &move_name(x, y))?;
    }
    assert_eq!(source(&kernel, VIEW_STATUS)?, "A draw.");
    Ok(())
}

// ANCHOR: reset
/// A new game clears exactly the board's squares — the ones a line passes through — and
/// nothing off it; each through the kernel, so the views recompute.
#[test]
fn a_reset_clears_the_board_and_only_the_board() -> Result<()> {
    let store = Arc::new(CellStore::default());
    let kernel = kernel_over(store.clone());
    sink(&kernel, &view_play_name(0, 0))?;
    sink(&kernel, &view_play_name(2, 2))?;
    let off = Request::new(Verb::Sink, iri(&stored_name(7, -3)))
        .with_arg("content", ArgRef::Inline(b"R".to_vec()));
    issue_in(&kernel, &Scope::empty(), off)?;
    source(&kernel, VIEW_BOARD)?;

    assert_eq!(sink(&kernel, VIEW_RESET)?, "The board is clear. X to play.");
    assert!(!cached(&kernel, VIEW_BOARD));
    assert_eq!(source(&kernel, VIEW_BOARD)?.matches("hx-post=").count(), 9);
    assert_eq!(source(&kernel, &stored_name(7, -3))?, "R");
    assert_eq!(sink(&kernel, RESET)?, "The board is clear");
    Ok(())
}
// ANCHOR_END: reset

// ANCHOR: views_cache
/// The views are composites: cached, recomputed by the move that touches them, and a read
/// or a refused move changes nothing.
#[test]
fn a_view_is_cached_until_a_move_touches_it() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let first = source(&kernel, VIEW_BOARD)?;
    source(&kernel, VIEW_STATUS)?;
    assert!(cached(&kernel, VIEW_BOARD) && cached(&kernel, VIEW_STATUS));

    // Off the board: refused, nothing written, nothing cut.
    sink(&kernel, &view_play_name(5, 5))?;
    assert!(cached(&kernel, VIEW_BOARD) && cached(&kernel, VIEW_STATUS));

    // The move cuts both. The reply then READ the status, so the status is cached again —
    // the new one — and the board waits for whoever asks for it next.
    assert_eq!(
        sink(&kernel, &view_play_name(0, 2))?,
        "X plays 0,2. O to play."
    );
    assert!(!cached(&kernel, VIEW_BOARD));
    assert!(cached(&kernel, VIEW_STATUS));
    assert_ne!(source(&kernel, VIEW_BOARD)?, first);
    Ok(())
}
// ANCHOR_END: views_cache

/// In a game's corridor the views show that game: the same names, the same templates, the
/// game's own marks.
#[test]
fn a_view_shows_the_game_it_is_asked_in() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let a = game("a", Arc::new(stored_space(Arc::default())))?;
    let b = game("b", Arc::new(stored_space(Arc::default())))?;
    issue_in(
        &kernel,
        &a,
        Request::new(Verb::Sink, iri(&view_play_name(1, 1))),
    )?;
    let in_a = issue_in(&kernel, &a, Request::new(Verb::Source, iri(VIEW_BOARD)))?;
    let in_b = issue_in(&kernel, &b, Request::new(Verb::Source, iri(VIEW_BOARD)))?;
    assert!(in_a.contains("X at 1,1"));
    assert!(!in_b.contains("X at 1,1"));
    assert_eq!(
        issue_in(&kernel, &b, Request::new(Verb::Source, iri(VIEW_STATUS)))?,
        "X to play."
    );
    Ok(())
}

/// A mark is text, so it is escaped on its way into the HTML: the store keeps any mark, and a
/// mark played straight into it must not become markup.
#[test]
fn a_mark_is_escaped_into_the_html() -> Result<()> {
    let kernel = kernel_over(Arc::default());
    let write = Request::new(Verb::Sink, iri(&stored_name(0, 0)))
        .with_arg("content", ArgRef::Inline(b"<b>\"&'".to_vec()));
    issue_in(&kernel, &Scope::empty(), write)?;
    let board = source(&kernel, VIEW_BOARD)?;
    assert!(
        board.contains(">&lt;b&gt;&quot;&amp;&#39;</button>"),
        "{board}"
    );
    assert!(!board.contains("<b>"));
    Ok(())
}

/// The views' verbs: a play and a reset are writes; reading one is refused.
#[test]
fn a_play_is_made_not_read() {
    let kernel = kernel_over(Arc::default());
    assert!(matches!(
        source(&kernel, &view_play_name(1, 1)),
        Err(Error::Endpoint(_))
    ));
    assert!(matches!(
        source(&kernel, VIEW_RESET),
        Err(Error::Endpoint(_))
    ));
    assert!(matches!(source(&kernel, RESET), Err(Error::Endpoint(_))));
}

/// The board template's squares are exactly the squares the line table's lines pass
/// through — the geometry lives in two places (the table for the rules, the template for
/// the face), so this is the check that they agree.
#[test]
fn the_board_template_shows_exactly_the_squares_of_the_lines() -> Result<()> {
    let board = TEMPLATES
        .iter()
        .find(|(name, _)| *name == "board")
        .expect("a board template")
        .1;
    // Every marker in the board is `$r{…:view:square:X:Y}`, and nothing else.
    let square = view_square_name(0, 0);
    let square = square.trim_end_matches("0:0");
    let mut shown: Vec<(i64, i64)> = board
        .split("$r{")
        .skip(1)
        .map(|marker| {
            let (at, _) = marker.split_once('}').expect("a closed marker");
            let (x, y) = at
                .strip_prefix(square)
                .and_then(|xy| xy.split_once(':'))
                .unwrap_or_else(|| panic!("`{at}` is not a square's view"));
            (x.parse().expect("x"), y.parse().expect("y"))
        })
        .collect();
    assert!(!board.contains("$h{") && !board.contains("$a{"));
    shown.sort_unstable();
    let line = CELLS.trim_end_matches("{list}");
    let mut on_lines: Vec<(i64, i64)> = lines()
        .rules()
        .iter()
        .filter_map(|rule| rule.to().strip_prefix(line))
        .flat_map(|list| list.split(','))
        .map(|member| {
            let (x, y) = member.split_once('.').expect("x.y");
            (x.parse().expect("x"), y.parse().expect("y"))
        })
        .collect();
    on_lines.sort_unstable();
    on_lines.dedup();
    assert_eq!(shown, on_lines);
    Ok(())
}

/// The game shell's paths are relative and game-free, so the host that serves the page
/// decides the game by where the page is. A template names resources only inside its
/// markers, and never a game: the game is the corridor a view is asked in.
#[test]
fn the_markup_names_no_game_and_no_host() {
    for (name, text) in TEMPLATES {
        for path in text.split("hx-").skip(1) {
            let value = path.split('"').nth(1).unwrap_or_default();
            if path.starts_with("get=") || path.starts_with("post=") {
                assert!(
                    value.starts_with("iki/tutorial/ttt/"),
                    "{name}: {value} must be a relative path to a game resource"
                );
            }
        }
        let outside: String = text
            .split('$')
            .enumerate()
            .map(|(i, piece)| match (i, piece.find('{')) {
                (0, _) | (_, None) => piece,
                // Skip the marker: up to its closing brace, counting `{x}` inside it.
                (_, Some(_)) => {
                    let mut depth = 0;
                    let end = piece
                        .char_indices()
                        .find(|&(_, c)| {
                            depth += match c {
                                '{' => 1,
                                '}' => -1,
                                _ => 0,
                            };
                            c == '}' && depth == 0
                        })
                        .map_or(piece.len(), |(at, _)| at + 1);
                    &piece[end..]
                }
            })
            .collect();
        assert!(
            !outside.contains("urn:"),
            "{name} names a resource outside a marker"
        );
        assert!(!text.contains("game:"), "{name} names a game");
    }
}
