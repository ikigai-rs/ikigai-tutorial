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
//! the kernel keeps both honest by itself. Read the book: `mdbook serve books/ikigai`, or
//! `./scripts/serve-with-drafts.sh` while the chapter is still a draft.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, ArgSpec, AsyncFnEndpoint, Description, EndpointSpace, Error, FnEndpoint,
    Invocation, InvokeFuture, Iri, Kernel, ReprType, Representation, Result, UriTemplate, Verb,
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

// ANCHOR: space
/// The game's space: the two cells, bound by template.
pub fn space() -> EndpointSpace {
    space_over(Arc::default())
}
// ANCHOR_END: space

/// [`space`], with the stored cells in a [`CellStore`] the caller keeps — so a test can
/// read the counter.
pub fn space_over(store: Arc<CellStore>) -> EndpointSpace {
    EndpointSpace::new()
        .bind(template(STORED), stored_cell(store))
        .bind(template(CELL), platonic_cell())
}

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
