//! The views are templates now, and every byte they answer is the byte they answered when
//! Rust filled them.
//!
//! ```text
//! cargo test -p tic-tac-toe --test parity
//! ```
//!
//! Until `ikigai-fn` 0.3.0 the game's views were `{{slot}}` templates that Rust code filled:
//! it read the winner and each cell, chose a square or status template, and escaped each
//! value on its way in. The Python and Deno apps fill those same `{{slot}}` templates today,
//! and part VI of the book compares their boards with the host's byte for byte. So the
//! conversion had to change no byte, and this file is the proof, after `ikigai-fn`'s own
//! (`tests/tic_tac_toe_board.rs` there, over this repository's templates).
//!
//! The REFERENCE is that filler, reduced to what the game used, over the `{{slot}}`
//! templates frozen in `tests/reference/` exactly as they were. Each test plays the same
//! game in two kernels: one through the views (`view:play`, `view:board`, …), the other
//! through the move alone, rendered by the reference. At every step the board, the status,
//! the reply and the page shell must be the same bytes and the same media type.

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Kernel, Representation, Request, Result, Verb};
use tic_tac_toe::{
    cell_name, kernel_over, move_name, stored_name, view_game_name, view_play_name, RESET, TURN,
    VIEW_BOARD, VIEW_RESET, VIEW_STATUS, WINNER,
};

// ---- the reference: the `{{slot}}` filler the views replaced ------------------------

/// A frozen `{{slot}}` template, its final newline dropped as the old endpoint dropped it.
fn reference(name: &str) -> &'static str {
    let text = match name {
        "board" => include_str!("reference/board.html"),
        "game" => include_str!("reference/game.html"),
        "reply" => include_str!("reference/reply.html"),
        "square-open" => include_str!("reference/square-open.html"),
        "square-taken" => include_str!("reference/square-taken.html"),
        "square-closed" => include_str!("reference/square-closed.html"),
        "status-turn" => include_str!("reference/status-turn.html"),
        "status-won" => include_str!("reference/status-won.html"),
        "status-draw" => include_str!("reference/status-draw.html"),
        other => panic!("no reference template `{other}`"),
    };
    text.strip_suffix('\n').unwrap_or(text)
}

/// What fills a slot: text, escaped as it goes in, or another template's output, raw.
enum Fill {
    Text(String),
    Html(String),
}

/// The old `fill`: every `{{name args…}}` replaced by what `value` says (the templates
/// here are well formed, so the refusals the old filler made are not needed).
fn fill(template: &str, mut value: impl FnMut(&str, &[i64]) -> Fill) -> String {
    let mut out = String::new();
    let mut rest = template;
    while let Some(open) = rest.find("{{") {
        out.push_str(&rest[..open]);
        let after = &rest[open + 2..];
        let close = after.find("}}").expect("a closed slot");
        let mut words = after[..close].split(' ');
        let name = words.next().expect("a name");
        let args: Vec<i64> = words.map(|w| w.parse().expect("an integer")).collect();
        match value(name, &args) {
            Fill::Text(text) => out.push_str(&escape(&text)),
            Fill::Html(html) => out.push_str(&html),
        }
        rest = &after[close + 2..];
    }
    out.push_str(rest);
    out
}

/// The old `escape`.
fn escape(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '&' => "&amp;".to_string(),
            '<' => "&lt;".to_string(),
            '>' => "&gt;".to_string(),
            '"' => "&quot;".to_string(),
            '\'' => "&#39;".to_string(),
            c => c.to_string(),
        })
        .collect()
}

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

fn issue(kernel: &Kernel, request: Request) -> Result<Representation> {
    block_on(kernel.issue(request, &Capability::root()))
}

fn text(kernel: &Kernel, name: &str) -> String {
    let repr = issue(kernel, Request::new(Verb::Source, iri(name))).expect(name);
    String::from_utf8(repr.bytes).expect("UTF-8")
}

/// The old `view:board`: each `{{square x y}}` the square template the cell calls for.
fn reference_board(kernel: &Kernel) -> String {
    let over = text(kernel, WINNER) != "-";
    fill(reference("board"), |_, at| {
        let (x, y) = (at[0], at[1]);
        let mark = text(kernel, &cell_name(x, y));
        let kind = match (mark != "-", over) {
            (true, _) => "square-taken",
            (false, false) => "square-open",
            (false, true) => "square-closed",
        };
        Fill::Html(fill(reference(kind), |slot, _| match slot {
            "x" => Fill::Text(x.to_string()),
            "y" => Fill::Text(y.to_string()),
            _ => Fill::Text(mark.clone()),
        }))
    })
}

/// The old `view:status`.
fn reference_status(kernel: &Kernel) -> String {
    let won = text(kernel, WINNER);
    let (kind, mark) = match won.as_str() {
        "-" => ("status-turn", text(kernel, TURN)),
        "draw" => ("status-draw", String::new()),
        _ => ("status-won", won.clone()),
    };
    fill(reference(kind), |_, _| Fill::Text(mark.clone()))
}

/// The old reply: what the write said (a refusal's Display included), then the status.
fn reference_reply(kernel: &Kernel, said: Result<Representation>) -> String {
    let message = match said {
        Ok(repr) => String::from_utf8(repr.bytes).expect("UTF-8"),
        Err(refused) => refused.to_string(),
    };
    let status = reference_status(kernel);
    fill(reference("reply"), |slot, _| match slot {
        "message" => Fill::Text(message.clone()),
        _ => Fill::Html(status.clone()),
    })
}

// ---- the two games ---------------------------------------------------------------------

/// One game played twice: `views` through the views, `refs` through the move and rendered
/// by the reference.
struct Twins {
    views: Kernel,
    refs: Kernel,
}

impl Twins {
    fn new() -> Twins {
        Twins {
            views: kernel_over(Arc::default()),
            refs: kernel_over(Arc::default()),
        }
    }

    /// The board, the status and the shell of games `a` and `root` agree, byte for byte and
    /// type for type. Answers the board.
    fn agree(&self) -> String {
        let view = |name: &str| issue(&self.views, Request::new(Verb::Source, iri(name)));
        let board = view(VIEW_BOARD).expect("the board view");
        let html = String::from_utf8(board.bytes).expect("UTF-8");
        assert_eq!(html, reference_board(&self.refs));
        assert_eq!(board.repr_type.to_string(), "text/html;charset=utf-8");
        let status = view(VIEW_STATUS).expect("the status view");
        assert_eq!(
            String::from_utf8(status.bytes).expect("UTF-8"),
            reference_status(&self.refs)
        );
        assert_eq!(status.repr_type, board.repr_type);
        for id in ["a", "root", "x-7"] {
            let shell = view(&view_game_name(id)).expect("the page shell");
            assert_eq!(
                String::from_utf8(shell.bytes).expect("UTF-8"),
                fill(reference("game"), |_, _| Fill::Text(id.to_string()))
            );
            assert_eq!(shell.repr_type, board.repr_type);
        }
        html
    }

    /// Play `(x, y)` through the view in one and through the move in the other; the replies
    /// are the same bytes, and so is everything after. Answers the reply.
    fn play(&self, x: i64, y: i64) -> String {
        let reply = issue(
            &self.views,
            Request::new(Verb::Sink, iri(&view_play_name(x, y))),
        )
        .expect("a play is answered, refused or not");
        let said = issue(&self.refs, Request::new(Verb::Sink, iri(&move_name(x, y))));
        let reply = String::from_utf8(reply.bytes).expect("UTF-8");
        assert_eq!(reply, reference_reply(&self.refs, said), "play {x},{y}");
        self.agree();
        reply
    }

    fn reset(&self) -> String {
        let reply = issue(&self.views, Request::new(Verb::Sink, iri(VIEW_RESET))).expect("reset");
        let said = issue(&self.refs, Request::new(Verb::Sink, iri(RESET)));
        let reply = String::from_utf8(reply.bytes).expect("UTF-8");
        assert_eq!(reply, reference_reply(&self.refs, said));
        self.agree();
        reply
    }

    /// Write `mark` straight into the store at `(x, y)`, in both.
    fn store(&self, x: i64, y: i64, mark: &str) {
        for kernel in [&self.views, &self.refs] {
            let write = Request::new(Verb::Sink, iri(&stored_name(x, y)))
                .with_arg("content", ArgRef::Inline(mark.as_bytes().to_vec()));
            issue(kernel, write).expect("the store keeps any mark");
        }
    }
}

// ---- the proof -------------------------------------------------------------------------

#[test]
fn a_fresh_board_is_the_same_bytes() {
    let twins = Twins::new();
    let board = twins.agree();
    assert_eq!(board.matches("play here").count(), 9, "{board}");
}

#[test]
fn a_game_in_progress_is_the_same_bytes_and_so_is_every_reply() {
    let twins = Twins::new();
    assert_eq!(twins.play(1, 1), "X plays 1,1. O to play.");
    twins.play(0, 0);
    // Refused: taken, and off the board.
    assert!(twins.play(1, 1).contains("1,1 is taken"));
    assert!(twins.play(3, 1).contains("3,1 is off the board"));
    let board = twins.agree();
    assert!(
        board.contains(r#"aria-label="X at 1,1">X</button>"#),
        "{board}"
    );
    assert_eq!(board.matches("play here").count(), 7);
}

/// Once someone has won, the empty squares close — the branch the Rust code took on the
/// winner, taken now by `conditional` in the `square-empty` template.
#[test]
fn a_won_game_is_the_same_bytes() {
    let twins = Twins::new();
    for (x, y) in [(0, 0), (0, 1), (1, 0), (1, 1), (2, 0)] {
        twins.play(x, y);
    }
    let board = twins.agree();
    assert_eq!(board.matches("the game is over").count(), 4, "{board}");
    assert!(!board.contains("play here"));
    assert_eq!(
        twins.play(2, 2),
        "invalid argument `x, y`: the game is over — X has won. X has won."
    );
    assert_eq!(twins.reset(), "The board is clear. X to play.");
}

#[test]
fn a_draw_is_the_same_bytes() {
    let twins = Twins::new();
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
        twins.play(x, y);
    }
    assert_eq!(text(&twins.views, VIEW_STATUS), "A draw.");
}

/// A mark is text: escaped into the HTML, and a marker inside it is NOT expanded. The
/// composed board splices the cell with `$h`, which terminates, exactly as the Rust filler
/// never looked inside a value. `$a{urn:iki:tutorial:ttt:winner}` stays those characters.
#[test]
fn a_hostile_mark_is_escaped_and_never_expanded() {
    let twins = Twins::new();
    let hostile = r#"<b>"&'$a{urn:iki:tutorial:ttt:winner}$$h{x}"#;
    twins.store(0, 0, hostile);
    let board = twins.agree();
    assert!(
        board.contains(
            r#">&lt;b&gt;&quot;&amp;&#39;$a{urn:iki:tutorial:ttt:winner}$$h{x}</button>"#
        ),
        "{board}"
    );
    assert!(!board.contains("<b>"));
    // A whole row of it, so it is in the lines the winner reads, and the reply that quotes
    // the refusal of the next move is escaped too.
    twins.store(1, 0, hostile);
    twins.store(2, 0, hostile);
    twins.play(1, 1);
    twins.play(0, 0);
}

/// The corridor carries through every marker: in game `b` the views show game `b`.
#[test]
fn the_views_answer_per_game_by_the_corridor() {
    let kernel = kernel_over(Arc::default());
    let game = tic_tac_toe::game("b", Arc::new(tic_tac_toe::stored_space(Arc::default()))).unwrap();
    let play = Request::new(Verb::Sink, iri(&view_play_name(2, 2)));
    let reply = block_on(kernel.issue_in(play, &Capability::root(), game.clone())).unwrap();
    assert_eq!(reply.bytes, b"X plays 2,2. O to play.");
    let board = |scope| {
        let request = Request::new(Verb::Source, iri(VIEW_BOARD));
        let repr = block_on(kernel.issue_in(request, &Capability::root(), scope)).unwrap();
        String::from_utf8(repr.bytes).unwrap()
    };
    assert!(board(game).contains("X at 2,2"));
    assert!(!board(ikigai_core::Scope::empty()).contains("X at 2,2"));
}
