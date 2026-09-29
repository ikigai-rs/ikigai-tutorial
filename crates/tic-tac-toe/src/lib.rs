//! The code taught by the book's applied chapter, **Tic-tac-toe**.
//!
//! The game is designed as resources before it is code, after *Resource Oriented Analysis
//! and Design*, Parts 1–9, by Peter Rodgers (NetKernel News 3.34–3.42, 2012). This crate
//! is ikigai's own version of that design, built one increment at a time; `README.md`
//! beside this file holds the plan for the whole arc.
//!
//! Increment 1 is the atom — the one piece of state a game of tic-tac-toe has:
//!
//! * `urn:iki:tutorial:ttt:stored:{x}:{y}` — the **stored cell**. What has been played at
//!   `(x, y)`, held in memory: `Source` reads the mark, `Sink` plays one, `Delete` clears
//!   it. A cell nobody has played is `NotFound`, because nothing is stored there.
//! * `urn:iki:tutorial:ttt:cell:{x}:{y}` — the **platonic cell**, which is what a reader
//!   asks for. It sources the stored cell and, where nothing has been played, answers the
//!   empty cell ([`EMPTY`]) instead of an error. Every cell of every board "already
//!   exists"; nobody has to create a game before reading one.
//!
//! Neither endpoint cuts or declares a golden thread, and that is the point of the chapter:
//! the kernel keeps both honest by itself.
//!
//! Increment 2 gathers cells into composites:
//!
//! * `urn:iki:tutorial:ttt:cells:{list}` — a **line of cells**, named by its members in
//!   order (`cells:0.0,1.0,2.0`). It sources each platonic cell and answers the marks
//!   side by side (`X-O`).
//! * `urn:iki:tutorial:ttt:row:{y}`, `column:{x}`, `diagonal:{n}` — **no endpoint**. Eight
//!   exact [`Alias`] rules ([`LINES`]) rename a line of cells, and the space the game
//!   exports is the endpoints wrapped in that table.
//! * `urn:iki:tutorial:ttt:board` — the three rows, read through the aliases, one per line.
//!
//! Increment 3 makes the rules resources, and puts the one constraint at the edge:
//!
//! * `urn:iki:tutorial:ttt:checkset:{x}:{y}` — the **CheckSet**: the names of the lines
//!   through a cell, read from [`LINES`]. Computed once and never un-cached.
//! * `urn:iki:tutorial:ttt:winner` and `urn:iki:tutorial:ttt:turn` — pure functions of the
//!   lines and the board, cacheable like them.
//! * `urn:iki:tutorial:ttt:move:{x}:{y}` — **Sink only**: plays the side to move, or
//!   refuses. The stored cell underneath still takes any mark anywhere.
//!
//! Increment 4 plays many games at once, and changes no name and no endpoint to do it:
//!
//! * A **game** is a resolution chain ([`game`]): one corridor, named
//!   `urn:iki:tutorial:ttt:game:{id}`, that binds the stored cell to that game's store. It
//!   sits ahead of the root for a request and everything the request gives rise to, so the
//!   board, the winner and the move all find this game's marks and nobody else's.
//! * The **store is a space**: [`space_with_store`] builds the game over whatever binds
//!   [`STORED`], and [`check_store`] is the contract such a space keeps — so a store in
//!   Python or TypeScript, served over the wire, is a game's store like the in-memory one.
//!
//! Increment 5 gives the game a face, and it is resources too:
//!
//! * `urn:iki:tutorial:ttt:template:{name}` — the game's HTML, in pieces, written in
//!   `ikigai-fn`'s template language ([`TEMPLATES`]): `$h{…}` for a value, `$r{…}` for a
//!   view, `{x}` for an argument. The same bytes for every host.
//! * `urn:iki:tutorial:ttt:view:board`, `view:square:{x}:{y}`, `view:status`,
//!   `view:reply` and `view:game:{game}` — each a template bound at a name
//!   ([`view`], `ikigai_fn::compose_over`), with no view code: which square or status to
//!   show is `urn:iki:fn:conditional` in a template. Composites like the rest, so cached,
//!   recomputed by the move that touches them, and per game through the corridor.
//! * `urn:iki:tutorial:ttt:view:play:{x}:{y}` and `view:reset` — the view's writes, `Sink`
//!   only: a move (or [`RESET`]) through the kernel, answered with `view:reply`.
//!
//! Read the book: `mdbook serve books/ikigai`, or `./scripts/serve-with-drafts.sh` while
//! the chapters are still drafts.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, Alias, AliasTable, ArgRef, ArgSpec, AsyncFnEndpoint, Capability, Description,
    EndpointSpace, Error, Exact, Expiry, Fallback, FnEndpoint, Invocation, InvokeFuture, Iri,
    Kernel, ReprType, Representation, Request, Result, Scope, Space, UriTemplate, Verb,
};
use ikigai_fn::ComposeOver;
use ikigai_vocab::TurtleRenderer;

/// `text/plain; charset=utf-8` as a [`ReprType`] — a local helper, as in `hello-camel`.
fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

/// The same media type as a string, for a [`Description`].
const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";

/// `xsd:integer`, the class a coordinate declares.
const XSD_INTEGER: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// `xsd:string`, the class a mark declares.
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

// ANCHOR: names
/// The stored cell: what has been played at `(x, y)`.
pub const STORED: &str = "urn:iki:tutorial:ttt:stored:{x}:{y}";

/// The platonic cell: what a reader asks for.
pub const CELL: &str = "urn:iki:tutorial:ttt:cell:{x}:{y}";

/// What the platonic cell answers where nothing has been played.
///
/// A hyphen rather than a space, because this is read as text: a single space is
/// invisible in a terminal and in the page's output pane, where it cannot be told apart
/// from "nothing came back". A hyphen also lines up in a monospaced board — `X - O`.
pub const EMPTY: &str = "-";
// ANCHOR_END: names

/// The stored cell's name at `(x, y)`, spelled the one way the endpoint accepts.
pub fn stored_name(x: i64, y: i64) -> String {
    format!("urn:iki:tutorial:ttt:stored:{x}:{y}")
}

/// The platonic cell's name at `(x, y)`.
pub fn cell_name(x: i64, y: i64) -> String {
    format!("urn:iki:tutorial:ttt:cell:{x}:{y}")
}

// ANCHOR: coordinate
/// A coordinate, captured from the name by the template.
///
/// **Any** integer: `-3` and `40` are cells as surely as `1` is. The model is loose on
/// purpose; a board's edges are a rule of one game, and a rule arrives at the edge of the
/// application, not in the grammar of its atoms.
///
/// But exactly one **spelling** per integer. `01`, `+1` and `-0` are refused, because two
/// names for one cell would be two golden threads over one piece of state: a write
/// through one name would cut its own thread and leave a cached read under the other
/// serving the old mark.
fn coordinate(inv: &Invocation<'_>, name: &str) -> Result<i64> {
    let text = inv
        .bindings
        .get(name)
        .ok_or_else(|| Error::MissingArgument(name.to_string()))?;
    plain_integer(name, text)
}

/// `text` as an integer, if it is spelled the one way that integer is spelled.
fn plain_integer(name: &str, text: &str) -> Result<i64> {
    match text.parse::<i64>() {
        Ok(value) if value.to_string() == text => Ok(value),
        _ => Err(Error::InvalidArgument {
            name: name.to_string(),
            detail: format!("`{text}` is not an integer in its plain form (e.g. 0, 2, -1)"),
        }),
    }
}
// ANCHOR_END: coordinate

/// The two coordinates, as a description declares them: captured from the name, not
/// passed as arguments.
fn coordinates() -> [ArgSpec; 2] {
    [
        ArgSpec::new("x")
            .summary("the column — any integer")
            .class(XSD_INTEGER)
            .binding(),
        ArgSpec::new("y")
            .summary("the row — any integer")
            .class(XSD_INTEGER)
            .binding(),
    ]
}

// ANCHOR: store
/// The marks that have been played, and how often the stored cell has been read.
///
/// The state lives outside the endpoint, where a test can hold it — the same shape as
/// `hello-camel`'s `TitleState`. The read counter is what lets a test prove that a cached
/// answer was *served*, with no code running, rather than recomputed to the same bytes.
#[derive(Debug, Default)]
pub struct CellStore {
    marks: Mutex<BTreeMap<(i64, i64), String>>,
    /// How many times the stored cell has actually been read — its `Source` or `Exists`
    /// run, rather than a cached answer served.
    pub reads: AtomicUsize,
}
// ANCHOR_END: store

// ANCHOR: stored
/// `ttt-stored`: the mark at `(x, y)`, held in memory.
///
/// `Source` answers the mark, `.cacheable()`, and says nothing about golden threads: the
/// kernel hangs every cacheable read from the thread named after the resource it read,
/// and cuts that thread after every successful `Sink` or `Delete` to the same name. A
/// cell nobody has played is [`Error::NotFound`] — the endpoint is bound, and there is
/// nothing there.
///
/// `Exists` answers `true` or `false`: is a mark played here? Never `NotFound`, since the
/// name is bound either way. Cacheable like the mark, so the same write cuts it.
///
/// `Sink` takes the mark as `content` and keeps any mark — `X`, `O`, or a Connect-Four
/// `R` — refusing only an empty one, since "no mark" is what `Delete` says. Whether this
/// player may play this cell now is a rule, and rules are for a later increment.
pub fn stored_cell(store: Arc<CellStore>) -> FnEndpoint {
    FnEndpoint::new("ttt-stored", move |inv: &Invocation<'_>| {
        let at = (coordinate(inv, "x")?, coordinate(inv, "y")?);
        let mut marks = store.marks.lock().expect("the cell store's lock");
        match inv.request.verb {
            Verb::Sink => {
                let mark = inv.inline_str("content")?.trim();
                if mark.is_empty() {
                    return Err(Error::InvalidArgument {
                        name: "content".to_string(),
                        detail: "an empty mark — to clear a cell, delete it".to_string(),
                    });
                }
                marks.insert(at, mark.to_string());
                Ok(Representation::new(text_plain_utf8(), b"ok".to_vec()))
            }
            Verb::Delete => {
                marks.remove(&at);
                Ok(Representation::new(text_plain_utf8(), b"ok".to_vec()))
            }
            Verb::Exists => {
                store.reads.fetch_add(1, Ordering::SeqCst);
                let played = if marks.contains_key(&at) {
                    "true"
                } else {
                    "false"
                };
                Ok(Representation::new(text_plain_utf8(), played.as_bytes().to_vec()).cacheable())
            }
            _ => {
                store.reads.fetch_add(1, Ordering::SeqCst);
                match marks.get(&at) {
                    Some(mark) => Ok(Representation::new(
                        text_plain_utf8(),
                        mark.clone().into_bytes(),
                    )
                    .cacheable()),
                    None => Err(Error::NotFound(format!(
                        "nothing has been played at {},{}",
                        at.0, at.1
                    ))),
                }
            }
        }
    })
    .with_description(
        Description::new("ttt-stored")
            .title("Stored cell")
            .summary("What has been played at (x, y): read it, play a mark, or clear it.")
            .verb(Verb::Meta)
            // Three verbs, three contracts, so each gets its own action — and each names
            // the coordinates, because an explicit action does not inherit flat inputs.
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("the mark played at (x, y); NotFound if none has been")
                    .input(coordinates()[0].clone())
                    .input(coordinates()[1].clone())
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("play a mark at (x, y)")
                    .input(coordinates()[0].clone())
                    .input(coordinates()[1].clone())
                    .input(
                        ArgSpec::new("content")
                            .summary("the mark to play, e.g. X or O")
                            .class(XSD_STRING),
                    )
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Exists)
                    .summary("true if a mark has been played at (x, y), false if none has")
                    .input(coordinates()[0].clone())
                    .input(coordinates()[1].clone())
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Delete)
                    .summary("clear the cell at (x, y)")
                    .input(coordinates()[0].clone())
                    .input(coordinates()[1].clone())
                    .output(TEXT_PLAIN_UTF8),
            ),
    )
}
// ANCHOR_END: stored

// ANCHOR: platonic
/// `ttt-cell`: the cell at `(x, y)` as a reader sees it — the mark, or [`EMPTY`].
///
/// A composite: it *resolves* the stored cell through the kernel and turns a `NotFound`
/// into the empty cell. Any other failure (a coordinate in the wrong spelling, say) is
/// passed on untouched: only "nothing has been played" means empty.
///
/// The empty answer is `.cacheable()`, and it is still correct after a mark is played,
/// with no code here to make it so. A sub-request that failed is recorded as a
/// dependency like one that succeeded, so the cached empty cell hangs from the stored
/// cell's thread — and the `Sink` that plays the cell cuts it.
pub fn platonic_cell() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("ttt-cell", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let (x, y) = (coordinate(inv, "x")?, coordinate(inv, "y")?);
            let stored = Iri::parse(stored_name(x, y))
                .map_err(|e| Error::Endpoint(format!("the stored cell's name: {e}")))?;
            let mark = match inv.source(&stored).await {
                Ok(played) => played.bytes,
                Err(Error::NotFound(_)) => EMPTY.as_bytes().to_vec(),
                Err(other) => return Err(other),
            };
            Ok(Representation::new(text_plain_utf8(), mark).cacheable())
        })
    })
    .with_description(
        Description::new("ttt-cell")
            .title("Cell")
            .summary("The cell at (x, y): the mark played there, or - where none has been.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(coordinates()[0].clone())
            .input(coordinates()[1].clone())
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: platonic

// ANCHOR: line_names
/// A line of cells: the members, in order, each spelled `x.y`, joined by `,`.
pub const CELLS: &str = "urn:iki:tutorial:ttt:cells:{list}";

/// The board: every row, top to bottom.
pub const BOARD: &str = "urn:iki:tutorial:ttt:board";
// ANCHOR_END: line_names

/// The name of the line through `members`, in order — spelled the one way the endpoint
/// accepts.
pub fn cells_name(members: &[(i64, i64)]) -> String {
    let list: Vec<String> = members.iter().map(|(x, y)| format!("{x}.{y}")).collect();
    format!("urn:iki:tutorial:ttt:cells:{}", list.join(","))
}

/// The name of row `y` — an alias, bound by [`LINES`], not by an endpoint.
pub fn row_name(y: i64) -> String {
    format!("urn:iki:tutorial:ttt:row:{y}")
}

// ANCHOR: members
/// The members of a line, read out of its name: `0.0,1.0,2.0` is `(0,0) (1,0) (2,0)`.
///
/// The same strictness as a single cell's coordinates, for the same reason. Every member
/// is two integers in their plain form, joined by one `.`; members are joined by one `,`;
/// nothing else — no spaces, no empty member, no trailing comma. A list has exactly one
/// spelling, so a line has exactly one name, and one name is one cache entry and one
/// golden thread. The model stays loose about *which* cells: any coordinates, any
/// length, repeats allowed.
fn members(inv: &Invocation<'_>) -> Result<Vec<(i64, i64)>> {
    let list = inv
        .bindings
        .get("list")
        .ok_or_else(|| Error::MissingArgument("list".to_string()))?;
    let refuse = |detail: String| Error::InvalidArgument {
        name: "list".to_string(),
        detail,
    };
    list.split(',')
        .map(|member| match member.split_once('.') {
            _ if member.is_empty() => Err(refuse(
                "an empty member — cells are x.y, joined by single commas".to_string(),
            )),
            Some((x, y)) => Ok((plain_integer("list", x)?, plain_integer("list", y)?)),
            None => Err(refuse(format!(
                "`{member}` is not a cell spelled x.y (e.g. 0.2)"
            ))),
        })
        .collect()
}
// ANCHOR_END: members

// ANCHOR: line
/// `ttt-cells`: a line of cells — the marks of its members, side by side, in order.
///
/// A composite over the platonic cell, and nothing more: it sources each member through
/// the kernel and concatenates the answers, so an empty cell reads `-` and a row reads
/// like `X-O`. It is `.cacheable()` and names no thread; its dependencies are its
/// sub-requests, so a move that changes one member cuts this line's cached answer and
/// leaves every line that does not pass through that cell alone.
pub fn line_of_cells() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("ttt-cells", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let mut marks = Vec::new();
            for (x, y) in members(inv)? {
                let cell = Iri::parse(cell_name(x, y))
                    .map_err(|e| Error::Endpoint(format!("a cell's name: {e}")))?;
                marks.extend(inv.source(&cell).await?.bytes);
            }
            Ok(Representation::new(text_plain_utf8(), marks).cacheable())
        })
    })
    .with_description(
        Description::new("ttt-cells")
            .title("Line of cells")
            .summary("The marks of the named cells, in order, side by side: X-O.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("list")
                    .summary("the cells, in order: x.y joined by commas, e.g. 0.0,1.0,2.0")
                    .class(XSD_STRING)
                    .binding(),
            )
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: line

// ANCHOR: lines
/// The rows, columns and diagonals of a 3×3 board, as names for lines of cells.
///
/// This table is where "three by three" lives — not in the cell, which takes any
/// coordinates, and not in the line, which takes any cells. `y` runs down the board, so
/// row 0 is the top row; a line reads left to right, or top to bottom, and diagonal 1
/// runs from the top-right corner to the bottom-left.
pub const LINES: &str = "\
exact urn:iki:tutorial:ttt:row:0      urn:iki:tutorial:ttt:cells:0.0,1.0,2.0
exact urn:iki:tutorial:ttt:row:1      urn:iki:tutorial:ttt:cells:0.1,1.1,2.1
exact urn:iki:tutorial:ttt:row:2      urn:iki:tutorial:ttt:cells:0.2,1.2,2.2
exact urn:iki:tutorial:ttt:column:0   urn:iki:tutorial:ttt:cells:0.0,0.1,0.2
exact urn:iki:tutorial:ttt:column:1   urn:iki:tutorial:ttt:cells:1.0,1.1,1.2
exact urn:iki:tutorial:ttt:column:2   urn:iki:tutorial:ttt:cells:2.0,2.1,2.2
exact urn:iki:tutorial:ttt:diagonal:0 urn:iki:tutorial:ttt:cells:0.0,1.1,2.2
exact urn:iki:tutorial:ttt:diagonal:1 urn:iki:tutorial:ttt:cells:2.0,1.1,0.2
";
// ANCHOR_END: lines

/// [`LINES`], parsed.
pub fn lines() -> AliasTable {
    AliasTable::parse(LINES).expect("the constant alias table parses")
}

// ANCHOR: board
/// `ttt-board`: the three rows, top to bottom, one per line of text.
///
/// It reads the rows by their alias names, through the kernel, like any other caller —
/// it does not know that a row is a line of cells, or that a line is cells. Cacheable,
/// with no thread named: the move that changes one cell reaches the board through the
/// one row that holds the cell, and through nothing else.
pub fn board() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("ttt-board", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let mut rows = Vec::new();
            for y in 0..3 {
                let row = Iri::parse(row_name(y))
                    .map_err(|e| Error::Endpoint(format!("a row's name: {e}")))?;
                rows.push(String::from_utf8_lossy(&inv.source(&row).await?.bytes).into_owned());
            }
            Ok(Representation::new(text_plain_utf8(), rows.join("\n").into_bytes()).cacheable())
        })
    })
    .with_description(
        Description::new("ttt-board")
            .title("Board")
            .summary("The board: row 0 (the top row) to row 2, one row per line.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: board

// ANCHOR: rule_names
/// Which lines pass through `(x, y)`: a list of their names.
pub const CHECKSET: &str = "urn:iki:tutorial:ttt:checkset:{x}:{y}";

/// Who has won: `X`, `O`, [`EMPTY`] for nobody yet, or [`DRAW`].
pub const WINNER: &str = "urn:iki:tutorial:ttt:winner";

/// Whose move it is: `X` or `O`, or [`EMPTY`] once the game is over.
pub const TURN: &str = "urn:iki:tutorial:ttt:turn";

/// Play the side to move at `(x, y)` — `Sink` only.
pub const MOVE: &str = "urn:iki:tutorial:ttt:move:{x}:{y}";

/// What the winner answers when every square is played and no line is won.
pub const DRAW: &str = "draw";
// ANCHOR_END: rule_names

/// The CheckSet's name at `(x, y)`.
pub fn checkset_name(x: i64, y: i64) -> String {
    format!("urn:iki:tutorial:ttt:checkset:{x}:{y}")
}

/// The move's name at `(x, y)`.
pub fn move_name(x: i64, y: i64) -> String {
    format!("urn:iki:tutorial:ttt:move:{x}:{y}")
}

/// Source `name` through the invocation and answer its text — a sub-request, so a
/// dependency of whatever the caller computes from it.
async fn text_of(inv: &Invocation<'_>, name: &str) -> Result<String> {
    let iri =
        Iri::parse(name).map_err(|e| Error::Endpoint(format!("the game's name {name}: {e}")))?;
    Ok(String::from_utf8_lossy(&inv.source(&iri).await?.bytes).into_owned())
}

// ANCHOR: checkset
/// The names of the lines `table` makes that pass through the cell spelled `member`
/// (`x.y`), in name order.
///
/// Read from the table, never listed by hand: a line passes through a cell when its
/// target's member list holds that cell. A string match is exact because a cell has one
/// spelling. The answer is the ALIAS names — `row:1`, not `cells:0.1,1.1,2.1` — because
/// they read as the game; each resolves to its line, and the line's name is the cache key.
/// Sorted, because a table keeps its rules most-specific-first, not in the order written.
fn lines_through<'t>(table: &'t AliasTable, member: &str) -> Vec<&'t str> {
    let line = CELLS.trim_end_matches("{list}");
    let mut names: Vec<&str> = table
        .rules()
        .iter()
        .filter(|rule| {
            rule.to()
                .strip_prefix(line)
                .is_some_and(|list| list.split(',').any(|m| m == member))
        })
        .map(|rule| rule.from())
        .collect();
    names.sort_unstable();
    names
}

/// `ttt-checkset`: the lines through `(x, y)`, one name per line of text.
///
/// A representation whose content is the names of other resources. It depends on its
/// name and a constant table and on nothing else — no sub-request, so no dependency any
/// `Sink` could cut — and `.cacheable()` is `Expiry::Never`: computed once in the life of
/// the kernel, and served from then on, whatever is played. A cell off the board is on no
/// line and answers the empty list; the cell itself is still a cell.
pub fn checkset(table: Arc<AliasTable>) -> FnEndpoint {
    FnEndpoint::new("ttt-checkset", move |inv: &Invocation<'_>| {
        let member = format!("{}.{}", coordinate(inv, "x")?, coordinate(inv, "y")?);
        let names = lines_through(&table, &member).join("\n");
        Ok(Representation::new(text_plain_utf8(), names.into_bytes()).cacheable())
    })
    .with_description(
        Description::new("ttt-checkset")
            .title("CheckSet")
            .summary(
                "The names of the lines through (x, y), one per line; none for a cell off \
                 the board.",
            )
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(coordinates()[0].clone())
            .input(coordinates()[1].clone())
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: checkset

// ANCHOR: winner
/// The mark that fills `line` — every mark one kind, and not the empty cell — if one does.
fn filled_by(line: &str) -> Option<char> {
    let mut marks = line.chars();
    let first = marks.next()?;
    (first.to_string() != EMPTY && marks.all(|mark| mark == first)).then_some(first)
}

/// `ttt-winner`: `X` or `O` if a line is filled by one of them, [`DRAW`] if every square
/// is played and none is, and [`EMPTY`] while the game goes on.
///
/// A pure function of the lines, read by their alias names from the table, through the
/// kernel. It names no thread: its dependencies are the eight lines, so a move recomputes
/// the lines through its square and this, and serves the others from cache. A board no
/// game reaches — both marks with a full line, played straight into the store — answers
/// the mark of whichever full line the table lists first.
pub fn winner(table: Arc<AliasTable>) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "ttt-winner",
        move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            let table = Arc::clone(&table);
            Box::pin(async move {
                let mut lines = Vec::new();
                for rule in table.rules() {
                    lines.push(text_of(inv, rule.from()).await?);
                }
                let answer = match lines.iter().find_map(|line| filled_by(line)) {
                    Some(mark) => mark.to_string(),
                    None if lines.iter().all(|line| !line.contains(EMPTY)) => DRAW.to_string(),
                    None => EMPTY.to_string(),
                };
                Ok(Representation::new(text_plain_utf8(), answer.into_bytes()).cacheable())
            })
        },
    )
    .with_description(
        Description::new("ttt-winner")
            .title("Winner")
            .summary("X or O once a line is theirs, draw on a full board, - until then.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: winner

// ANCHOR: turn
/// `ttt-turn`: whose move it is — `X` when the marks are level (so `X` opens), `O` when
/// `X` is ahead — and [`EMPTY`] once the winner says the game is over.
///
/// A pure function of two resources, the winner and the board, and cacheable for the same
/// reason they are.
pub fn turn() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("ttt-turn", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let side = if text_of(inv, WINNER).await? != EMPTY {
                EMPTY
            } else {
                let board = text_of(inv, BOARD).await?;
                let count = |mark: char| board.chars().filter(|&c| c == mark).count();
                if count('X') <= count('O') {
                    "X"
                } else {
                    "O"
                }
            };
            Ok(Representation::new(text_plain_utf8(), side.as_bytes().to_vec()).cacheable())
        })
    })
    .with_description(
        Description::new("ttt-turn")
            .title("Turn")
            .summary("Whose move it is: X (who opens) or O; - once the game is over.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: turn

// ANCHOR: move
/// `ttt-move`: play the side to move at `(x, y)` — the one place the rules are enforced.
///
/// Each check reads a resource the game already has, and the move itself is a `Sink` to
/// the stored cell issued through the invocation — the same request a player could make
/// by hand, and so the same cut of the stored cell's thread. Nothing about turns or boards
/// moves into the stored cell, which still takes any mark at any square.
///
/// Refused, with [`Error::InvalidArgument`]: a square no line passes through (off the
/// board), any square once the game is over, a square already played, and a `content`
/// mark that is not the side to play. `content` is optional; left out, the move plays
/// whoever's turn it is.
pub fn play_move() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("ttt-move", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            if inv.request.verb != Verb::Sink {
                return Err(Error::Endpoint(
                    "a move is played, not read: `sink` it (read the board instead)".into(),
                ));
            }
            let (x, y) = (coordinate(inv, "x")?, coordinate(inv, "y")?);
            let square = |detail: String| Error::InvalidArgument {
                name: "x, y".to_string(),
                detail,
            };
            if text_of(inv, &checkset_name(x, y)).await?.is_empty() {
                return Err(square(format!(
                    "{x},{y} is off the board — no line passes through it"
                )));
            }
            match text_of(inv, WINNER).await?.as_str() {
                won if won == EMPTY => {}
                DRAW => return Err(square("the game is over — a draw".to_string())),
                mark => return Err(square(format!("the game is over — {mark} has won"))),
            }
            let there = text_of(inv, &cell_name(x, y)).await?;
            if there != EMPTY {
                return Err(square(format!("{x},{y} is taken — {there} played there")));
            }
            let side = text_of(inv, TURN).await?;
            let offered = match inv.inline_str("content") {
                Ok(mark) => mark.trim(),
                Err(Error::MissingArgument(_)) => "",
                Err(other) => return Err(other),
            };
            if !offered.is_empty() && offered != side {
                return Err(Error::InvalidArgument {
                    name: "content".to_string(),
                    detail: format!("it is {side}'s turn, not {offered}'s"),
                });
            }
            let stored = Iri::parse(stored_name(x, y))
                .map_err(|e| Error::Endpoint(format!("the stored cell's name: {e}")))?;
            let play = Request::new(Verb::Sink, stored)
                .with_arg("content", ArgRef::Inline(side.clone().into_bytes()));
            inv.issue(play).await?;
            Ok(Representation::new(
                text_plain_utf8(),
                format!("{side} plays {x},{y}").into_bytes(),
            ))
        })
    })
    .with_description(
        Description::new("ttt-move")
            .title("Move")
            .summary(
                "Play the side to move at (x, y), if the rules allow it: on the board, the \
                 game not over, the square free, and the mark (if given) the side to play.",
            )
            .verb(Verb::Sink)
            .verb(Verb::Meta)
            .input(coordinates()[0].clone())
            .input(coordinates()[1].clone())
            .input(
                ArgSpec::new("content")
                    .summary(
                        "the mark to play — checked against the turn; left out, the turn plays",
                    )
                    .class(XSD_STRING)
                    .one_of(["X", "O"])
                    .optional(),
            )
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: move

// ANCHOR: view_names
/// A template: HTML in `ikigai-fn`'s template language, the same bytes for every host.
pub const TEMPLATE: &str = "urn:iki:tutorial:ttt:template:{name}";

/// The board as HTML: one button per square, each empty one a move.
pub const VIEW_BOARD: &str = "urn:iki:tutorial:ttt:view:board";

/// One square as HTML: open, taken or closed, as the cell and the winner say.
pub const VIEW_SQUARE: &str = "urn:iki:tutorial:ttt:view:square:{x}:{y}";

/// Whose turn it is, or who won, as one line of text.
pub const VIEW_STATUS: &str = "urn:iki:tutorial:ttt:view:status";

/// What a write answers: its `message` argument, then the status.
pub const VIEW_REPLY: &str = "urn:iki:tutorial:ttt:view:reply";

/// The page shell of game `{game}`: the status line, the board and the New game button.
pub const VIEW_GAME: &str = "urn:iki:tutorial:ttt:view:game:{game}";

/// Play the side to move at `(x, y)`, and answer with what happened and the status —
/// `Sink` only.
pub const VIEW_PLAY: &str = "urn:iki:tutorial:ttt:view:play:{x}:{y}";

/// Clear the board, and answer with what happened and the status — `Sink` only.
pub const VIEW_RESET: &str = "urn:iki:tutorial:ttt:view:reset";

/// Clear every square of the board — `Sink` only.
pub const RESET: &str = "urn:iki:tutorial:ttt:reset";

/// The one resource the views use that the game does not bind: `ikigai-fn`'s
/// `conditional`, at its conventional name. A host binds it ([`conditional_space`]).
pub const CONDITIONAL: &str = "urn:iki:fn:conditional";
// ANCHOR_END: view_names

/// Every template, by the name it is resolved at: `template:{name}`.
///
/// The files live beside this crate's source (`templates/*.html`) so they can be read and
/// edited as HTML. A file's final newline is not part of the template.
pub const TEMPLATES: &[(&str, &str)] = &[
    ("game", include_str!("../templates/game.html")),
    ("board", include_str!("../templates/board.html")),
    ("square", include_str!("../templates/square.html")),
    (
        "square-empty",
        include_str!("../templates/square-empty.html"),
    ),
    ("square-open", include_str!("../templates/square-open.html")),
    (
        "square-taken",
        include_str!("../templates/square-taken.html"),
    ),
    (
        "square-closed",
        include_str!("../templates/square-closed.html"),
    ),
    ("status", include_str!("../templates/status.html")),
    ("status-over", include_str!("../templates/status-over.html")),
    ("status-turn", include_str!("../templates/status-turn.html")),
    ("status-won", include_str!("../templates/status-won.html")),
    ("status-draw", include_str!("../templates/status-draw.html")),
    ("reply", include_str!("../templates/reply.html")),
];

/// The template `name`'s resource name.
pub fn template_name(name: &str) -> String {
    format!("urn:iki:tutorial:ttt:template:{name}")
}

/// The name of the view that plays `(x, y)`.
pub fn view_play_name(x: i64, y: i64) -> String {
    format!("urn:iki:tutorial:ttt:view:play:{x}:{y}")
}

/// The name of the view of the square at `(x, y)`.
pub fn view_square_name(x: i64, y: i64) -> String {
    format!("urn:iki:tutorial:ttt:view:square:{x}:{y}")
}

/// The name of game `id`'s page shell.
pub fn view_game_name(id: &str) -> String {
    format!("urn:iki:tutorial:ttt:view:game:{id}")
}

/// `text/html; charset=utf-8` as a [`ReprType`].
fn text_html_utf8() -> ReprType {
    ReprType::new("text/html").with_param("charset", "utf-8")
}

/// The same media type as a string, for a [`Description`].
const TEXT_HTML_UTF8: &str = "text/html;charset=utf-8";

/// `ttt-template`: the template `name`, as `text/html`, computed once.
pub fn templates() -> FnEndpoint {
    FnEndpoint::new("ttt-template", |inv: &Invocation<'_>| {
        let name = inv
            .bindings
            .get("name")
            .ok_or_else(|| Error::MissingArgument("name".to_string()))?;
        let (_, text) = TEMPLATES
            .iter()
            .find(|(known, _)| *known == name)
            .ok_or_else(|| Error::NotFound(format!("there is no template named `{name}`")))?;
        let text = text.strip_suffix('\n').unwrap_or(text);
        Ok(Representation::new(text_html_utf8(), text.as_bytes().to_vec()).cacheable())
    })
    .with_description(
        Description::new("ttt-template")
            .title("Template")
            .summary(
                "A piece of the game's HTML, with $h{…}, $r{…} and $a{…} markers that \
                 ikigai-fn's compose fills.",
            )
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("name")
                    .summary("which template")
                    .class(XSD_STRING)
                    .one_of(TEMPLATES.iter().map(|(name, _)| *name))
                    .binding(),
            )
            .output(TEXT_HTML_UTF8),
    )
}

// ANCHOR: view
/// A view: the template `name`, filled by `ikigai-fn`'s template language and bound at a
/// name of its own. The variables the name captures (`{x}`, `{y}`, `{game}`) and the
/// request's arguments (`message`) are the template's arguments.
///
/// There is no view code. The template names what it shows — a cell, the winner, another
/// view — and `urn:iki:fn:conditional` inside it chooses between templates. Every one of
/// those is sourced through the kernel, so a view is a composite like the rest: cacheable,
/// recomputed by the move that touches what it read, and per game through the corridor.
pub fn view(name: &str) -> ComposeOver {
    ikigai_fn::compose_over(Iri::parse(template_name(name)).expect("a template's name parses"))
}
// ANCHOR_END: view

// ANCHOR: view_play
/// What a view's `Sink` answers: `view:reply`, the `reply` template filled with `message`
/// — what the write said — and the status.
///
/// `message` is whatever the write said, success or refusal alike, as text: the view does
/// not look inside an error to decide how to show it, so a refusal of any kind reaches the
/// player in the kernel's own words. It is an ARGUMENT of the reply, and the template
/// splices it with `$h`, so it is escaped and never expanded, whatever the error says.
async fn reply(inv: &Invocation<'_>, outcome: Result<Representation>) -> Result<Representation> {
    let message = match outcome {
        Ok(said) => String::from_utf8_lossy(&said.bytes).into_owned(),
        Err(refused) => refused.to_string(),
    };
    let reply = Request::new(Verb::Source, name_iri(VIEW_REPLY)?)
        .with_arg("message", ArgRef::Inline(message.into_bytes()));
    let answer = inv.issue(reply).await?;
    // A write's answer is not cacheable: it says what THIS write did.
    Ok(Representation::new(answer.repr_type, answer.bytes))
}

/// `ttt-view-play`: play the side to move at `(x, y)` — a `Sink` to the move, through the
/// kernel — and answer the reply: `X plays 1,1. O to play.` A refused move is answered, not
/// failed: `…1,1 is taken — X played there. O to play.` This is the view's one write, and it
/// changes nothing about the move: the rules are still the move's.
///
/// It stays code because it is a SEQUENCE — write, then read the reply — and composition
/// only reads: a template's markers are all `Source` requests.
pub fn view_play() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "ttt-view-play",
        |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            Box::pin(async move {
                sink_only(
                    inv,
                    "a play is made, not read: `sink` it (read the board view)",
                )?;
                let (x, y) = (coordinate(inv, "x")?, coordinate(inv, "y")?);
                let play = Request::new(Verb::Sink, name_iri(&move_name(x, y))?);
                let outcome = inv.issue(play).await;
                reply(inv, outcome).await
            })
        },
    )
    .with_description(
        Description::new("ttt-view-play")
            .title("Play, as a view")
            .summary(
                "Play the side to move at (x, y) and answer, as HTML text, what happened and \
                 whose turn it is — a refused move included.",
            )
            .verb(Verb::Sink)
            .verb(Verb::Meta)
            .input(coordinates()[0].clone())
            .input(coordinates()[1].clone())
            .output(TEXT_HTML_UTF8),
    )
}
// ANCHOR_END: view_play

/// `ttt-view-reset`: clear the board ([`RESET`]) and answer the reply.
pub fn view_reset() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "ttt-view-reset",
        |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            Box::pin(async move {
                sink_only(inv, "a reset is made, not read: `sink` it")?;
                let outcome = inv.issue(Request::new(Verb::Sink, name_iri(RESET)?)).await;
                reply(inv, outcome).await
            })
        },
    )
    .with_description(
        Description::new("ttt-view-reset")
            .title("New game, as a view")
            .summary("Clear the board and answer, as HTML text, that it is clear and who opens.")
            .verb(Verb::Sink)
            .verb(Verb::Meta)
            .output(TEXT_HTML_UTF8),
    )
}

// ANCHOR: reset
/// `ttt-reset`: clear every square a line of `table` passes through — a `Delete` of each
/// stored cell, issued through the kernel, so each one cuts its own thread.
///
/// The squares come from the table, like the CheckSet's lines, so a bigger board's table
/// resets a bigger board. The stored cell's `Delete` succeeds on an empty square, so a reset
/// of a fresh board is not an error.
pub fn reset(table: Arc<AliasTable>) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "ttt-reset",
        move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            let table = Arc::clone(&table);
            Box::pin(async move {
                sink_only(inv, "a reset is made, not read: `sink` it")?;
                let line = CELLS.trim_end_matches("{list}");
                let mut squares = std::collections::BTreeSet::new();
                for rule in table.rules() {
                    if let Some(list) = rule.to().strip_prefix(line) {
                        for member in list.split(',') {
                            let (x, y) = member.split_once('.').ok_or_else(|| {
                                Error::Endpoint(format!("the line table's member `{member}`"))
                            })?;
                            squares.insert((plain_integer("x", x)?, plain_integer("y", y)?));
                        }
                    }
                }
                for (x, y) in squares {
                    inv.issue(Request::new(Verb::Delete, name_iri(&stored_name(x, y))?))
                        .await?;
                }
                Ok(Representation::new(
                    text_plain_utf8(),
                    b"The board is clear".to_vec(),
                ))
            })
        },
    )
    .with_description(
        Description::new("ttt-reset")
            .title("New game")
            .summary("Clear every square of the board, so the next move opens a new game.")
            .verb(Verb::Sink)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: reset

/// Refuse any verb but `Sink`, saying `why`.
fn sink_only(inv: &Invocation<'_>, why: &str) -> Result<()> {
    if inv.request.verb == Verb::Sink {
        Ok(())
    } else {
        Err(Error::Endpoint(why.to_string()))
    }
}

/// A name the game builds, parsed.
fn name_iri(name: &str) -> Result<Iri> {
    Iri::parse(name).map_err(|e| Error::Endpoint(format!("the game's name {name}: {e}")))
}

// ANCHOR: space
/// The game's space: the endpoints, wrapped in the table that names the lines.
pub fn space() -> Alias {
    space_over(Arc::default())
}

/// [`space`], with the stored cells in a [`CellStore`] the caller keeps — so a test can
/// read the counter.
pub fn space_over(store: Arc<CellStore>) -> Alias {
    space_with_store(Arc::new(stored_space(store)))
}

/// The game over ANY store: every composite, then `store` for the stored cells, all
/// wrapped in the table that names the lines.
///
/// Nothing here holds state. `store` is whatever binds [`STORED`] — the in-memory
/// [`stored_space`], or a peer in another process or language — and the composites reach
/// it by name, through the kernel, so they cannot tell which one answered.
pub fn space_with_store(store: Arc<dyn Space>) -> Alias {
    // One table, shared: the aliases rewrite through it, and the rules read it.
    let table = Arc::new(lines());
    let composites = EndpointSpace::new()
        .bind(template(CELL), platonic_cell())
        .bind(template(CELLS), line_of_cells())
        .bind(template(BOARD), board())
        .bind(template(CHECKSET), checkset(Arc::clone(&table)))
        .bind(template(WINNER), winner(Arc::clone(&table)))
        .bind(template(TURN), turn())
        .bind(template(MOVE), play_move())
        .bind(template(RESET), reset(Arc::clone(&table)))
        .bind(template(TEMPLATE), templates())
        // ANCHOR: views
        .bind(template(VIEW_BOARD), view("board"))
        .bind(template(VIEW_SQUARE), view("square"))
        .bind(template(VIEW_STATUS), view("status"))
        .bind(template(VIEW_REPLY), view("reply"))
        .bind(template(VIEW_GAME), view("game"))
        // ANCHOR_END: views
        .bind(template(VIEW_PLAY), view_play())
        .bind(template(VIEW_RESET), view_reset());
    Alias::new(
        table,
        Arc::new(Fallback::new(vec![Arc::new(composites), store])),
    )
}

/// A store: the stored cell over one [`CellStore`], and nothing else.
pub fn stored_space(store: Arc<CellStore>) -> EndpointSpace {
    EndpointSpace::new().bind(template(STORED), stored_cell(store))
}
// ANCHOR_END: space

// ANCHOR: game
/// A game: the name of the corridor it is played in.
///
/// Not a resource. Nothing is bound at this name and nothing resolves it; it is the name a
/// resolution chain carries, and so the name the cache files a game's answers under.
pub const GAME: &str = "urn:iki:tutorial:ttt:game:{id}";

/// The corridor name of game `id`. An id is letters, digits and `-`, so a game is named by
/// one segment, in one spelling.
pub fn game_name(id: &str) -> Result<Iri> {
    if id.is_empty() || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-') {
        return Err(Error::InvalidArgument {
            name: "game".to_string(),
            detail: format!("`{id}` is not a game id: letters, digits and - only (e.g. a, b, 7)"),
        });
    }
    Iri::parse(format!("urn:iki:tutorial:ttt:game:{id}"))
        .map_err(|e| Error::Endpoint(format!("a game's name: {e}")))
}

/// Game `id`, played over `store`: a resolution chain whose one corridor is the store.
///
/// Resolve anything in it — a cell, the board, the winner, a move — and that request, and
/// every sub-request it gives rise to, looks for `stored:{x}:{y}` in `store` ahead of the
/// root. Every other name falls through to the root, so the game above the atom is the same
/// names and the same code in every game. Only what the corridor binds differs.
///
/// The name is a claim: *any corridor named `game:{id}` holds these doors*. The cache keys on
/// it, so build a game's corridor once and keep it, and never give two stores one id.
/// Injecting a corridor is the host's authority — whoever may push one chooses the game.
pub fn game(id: &str, store: Arc<dyn Space>) -> Result<Scope> {
    let name = game_name(id)?;
    // `Scope::with_named` refuses (panics on) a space that names itself otherwise; a store
    // is anyone's space, so say it as an error instead.
    if let Some(own) = store.id() {
        if own.as_str() != name.as_str() {
            return Err(Error::InvalidArgument {
                name: "game".to_string(),
                detail: format!(
                    "this store names itself `{}`, so it cannot be game `{id}`",
                    own.as_str()
                ),
            });
        }
    }
    Ok(Scope::empty().with_named(name, store))
}
// ANCHOR_END: game

// ANCHOR: contract
/// Check `store` against the **store contract** — what any space must do to be a game's
/// store, whatever language it is written in:
///
/// 1. It binds [`STORED`], `urn:iki:tutorial:ttt:stored:{x}:{y}`, for any integer `x`, `y`
///    in its one plain spelling; `01`, `+1` and `-0` are `InvalidArgument` naming `x` or `y`.
/// 2. `Source` answers the mark as `text/plain`, its bytes exactly the mark (no trailing
///    newline), cacheable. A cell nothing was played at is `NotFound` — not an empty
///    answer, and not `Unresolved`.
/// 3. `Sink` takes the mark as `content`, trimmed, and keeps any non-empty mark; an empty
///    one is `InvalidArgument` naming `content`.
/// 4. `Exists` answers `true` where a mark is played and `false` where none is — never
///    `NotFound` — as `text/plain`, cacheable.
/// 5. `Delete` clears the cell, and succeeds whether or not anything was there.
///
/// It WRITES: it plays and clears `(7, -3)`, a square off the board, and leaves it clear.
/// Point it at a scratch store, not a game in progress.
pub async fn check_store(store: Arc<dyn Space>) -> std::result::Result<(), String> {
    let kernel = Kernel::new(store);
    let root = Capability::root();
    let at = |name: &str| Iri::parse(name).expect("a constant name");
    let here = at(&stored_name(7, -3));
    let issue = |request: Request| kernel.issue(request, &root);
    let with = |verb: Verb, content: &str| {
        Request::new(verb, here.clone())
            .with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()))
    };
    let read = || issue(Request::new(Verb::Source, here.clone()));

    for round in ["first", "second"] {
        issue(Request::new(Verb::Delete, here.clone()))
            .await
            .map_err(|e| format!("Delete must succeed on any cell ({round} delete): {e}"))?;
    }
    match read().await {
        Err(Error::NotFound(_)) => {}
        other => return Err(format!("an unplayed cell must be NotFound, got {other:?}")),
    }
    exists(&issue, &here, "false").await?;
    issue(with(Verb::Sink, " X\n"))
        .await
        .map_err(|e| format!("Sink of `content` must succeed: {e}"))?;
    let played = read()
        .await
        .map_err(|e| format!("a played cell must answer: {e}"))?;
    if played.bytes != b"X" {
        return Err(format!(
            "a played cell answers exactly its mark, trimmed: expected `X`, got {:?}",
            String::from_utf8_lossy(&played.bytes)
        ));
    }
    if !played.repr_type.to_string().starts_with("text/plain") {
        return Err(format!("a mark is text/plain, not {}", played.repr_type));
    }
    if played.expiry == Expiry::Always {
        return Err(
            "a mark must be cacheable, or nothing above the store can be cached".to_string(),
        );
    }
    exists(&issue, &here, "true").await?;
    match issue(with(Verb::Sink, "  ")).await {
        Err(Error::InvalidArgument { name, .. }) if name == "content" => {}
        other => return Err(format!("an empty mark must be refused, got {other:?}")),
    }
    for spelling in [
        "stored:07:-3",
        "stored:+7:-3",
        "stored:7:-03",
        "stored:0:-0",
    ] {
        let name = format!("urn:iki:tutorial:ttt:{spelling}");
        for verb in [Verb::Source, Verb::Exists] {
            match issue(Request::new(verb, at(&name))).await {
                Err(Error::InvalidArgument { name: arg, .. }) if arg == "x" || arg == "y" => {}
                other => {
                    return Err(format!(
                        "{name} must be refused by {verb:?} (one spelling), got {other:?}"
                    ))
                }
            }
        }
    }
    issue(Request::new(Verb::Delete, here.clone()))
        .await
        .map_err(|e| format!("Delete must clear a played cell: {e}"))?;
    match read().await {
        Err(Error::NotFound(_)) => {}
        other => {
            return Err(format!(
                "a cleared cell must be NotFound again, got {other:?}"
            ))
        }
    }
    exists(&issue, &here, "false").await
}

/// Clause 4 of the contract: `Exists` at `here` answers exactly `expected`, cacheably.
async fn exists<F, Fut>(issue: &F, here: &Iri, expected: &str) -> std::result::Result<(), String>
where
    F: Fn(Request) -> Fut,
    Fut: std::future::Future<Output = Result<Representation>>,
{
    match issue(Request::new(Verb::Exists, here.clone())).await {
        Ok(answer) if answer.bytes == expected.as_bytes() && answer.expiry != Expiry::Always => {
            Ok(())
        }
        Ok(answer) if answer.bytes == expected.as_bytes() => Err(format!(
            "Exists must be cacheable like the mark (it answered `{expected}`, uncacheable)"
        )),
        Ok(answer) => Err(format!(
            "Exists must answer `{expected}` here, got {:?}",
            String::from_utf8_lossy(&answer.bytes)
        )),
        Err(e) => Err(format!(
            "Exists must answer true or false, never fail (expected `{expected}`): {e}"
        )),
    }
}
// ANCHOR_END: contract

/// A constant template. Both are literals in this file, and a test parses them.
fn template(source: &str) -> UriTemplate {
    UriTemplate::parse(source).expect("a constant template parses")
}

/// What the views need that the game does not bind: `ikigai-fn`'s `conditional`, at
/// [`CONDITIONAL`], and nothing else from the library. A host that already mounts
/// `ikigai_fn::space()` (the book's page does) has it.
pub fn conditional_space() -> EndpointSpace {
    EndpointSpace::new().bind(Exact::new(CONDITIONAL), ikigai_fn::conditional())
}

/// A host for the game alone: a kernel over [`space`] and [`conditional_space`], with the
/// Meta renderer that lets it describe itself.
pub fn kernel() -> Kernel {
    kernel_over(Arc::default())
}

/// [`kernel`] over [`space_over`].
pub fn kernel_over(store: Arc<CellStore>) -> Kernel {
    let space = Fallback::new(vec![
        Arc::new(space_over(store)) as Arc<dyn Space>,
        Arc::new(conditional_space()),
    ]);
    Kernel::with_meta_renderer(Arc::new(space), Arc::new(TurtleRenderer))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_names_are_the_templates_they_claim_to_be() {
        assert_eq!(template(STORED).source(), STORED);
        assert_eq!(template(CELL).source(), CELL);
        assert_eq!(stored_name(0, -1), "urn:iki:tutorial:ttt:stored:0:-1");
        assert_eq!(cell_name(2, 1), "urn:iki:tutorial:ttt:cell:2:1");
        assert_eq!(template(CELLS).source(), CELLS);
        assert_eq!(template(BOARD).source(), BOARD);
        assert_eq!(
            cells_name(&[(0, 0), (1, 0), (-2, 7)]),
            "urn:iki:tutorial:ttt:cells:0.0,1.0,-2.7"
        );
        assert_eq!(row_name(2), "urn:iki:tutorial:ttt:row:2");
        for (source, name) in [(CHECKSET, checkset_name(1, -1)), (MOVE, move_name(1, -1))] {
            assert_eq!(template(source).source(), source);
            assert!(name.ends_with(":1:-1"), "{name}");
        }
        assert_eq!(template(WINNER).source(), WINNER);
        assert_eq!(template(TURN).source(), TURN);
    }

    #[test]
    fn a_line_is_filled_by_one_mark_three_times_and_never_by_the_empty_cell() {
        assert_eq!(filled_by("XXX"), Some('X'));
        assert_eq!(filled_by("OOO"), Some('O'));
        assert_eq!(filled_by("XX-"), None);
        assert_eq!(filled_by("XOX"), None);
        assert_eq!(filled_by("---"), None);
        assert_eq!(filled_by(""), None);
    }

    /// The CheckSet is read from the table, so it answers for whatever table it is given:
    /// four lines through a 3×3 center, three through a corner, two through an edge — and,
    /// over another table, that table's lines.
    #[test]
    fn the_lines_through_a_cell_come_from_the_table() {
        let table = lines();
        let through = |member: &str| lines_through(&table, member).len();
        assert_eq!(through("1.1"), 4);
        assert_eq!(through("0.0"), 3);
        assert_eq!(through("1.0"), 2);
        assert_eq!(through("3.1"), 0);
        assert_eq!(
            lines_through(&table, "2.2"),
            [
                "urn:iki:tutorial:ttt:column:2",
                "urn:iki:tutorial:ttt:diagonal:0",
                "urn:iki:tutorial:ttt:row:2"
            ]
        );
        let other =
            AliasTable::parse("exact urn:x:long urn:iki:tutorial:ttt:cells:0.0,1.0,2.0,3.0\n")
                .expect("parses");
        assert_eq!(lines_through(&other, "3.0"), ["urn:x:long"]);
        // `1.0` is a member; `11.0` merely contains the text `1.0`.
        let tricky =
            AliasTable::parse("exact urn:x:t urn:iki:tutorial:ttt:cells:11.0\n").expect("parses");
        assert!(lines_through(&tricky, "1.0").is_empty());
    }

    /// The eight rules, checked against the geometry they claim: each alias names the
    /// line [`cells_name`] would spell for those coordinates. A typo in [`LINES`] — a
    /// column that is really a row, a diagonal with a cell off it — fails here.
    #[test]
    fn the_table_names_the_eight_lines_of_a_three_by_three_board() {
        let mut expected: Vec<(String, String)> = Vec::new();
        for i in 0..3 {
            expected.push((
                format!("urn:iki:tutorial:ttt:row:{i}"),
                cells_name(&[(0, i), (1, i), (2, i)]),
            ));
            expected.push((
                format!("urn:iki:tutorial:ttt:column:{i}"),
                cells_name(&[(i, 0), (i, 1), (i, 2)]),
            ));
        }
        expected.push((
            "urn:iki:tutorial:ttt:diagonal:0".into(),
            cells_name(&[(0, 0), (1, 1), (2, 2)]),
        ));
        expected.push((
            "urn:iki:tutorial:ttt:diagonal:1".into(),
            cells_name(&[(2, 0), (1, 1), (0, 2)]),
        ));
        expected.sort();

        let table = lines();
        let mut got: Vec<(String, String)> = table
            .rules()
            .iter()
            .map(|rule| {
                assert_eq!(rule.kind().keyword(), "exact", "{}", rule.from());
                (rule.from().to_string(), rule.to().to_string())
            })
            .collect();
        got.sort();
        assert_eq!(got, expected);
    }

    #[test]
    fn the_stored_cell_declares_one_action_per_verb_each_naming_the_coordinates() {
        use ikigai_core::{Endpoint, InputSource};
        let actions = stored_cell(Arc::default()).describe().action_specs();
        let verbs: Vec<Verb> = actions.iter().map(|a| a.verb).collect();
        assert_eq!(
            verbs,
            vec![Verb::Source, Verb::Sink, Verb::Exists, Verb::Delete]
        );
        for action in &actions {
            let bound: Vec<&str> = action
                .inputs
                .iter()
                .filter(|i| i.source == InputSource::Binding)
                .map(|i| i.name.as_str())
                .collect();
            assert_eq!(bound, ["x", "y"], "{:?}", action.verb);
        }
        let sink = actions.iter().find(|a| a.verb == Verb::Sink).unwrap();
        assert!(sink
            .inputs
            .iter()
            .any(|i| i.name == "content" && i.source == InputSource::Argument));
    }

    #[test]
    fn the_empty_cell_is_visible_text() {
        assert!(!EMPTY.trim().is_empty());
    }
}
