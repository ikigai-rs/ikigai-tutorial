//! The code taught by two places in the ikigai Book: the section "A format that is not RDF" in
//! **Transreption**, and the applied chapter **Short names in a space**.
//!
//! A small shop keeps its stock as a CSV table, the way a spreadsheet exports it. Nothing here
//! turns the table into RDF, and nothing here is a CSV parser or a query language:
//!
//! | name | kind | what it is |
//! |---|---|---|
//! | `urn:iki:tutorial:shop:stock` | atom | the stock, as `text/csv`; Sink `content` replaces it |
//! | `urn:json:query` | composition (ikigai-json) | jq over any resource; a CSV one is read through the `text/csv` to `application/json` transreptor |
//! | `urn:iki:tutorial:shop:prefixes` | atom | the prefix table, a declaration (the one "A space that changes while it runs" wrote) |
//! | `urn:iki:space:dynamic:{name}` | dynamic space | the prefix table, bound: where the short names exist |
//! | `urn:iki:tutorial:shop:template:{summary,row,records}` | artifact | three templates, written with the short names |
//! | `urn:iki:tutorial:shop:summary`, `…:records` | composition | `summary` and `records` filled (`compose_over`) |
//!
//! The only code is the two atoms. Everything between them and the one-line summary is a
//! composition of published resources: ikigai-json's `urn:json:query`, ikigai-fn's `compose`,
//! `each` and `$j`, and ikigai-fn's `prefix_space`.
//!
//! [`Host`] runs the chapter's listings: the root (the shop's names, ikigai-json and ikigai-fn),
//! the prefix space, and the loop that rebinds it when its table is written. A request issued
//! in the prefix space ([`Host::read_in`]) carries it as a CORRIDOR, which every sub-request
//! inherits; a request issued without it ([`Host::read`]) finds no short name at all.

use std::sync::{Arc, Mutex};

use futures::executor::{block_on, LocalPool};
use futures::task::LocalSpawnExt;
use ikigai_core::{
    ActionSpec, ArgRef, ArgSpec, Capability, Description, DynamicSpace, EndpointSpace, Error,
    Exact, Fallback, FnEndpoint, Invocation, Iri, Kernel, ReprType, Representation, Request, Scope,
    Space, Verb,
};
use live_space::DeclarationState;

// The two libraries the listings call, re-exported so a listing can name them through this
// crate: the book's test build holds two releases of ikigai-fn (ikigai-ui, in the UI book, links
// the older), so `extern crate ikigai_fn` would be ambiguous there.
pub use ikigai_fn;
pub use ikigai_json;

// ANCHOR: names
/// The stock: a CSV table with a header row.
pub const STOCK: &str = "urn:iki:tutorial:shop:stock";

/// Where the prefix table lives. A write to it changes which short names exist.
pub const PREFIXES: &str = "urn:iki:tutorial:shop:prefixes";

/// The one-line summary: `template:summary`, filled.
pub const SUMMARY: &str = "urn:iki:tutorial:shop:summary";

/// The stock's records, as one JSON array: `template:records`, filled.
pub const RECORDS: &str = "urn:iki:tutorial:shop:records";

/// One row of the stock list, filled once per record by `urn:iki:fn:each`.
pub const ROW: &str = "urn:iki:tutorial:shop:template:row";
// ANCHOR_END: names

/// `stock.csv`: the stock the shop starts with.
pub const STOCK_CSV: &str = include_str!("../stock.csv");

/// `prefixes.arrangement`: the shop's prefix table, `json:` and `shop:`.
pub const PREFIX_TABLE: &str = include_str!("../prefixes.arrangement");

/// The templates, by the last segment of their names, `urn:iki:tutorial:shop:template:{name}`.
pub const TEMPLATES: [(&str, &str); 3] = [
    ("summary", include_str!("../templates/summary.txt")),
    ("row", include_str!("../templates/row.txt")),
    ("records", include_str!("../templates/records.txt")),
];

/// The largest table the stock accepts.
pub const MAX_STOCK: usize = 16 * 1024;

const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

// Bare `text/csv`, with no `charset`: a transreptor is selected on the media type as served, and
// ikigai-json 0.1.0 declares `text/csv`, so `text/csv; charset=utf-8` would match none.
const CSV: &str = "text/csv";

/// What the stock holds: the table last written.
#[derive(Debug)]
pub struct StockState {
    table: Mutex<String>,
}

impl Default for StockState {
    fn default() -> Self {
        StockState {
            table: Mutex::new(STOCK_CSV.to_string()),
        }
    }
}

// ANCHOR: stock
/// `shop-stock`: the stock, an atom. `Source` answers the table as `text/csv`, cached (it hangs
/// from its own name, which the kernel cuts after every write); `Sink` replaces it with
/// `content`. It does not read the table: a table jq cannot be given is refused by the
/// transreptor, with the CSV reader's reason, when something asks for it as JSON.
pub fn stock(state: Arc<StockState>) -> FnEndpoint {
    FnEndpoint::new("shop-stock", move |inv: &Invocation<'_>| {
        let mut table = state
            .table
            .lock()
            .map_err(|_| Error::Endpoint("shop-stock is poisoned".to_string()))?;
        if inv.request.verb == Verb::Sink {
            let text = inv.inline_str("content")?;
            if text.len() > MAX_STOCK {
                return Err(Error::InvalidArgument {
                    name: "content".to_string(),
                    detail: format!("{} bytes; the stock holds at most {MAX_STOCK}", text.len()),
                });
            }
            *table = text.to_string();
            return Ok(Representation::new(
                ReprType::new("text/plain"),
                b"ok".to_vec(),
            ));
        }
        Ok(Representation::new(ReprType::new(CSV), table.clone().into_bytes()).cacheable())
    })
    .with_description(
        Description::new("shop-stock")
            .title("The shop's stock")
            .summary(
                "The stock as a CSV table (item, shelf, count). Reading it is cached; writing \
                 it cuts its thread, so everything read from it recomputes.",
            )
            .action(ActionSpec::new(Verb::Source).output(CSV))
            .action(
                ActionSpec::new(Verb::Sink)
                    .input(
                        ArgSpec::new("content")
                            .summary(format!("the new table, at most {MAX_STOCK} bytes"))
                            .class(XSD_STRING),
                    )
                    .output("text/plain"),
            )
            .verb(Verb::Meta),
    )
}
// ANCHOR_END: stock

/// A template, served as it is written, markers and all.
fn template(name: &'static str, text: &'static str) -> FnEndpoint {
    FnEndpoint::new(
        format!("shop-template-{name}"),
        move |_: &Invocation<'_>| {
            Ok(
                Representation::new(ReprType::new("text/plain"), text.as_bytes().to_vec())
                    .cacheable(),
            )
        },
    )
}

fn iri(text: &str) -> Iri {
    Iri::parse(text).expect("a constant IRI")
}

// ANCHOR: space
/// The shop's names: the stock, the prefix table, the three templates, and the two views that
/// fill them. Nothing here knows that the templates' names are short: they are filled by
/// whatever the request's resolution chain makes of them.
pub fn space(stock_state: Arc<StockState>, prefixes: Arc<DeclarationState>) -> EndpointSpace {
    let mut space = EndpointSpace::new()
        .bind(Exact::new(STOCK), stock(stock_state))
        .bind(Exact::new(PREFIXES), live_space::declaration(prefixes));
    for (name, text) in TEMPLATES {
        space = space.bind(
            Exact::new(format!("urn:iki:tutorial:shop:template:{name}")),
            template(name, text),
        );
    }
    space
        .bind(
            Exact::new(SUMMARY),
            ikigai_fn::compose_over(iri("urn:iki:tutorial:shop:template:summary"))
                .named("shop-summary"),
        )
        .bind(
            Exact::new(RECORDS),
            ikigai_fn::compose_over(iri("urn:iki:tutorial:shop:template:records"))
                .named("shop-records"),
        )
}

/// The prefix space called `name`: an empty dynamic space built from [`PREFIXES`], which binds
/// only a prefix table (ikigai-fn's `prefix_space` checks it), and is named
/// `urn:iki:space:dynamic:{name}`.
pub fn prefixes(name: &str) -> DynamicSpace {
    ikigai_fn::prefix_space(name, iri(PREFIXES))
}

/// The host's root: the shop's names, then ikigai-json's (`urn:json:*`), then ikigai-fn's
/// (`urn:iki:fn:*`). The prefix space is not in it: it is a corridor a request carries.
pub fn root(stock_state: Arc<StockState>, prefixes: Arc<DeclarationState>) -> Fallback {
    Fallback::new(vec![
        Arc::new(space(stock_state, prefixes)) as Arc<dyn Space>,
        Arc::new(ikigai_json::space()),
        Arc::new(ikigai_fn::space()),
    ])
}
// ANCHOR_END: space

/// A `Source` of `iri`, with `args` inline: the request every listing reads with.
pub fn source(iri: &str, args: &[(&str, &str)]) -> Request {
    let mut request = Request::new(Verb::Source, Iri::parse(iri).expect("an IRI"));
    for (name, value) in args {
        request = request.with_arg(*name, ArgRef::Inline(value.as_bytes().to_vec()));
    }
    request
}

/// A `Sink` of `content` to `iri`.
pub fn sink(iri: &str, content: &str) -> Request {
    Request::new(Verb::Sink, Iri::parse(iri).expect("an IRI"))
        .with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()))
}

/// The chapter's host, started: the root, the prefix space, and the loop that rebinds the
/// space whenever its table is written.
pub struct Host {
    /// The kernel every listing asks.
    pub kernel: Arc<Kernel>,
    /// The prefix space, for its status and its thread.
    pub space: Arc<DynamicSpace>,
    /// The corridor a request carries to be issued in the prefix space.
    pub corridor: Scope,
    pool: LocalPool,
}

impl Host {
    // ANCHOR: start
    /// Make the root, an empty prefix space called `name`, and the loop that rebinds it from
    /// [`PREFIXES`]; the corridor is the space, alone.
    pub fn start(name: &str) -> Host {
        let kernel = Arc::new(Kernel::with_meta_renderer(
            Arc::new(root(Arc::default(), Arc::default())),
            Arc::new(ikigai_vocab::TurtleRenderer),
        ));
        let space = Arc::new(prefixes(name));
        let corridor = Scope::empty().with(Arc::clone(&space) as Arc<dyn Space>);
        let pool = LocalPool::new();
        let (looping, on) = (Arc::clone(&space), Arc::clone(&kernel));
        pool.spawner()
            .spawn_local(async move {
                let stopped = looping.drive(&on, &Capability::root()).await;
                panic!("the prefix space's loop stopped: {stopped}");
            })
            .expect("spawn the rebuild loop");
        let mut host = Host {
            kernel,
            space,
            corridor,
            pool,
        };
        host.settle();
        host
    }
    // ANCHOR_END: start

    /// Write `content` to `iri` under root, then let the loop run until it has nothing left to
    /// do (a write to [`PREFIXES`] rebinds the space).
    pub fn write(&mut self, iri: &str, content: &str) {
        block_on(self.kernel.issue(sink(iri, content), &Capability::root())).expect("the write");
        self.settle();
    }

    /// Run the host's loop until it waits for the next cut.
    pub fn settle(&mut self) {
        self.pool.run_until_stalled();
    }

    /// `request` under root, outside the prefix space, its answer as text.
    pub fn read(&self, request: Request) -> Result<String, Error> {
        let answer = block_on(self.kernel.issue(request, &Capability::root()))?;
        Ok(String::from_utf8_lossy(&answer.bytes).into_owned())
    }

    // ANCHOR: read_in
    /// `request` under root, issued IN the prefix space: the space rides the request as a
    /// corridor, so every sub-request it gives rise to, at any depth, resolves the short names.
    pub fn read_in(&self, request: Request) -> Result<String, Error> {
        let root = Capability::root();
        let answer = block_on(self.kernel.issue_in(request, &root, self.corridor.clone()))?;
        Ok(String::from_utf8_lossy(&answer.bytes).into_owned())
    }
    // ANCHOR_END: read_in

    /// Whether `request`, under root and outside the prefix space, would be served from the
    /// cache now.
    pub fn is_cached(&self, request: &Request) -> bool {
        self.kernel.is_cached(request, &Capability::root())
    }

    /// Whether `request`, under root and in the prefix space, would be served from the cache now.
    pub fn is_cached_in(&self, request: &Request) -> bool {
        self.kernel
            .is_cached_in(request, &Capability::root(), &self.corridor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_summary_is_composed_in_the_prefix_space_and_nowhere_else() {
        let mut host = Host::start("shop");
        assert!(matches!(
            host.read_in(source(SUMMARY, &[])),
            Err(Error::Unresolved(_))
        ));
        host.write(PREFIXES, PREFIX_TABLE);
        assert_eq!(
            host.read_in(source(SUMMARY, &[])).unwrap(),
            "4 items, 49 on the shelves; out of stock: sugar."
        );
        assert!(host.read(source(SUMMARY, &[])).is_err());
    }

    #[test]
    fn each_fills_one_row_per_record() {
        let mut host = Host::start("shop");
        host.write(PREFIXES, PREFIX_TABLE);
        let list = source(
            "urn:iki:fn:each",
            &[("src", RECORDS), ("template", ROW), ("separator", ", ")],
        );
        assert_eq!(
            host.read_in(list).unwrap(),
            "flour (12), sugar (0), rice (30), lentils (7)"
        );
    }
}
