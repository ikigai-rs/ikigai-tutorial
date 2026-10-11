//! The code taught by "A sentence becomes a draft".
//!
//! `ikigai-nl` turns a sentence into a SPARQL query with a model as the drafter and never the
//! actor: every step is a resource read or write under the CALLER's capability, and the most
//! the model's text can become is a DRAFT script, private to the person who asked, which a
//! person reviews and publishes. This crate mounts it on the previous chapters' host (the
//! script host and the store, with the same items in it), beside the vocabulary the grounding
//! cites and a model door.
//!
//! The model door is a [`stub`] in every gate: a canned draft per attempt, so what is under
//! test is the funnel around the model, never the model, and nothing reaches a network. A
//! reader with a local Ollama swaps in the real `urn:llm:ask` ([`Model::Ollama`]).

use std::io::Read;
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ikigai_core::{Capability, Fallback, Kernel, Space, Verb};
use ikigai_http::{HttpRequest, HttpResponse, HttpTransport};
use ikigai_nl::{Examples, SpaceConfig};
use ikigai_script::authority::{same_for_all, Ceiling};
use script_host::{issue, READ_ITEMS, READ_PAYROLL};

pub mod stub;

/// The ask the chapter repairs: the stub's first draft names a predicate no graph uses.
pub const ASK_TITLES: &str = "the titles of the open items";

/// The ask from the live demo (gonk, 2026-10-11), word for word.
pub const ASK_P2: &str =
    "the number and title of every open item with priority 2 or higher (0 is highest), ordered by number";

// ANCHOR: book_count
/// The first page's ask, and what the stub answers it with: a count, which passes the check
/// the first time.
pub const ASK_BOOKS: &str = "How many books are on the shelf?";

/// The stub's draft for [`ASK_BOOKS`].
pub const BOOK_COUNT: &str = "PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT (COUNT(?book) AS ?books) WHERE {
  GRAPH <urn:iki:tutorial:shelf> { ?book dcterms:title ?title }
}";
// ANCHOR_END: book_count

// ANCHOR: drafts
/// The stub's first draft for [`ASK_TITLES`]: `ledger:state` is not a predicate the items
/// graph uses (it says `ledger:status`), and the check says so.
pub const TITLES_FIRST: &str = "PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?title WHERE {
  GRAPH <urn:iki:tutorial:items> { ?item ledger:state ledger:open ; dcterms:title ?title }
} ORDER BY ?title";

/// Its repair, after the model is shown what was refused.
pub const TITLES_REPAIRED: &str = "PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?title WHERE {
  GRAPH <urn:iki:tutorial:items> { ?item ledger:status ledger:open ; dcterms:title ?title }
} ORDER BY ?title";

/// What a 70B model drafted for [`ASK_P2`] in the live demo, with the demo's graph renamed to
/// this book's: a string `\"open\"` where the status is the IRI `ledger:open`, `FROM` paired with
/// `GRAPH`, and the priority bound named `limit`.
pub const P2_DRAFT: &str = "# @param limit xsd:integer optional default 2
SELECT ?number ?title
FROM <urn:iki:tutorial:items>
WHERE {
  GRAPH <urn:iki:tutorial:items> {
    ?item a <https://ikigai-rs.dev/ns/ledger#Item> ;
      <https://ikigai-rs.dev/ns/ledger#status> \"open\" ;
      <https://ikigai-rs.dev/ns/ledger#priority> ?priority ;
      <https://ikigai-rs.dev/ns/ledger#number> ?number ;
      <http://purl.org/dc/terms/title> ?title .
    FILTER (?priority <= ?limit)
  }
}
ORDER BY ?number";

/// The query a person writes after reading [`P2_DRAFT`]: the status as the IRI it is, one
/// `GRAPH` and no `FROM`, and the bound written into the text.
pub const P2_CORRECTED: &str = "PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title
WHERE {
  GRAPH <urn:iki:tutorial:items> {
    ?item a ledger:Item ;
          ledger:status ledger:open ;
          ledger:priority ?priority ;
          ledger:number ?number ;
          dcterms:title ?title .
    FILTER (?priority <= 2)
  }
}
ORDER BY ?number";
// ANCHOR_END: drafts

/// Which model the drafter asks.
#[derive(Clone)]
pub enum Model {
    /// The [`stub`]: canned drafts, recorded prompts, no network.
    Stub(Arc<Mutex<stub::Stub>>),
    /// `ikigai-llm`'s `urn:llm:ask` over a local Ollama, at this model.
    Ollama(String),
}

impl Model {
    /// The stub, scripted with this chapter's drafts.
    pub fn stub() -> Model {
        Model::Stub(Arc::new(Mutex::new(stub::Stub::new(&[
            (ASK_TITLES, &[TITLES_FIRST, TITLES_REPAIRED]),
            (ASK_P2, &[P2_DRAFT]),
            (ASK_BOOKS, &[BOOK_COUNT]),
        ]))))
    }

    fn space(&self) -> Arc<dyn Space> {
        match self {
            Model::Stub(stub) => Arc::new(stub::space(Arc::clone(stub))),
            Model::Ollama(model) => Arc::new(ikigai_llm::space(
                Arc::new(Ureq),
                ikigai_llm::OpenAiConfig::ollama(model.as_str()),
            )),
        }
    }
}

// ANCHOR: host
/// The drafting host: the previous chapters' script host and store, then `ikigai-nl`, the
/// vocabulary its grounding cites, and a model door. The grounding's worked examples are named
/// rather than read from the live catalog, so a grounding stays cached until what it read
/// changes.
pub fn kernel(model: &Model) -> Arc<Kernel> {
    kernel_with(model, Vec::new())
}

/// [`kernel`], with `yours` mounted after the script host and the store and in front of
/// `ikigai-nl`, the vocabulary and the model: for the exercises, a space of the reader's own
/// binds names the drafting host does not (or answers `urn:llm:ask` before the model does).
pub fn kernel_with(model: &Model, yours: Vec<Arc<dyn Space>>) -> Arc<Kernel> {
    let nl = SpaceConfig::new().examples(Examples::Named(vec!["open-items".to_string()]));
    let mut beside = yours;
    beside.extend([
        Arc::new(ikigai_nl::space(nl)) as Arc<dyn Space>,
        Arc::new(ikigai_vocab::space()) as Arc<dyn Space>,
        model.space(),
    ]);
    script_host::kernel_beside(same_for_all(Ceiling::unbounded()), beside)
}
// ANCHOR_END: host

/// The first page's drafting host: the script host in front of a store holding the shelf of
/// three books (`script_host::first`), then `ikigai-nl`, the vocabulary and a model door. No
/// worked examples: the shelf is all there is to see.
pub fn shelf_kernel(model: &Model) -> Arc<Kernel> {
    let nl = SpaceConfig::new().examples(Examples::Named(Vec::new()));
    script_host::kernel_over(
        same_for_all(Ceiling::unbounded()),
        vec![
            Arc::new(ikigai_nl::space(nl)) as Arc<dyn Space>,
            Arc::new(ikigai_vocab::space()) as Arc<dyn Space>,
            model.space(),
        ],
        Some(script_host::first::BOOKS),
    )
}

/// The host's spaces with the stub as the model, for the book's name gate.
pub fn space() -> Fallback {
    let mut spaces = script_host::root(same_for_all(Ceiling::unbounded()), None);
    spaces.push(Arc::new(ikigai_nl::space(SpaceConfig::new())));
    spaces.push(Arc::new(ikigai_vocab::space()));
    spaces.push(Model::stub().space());
    Fallback::new(spaces)
}

/// Publish the first chapter's `open-items` as the host, so the grounding has one worked
/// example to show.
pub fn publish_example(kernel: &Kernel) {
    issue(
        kernel,
        &Capability::root(),
        Verb::Sink,
        "urn:script:open-items",
        &[("language", "sparql"), ("content", script_host::OPEN_ITEMS)],
    )
    .expect("publishing the example");
}

// ANCHOR: people
/// Drafts. Reads the items graph and the example, may reach the model (which, like every
/// call the drafter makes, runs under HER capability), and may write, read and run the one
/// script name she drafts into.
pub fn alice() -> Capability {
    person(
        "alice",
        &[
            READ_ITEMS,
            "urn:cap:script:read:open-items",
            "urn:cap:net:localhost:11434",
            "urn:cap:script:write:open-titles",
            "urn:cap:script:read:open-titles",
            "urn:cap:script:write:open-p2",
            "urn:cap:script:read:open-p2",
            "urn:cap:script:run:open-p2",
        ],
    )
}

/// Holds the same read grant on `open-p2` as Alice, and the payroll graph, but drafted
/// nothing.
pub fn bob() -> Capability {
    person(
        "bob",
        &[READ_ITEMS, READ_PAYROLL, "urn:cap:script:read:open-p2"],
    )
}

/// Holds nothing but the network: may ask, can see nothing.
pub fn carol() -> Capability {
    person("carol", &["urn:cap:net:localhost:11434"])
}
// ANCHOR_END: people

fn person(name: &str, scopes: &[&str]) -> Capability {
    Capability::scoped(scopes.iter().copied())
        .with_principal(&format!("urn:iki:tutorial:person:{name}"))
        .expect("a principal IRI")
}

/// The HTTP seam `ikigai-llm` reaches a server through, filled with a blocking client, as
/// that crate's own `ask` example fills it. Only `--ollama` uses it.
struct Ureq;

#[async_trait]
impl HttpTransport for Ureq {
    async fn send(&self, request: HttpRequest) -> Result<HttpResponse, String> {
        let mut req = ureq::request(request.method.as_str(), &request.url);
        for (name, value) in &request.headers {
            req = req.set(name, value);
        }
        let (status, mut reader) = match req.send_bytes(&request.body) {
            Ok(response) => (response.status(), response.into_reader()),
            Err(ureq::Error::Status(code, response)) => (code, response.into_reader()),
            Err(e) => return Err(e.to_string()),
        };
        let mut body = Vec::new();
        reader.read_to_end(&mut body).map_err(|e| e.to_string())?;
        Ok(HttpResponse {
            status,
            headers: vec![],
            body,
        })
    }
}
