//! The code taught by the book's applied chapter, **Tic-tac-toe**.
//!
//! The game is designed as resources before it is code, after *Resource Oriented Analysis
//! and Design*, Parts 1–9, by Peter Rodgers (NetKernel News 3.34–3.42, 2012). This crate
//! is ikigai's own version of that design, built one increment at a time; `README.md`
//! beside this file holds the plan for the whole arc.
//!
//! Increment 1 is the atom — the one piece of state a game of noughts and crosses has:
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
//! Read the book: `mdbook serve books/ikigai`, or `./scripts/serve-with-drafts.sh` while
//! the chapters are still drafts.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, Alias, AliasTable, ArgSpec, AsyncFnEndpoint, Description, EndpointSpace, Error,
    FnEndpoint, Invocation, InvokeFuture, Iri, Kernel, ReprType, Representation, Result,
    UriTemplate, Verb,
};
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
    /// How many times the stored cell's `Source` has actually run.
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

// ANCHOR: space
/// The game's space: the endpoints, wrapped in the table that names the lines.
pub fn space() -> Alias {
    space_over(Arc::default())
}

/// [`space`], with the stored cells in a [`CellStore`] the caller keeps — so a test can
/// read the counter.
pub fn space_over(store: Arc<CellStore>) -> Alias {
    let endpoints = EndpointSpace::new()
        .bind(template(STORED), stored_cell(store))
        .bind(template(CELL), platonic_cell())
        .bind(template(CELLS), line_of_cells())
        .bind(template(BOARD), board());
    Alias::new(Arc::new(lines()), Arc::new(endpoints))
}
// ANCHOR_END: space

/// A constant template. Both are literals in this file, and a test parses them.
fn template(source: &str) -> UriTemplate {
    UriTemplate::parse(source).expect("a constant template parses")
}

/// A host for the game alone: a kernel over [`space`], with the Meta renderer that lets
/// it describe itself.
pub fn kernel() -> Kernel {
    kernel_over(Arc::default())
}

/// [`kernel`] over [`space_over`].
pub fn kernel_over(store: Arc<CellStore>) -> Kernel {
    Kernel::with_meta_renderer(Arc::new(space_over(store)), Arc::new(TurtleRenderer))
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
        assert_eq!(verbs, vec![Verb::Source, Verb::Sink, Verb::Delete]);
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
