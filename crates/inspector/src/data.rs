//! **Views of data that is not a graph**: one table of planets, held as a JSON document and as a
//! CSV table, and ONE view of it that does not know which it was read from.
//!
//! The graph views read the kernel's readouts as RDF because the kernel publishes them that way.
//! These read the form the data already has, and nothing here turns either source into RDF.
//! The model is a JSON array of records: the JSON source already is one, and the CSV source
//! becomes one by transreption (CSV to JSON, a record per row, keyed by the header).
//!
//! | name | kind | what it is |
//! |---|---|---|
//! | `data:planets.json`, `data:planets.csv` | atom | the data, readable and writable (Sink `content`) |
//! | `template:records-json`, `template:records-csv` | artifact | where each source's records come from |
//! | `records:json`, `records:csv` | composition | the source's records, as a JSON array |
//! | `template:planets`, `template:planet-row` | artifact | the view, and one row of it |
//! | `fragment:planets-{source}` | composition | `template:planets` over `records:{source}` |
//! | `urn:iki:fn:each` | composition (ikigai-fn 0.6) | the row template filled once per record, fields as arguments |
//! | `urn:json:query` | composition (ikigai-json 0.1) | the CSV atom read as JSON, through ikigai-json's CSV transreptor |
//!
//! `{source}` is `json` or `csv`. The only code here is the two atoms: everything between them
//! and the page is a composition of published resources. (While this was written, the iteration
//! and the CSV-to-JSON step were labeled stopgaps; ikigai-fn 0.6.0's `each` and ikigai-json
//! 0.1.0's `query` replaced them with the same arguments.)

use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, ArgSpec, Description, EndpointSpace, Error, Exact, FnEndpoint, Invocation,
    ReprType, Representation, Verb,
};

use crate::{composed, template, BASE};

/// The capability that writes the planet data (either source).
pub const CAP_DATA: &str = "urn:cap:iki:tutorial:inspect:data";

/// The sources the one view can be read from.
pub const SOURCES: [&str; 2] = ["json", "csv"];

/// The largest document a source atom accepts.
pub const MAX_DATA: usize = 16 * 1024;

const TEXT: &str = "text/plain; charset=utf-8";
const JSON: &str = "application/json";
// Bare `text/csv`, no `charset`: the kernel plans a transreption on the media type as written,
// and `text/csv; charset=utf-8` matches no registered transreptor (ikigai-json 0.1.0 declares
// `text/csv`), so a query over it is refused. Reported to the hub; UTF-8 is RFC 4180's default.
const CSV: &str = "text/csv";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// The planets, as a JSON document a reader would recognize: an array of objects.
pub const PLANETS_JSON: &str = include_str!("../artifacts/planets.json");

/// The same planets, as a CSV table with a header row.
pub const PLANETS_CSV: &str = include_str!("../artifacts/planets.csv");

/// A data atom: Source reads it (cacheable: it hangs from its own name, which a Sink cuts);
/// Sink replaces it with `content`. It does not check what it is given: a document the view
/// cannot read makes the view fail, with the parser's reason.
fn atom_endpoint(
    id: &'static str,
    label: &'static str,
    media: &'static str,
    initial: &str,
) -> FnEndpoint {
    let held = Arc::new(Mutex::new(initial.to_string()));
    FnEndpoint::new(id, move |inv: &Invocation<'_>| {
        let mut held = held
            .lock()
            .map_err(|_| Error::Endpoint(format!("{id} is poisoned")))?;
        if inv.request.verb == Verb::Sink {
            let text = inv.inline_str("content")?;
            if text.len() > MAX_DATA {
                return Err(Error::InvalidArgument {
                    name: "content".to_string(),
                    detail: format!("{} bytes; a source holds at most {MAX_DATA}", text.len()),
                });
            }
            *held = text.to_string();
            return Ok(Representation::new(
                ReprType::new(TEXT),
                b"written".to_vec(),
            ));
        }
        Ok(Representation::new(ReprType::new(media), held.clone().into_bytes()).cacheable())
    })
    .with_description(
        Description::new(id)
            .title(format!("The planets, as {label}"))
            .summary(format!(
                "The eight planets as {label}. Reading it is cached; writing it cuts its \
                 thread, so the view over it recomputes and the view over the other source \
                 does not."
            ))
            .action(ActionSpec::new(Verb::Source).output(media))
            .action(
                ActionSpec::new(Verb::Sink)
                    .requires(CAP_DATA)
                    .input(
                        ArgSpec::new("content")
                            .summary(format!("the new document, at most {MAX_DATA} bytes"))
                            .class(XSD_STRING),
                    )
                    .output(TEXT),
            )
            .verb(Verb::Meta),
    )
}

/// The data views' names. Bound before the graph views' `fragment:{view}`, so
/// `fragment:planets-json` is this composition and not a graph view.
pub fn space(space: EndpointSpace) -> EndpointSpace {
    // ANCHOR: bindings
    space
        .bind(
            Exact::new(format!("{BASE}data:planets.json")),
            atom_endpoint("inspect-data-json", "a JSON document", JSON, PLANETS_JSON),
        )
        .bind(
            Exact::new(format!("{BASE}data:planets.csv")),
            atom_endpoint("inspect-data-csv", "a CSV table", CSV, PLANETS_CSV),
        )
        .bind(
            Exact::new(format!("{BASE}records:json")),
            composed(
                "urn:iki:tutorial:inspect:template:records-json",
                "inspect-records-json",
            ),
        )
        .bind(
            Exact::new(format!("{BASE}records:csv")),
            composed(
                "urn:iki:tutorial:inspect:template:records-csv",
                "inspect-records-csv",
            ),
        )
        .bind(
            template(&format!("{BASE}fragment:planets-{{source}}")),
            composed(
                "urn:iki:tutorial:inspect:template:planets",
                "inspect-fragment-planets",
            ),
        )
        .bind(Exact::new("urn:iki:fn:each"), ikigai_fn::each())
    // ANCHOR_END: bindings
}
