//! The code taught by the book's spreadsheet arc.
//!
//! A spreadsheet, designed as resources before it is code. The one thing it stores is what
//! a person typed into each cell. Everything else is a question about that, asked by name:
//! what the formula in a cell compiles to, which cells it reads, what its value is. The
//! kernel caches every answer and cuts it when what it was computed from changes, so a
//! recalculation engine is never written: the golden threads are the dependency graph.
//!
//! The resources, designed as names before code:
//!
//! * `urn:iki:tutorial:sheet:input:{ref}` — what was typed into the cell `ref` (`A1` to
//!   `D6`): a number, some text, or a formula starting with `=`. The **atom**: the only
//!   state, set by `Sink`, cleared by `Delete`. A formula is served as `text/x-formula`,
//!   anything else as `text/plain`.
//! * `urn:iki:tutorial:sheet:compile` — the compiler, a **transreptor** from
//!   `text/x-formula` to `text/x-sexpr`, found by what it declares.
//! * `urn:iki:tutorial:sheet:formula:{ref}` — the cell's formula, compiled: the input run
//!   through whatever transreptor the kernel plans. Code, as a resource.
//! * `urn:iki:tutorial:sheet:refs:{ref}` — the cells the formula reads, one name per line.
//! * `urn:iki:tutorial:sheet:precedents:{ref}` — every cell its value depends on, however
//!   far back. A cell among its own precedents is in a cycle.
//! * `urn:iki:tutorial:sheet:range:{from}:{to}` — the cells in a rectangle, one name per
//!   line, for `SUM`.
//! * `urn:iki:tutorial:sheet:cell:{ref}` — the cell's **value**: a literal passes through, a
//!   formula is evaluated over the values of the cells it reads. Errors are values:
//!   `#REF`, `#DIV/0`, `#CYCLE`, `#VALUE`, `#SYNTAX`, `#N/A`.
//! * `urn:iki:tutorial:sheet:feed:{name}` — the latest value written to a feed: the second
//!   **atom**, written by something that is not the sheet (part V). `FEED(name)` reads it.
//! * `urn:iki:tutorial:sheet:tick:{name}` — `Sink` only: the market moves. Reads the feed,
//!   and writes the next price to it: the outside world, simulated.
//! * `urn:iki:tutorial:sheet:template:{name}` — the sheet's HTML.
//! * `urn:iki:tutorial:sheet:view:cell:{ref}` — the value as HTML, escaped by the `$h{…}`
//!   marker: the `cell` template, composed.
//! * `urn:iki:tutorial:sheet:view:grid` — the `grid` template, composed: every cell's view.
//! * `urn:iki:tutorial:sheet:view:cached:{ref}` — `true` or `false`: would a read of the
//!   cell's value be served from the cache right now? The kernel's probe,
//!   `urn:kernel:cached`, given a name by a one-marker template.
//! * `urn:iki:tutorial:sheet:view:cost:{ref}` and `urn:iki:tutorial:sheet:view:cost` —
//!   what drawing the grid again will cost, cell by cell: `computed` or `cached`.
//! * `urn:iki:tutorial:sheet:view:edit` — `Sink` only: the formula bar's write.
//!
//! Every view but the edit is a template bound at a name with `ikigai_fn::compose_over`:
//! a composition, with no code of its own.
//!
//! And two names the sheet reads but does not serve, from the time chapter's crate:
//! `urn:iki:tutorial:time:instant` for `NOW()` and `urn:iki:tutorial:time:today` for `TODAY()`
//! (part IV). A host that binds the sheet binds those beside it.
//!
//! Read the book: `./scripts/serve-with-drafts.sh` while the chapters are still drafts.

pub mod formula;

use std::collections::{BTreeMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, ArgRef, ArgSpec, AsyncFnEndpoint, Description, EndpointSpace, Error, Fallback,
    FnEndpoint, Invocation, InvokeFuture, Iri, Kernel, ReprType, Representation, Request, Result,
    Space, TransreptionPolicy, UriTemplate, Verb,
};
use ikigai_fn::ComposeOver;
use ikigai_vocab::TurtleRenderer;

pub use formula::{CellRef, Expr};

/// `text/plain; charset=utf-8` as a [`ReprType`].
fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

/// The same media type as a string, for a [`Description`].
const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";

/// `text/html; charset=utf-8` as a [`ReprType`].
fn text_html_utf8() -> ReprType {
    ReprType::new("text/html").with_param("charset", "utf-8")
}

/// The same media type as a string, for a [`Description`].
const TEXT_HTML_UTF8: &str = "text/html;charset=utf-8";

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

// ANCHOR: names
/// What was typed into a cell: the atom.
pub const INPUT: &str = "urn:iki:tutorial:sheet:input:{ref}";

/// The compiler: formula text in, an s-expression out.
pub const COMPILE: &str = "urn:iki:tutorial:sheet:compile";

/// A cell's formula, compiled.
pub const FORMULA: &str = "urn:iki:tutorial:sheet:formula:{ref}";

/// The cells a cell's formula reads.
pub const REFS: &str = "urn:iki:tutorial:sheet:refs:{ref}";

/// Every cell a cell's value depends on, however far back.
pub const PRECEDENTS: &str = "urn:iki:tutorial:sheet:precedents:{ref}";

/// The cells in a rectangle.
pub const RANGE: &str = "urn:iki:tutorial:sheet:range:{from}:{to}";

/// A cell's value.
pub const CELL: &str = "urn:iki:tutorial:sheet:cell:{ref}";

/// A piece of the sheet's HTML.
pub const TEMPLATE: &str = "urn:iki:tutorial:sheet:template:{name}";

/// A cell's value, as HTML.
pub const VIEW_CELL: &str = "urn:iki:tutorial:sheet:view:cell:{ref}";

/// The grid, as HTML.
pub const VIEW_GRID: &str = "urn:iki:tutorial:sheet:view:grid";

/// The formula bar's write: set or clear one cell's input.
pub const VIEW_EDIT: &str = "urn:iki:tutorial:sheet:view:edit";
// ANCHOR_END: names

// ANCHOR: cost_names
/// `true` or `false`: would a read of the cell's value be served from the cache right now?
pub const VIEW_CACHED: &str = "urn:iki:tutorial:sheet:view:cached:{ref}";

/// One cell of the cost of drawing the grid again: `computed` or `cached`.
pub const VIEW_COST_CELL: &str = "urn:iki:tutorial:sheet:view:cost:{ref}";

/// The cost of drawing the grid again, as a table the shape of the grid.
pub const VIEW_COST: &str = "urn:iki:tutorial:sheet:view:cost";
// ANCHOR_END: cost_names

// ANCHOR: feed_names
/// The latest value written to a feed: an atom the sheet reads and never writes.
pub const FEED: &str = "urn:iki:tutorial:sheet:feed:{name}";

/// The market moves: `Sink` only. The next price, written to the feed.
pub const TICK: &str = "urn:iki:tutorial:sheet:tick:{name}";
// ANCHOR_END: feed_names

// ANCHOR: time_names
/// What `NOW()` reads: the time chapter's current minute, the date and the time as one
/// reading.
pub const TIME_INSTANT: &str = "urn:iki:tutorial:time:instant";

/// What `TODAY()` reads: the time chapter's date.
pub const TIME_TODAY: &str = "urn:iki:tutorial:time:today";
// ANCHOR_END: time_names

/// The media type of a formula as typed: `=A1*2`.
pub const FORMULA_TYPE: &str = "text/x-formula";

/// The media type of a compiled formula, an s-expression — the type `ikigai-sexpr` reads.
pub const SEXPR_TYPE: &str = "text/x-sexpr";

// ANCHOR: errors
/// A reference to a cell that is not on the sheet.
pub const REF_ERROR: &str = "#REF";
/// A division by zero.
pub const DIV_ERROR: &str = "#DIV/0";
/// A cell whose value depends on itself.
pub const CYCLE_ERROR: &str = "#CYCLE";
/// Text where a number was needed.
pub const VALUE_ERROR: &str = "#VALUE";
/// A formula that does not compile.
pub const SYNTAX_ERROR: &str = "#SYNTAX";
/// A value that is not available: a feed nothing has written to, or the time on a kernel
/// with no clock.
pub const NA_ERROR: &str = "#N/A";
// ANCHOR_END: errors

const ERRORS: [&str; 6] = [
    REF_ERROR,
    DIV_ERROR,
    CYCLE_ERROR,
    VALUE_ERROR,
    SYNTAX_ERROR,
    NA_ERROR,
];

/// The most bytes one cell's input may hold.
const MAX_INPUT: usize = 1000;

/// The name of the input of `cell`.
pub fn input_name(cell: CellRef) -> String {
    format!("urn:iki:tutorial:sheet:input:{cell}")
}

/// The name of the compiled formula of `cell`.
pub fn formula_name(cell: CellRef) -> String {
    format!("urn:iki:tutorial:sheet:formula:{cell}")
}

/// The name of the references of `cell`.
pub fn refs_name(cell: CellRef) -> String {
    format!("urn:iki:tutorial:sheet:refs:{cell}")
}

/// The name of the precedents of `cell`.
pub fn precedents_name(cell: CellRef) -> String {
    format!("urn:iki:tutorial:sheet:precedents:{cell}")
}

/// The name of the value of `cell`.
pub fn cell_name(cell: CellRef) -> String {
    format!("urn:iki:tutorial:sheet:cell:{cell}")
}

/// The name of the view of `cell`: its value, as HTML.
pub fn view_cell_name(cell: CellRef) -> String {
    format!("urn:iki:tutorial:sheet:view:cell:{cell}")
}

/// The name of the range from `top_left` to `bottom_right`.
pub fn range_name(top_left: CellRef, bottom_right: CellRef) -> String {
    format!("urn:iki:tutorial:sheet:range:{top_left}:{bottom_right}")
}

/// The name of the feed `name`.
pub fn feed_name(name: &str) -> String {
    format!("urn:iki:tutorial:sheet:feed:{name}")
}

/// The name of the market's move on the feed `name`.
pub fn tick_name(name: &str) -> String {
    format!("urn:iki:tutorial:sheet:tick:{name}")
}

/// The name of the template `name`.
pub fn template_name(name: &str) -> String {
    format!("urn:iki:tutorial:sheet:template:{name}")
}

/// The cell a value's name names: `urn:iki:tutorial:sheet:cell:A1` is `A1`.
pub fn cell_of(name: &str) -> Option<CellRef> {
    name.strip_prefix("urn:iki:tutorial:sheet:cell:")
        .and_then(CellRef::parse)
}

fn iri(name: &str) -> Result<Iri> {
    Iri::parse(name).map_err(|e| Error::Endpoint(format!("the sheet's name {name}: {e}")))
}

/// The text of the resource `name`, read through the kernel.
async fn text_of(inv: &Invocation<'_>, name: &str) -> Result<String> {
    Ok(String::from_utf8_lossy(&inv.source(&iri(name)?).await?.bytes).into_owned())
}

// ANCHOR: cell_binding
/// The cell a name's `{ref}` (or other binding) names, on the sheet, in its one spelling.
fn cell_binding(inv: &Invocation<'_>, binding: &str) -> Result<CellRef> {
    let text = inv
        .bindings
        .get(binding)
        .ok_or_else(|| Error::MissingArgument(binding.to_string()))?;
    on_sheet(binding, text)
}

/// `text` as a cell of the sheet, or a refusal naming `argument` and saying what to fix.
fn on_sheet(argument: &str, text: &str) -> Result<CellRef> {
    let refuse = |detail: String| Error::InvalidArgument {
        name: argument.to_string(),
        detail,
    };
    let cell = CellRef::parse(text).ok_or_else(|| {
        refuse(format!(
            "`{text}` is not a cell: a capital letter for the column, then the row (e.g. B3)"
        ))
    })?;
    if !cell.on_sheet() {
        return Err(refuse(format!(
            "{cell} is off the sheet, which is columns {}–{} and rows 1–{}",
            &formula::COLUMNS[..1],
            &formula::COLUMNS[formula::COLUMNS.len() - 1..],
            formula::ROWS
        )));
    }
    Ok(cell)
}
// ANCHOR_END: cell_binding

fn ref_input(summary: &str) -> ArgSpec {
    ArgSpec::new("ref")
        .summary(summary)
        .class(XSD_STRING)
        .binding()
}

// ANCHOR: store
/// What has been typed into each cell. The sheet's one piece of state.
#[derive(Debug, Default)]
pub struct InputStore {
    inputs: Mutex<BTreeMap<CellRef, String>>,
}
// ANCHOR_END: store

// ANCHOR: input
/// `sheet-input`: what was typed into the cell `ref`, held in memory.
///
/// `Source` answers the text as typed, `.cacheable()` with no thread named: the kernel
/// hangs it from its own name, and cuts that after every `Sink` or `Delete` here. The media
/// type says what the text is. A formula, which starts with `=`, is `text/x-formula`;
/// anything else is `text/plain`. A cell nobody has typed into is `NotFound`.
///
/// `Sink` stores `content`, trimmed, and answers what it stored; an empty input is refused,
/// since "nothing" is what `Delete` says.
pub fn input(store: Arc<InputStore>) -> FnEndpoint {
    FnEndpoint::new("sheet-input", move |inv: &Invocation<'_>| {
        let at = cell_binding(inv, "ref")?;
        let mut inputs = store.inputs.lock().expect("the input store's lock");
        match inv.request.verb {
            Verb::Sink => {
                let typed = inv.inline_str("content")?.trim();
                if typed.is_empty() {
                    return Err(Error::InvalidArgument {
                        name: "content".to_string(),
                        detail: "an empty input — to clear a cell, delete it".to_string(),
                    });
                }
                if typed.len() > MAX_INPUT {
                    return Err(Error::InvalidArgument {
                        name: "content".to_string(),
                        detail: format!("{} bytes; a cell holds at most {MAX_INPUT}", typed.len()),
                    });
                }
                inputs.insert(at, typed.to_string());
                Ok(Representation::new(
                    text_plain_utf8(),
                    typed.as_bytes().to_vec(),
                ))
            }
            Verb::Delete => {
                inputs.remove(&at);
                Ok(Representation::new(text_plain_utf8(), b"ok".to_vec()))
            }
            _ => match inputs.get(&at) {
                Some(typed) => {
                    let media = if typed.starts_with('=') {
                        ReprType::new(FORMULA_TYPE)
                    } else {
                        text_plain_utf8()
                    };
                    Ok(Representation::new(media, typed.clone().into_bytes()).cacheable())
                }
                None => Err(Error::NotFound(format!("nothing has been typed into {at}"))),
            },
        }
    })
    .with_description(
        Description::new("sheet-input")
            .title("Input")
            .summary("What was typed into a cell: read it, type into it, or clear it.")
            .verb(Verb::Meta)
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("the input as typed; text/x-formula for a formula, NotFound if empty")
                    .input(ref_input("the cell, A1 to D6"))
                    .output(TEXT_PLAIN_UTF8)
                    .output(FORMULA_TYPE),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("type into the cell, and answer what was stored")
                    .input(ref_input("the cell, A1 to D6"))
                    .input(
                        ArgSpec::new("content")
                            .summary("a number, some text, or a formula starting with =")
                            .class(XSD_STRING),
                    )
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Delete)
                    .summary("clear the cell")
                    .input(ref_input("the cell, A1 to D6"))
                    .output(TEXT_PLAIN_UTF8),
            ),
    )
}
// ANCHOR_END: input

// ANCHOR: compile
/// `sheet-compile`: a transreptor from formula text to its compiled form, an s-expression.
///
/// A pure function of `content`, so `.cacheable()`: the kernel keeps one entry per formula
/// text, and a formula typed into two cells is compiled once. It never refuses a formula:
/// one that does not parse compiles to `(syntax-error "…")`, and its value is `#SYNTAX`.
///
/// The description is what makes it the compiler. `.transreptor(from, to)` is what the
/// kernel selects on, so `formula:{ref}` finds it by what it converts, never by its name.
/// And `.lossy()` is true: compiling forgets how the formula was spelled (`=a1 +2` and
/// `=A1+2` compile alike), so the formula cannot be recovered from what comes out.
pub fn compile() -> FnEndpoint {
    FnEndpoint::new("sheet-compile", |inv: &Invocation<'_>| {
        let compiled = formula::compile(inv.inline_str("content")?);
        Ok(
            Representation::new(ReprType::new(SEXPR_TYPE), compiled.to_string().into_bytes())
                .cacheable(),
        )
    })
    .with_description(
        Description::new("sheet-compile")
            .title("Compile a formula")
            .summary("Compiles a formula (=A1*2) to an s-expression ((* A1 2)).")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("content")
                    .summary("the formula, starting with =")
                    .class(XSD_STRING),
            )
            .input(
                ArgSpec::new("as")
                    .summary("the target type; only text/x-sexpr")
                    .class(XSD_STRING)
                    .optional(),
            )
            .output(SEXPR_TYPE)
            .transreptor([FORMULA_TYPE], [SEXPR_TYPE])
            .lossy(),
    )
}
// ANCHOR_END: compile

// ANCHOR: formula
/// `sheet-formula`: the cell's formula, compiled.
///
/// It reads the input, and asks the kernel for a plan from what the input is
/// (`text/x-formula`) to what this answers (`text/x-sexpr`), then runs the plan through the
/// kernel. The compiler is whatever transreptor the plan names; this code does not know it.
/// The consent to a lossy step is this endpoint's to give, because it is the one that knows
/// losing the spelling is what compiling means.
///
/// Cacheable, with no thread named: it read the input, so the kernel hangs it from the
/// input's thread, and typing into that cell, and only that cell, recompiles it. A cell
/// holding no formula is `NotFound`.
pub fn formula() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "sheet-formula",
        |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            Box::pin(async move {
                let at = cell_binding(inv, "ref")?;
                let typed = inv.source(&iri(&input_name(at))?).await?;
                if typed.repr_type.media_type != FORMULA_TYPE {
                    return Err(Error::NotFound(format!("{at} holds no formula")));
                }
                let plan = inv
                    .select_transreptor_with(
                        FORMULA_TYPE,
                        SEXPR_TYPE,
                        &TransreptionPolicy::allow_lossy(),
                    )
                    .ok_or_else(|| {
                        Error::Endpoint(format!("nothing converts {FORMULA_TYPE} to {SEXPR_TYPE}"))
                    })?;
                let mut compiled = typed;
                for step in plan {
                    let request = Request::new(Verb::Source, iri(&step.endpoint)?)
                        .with_arg("content", ArgRef::Inline(compiled.bytes))
                        .with_arg("as", ArgRef::Inline(step.to.into_bytes()));
                    compiled = inv.issue(request).await?;
                }
                Ok(Representation::new(compiled.repr_type, compiled.bytes).cacheable())
            })
        },
    )
    .with_description(
        Description::new("sheet-formula")
            .title("Compiled formula")
            .summary("The cell's formula, compiled to an s-expression; NotFound if it holds none.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(ref_input("the cell, A1 to D6"))
            .output(SEXPR_TYPE),
    )
}
// ANCHOR_END: formula

/// The cell's input, or `None` for a cell nobody has typed into. Read through the kernel,
/// so whatever asks hangs from the input's thread either way: an answer built on an empty
/// cell is cut by the first thing typed into it.
async fn typed_into(inv: &Invocation<'_>, at: CellRef) -> Result<Option<Representation>> {
    match inv.source(&iri(&input_name(at))?).await {
        Ok(typed) => Ok(Some(typed)),
        Err(Error::NotFound(_)) => Ok(None),
        Err(other) => Err(other),
    }
}

/// The cell's formula, compiled and read back, or `None` if `typed` (the cell's input, as
/// [`typed_into`] read it) is not a formula.
///
/// The input is read FIRST, and the formula only when the input is one: a fallback on the
/// atom's own `NotFound` says exactly what is missing. (Before core 0.1.82 it was also the only
/// safe order: a `NotFound` from `formula:{ref}` hung whatever caught it from a thread named
/// `formula:{ref}`, which no `Sink` cuts. Since then a failure carries the threads it read on
/// its way to failing, ledger item 611.)
async fn compiled(
    inv: &Invocation<'_>,
    at: CellRef,
    typed: Option<&Representation>,
) -> Result<Option<Expr>> {
    if !typed.is_some_and(|typed| typed.repr_type.media_type == FORMULA_TYPE) {
        return Ok(None);
    }
    let text = text_of(inv, &formula_name(at)).await?;
    let expr = formula::read(&text).map_err(|why| {
        Error::Endpoint(format!(
            "{}: `{text}` does not read back: {why}",
            formula_name(at)
        ))
    })?;
    Ok(Some(expr))
}

/// A list of names as a representation: one per line.
fn name_list(names: &[String]) -> Representation {
    Representation::new(text_plain_utf8(), names.join("\n").into_bytes()).cacheable()
}

/// The lines of a list of names, none of them empty.
fn lines_of(text: &str) -> impl Iterator<Item = &str> {
    text.lines().filter(|line| !line.is_empty())
}

// ANCHOR: refs
/// `sheet-refs`: the cells the cell's formula reads, one name per line, each once.
///
/// Read from the compiled form, not the text: the cells it names and the cells of each
/// range, which `range:{from}:{to}` lists. A cell off the sheet is not listed (nothing is
/// read for it; its value is `#REF`). A cell holding no formula reads nothing: the empty list.
pub fn refs() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("sheet-refs", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let at = cell_binding(inv, "ref")?;
            let mut names: Vec<String> = Vec::new();
            let typed = typed_into(inv, at).await?;
            if let Some(expr) = compiled(inv, at, typed.as_ref()).await? {
                let (cells, ranges) = expr.names();
                for (from, to) in ranges
                    .into_iter()
                    .filter(|(a, b)| a.on_sheet() && b.on_sheet())
                {
                    for name in lines_of(&text_of(inv, &range_name(from, to)).await?) {
                        names.push(name.to_string());
                    }
                }
                names.extend(cells.into_iter().filter(CellRef::on_sheet).map(cell_name));
            }
            let mut once = Vec::new();
            for name in names {
                if !once.contains(&name) {
                    once.push(name);
                }
            }
            Ok(name_list(&once))
        })
    })
    .with_description(
        Description::new("sheet-refs")
            .title("References")
            .summary("The cells a cell's formula reads, one name per line.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(ref_input("the cell, A1 to D6"))
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: refs

// ANCHOR: precedents
/// `sheet-precedents`: every cell the cell's value depends on, however far back, one name
/// per line, nearest first.
///
/// A walk over `refs:{ref}`: the cells this one reads, then the cells those read, and so on,
/// each visited once. It reads formulas and never values, so it ends even when the formulas
/// go round in a circle, and a cell that turns up among its own precedents is in one. That
/// is how a cell's value can say `#CYCLE` instead of chasing itself down to the kernel's
/// depth bound.
pub fn precedents() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "sheet-precedents",
        |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            Box::pin(async move {
                let at = cell_binding(inv, "ref")?;
                let mut found: Vec<String> = Vec::new();
                let mut queue = VecDeque::from([at]);
                while let Some(next) = queue.pop_front() {
                    for name in lines_of(&text_of(inv, &refs_name(next)).await?) {
                        if found.iter().any(|seen| seen == name) {
                            continue;
                        }
                        found.push(name.to_string());
                        if let Some(cell) = cell_of(name).filter(|cell| *cell != at) {
                            queue.push_back(cell);
                        }
                    }
                }
                Ok(name_list(&found))
            })
        },
    )
    .with_description(
        Description::new("sheet-precedents")
            .title("Precedents")
            .summary("Every cell a cell's value depends on, however far back, one name per line.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(ref_input("the cell, A1 to D6"))
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: precedents

// ANCHOR: range
/// `sheet-range`: the cells from `from` (top-left) to `to` (bottom-right), row by row, one
/// name per line. A pure function of its name, so it is computed once.
///
/// One spelling per rectangle: `A1:B2`, never `B2:A1` or `A2:B1`, so a range has one name
/// and one cache entry. The compiler writes that spelling whatever order a formula used.
pub fn range() -> FnEndpoint {
    FnEndpoint::new("sheet-range", |inv: &Invocation<'_>| {
        let (from, to) = (cell_binding(inv, "from")?, cell_binding(inv, "to")?);
        if from.column > to.column || from.row > to.row {
            let (top_left, bottom_right) = (
                CellRef::new(from.column.min(to.column), from.row.min(to.row)),
                CellRef::new(from.column.max(to.column), from.row.max(to.row)),
            );
            return Err(Error::InvalidArgument {
                name: "from, to".to_string(),
                detail: format!("{from}:{to} is spelled top-left first: {top_left}:{bottom_right}"),
            });
        }
        let names: Vec<String> = (from.row..=to.row)
            .flat_map(|row| (from.column..=to.column).map(move |c| cell_name(CellRef::new(c, row))))
            .collect();
        Ok(name_list(&names))
    })
    .with_description(
        Description::new("sheet-range")
            .title("Range")
            .summary("The cells in a rectangle, row by row, one name per line.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("from")
                    .summary("the top-left cell")
                    .class(XSD_STRING)
                    .binding(),
            )
            .input(
                ArgSpec::new("to")
                    .summary("the bottom-right cell")
                    .class(XSD_STRING)
                    .binding(),
            )
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: range

/// A value, as a formula sees one.
#[derive(Clone, Debug, PartialEq)]
enum Value {
    Number(f64),
    Empty,
    Text(String),
    Error(&'static str),
}

impl Value {
    /// A cell's value, read from its text: a number, nothing, one of the errors, or text.
    fn of(text: &str) -> Value {
        if text.is_empty() {
            return Value::Empty;
        }
        if let Some(error) = ERRORS.iter().find(|e| **e == text) {
            return Value::Error(error);
        }
        match formula::parse_number(text) {
            Some(n) => Value::Number(n),
            None => Value::Text(text.to_string()),
        }
    }

    /// As a number for arithmetic: an empty cell is 0, text is `#VALUE`, an error is itself.
    fn number(self) -> std::result::Result<f64, &'static str> {
        match self {
            Value::Number(n) => Ok(n),
            Value::Empty => Ok(0.0),
            Value::Text(_) => Err(VALUE_ERROR),
            Value::Error(e) => Err(e),
        }
    }

    /// The value as a cell shows it. A formula that names an empty cell is 0.
    fn text(self) -> String {
        match self {
            Value::Number(n) => formula::number_text(n),
            Value::Empty => "0".to_string(),
            Value::Text(text) => text,
            Value::Error(e) => e.to_string(),
        }
    }
}

type Evaluated<'a> = Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>>;

// ANCHOR: eval
/// Evaluate `expr`, reading every cell it names through the kernel.
///
/// Each read is a sub-request, so the value being computed depends on exactly the cells
/// this evaluation read, and on nothing else: the kernel records them as it goes. Nothing
/// here writes down a dependency. A sheet error is a [`Value::Error`], and the first one met
/// (left to right) is the answer; a failure of the kernel's own is an `Err` and passes on.
fn eval<'a>(inv: &'a Invocation<'_>, expr: &'a Expr) -> Evaluated<'a> {
    Box::pin(async move {
        let arithmetic = |result: std::result::Result<f64, &'static str>| match result {
            Ok(n) if n.is_finite() => Value::Number(n),
            Ok(_) => Value::Error(VALUE_ERROR),
            Err(e) => Value::Error(e),
        };
        Ok(match expr {
            Expr::Number(n) => Value::Number(*n),
            Expr::Cell(cell) => read_cell(inv, *cell).await?,
            Expr::Negate(inner) => arithmetic(eval(inv, inner).await?.number().map(|n| -n)),
            Expr::Binary(op, left, right) => {
                let left = match eval(inv, left).await?.number() {
                    Ok(n) => n,
                    Err(e) => return Ok(Value::Error(e)),
                };
                let right = match eval(inv, right).await?.number() {
                    Ok(n) => n,
                    Err(e) => return Ok(Value::Error(e)),
                };
                arithmetic(match op {
                    formula::Op::Add => Ok(left + right),
                    formula::Op::Subtract => Ok(left - right),
                    formula::Op::Multiply => Ok(left * right),
                    formula::Op::Divide if right == 0.0 => Err(DIV_ERROR),
                    formula::Op::Divide => Ok(left / right),
                })
            }
            Expr::Sum(args) => {
                let mut total = 0.0;
                for arg in args {
                    match sum_of(inv, arg).await? {
                        Ok(n) => total += n,
                        Err(e) => return Ok(Value::Error(e)),
                    }
                }
                arithmetic(Ok(total))
            }
            Expr::Now => read_time(inv, TIME_INSTANT).await?,
            Expr::Today => read_time(inv, TIME_TODAY).await?,
            Expr::Feed(name) => read_feed(inv, name).await?,
            // A range means something only as an argument of SUM.
            Expr::Range(..) => Value::Error(VALUE_ERROR),
            Expr::SyntaxError(_) => Value::Error(SYNTAX_ERROR),
        })
    })
}

/// The value of `cell`, read through the kernel; `#REF` for a cell off the sheet, which is
/// never asked for, since no name answers for it.
async fn read_cell(inv: &Invocation<'_>, cell: CellRef) -> Result<Value> {
    if !cell.on_sheet() {
        return Ok(Value::Error(REF_ERROR));
    }
    Ok(Value::of(&text_of(inv, &cell_name(cell)).await?))
}

// ANCHOR: read_time
/// `NOW()` or `TODAY()`: the text of the time chapter's `instant` or `today`, read through
/// the kernel, so the value that asked inherits its deadline. A kernel with no clock has no
/// time to give, and never will (its clock is fixed when it is built), so that is `#N/A`,
/// and nothing is asked.
async fn read_time(inv: &Invocation<'_>, name: &str) -> Result<Value> {
    if inv.now().is_none() {
        return Ok(Value::Error(NA_ERROR));
    }
    Ok(Value::Text(text_of(inv, name).await?))
}
// ANCHOR_END: read_time

// ANCHOR: read_feed
/// `FEED(name)`: the latest value written to the feed, read through the kernel.
///
/// A feed nothing has written to is `#N/A`. That fallback is safe to cache: the value that
/// caught the `NotFound` hangs from the feed's golden thread, so the first write to the feed
/// cuts it. It catches the ATOM's own `NotFound`, which says exactly what is missing; before
/// core 0.1.82 that was also the only safe choice, since a composite's `NotFound` hung the
/// fallback from a thread nobody cuts.
async fn read_feed(inv: &Invocation<'_>, name: &str) -> Result<Value> {
    match inv.source(&iri(&feed_name(name))?).await {
        Ok(latest) => Ok(Value::of(&String::from_utf8_lossy(&latest.bytes))),
        Err(Error::NotFound(_)) => Ok(Value::Error(NA_ERROR)),
        Err(other) => Err(other),
    }
}
// ANCHOR_END: read_feed

/// What one argument of SUM adds. A range, or a cell named on its own, adds its numbers
/// and skips its empty cells and its text, as spreadsheets do; anything else is arithmetic.
async fn sum_of(
    inv: &Invocation<'_>,
    arg: &Expr,
) -> Result<std::result::Result<f64, &'static str>> {
    let cells = match arg {
        Expr::Range(from, to) if from.on_sheet() && to.on_sheet() => {
            let listed = text_of(inv, &range_name(*from, *to)).await?;
            lines_of(&listed).filter_map(cell_of).collect()
        }
        Expr::Range(..) => return Ok(Err(REF_ERROR)),
        Expr::Cell(cell) => vec![*cell],
        other => return Ok(eval(inv, other).await?.number()),
    };
    let mut total = 0.0;
    for cell in cells {
        match read_cell(inv, cell).await? {
            Value::Number(n) => total += n,
            Value::Error(e) => return Ok(Err(e)),
            Value::Empty | Value::Text(_) => {}
        }
    }
    Ok(Ok(total))
}
// ANCHOR_END: eval

// ANCHOR: cell
/// `sheet-cell`: the cell's value.
///
/// A cell nobody has typed into is empty. A literal (a number, some text) passes through as
/// typed. A formula is evaluated: first the cell's precedents are read, and a cell among its
/// own is `#CYCLE`; otherwise the compiled form is read back and evaluated over the values
/// of the cells it names.
///
/// Every answer is `.cacheable()` and names no thread. It hangs from what it read: the
/// input, the compiled formula, the precedents, and the values of the cells the formula
/// named. So typing into a cell cuts that cell's value, and the values that read it, and the
/// values that read those, and nothing else.
pub fn cell() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("sheet-cell", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let at = cell_binding(inv, "ref")?;
            let typed = typed_into(inv, at).await?;
            let value = match &typed {
                None => String::new(),
                Some(literal) if literal.repr_type.media_type != FORMULA_TYPE => {
                    String::from_utf8_lossy(&literal.bytes).into_owned()
                }
                Some(_) => {
                    let precedents = text_of(inv, &precedents_name(at)).await?;
                    if lines_of(&precedents).any(|name| cell_of(name) == Some(at)) {
                        CYCLE_ERROR.to_string()
                    } else {
                        let expr = compiled(inv, at, typed.as_ref()).await?.ok_or_else(|| {
                            Error::Endpoint(format!("{at} held a formula and then did not"))
                        })?;
                        eval(inv, &expr).await?.text()
                    }
                }
            };
            Ok(Representation::new(text_plain_utf8(), value.into_bytes()).cacheable())
        })
    })
    .with_description(
        Description::new("sheet-cell")
            .title("Cell")
            .summary(
                "A cell's value: a literal as typed, or its formula evaluated. Errors are \
                 values: #REF, #DIV/0, #CYCLE, #VALUE, #SYNTAX, #N/A.",
            )
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(ref_input("the cell, A1 to D6"))
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: cell

/// `text`, safe to put in HTML text or a quoted attribute: what the formula bar's reply is
/// written with. (The grid needs nothing of the kind: its template splices each value with
/// compose's `$h{…}` marker, which escapes it and never expands it.)
///
/// ```
/// assert_eq!(
///     spreadsheet::escape("<b>\"it's\" & more</b>"),
///     "&lt;b&gt;&quot;it&#39;s&quot; &amp; more&lt;/b&gt;"
/// );
/// ```
pub fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

/// The feed a name's `{name}` names, in its one spelling.
fn feed_binding(inv: &Invocation<'_>) -> Result<String> {
    let name = inv
        .bindings
        .get("name")
        .ok_or_else(|| Error::MissingArgument("name".to_string()))?;
    if !formula::feed_name(name) {
        return Err(Error::InvalidArgument {
            name: "name".to_string(),
            detail: format!(
                "`{name}` is not a feed's name: a lower-case letter, then lower-case letters and digits (e.g. acme)"
            ),
        });
    }
    Ok(name.to_string())
}

fn feed_input() -> ArgSpec {
    ArgSpec::new("name")
        .summary("the feed's name: lower-case letters and digits, e.g. acme")
        .class(XSD_STRING)
        .binding()
}

// ANCHOR: feed_store
/// The latest value written to each feed. The sheet's second piece of state, and the first
/// the sheet itself never writes.
#[derive(Debug, Default)]
pub struct FeedStore {
    latest: Mutex<BTreeMap<String, String>>,
}
// ANCHOR_END: feed_store

// ANCHOR: feed
/// `sheet-feed`: the latest value written to the feed `name`, held in memory.
///
/// The same shape as the input: `Source` answers what was written, `.cacheable()` with no
/// thread named, so the kernel hangs it from the feed's own name and cuts that on every
/// `Sink` or `Delete` here. A feed nothing has written to is `NotFound`. `Sink` stores
/// `content`, trimmed, and answers it.
///
/// Nothing in the sheet writes here. A price service, a sensor, a person at a shell: whatever
/// writes a feed is outside the sheet, and the sheet finds out the way it finds out about
/// anything, by reading a name whose thread was cut.
pub fn feed(store: Arc<FeedStore>) -> FnEndpoint {
    FnEndpoint::new("sheet-feed", move |inv: &Invocation<'_>| {
        let name = feed_binding(inv)?;
        let mut latest = store.latest.lock().expect("the feed store's lock");
        match inv.request.verb {
            Verb::Sink => {
                let written = inv.inline_str("content")?.trim();
                if written.is_empty() || written.len() > MAX_INPUT {
                    return Err(Error::InvalidArgument {
                        name: "content".to_string(),
                        detail: format!(
                            "{} bytes; a feed holds 1 to {MAX_INPUT} (to clear it, delete it)",
                            written.len()
                        ),
                    });
                }
                latest.insert(name, written.to_string());
                Ok(Representation::new(
                    text_plain_utf8(),
                    written.as_bytes().to_vec(),
                ))
            }
            Verb::Delete => {
                latest.remove(&name);
                Ok(Representation::new(text_plain_utf8(), b"ok".to_vec()))
            }
            _ => match latest.get(&name) {
                Some(written) => Ok(Representation::new(
                    text_plain_utf8(),
                    written.clone().into_bytes(),
                )
                .cacheable()),
                None => Err(Error::NotFound(format!(
                    "nothing has been written to the feed `{name}`"
                ))),
            },
        }
    })
    .with_description(
        Description::new("sheet-feed")
            .title("Feed")
            .summary("The latest value written to a feed: read it, write it, or clear it.")
            .verb(Verb::Meta)
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("the latest value written; NotFound if nothing has been")
                    .input(feed_input())
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("write the feed's latest value, and answer it")
                    .input(feed_input())
                    .input(
                        ArgSpec::new("content")
                            .summary("the value: a number, or some text")
                            .class(XSD_STRING),
                    )
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Delete)
                    .summary("clear the feed")
                    .input(feed_input())
                    .output(TEXT_PLAIN_UTF8),
            ),
    )
}
// ANCHOR_END: feed

/// Where a feed nothing has written to starts, in cents: 100.
const OPENING_CENTS: u64 = 10_000;

// ANCHOR: next_price
/// The market, simulated: the price after `cents`, in cents, somewhere within 1.50 of it and
/// never below one cent.
///
/// A pure function of the price, so the chapter's outputs are the same on every run: a
/// generator seeded by the price, not by the time. The sheet never sees this; it sees only
/// what gets written to the feed.
///
/// ```
/// use spreadsheet::next_price;
/// assert_eq!(next_price(10_000), next_price(10_000));
/// assert!(next_price(10_000).abs_diff(10_000) <= 150);
/// assert!(next_price(1) >= 1);
/// ```
pub fn next_price(cents: u64) -> u64 {
    let mixed = cents
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    let step = (mixed >> 33) % 301;
    (cents.saturating_add(step)).saturating_sub(150).max(1)
}
// ANCHOR_END: next_price

// ANCHOR: tick
/// `sheet-tick`: the market moves. `Sink` only.
///
/// Reads the feed `name` through the kernel (a feed nothing has written to opens at 100),
/// writes [`next_price`] to it through the kernel, and answers the new price. The write is an
/// ordinary `Sink` on `feed:{name}`, so it cuts the feed's thread exactly as a price service
/// writing the same name would; this endpoint is here only because the page has no price
/// service. A feed holding something that is not a price is a `Conflict`: the market does not
/// move a feed somebody has set to `halted`.
pub fn tick() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("sheet-tick", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            if inv.request.verb != Verb::Sink {
                return Err(Error::Endpoint(
                    "the market moves when it is sunk: `sink` it (and read the feed)".to_string(),
                ));
            }
            let name = feed_binding(inv)?;
            let target = iri(&feed_name(&name))?;
            let cents = match inv.source(&target).await {
                Ok(latest) => {
                    let text = String::from_utf8_lossy(&latest.bytes).into_owned();
                    formula::parse_number(&text)
                        .filter(|n| *n > 0.0 && *n < 1e12)
                        .map(|n| (n * 100.0).round() as u64)
                        .ok_or_else(|| {
                            Error::Conflict(format!(
                                "the feed `{name}` holds `{text}`, which is not a price"
                            ))
                        })?
                }
                Err(Error::NotFound(_)) => OPENING_CENTS,
                Err(other) => return Err(other),
            };
            let price = formula::number_text(next_price(cents) as f64 / 100.0);
            let write = Request::new(Verb::Sink, target)
                .with_arg("content", ArgRef::Inline(price.clone().into_bytes()));
            inv.issue(write).await?;
            Ok(Representation::new(text_plain_utf8(), price.into_bytes()))
        })
    })
    .with_description(
        Description::new("sheet-tick")
            .title("The market moves")
            .summary("Writes the feed's next price to it, and answers the price. Sink only.")
            .verb(Verb::Sink)
            .verb(Verb::Meta)
            .input(feed_input())
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: tick

/// Every template, by the name it is resolved at: `template:{name}`. A file's final
/// newline is not part of the template.
pub const TEMPLATES: &[(&str, &str)] = &[
    ("grid", include_str!("../templates/grid.html")),
    ("cell", include_str!("../templates/cell.html")),
    ("cached", include_str!("../templates/cached.html")),
    ("cost", include_str!("../templates/cost.html")),
    ("cost-cell", include_str!("../templates/cost-cell.html")),
    ("served", include_str!("../templates/served.html")),
    ("computed", include_str!("../templates/computed.html")),
    ("page", include_str!("../templates/page.html")),
    ("live", include_str!("../templates/live.html")),
    ("market", include_str!("../templates/market.html")),
];

/// `sheet-template`: the template `name`, as `text/html`, computed once.
pub fn templates() -> FnEndpoint {
    FnEndpoint::new("sheet-template", |inv: &Invocation<'_>| {
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
        Description::new("sheet-template")
            .title("Template")
            .summary("A piece of the sheet's HTML.")
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
/// A view: the template `name`, filled by `urn:iki:fn:compose`'s rules and bound at a name
/// of its own. The variables the name captures (`{ref}`) are the template's arguments.
/// There is no code here that reads a cell: the template names what it shows.
pub fn view(name: &str) -> ComposeOver {
    ikigai_fn::compose_over(iri(&template_name(name)).expect("a template's name is a name"))
}
// ANCHOR_END: view

// ANCHOR: view_edit
/// `sheet-view-edit`: the formula bar's write. `Sink` only.
///
/// `ref` is the cell as a person typed it (`b3` will do: this is the human edge, so it is
/// forgiving here and nowhere else); `content` is what to type into it. Empty content
/// clears the cell. The write is a `Sink` (or a `Delete`) to the cell's input, through the
/// kernel, and the answer is one line of HTML text saying what happened, a refusal included,
/// in the kernel's own words.
pub fn view_edit() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "sheet-view-edit",
        |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            Box::pin(async move {
                if inv.request.verb != Verb::Sink {
                    return Err(Error::Endpoint(
                        "an edit is made, not read: `sink` it (read the grid view)".to_string(),
                    ));
                }
                let typed_ref = inv.inline_str("ref")?.trim().to_ascii_uppercase();
                let content = inv
                    .inline_str("content")
                    .unwrap_or_default()
                    .trim()
                    .to_string();
                let said = match on_sheet("ref", &typed_ref) {
                    Err(refused) => refused.to_string(),
                    Ok(at) => {
                        let target = iri(&input_name(at))?;
                        let outcome = if content.is_empty() {
                            inv.issue(Request::new(Verb::Delete, target)).await
                        } else {
                            let request = Request::new(Verb::Sink, target)
                                .with_arg("content", ArgRef::Inline(content.clone().into_bytes()));
                            inv.issue(request).await
                        };
                        match outcome {
                            Ok(_) if content.is_empty() => format!("{at} is clear."),
                            Ok(_) => format!("{at} is now {content}."),
                            Err(refused) => refused.to_string(),
                        }
                    }
                };
                Ok(Representation::new(
                    text_html_utf8(),
                    escape(&said).into_bytes(),
                ))
            })
        },
    )
    .with_description(
        Description::new("sheet-view-edit")
            .title("Edit, as a view")
            .summary(
                "Type into a cell (or clear it) and answer, as HTML text, what happened — a \
                 refusal included.",
            )
            .verb(Verb::Sink)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("ref")
                    .summary("the cell, e.g. B3 (any case)")
                    .class(XSD_STRING),
            )
            .input(
                ArgSpec::new("content")
                    .summary("what to type into it; empty clears it")
                    .class(XSD_STRING)
                    .optional(),
            )
            .output(TEXT_HTML_UTF8),
    )
}
// ANCHOR_END: view_edit

fn template(source: &str) -> UriTemplate {
    UriTemplate::parse(source).expect("a constant template parses")
}

// ANCHOR: space
/// Every name in the sheet, with the inputs in a store of their own.
pub fn space() -> EndpointSpace {
    space_over(Arc::default())
}

/// [`space`], with the inputs in an [`InputStore`] the caller keeps.
pub fn space_over(store: Arc<InputStore>) -> EndpointSpace {
    space_with(store, Arc::default())
}

/// [`space`], with the inputs and the feeds in stores the caller keeps.
pub fn space_with(inputs: Arc<InputStore>, feeds: Arc<FeedStore>) -> EndpointSpace {
    EndpointSpace::new()
        .bind(template(INPUT), input(inputs))
        .bind(template(FEED), feed(feeds))
        .bind(template(TICK), tick())
        .bind(template(COMPILE), compile())
        .bind(template(FORMULA), formula())
        .bind(template(REFS), refs())
        .bind(template(PRECEDENTS), precedents())
        .bind(template(RANGE), range())
        .bind(template(CELL), cell())
        .bind(template(TEMPLATE), templates())
        // ANCHOR: views
        .bind(template(VIEW_CELL), view("cell"))
        .bind(template(VIEW_GRID), view("grid"))
        .bind(template(VIEW_CACHED), view("cached"))
        .bind(template(VIEW_COST_CELL), view("cost-cell"))
        .bind(template(VIEW_COST), view("cost"))
        // ANCHOR_END: views
        .bind(template(VIEW_EDIT), view_edit())
}

/// A host for the sheet alone: its names and `ikigai-fn`'s (for `conditional`), with the Meta
/// renderer that lets the kernel read what the compiler declares.
pub fn kernel() -> Kernel {
    kernel_over(Arc::default())
}

/// [`kernel`], over an [`InputStore`] the caller keeps.
pub fn kernel_over(store: Arc<InputStore>) -> Kernel {
    let space = Fallback::new(vec![
        Arc::new(space_over(store)) as Arc<dyn Space>,
        Arc::new(ikigai_fn::space()),
    ]);
    Kernel::with_meta_renderer(Arc::new(space), Arc::new(TurtleRenderer))
}
// ANCHOR_END: space

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_value_reads_as_what_it_is() {
        assert_eq!(Value::of(""), Value::Empty);
        assert_eq!(Value::of("-2.5"), Value::Number(-2.5));
        assert_eq!(Value::of("#DIV/0"), Value::Error(DIV_ERROR));
        assert_eq!(Value::of("#div/0"), Value::Text("#div/0".to_string()));
        assert_eq!(Value::of("hello").number(), Err(VALUE_ERROR));
        assert_eq!(Value::Empty.number(), Ok(0.0));
        assert_eq!(Value::Empty.text(), "0");
    }

    #[test]
    fn a_value_name_gives_back_its_cell() {
        assert_eq!(
            cell_of("urn:iki:tutorial:sheet:cell:B3"),
            CellRef::parse("B3")
        );
        assert_eq!(cell_of("urn:iki:tutorial:sheet:input:B3"), None);
        assert_eq!(cell_of("urn:iki:tutorial:sheet:cell:b3"), None);
    }
}
