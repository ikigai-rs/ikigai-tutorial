//! **A fold, as a worked example**: an event-sourced resource, built from the names up.
//!
//! The design note *Events, streams and resources* (ikigai-core
//! `docs/design/events-and-resources.md`, section 2.7 and section 9, item 9) says event
//! sourcing is "a resource composed by folding a log", and that because entries never
//! change, "the state after entry n is a pure function of an immutable prefix". This crate
//! is that sentence as code, over the book's game: a tic-tac-toe game is the log of its
//! moves, and the board is a fold over them.
//!
//! The resources, designed as names before code:
//!
//! * `urn:iki:tutorial:fold:log` — the log. **Sink** appends one move (`content` is a cell,
//!   `1` to `9`, read left to right and top to bottom) and answers the move's position;
//!   **Source** answers the head, the number of moves so far. The only thing that moves.
//! * `urn:iki:tutorial:fold:entry:seq:{n}` — the move at position `n`, `X5` or `O1`. An
//!   **atom** that never changes once written, so it is cacheable forever. It has no Sink of
//!   its own: the only way to write an entry is to append it.
//! * `urn:iki:tutorial:fold:checkpoint:seq:{n}` — the board after `n` moves, for `n` a
//!   multiple of [`EVERY`]: the previous checkpoint with the [`EVERY`] entries after it
//!   applied. Cacheable forever.
//! * `urn:iki:tutorial:fold:state:seq:{n}` — the board after `n` moves, for any `n`:
//!   `state:seq:{n-1}` with entry `n` applied, or the checkpoint itself at a multiple of
//!   [`EVERY`]. Cacheable forever, because a prefix of the log never changes.
//! * `urn:iki:tutorial:fold:state` — the board now: `state:seq:{head}`. It reads the log's
//!   head, so it hangs from the log's golden thread and is the one composite an append cuts.
//!
//! What the fold does NOT do, said plainly: the design note asks for checkpoints "by spill"
//! (`urn:stream:spill`), so that a cold read survives a restart without recursing through
//! history. A spill needs a host spill facility (`Kernel::with_spill`, a file-backed one
//! lives in ikigai-fs) and a disk-budget grant (`urn:cap:stream:spill:<bytes>`), and makes a
//! ONE-SHOT body re-readable. Every answer here is already a buffer, held in the kernel's
//! cache, so a checkpoint here is a composite cached forever: it bounds how many entries one
//! fold step reads, and a cold read recurses through checkpoints only (`n / EVERY` deep),
//! never through every entry.
//!
//! Nor does a spill make a checkpoint survive a restart. ikigai-fs's `FileSpill` unlinks
//! its backing file when the last holder of the replay handle drops it, so a spill makes a
//! body re-readable for that handle's lifetime, not durable. A checkpoint that survives a
//! restart needs a durable store the host binds: an atom written at the checkpoint. This
//! example does not build one; a kernel started afresh folds again from the log.
//!
//! The representation of a board is three lines of text:
//!
//! ```text
//! X.O
//! .X.
//! ..O
//! ```
//!
//! followed by one more line: `next: X`, `next: O`, `winner: X`, `winner: O` or `draw`.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, ArgSpec, AsyncFnEndpoint, Description, EndpointSpace, Error, Invocation,
    InvokeFuture, Iri, Kernel, ReprType, Representation, Result, UriTemplate, Verb,
};

// ANCHOR: names
/// The log: Sink appends a move, Source answers the head (the number of moves).
pub const LOG: &str = "urn:iki:tutorial:fold:log";

/// The move at position `n`, an atom that never changes.
pub const ENTRY: &str = "urn:iki:tutorial:fold:entry:seq:{n}";

/// The board after `n` moves, `n` a multiple of [`EVERY`].
pub const CHECKPOINT: &str = "urn:iki:tutorial:fold:checkpoint:seq:{n}";

/// The board after `n` moves, any `n`.
pub const STATE_AT: &str = "urn:iki:tutorial:fold:state:seq:{n}";

/// The board now: the one moving composite.
pub const STATE: &str = "urn:iki:tutorial:fold:state";
// ANCHOR_END: names

/// How many entries lie between two checkpoints.
pub const EVERY: u64 = 3;

/// The name of the entry at position `n`.
pub fn entry_name(n: u64) -> String {
    format!("urn:iki:tutorial:fold:entry:seq:{n}")
}

/// The name of the checkpoint at position `n`.
pub fn checkpoint_name(n: u64) -> String {
    format!("urn:iki:tutorial:fold:checkpoint:seq:{n}")
}

/// The name of the board after `n` moves.
pub fn state_name(n: u64) -> String {
    format!("urn:iki:tutorial:fold:state:seq:{n}")
}

fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#nonNegativeInteger";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

// ---------------------------------------------------------------------------------------
// The game: a board, and the one pure step the fold repeats.
// ---------------------------------------------------------------------------------------

/// A board: nine cells, each `X`, `O` or `.`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Board(pub [u8; 9]);

const LINES: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

impl Board {
    /// The board before any move.
    pub fn empty() -> Self {
        Board([b'.'; 9])
    }

    /// The player whose line is complete, if any.
    pub fn winner(&self) -> Option<char> {
        LINES.iter().find_map(|[a, b, c]| {
            let cell = self.0[*a];
            (cell != b'.' && cell == self.0[*b] && cell == self.0[*c]).then_some(cell as char)
        })
    }

    /// The player to move, or `None` once the game is over.
    pub fn next(&self) -> Option<char> {
        if self.winner().is_some() || !self.0.contains(&b'.') {
            return None;
        }
        let played = self.0.iter().filter(|c| **c != b'.').count();
        Some(if played % 2 == 0 { 'X' } else { 'O' })
    }

    /// The board's text: three rows, then the outcome line.
    pub fn text(&self) -> String {
        let rows: Vec<&str> = self
            .0
            .chunks(3)
            .map(|row| std::str::from_utf8(row).expect("cells are ASCII"))
            .collect();
        let outcome = match (self.winner(), self.next()) {
            (Some(player), _) => format!("winner: {player}"),
            (None, Some(player)) => format!("next: {player}"),
            (None, None) => "draw".to_string(),
        };
        format!("{}\n{}", rows.join("\n"), outcome)
    }

    /// Read a board back from its own text.
    pub fn parse(text: &str) -> Option<Self> {
        let cells: Vec<u8> = text.lines().take(3).flat_map(|row| row.bytes()).collect();
        let cells: [u8; 9] = cells.try_into().ok()?;
        cells
            .iter()
            .all(|c| matches!(c, b'X' | b'O' | b'.'))
            .then_some(Board(cells))
    }

    // ANCHOR: apply
    /// The fold's one step: the board with `entry` played on it. Pure, so the board after
    /// move `n` depends on nothing but moves `1..=n`.
    pub fn apply(&self, entry: &Entry) -> Result<Board> {
        let expected = self.next().ok_or_else(|| {
            Error::Conflict("the game is over: no move follows a win or a draw".to_string())
        })?;
        if entry.player != expected {
            return Err(Error::Conflict(format!(
                "it is {expected}'s move, not {}'s",
                entry.player
            )));
        }
        let at = usize::from(entry.cell - 1);
        if self.0[at] != b'.' {
            return Err(Error::Conflict(format!(
                "cell {} is taken: {} played there",
                entry.cell, self.0[at] as char
            )));
        }
        let mut next = self.clone();
        next.0[at] = entry.player as u8;
        Ok(next)
    }
    // ANCHOR_END: apply
}

/// One move: who played, and where (`1` to `9`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub player: char,
    pub cell: u8,
}

impl Entry {
    /// `X5`.
    pub fn text(&self) -> String {
        format!("{}{}", self.player, self.cell)
    }

    /// Read `X5` back.
    pub fn parse(text: &str) -> Option<Self> {
        let mut chars = text.chars();
        let player = chars.next().filter(|p| matches!(p, 'X' | 'O'))?;
        let cell = cell(chars.as_str())?;
        Some(Entry { player, cell })
    }
}

fn cell(text: &str) -> Option<u8> {
    match text.as_bytes() {
        [d @ b'1'..=b'9'] => Some(d - b'0'),
        _ => None,
    }
}

// ---------------------------------------------------------------------------------------
// The log's store: the entries, in order. The only mutable state in the crate.
// ---------------------------------------------------------------------------------------

// ANCHOR: store
/// The moves, in order, and a count of how often the fold's step has run (so a test can
/// see which reads were answered from the cache and which computed).
#[derive(Debug, Default)]
pub struct Log {
    entries: Mutex<Vec<Entry>>,
    steps: AtomicUsize,
}
// ANCHOR_END: store

impl Log {
    /// How many times [`Board::apply`] has run inside a fold.
    pub fn steps(&self) -> usize {
        self.steps.load(Ordering::SeqCst)
    }

    fn head(&self) -> u64 {
        self.entries.lock().expect("the log's lock").len() as u64
    }

    fn at(&self, n: u64) -> Option<Entry> {
        let index = usize::try_from(n.checked_sub(1)?).ok()?;
        self.entries
            .lock()
            .expect("the log's lock")
            .get(index)
            .copied()
    }
}

fn iri(name: &str) -> Result<Iri> {
    Iri::parse(name).map_err(|e| Error::Endpoint(format!("the name {name}: {e}")))
}

async fn text_of(inv: &Invocation<'_>, name: &str) -> Result<String> {
    Ok(String::from_utf8_lossy(&inv.source(&iri(name)?).await?.bytes).into_owned())
}

fn position(inv: &Invocation<'_>) -> Result<u64> {
    let n = inv
        .bindings
        .get("n")
        .ok_or_else(|| Error::MissingArgument("n".to_string()))?;
    let spelled =
        !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()) && (n == "0" || !n.starts_with('0'));
    spelled
        .then(|| n.parse().ok())
        .flatten()
        .ok_or_else(|| Error::InvalidArgument {
            name: "n".to_string(),
            detail: format!("`{n}` is not a position: a whole number with no leading zeros"),
        })
}

fn position_input() -> ArgSpec {
    ArgSpec::new("n")
        .summary("a position in the log: 0 before any move, then 1, 2, ...")
        .class(XSD_INTEGER)
        .binding()
}

fn board(repr_of: &str, text: &str) -> Result<Board> {
    Board::parse(text).ok_or_else(|| {
        Error::Endpoint(format!("{repr_of} answered `{text}`, which is not a board"))
    })
}

async fn entry_at(inv: &Invocation<'_>, n: u64) -> Result<Entry> {
    let name = entry_name(n);
    let text = text_of(inv, &name).await?;
    Entry::parse(&text)
        .ok_or_else(|| Error::Endpoint(format!("{name} answered `{text}`, which is not a move")))
}

// ANCHOR: fold
/// The board at `base` (read by name, so from the cache when it is there) with the entries
/// `base + 1 ..= n` applied. The only place the fold's step runs.
async fn fold(inv: &Invocation<'_>, log: &Log, base: &str, from: u64, to: u64) -> Result<Board> {
    let mut board = board(base, &text_of(inv, base).await?)?;
    for n in from..=to {
        let entry = entry_at(inv, n).await?;
        log.steps.fetch_add(1, Ordering::SeqCst);
        board = board.apply(&entry)?;
    }
    Ok(board)
}
// ANCHOR_END: fold

fn answer(board: &Board) -> Representation {
    Representation::new(text_plain_utf8(), board.text().into_bytes()).cacheable()
}

// ANCHOR: log
/// `fold-log`: Source answers the head; Sink appends one move.
///
/// An append is checked against the board at the head before it is kept, so a log only
/// ever holds a game that can be played. The Sink cuts the log's golden thread, and that
/// is the only cut the whole crate makes: nothing else is ever written.
pub fn log(log: Arc<Log>) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "fold-log",
        move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            let log = Arc::clone(&log);
            Box::pin(async move {
                match inv.request.verb {
                    Verb::Sink => {
                        let content = inv.inline_str("content")?.trim();
                        let cell = cell(content).ok_or_else(|| Error::InvalidArgument {
                            name: "content".to_string(),
                            detail: format!(
                                "`{content}` is not a cell: 1 to 9, left to right, top to bottom"
                            ),
                        })?;
                        let head = log.head();
                        let now =
                            board(&state_name(head), &text_of(inv, &state_name(head)).await?)?;
                        let player = now.next().ok_or_else(|| {
                            Error::Conflict(
                                "the game is over: no move follows a win or a draw".to_string(),
                            )
                        })?;
                        let entry = Entry { player, cell };
                        now.apply(&entry)?;
                        let mut entries = log.entries.lock().expect("the log's lock");
                        if entries.len() as u64 != head {
                            return Err(Error::Conflict(
                                "another move was appended while this one was checked".to_string(),
                            ));
                        }
                        entries.push(entry);
                        Ok(Representation::new(
                            text_plain_utf8(),
                            (head + 1).to_string().into_bytes(),
                        ))
                    }
                    _ => Ok(Representation::new(
                        text_plain_utf8(),
                        log.head().to_string().into_bytes(),
                    )
                    .cacheable()),
                }
            })
        },
    )
    .with_description(
        Description::new("fold-log")
            .title("The game's log")
            .summary(
                "The moves of one tic-tac-toe game, in order: read the head, or append a move.",
            )
            .verb(Verb::Meta)
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("the head: how many moves the log holds")
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary(
                        "append a move for the player whose turn it is, and answer its position",
                    )
                    .input(
                        ArgSpec::new("content")
                            .summary("a cell, 1 to 9, left to right and top to bottom")
                            .class(XSD_STRING),
                    )
                    .output(TEXT_PLAIN_UTF8),
            ),
    )
}
// ANCHOR_END: log

// ANCHOR: entry
/// `fold-entry`: the move at position `n`. Cacheable forever: an entry is never rewritten.
pub fn entry(log: Arc<Log>) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "fold-entry",
        move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            let log = Arc::clone(&log);
            Box::pin(async move {
                let n = position(inv)?;
                let entry = log.at(n).ok_or_else(|| {
                    Error::NotFound(format!("the log has no move at position {n}"))
                })?;
                Ok(Representation::new(text_plain_utf8(), entry.text().into_bytes()).cacheable())
            })
        },
    )
    .with_description(
        Description::new("fold-entry")
            .title("A move")
            .summary("The move at a position in the log, e.g. X5. Never changes once written.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(position_input())
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: entry

// ANCHOR: checkpoint
/// `fold-checkpoint`: the board after `n` moves, `n` a multiple of [`EVERY`]; the previous
/// checkpoint with the `EVERY` entries after it applied.
pub fn checkpoint(log: Arc<Log>) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("fold-checkpoint", move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        let log = Arc::clone(&log);
        Box::pin(async move {
            let n = position(inv)?;
            if n % EVERY != 0 {
                return Err(Error::InvalidArgument {
                    name: "n".to_string(),
                    detail: format!("a checkpoint falls every {EVERY} moves, and {n} is not one"),
                });
            }
            if n == 0 {
                return Ok(answer(&Board::empty()));
            }
            let board = fold(inv, &log, &checkpoint_name(n - EVERY), n - EVERY + 1, n).await?;
            Ok(answer(&board))
        })
    })
    .with_description(
        Description::new("fold-checkpoint")
            .title("A checkpoint")
            .summary("The board after a multiple of three moves: the previous checkpoint and three entries.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(position_input())
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: checkpoint

// ANCHOR: state_at
/// `fold-state-at`: the board after `n` moves, `apply(state:seq:{n-1}, entry:seq:{n})`, or
/// the checkpoint itself when `n` is a multiple of [`EVERY`]. Cacheable forever: it reads
/// only entries, earlier positions and checkpoints, and none of them ever changes. So a warm
/// append costs one step, and a cold read recurses at most `EVERY - 1` positions before it
/// reaches a checkpoint, then through checkpoints only.
pub fn state_at(log: Arc<Log>) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "fold-state-at",
        move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            let log = Arc::clone(&log);
            Box::pin(async move {
                let n = position(inv)?;
                if n % EVERY == 0 {
                    // At a checkpoint the board IS the checkpoint: the recursion stops here.
                    let name = checkpoint_name(n);
                    return Ok(answer(&board(&name, &text_of(inv, &name).await?)?));
                }
                let board = fold(inv, &log, &state_name(n - 1), n, n).await?;
                Ok(answer(&board))
            })
        },
    )
    .with_description(
        Description::new("fold-state-at")
            .title("The board at a position")
            .summary("The board after n moves: a fold of an immutable prefix of the log.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(position_input())
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: state_at

// ANCHOR: state
/// `fold-state`: the board now. It reads the head, then the board at the head, so it hangs
/// from the log's golden thread: an append cuts it, and nothing else does.
pub fn state() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("fold-state", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let head = text_of(inv, LOG).await?;
            let head: u64 = head.parse().map_err(|_| {
                Error::Endpoint(format!("{LOG} answered `{head}`, which is not a head"))
            })?;
            let board = board(&state_name(head), &text_of(inv, &state_name(head)).await?)?;
            Ok(answer(&board))
        })
    })
    .with_description(
        Description::new("fold-state")
            .title("The board now")
            .summary("The board after every move in the log: the fold at the head.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: state

fn template(source: &str) -> UriTemplate {
    UriTemplate::parse(source).expect("a constant template parses")
}

// ANCHOR: space
/// The crate's names over a log of its own.
pub fn space() -> EndpointSpace {
    space_over(Arc::default())
}

/// The crate's names over `log`, which the caller keeps to look inside.
pub fn space_over(log: Arc<Log>) -> EndpointSpace {
    EndpointSpace::new()
        .bind(template(LOG), self::log(Arc::clone(&log)))
        .bind(template(ENTRY), entry(Arc::clone(&log)))
        .bind(template(CHECKPOINT), checkpoint(Arc::clone(&log)))
        .bind(template(STATE_AT), state_at(log))
        .bind(template(STATE), state())
}

/// A kernel over the crate's names and `log`.
pub fn kernel(log: Arc<Log>) -> Kernel {
    Kernel::new(Arc::new(space_over(log)))
}
// ANCHOR_END: space

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_board_reads_back_its_own_text() {
        let mut board = Board::empty();
        for (player, cell) in [('X', 5), ('O', 1), ('X', 9)] {
            board = board.apply(&Entry { player, cell }).unwrap();
            assert_eq!(Board::parse(&board.text()), Some(board.clone()));
        }
        assert_eq!(board.text(), "O..\n.X.\n..X\nnext: O");
    }

    #[test]
    fn an_entry_has_one_spelling() {
        assert_eq!(
            Entry::parse("X5"),
            Some(Entry {
                player: 'X',
                cell: 5
            })
        );
        for refused in ["", "X", "X0", "X10", "x5", "Z5", "5", "X5 "] {
            assert_eq!(Entry::parse(refused), None, "{refused:?}");
        }
    }
}
