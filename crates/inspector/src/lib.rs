//! **The kernel inspector**: the demo app of *Building ikigai UIs*, as resources.
//!
//! The inspector shows a kernel to the person running it: what its cache holds, which golden
//! threads have been cut, how its spaces are arranged, what its endpoints say about
//! themselves, and its scheduler. Each of those is already a resource the kernel answers
//! (`urn:kernel:cache`, `urn:kernel:threads`, `urn:kernel:topology`, `urn:kernel:catalog`,
//! `urn:kernel:scheduler`), so the inspector writes no code to find any of it. It reads each one
//! **under the caller's capability**, so it shows only what its viewer may inspect, and turns its
//! Turtle face into a page with XSLT.
//!
//! This crate is the model and the artifacts. A delivery target is a thin crate of its own that
//! serves these names: `crates/inspector-htmx` serves them over HTTP as htmx pages, and
//! `crates/ui-book-wasm` answers them in the book's own pages.
//!
//! ## The names
//!
//! Every name is under [`BASE`], `urn:iki:tutorial:inspect:`.
//!
//! | name | kind | what it is |
//! |---|---|---|
//! | `xsl:{cache,threads,topology,catalog,entry,thread}` | artifact | one XSLT stylesheet per view, `xsl:value-of` only |
//! | `template:{view,scheduler,note,card}` | artifact | compose templates: the fragment shapes |
//! | `header`, `inspector.css`, `live.js` | artifact | the site header, the inspector's own rules, and the script that hears cuts |
//! | `graph:{view}` | composition (code) | the view's input: the kernel's Turtle face, with the links a page needs, as RDF/XML |
//! | `fragment:{view}` | composition | `urn:xslt:transform` over `graph:{view}` with `xsl:{view}` |
//! | `fragment:scheduler` | composition | the scheduler's TEXT face, labeled: it has no Turtle face yet |
//! | `title:{name}` | data | a page's title |
//! | `page:{name}` | composition | ikigai-ui's shell over the fragment (`ikigai_ui::pages`) |
//! | `demo:note` | **atom** | the one thing here that holds state: a note you can write |
//! | `demo:card` | composition | the note in a card, cached, so a write is a cut you can watch |
//! | `data:planets.{json,csv}`, `fragment:planets-{source}` | atom, composition | a view of data that is NOT a graph, over two sources; see [`data`] |
//!
//! `{view}` is `cache`, `threads`, `topology` or `catalog`, or a detail: `entry:{id}` (one cache
//! entry and the threads it hangs from, `{id}` the hex of its `urn:ikigai:cache:entry:{id}`) and
//! `thread:{name}` (one golden thread and the entries hanging from it).
//!
//! ⚠ **A detail page's name reaches its fragment percent-encoded.** `page:entry:8a03…` composes
//! `fragment:entry%3A8a03…`, because compose encodes a template argument spliced into an IRI as
//! ONE segment (ikigai-ui documents it on `pages`). So the detail names are bound in both
//! spellings, and `graph:{view}` decodes a view that arrives as one encoded segment (no raw `:`
//! in it) exactly once.
//!
//! ## Two deliveries, one set of names (UI conventions D17)
//!
//! Every artifact is compiled in. A host that binds a directory at [`DIR`] (ikigai-fs's
//! `FileEndpoint` under `urn:iki:tutorial:inspect:dir:{file}`) replaces any artifact with the
//! file of the same name there, and only on that file's own absence falls back to the
//! compiled-in bytes, so a write to the file recomputes every page over it.

#![forbid(unsafe_code)]

pub mod data;

use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, ArgRef, ArgSpec, AsyncFnEndpoint, Description, EndpointSpace, Error, Exact,
    Fallback, FnEndpoint, Invocation, InvokeFuture, Iri, Kernel, ReprType, Representation, Request,
    Space, UriTemplate, Verb,
};
use ikigai_vocab::TurtleRenderer;

/// Every name the inspector binds starts here.
pub const BASE: &str = "urn:iki:tutorial:inspect:";

/// The page family [`ikigai_ui::pages`] is given: the shell reaches this crate's header, titles
/// and fragments through `urn:iki:ui:{header,title,main}:inspect…`.
pub const FAMILY: &str = "inspect";

/// Where an artifact looks for its override: `urn:iki:tutorial:inspect:dir:{file}`.
pub const DIR: &str = "urn:iki:tutorial:inspect:dir:";

/// The list views, in the order the header names them.
pub const VIEWS: [&str; 8] = [
    "cache",
    "threads",
    "topology",
    "catalog",
    "scheduler",
    "note",
    "planets-json",
    "planets-csv",
];

/// The capability the kernel's own readouts require. Every view of the kernel needs it, and
/// [`graph`](self) declares it, so the action manifold never offers a view its caller cannot
/// read.
pub const CAP_INSPECT: &str = "urn:cap:kernel:inspect";

/// The capability that writes the demo note.
pub const CAP_NOTE: &str = "urn:cap:iki:tutorial:inspect:note";

/// The namespace of the few terms the inspector adds to a graph: the links a page needs.
pub const TERMS: &str = "urn:iki:tutorial:inspect:term#";

const HTML: &str = "text/html; charset=utf-8";
const JS: &str = "text/javascript";
const CSS: &str = "text/css";
const XSLT: &str = "application/xslt+xml";
const TEXT: &str = "text/plain; charset=utf-8";
const TURTLE: &str = "text/turtle";
const RDF_XML: &str = "application/rdf+xml";
const JSON: &str = "application/json";
const OCTET_STREAM: &str = "application/octet-stream";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const IK: &str = "https://ikigai-rs.dev/ns#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

// --- the artifacts ---------------------------------------------------------------------------

/// One artifact: its name, its endpoint id, the file that overrides it, its media type and its
/// compiled-in bytes.
#[derive(Debug, Clone, Copy)]
pub struct Artifact {
    /// The artifact's IRI.
    pub iri: &'static str,
    /// Its endpoint id.
    pub id: &'static str,
    /// The file under [`DIR`] that overrides it, and its name under `artifacts/`.
    pub file: &'static str,
    /// The media type it is served with, from either delivery.
    pub media_type: &'static str,
    /// The compiled-in delivery.
    pub bytes: &'static [u8],
}

macro_rules! artifact {
    ($name:literal, $id:literal, $file:literal, $type:expr) => {
        Artifact {
            iri: concat!("urn:iki:tutorial:inspect:", $name),
            id: $id,
            file: $file,
            media_type: $type,
            bytes: include_bytes!(concat!("../artifacts/", $file)),
        }
    };
}

/// Every artifact: the whole set a directory can override.
pub const ARTIFACTS: [Artifact; 17] = [
    artifact!("header", "inspect-header", "header.html", HTML),
    artifact!("inspector.css", "inspect-css", "inspector.css", CSS),
    artifact!("live.js", "inspect-live-js", "live.js", JS),
    artifact!("template:view", "inspect-template-view", "view.html", HTML),
    artifact!(
        "template:scheduler",
        "inspect-template-scheduler",
        "scheduler.html",
        HTML
    ),
    artifact!("template:note", "inspect-template-note", "note.html", HTML),
    artifact!("template:card", "inspect-template-card", "card.html", HTML),
    artifact!(
        "template:planets",
        "inspect-template-planets",
        "planets.html",
        HTML
    ),
    artifact!(
        "template:records-json",
        "inspect-template-records-json",
        "records-json.tmpl",
        JSON
    ),
    artifact!(
        "template:records-csv",
        "inspect-template-records-csv",
        "records-csv.tmpl",
        JSON
    ),
    artifact!(
        "template:planet-row",
        "inspect-template-planet-row",
        "planet-row.html",
        HTML
    ),
    artifact!("xsl:cache", "inspect-xsl-cache", "cache.xsl", XSLT),
    artifact!("xsl:threads", "inspect-xsl-threads", "threads.xsl", XSLT),
    artifact!("xsl:topology", "inspect-xsl-topology", "topology.xsl", XSLT),
    artifact!("xsl:catalog", "inspect-xsl-catalog", "catalog.xsl", XSLT),
    artifact!("xsl:entry", "inspect-xsl-entry", "entry.xsl", XSLT),
    artifact!("xsl:thread", "inspect-xsl-thread", "thread.xsl", XSLT),
];

/// The artifact named `iri`, or `None`.
pub fn artifact(iri: &str) -> Option<&'static Artifact> {
    ARTIFACTS.iter().find(|a| a.iri == iri)
}

/// An artifact: the directory's file if there is one, else the compiled-in bytes.
///
/// The same rule as ikigai-ui's own artifacts, written again here because ikigai-ui keeps its
/// artifact endpoint private: the fallback is taken ONLY on the directory atom's own `NotFound`
/// (or `Unresolved`, when no directory is bound), never on a denial or any other failure, so the
/// answer hangs from that atom's golden thread and is cut when the file appears or changes.
fn artifact_endpoint(a: &'static Artifact) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(a.id, move |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let file = Iri::parse(format!("{DIR}{}", a.file))
                .map_err(|e| Error::Endpoint(format!("{}: no override IRI: {e}", a.iri)))?;
            let request = Request::new(Verb::Source, file)
                .with_arg("as", ArgRef::Inline(OCTET_STREAM.as_bytes().to_vec()));
            let bytes = match inv.issue(request).await {
                Ok(rep) => rep.bytes,
                Err(Error::NotFound(_)) | Err(Error::Unresolved(_)) => a.bytes.to_vec(),
                Err(other) => return Err(other),
            };
            Ok(Representation::new(ReprType::new(a.media_type), bytes).cacheable())
        })
    })
    .with_description(
        Description::new(a.id)
            .title(format!("The inspector artifact {}", a.iri))
            .summary(format!(
                "A UI artifact of the kernel inspector: compiled in, or the file {} in the \
                 directory a host binds at {DIR}, which wins when it is there.",
                a.file
            ))
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(a.media_type),
    )
}

fn by_name(iri: &str) -> &'static Artifact {
    artifact(iri).unwrap_or_else(|| panic!("{iri} is in ARTIFACTS"))
}

// --- the graph a view transforms ---------------------------------------------------------------

/// What `graph:{view}` was asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum View {
    /// `cache`, `threads`, `topology` or `catalog`: that `urn:kernel:*` resource.
    List(&'static str),
    /// `entry:{id}`: one cache entry, and the threads it hangs from.
    Entry(String),
    /// `thread:{name}`: one golden thread, and the entries hanging from it.
    Thread(String),
}

impl View {
    /// Read a view's name. One that arrives as a single percent-encoded segment (it holds no raw
    /// `:`, as every detail name does) is decoded once first; see the crate doc.
    ///
    /// ```
    /// use inspector::View;
    /// assert_eq!(View::parse("cache"), Some(View::List("cache")));
    /// assert_eq!(View::parse("entry:8a03"), Some(View::Entry("8a03".into())));
    /// assert_eq!(View::parse("entry%3A8a03"), Some(View::Entry("8a03".into())));
    /// assert_eq!(
    ///     View::parse("thread%3Aurn%3Ademo%3Anote"),
    ///     Some(View::Thread("urn:demo:note".into()))
    /// );
    /// assert_eq!(View::parse("scheduler"), None);
    /// ```
    pub fn parse(name: &str) -> Option<View> {
        let decoded;
        let name = if name.contains(':') {
            name
        } else {
            decoded = percent_decode(name)?;
            decoded.as_str()
        };
        if let Some(list) = ["cache", "threads", "topology", "catalog"]
            .into_iter()
            .find(|v| *v == name)
        {
            return Some(View::List(list));
        }
        if let Some(id) = name.strip_prefix("entry:") {
            return (!id.is_empty() && id.bytes().all(|b| b.is_ascii_hexdigit()))
                .then(|| View::Entry(id.to_string()));
        }
        name.strip_prefix("thread:")
            .filter(|t| !t.is_empty())
            .map(|t| View::Thread(t.to_string()))
    }

    /// The kernel resource the view reads.
    fn source(&self) -> &'static str {
        match self {
            View::List("threads") => "urn:kernel:threads",
            View::List("topology") => "urn:kernel:topology",
            View::List("catalog") => "urn:kernel:catalog",
            _ => "urn:kernel:cache",
        }
    }
}

/// One segment, percent-decoded (`percent-encoding`, the crate `url` and htmx's own server
/// side use), or `None` when the bytes are not UTF-8.
fn percent_decode(s: &str) -> Option<String> {
    percent_encoding::percent_decode_str(s)
        .decode_utf8()
        .ok()
        .map(|c| c.into_owned())
}

/// Add the links a page needs to a kernel readout, keep only what a detail view shows, and
/// return the result as Turtle.
///
/// XSLT 1.0 cannot percent-encode, so a link to a thread's page cannot be built in the
/// stylesheet; it arrives as data on the node it belongs to (`insp:page`, `insp:fragment`), the
/// way gonk's renderer hands its stylesheet everything a template would compute.
fn enrich(turtle: &str, view: &View) -> Result<String, Error> {
    use oxrdf::{Graph, Literal, NamedNode, NamedNodeRef, NamedOrBlankNodeRef, Triple};
    let parse_err = |e: String| Error::Endpoint(format!("the kernel's Turtle did not parse: {e}"));
    let mut graph = Graph::new();
    for triple in oxttl::TurtleParser::new().for_slice(turtle.as_bytes()) {
        graph.insert(&triple.map_err(|e| parse_err(e.to_string()))?);
    }
    let named = |iri: &str| NamedNode::new(iri).map_err(|e| Error::NotFound(format!("{iri}: {e}")));
    let ty = NamedNodeRef::new_unchecked(RDF_TYPE);
    let entry_class = named(&format!("{IK}CacheEntry"))?;
    let thread_class = named(&format!("{IK}GoldenThread"))?;
    let hangs_from = named(&format!("{IK}hangsFrom"))?;
    let page = named(&format!("{TERMS}page"))?;
    let fragment = named(&format!("{TERMS}fragment"))?;

    let subjects = |class: &NamedNode| -> Vec<NamedNode> {
        graph
            .subjects_for_predicate_object(ty, class)
            .filter_map(|s| match s {
                oxrdf::NamedOrBlankNodeRef::NamedNode(n) => Some(n.into_owned()),
                _ => None,
            })
            .collect()
    };
    let entries = subjects(&entry_class);
    let threads = subjects(&thread_class);

    // A detail keeps its focus and what is linked to it, and nothing else.
    let keep: Option<Vec<NamedNode>> = match view {
        View::List(_) => None,
        View::Entry(id) => {
            let focus = named(&format!("urn:ikigai:cache:entry:{id}"))?;
            if !entries.contains(&focus) {
                return Err(Error::NotFound(format!(
                    "the cache holds no entry {id} for you (it may have been evicted)"
                )));
            }
            let mut keep = vec![focus.clone()];
            keep.extend(
                graph
                    .objects_for_subject_predicate(&focus, &hangs_from)
                    .filter_map(|o| match o {
                        oxrdf::TermRef::NamedNode(n) => Some(n.into_owned()),
                        _ => None,
                    }),
            );
            Some(keep)
        }
        View::Thread(name) => {
            let focus = named(&format!("urn:ikigai:thread:{name}"))?;
            if !threads.contains(&focus) {
                return Err(Error::NotFound(format!(
                    "no entry you cached hangs from the thread {name}"
                )));
            }
            let mut keep = vec![focus.clone()];
            keep.extend(
                graph
                    .subjects_for_predicate_object(&hangs_from, &focus)
                    .filter_map(|s| match s {
                        oxrdf::NamedOrBlankNodeRef::NamedNode(n) => Some(n.into_owned()),
                        _ => None,
                    }),
            );
            Some(keep)
        }
    };

    let mut links: Vec<Triple> = Vec::new();
    let link = |node: &NamedNode, path: String| {
        [
            Triple::new(
                node.clone(),
                page.clone(),
                Literal::new_simple_literal(format!("/inspect/{path}")),
            ),
            Triple::new(
                node.clone(),
                fragment.clone(),
                Literal::new_simple_literal(format!("/inspect/fragment/{path}")),
            ),
        ]
    };
    for e in &entries {
        let id = e.as_str().trim_start_matches("urn:ikigai:cache:entry:");
        links.extend(link(
            e,
            format!("entry/{}", ikigai_fn::html::url_segment(id)),
        ));
    }
    for t in &threads {
        let name = t.as_str().trim_start_matches("urn:ikigai:thread:");
        links.extend(link(
            t,
            format!("thread/{}", ikigai_fn::html::url_segment(name)),
        ));
    }
    // The topology states order as RDF lists (a chain's layers, a space's doors), which a
    // stylesheet cannot walk; each node gets its place in resolution order instead, and each
    // door the space it belongs to.
    if matches!(view, View::List("topology")) {
        links.extend(resolution_order(&graph)?);
    }
    for t in links {
        graph.insert(&t);
    }

    let mut out = oxttl::TurtleSerializer::new()
        .with_prefix("ik", IK)
        .map_err(|e| Error::Endpoint(e.to_string()))?
        .with_prefix("insp", TERMS)
        .map_err(|e| Error::Endpoint(e.to_string()))?
        .for_writer(Vec::new());
    let mut triples: Vec<_> = graph.iter().collect();
    triples.sort_by_key(|t| t.to_string());
    for t in triples {
        let wanted = match (&keep, t.subject) {
            (None, _) => true,
            (Some(keep), NamedOrBlankNodeRef::NamedNode(s)) => keep.iter().any(|k| k.as_ref() == s),
            (Some(_), _) => false,
        };
        if wanted {
            out.serialize_triple(t)
                .map_err(|e| Error::Endpoint(e.to_string()))?;
        }
    }
    let bytes = out.finish().map_err(|e| Error::Endpoint(e.to_string()))?;
    String::from_utf8(bytes).map_err(|e| Error::Endpoint(e.to_string()))
}

/// Every node of a topology in the order resolution consults it, depth first from the root
/// chain: `insp:order` (zero-padded, so it sorts as text) on each, and `insp:space` (the space's
/// IRI) on each door.
fn resolution_order(graph: &oxrdf::Graph) -> Result<Vec<oxrdf::Triple>, Error> {
    use oxrdf::{Literal, NamedNode, NamedNodeRef, TermRef, Triple};
    let iri = |s: &str| NamedNodeRef::new_unchecked(s).into_owned();
    let (first, rest) = (
        iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#first"),
        iri("http://www.w3.org/1999/02/22-rdf-syntax-ns#rest"),
    );
    let lists = [iri(&format!("{IK}layers")), iri(&format!("{IK}doors"))];
    let inner = iri(&format!("{IK}space"));
    let doors = iri(&format!("{IK}doors"));
    let (order, in_space) = (iri(&format!("{TERMS}order")), iri(&format!("{TERMS}space")));
    let object = |s: &NamedNode, p: &NamedNode| -> Vec<NamedNode> {
        graph
            .objects_for_subject_predicate(s, p)
            .filter_map(|o| match o {
                TermRef::NamedNode(n) => Some(n.into_owned()),
                _ => None,
            })
            .collect()
    };
    let members = |head: &NamedNode| -> Vec<NamedNode> {
        let mut out = Vec::new();
        let mut at = Some(head.clone());
        while let Some(cell) = at.take() {
            if out.len() > 10_000 {
                break; // a cyclic list is the kernel's bug, not a reason to hang a page
            }
            out.extend(object(&cell, &first));
            at = object(&cell, &rest)
                .into_iter()
                .find(|n| n.as_str() != "http://www.w3.org/1999/02/22-rdf-syntax-ns#nil");
        }
        out
    };
    let mut seen: Vec<NamedNode> = Vec::new();
    let mut triples = Vec::new();
    let mut stack = vec![(iri("urn:ikigai:chain:root"), None::<NamedNode>)];
    while let Some((node, owner)) = stack.pop() {
        if seen.contains(&node) {
            continue;
        }
        triples.push(Triple::new(
            node.clone(),
            order.clone(),
            Literal::new_simple_literal(format!("{:05}", seen.len())),
        ));
        if let Some(space) = owner {
            triples.push(Triple::new(
                node.clone(),
                in_space.clone(),
                Literal::new_simple_literal(space.as_str()),
            ));
        }
        seen.push(node.clone());
        // Children pushed in reverse, so they pop in order.
        let mut children: Vec<(NamedNode, Option<NamedNode>)> = Vec::new();
        for p in &lists {
            for head in object(&node, p) {
                for m in members(&head) {
                    let owner = (p == &doors).then(|| node.clone());
                    children.push((m, owner));
                }
            }
        }
        for s in object(&node, &inner) {
            children.push((s, None));
        }
        stack.extend(children.into_iter().rev());
    }
    Ok(triples)
}

/// `graph:{view}`: the view's input, as RDF/XML.
///
/// Reads the kernel's own resource under the CALLER's capability (a viewer without
/// `urn:cap:kernel:inspect` is refused, by the kernel, for the kernel's reason), adds the links
/// a page needs, and transrepts the Turtle to RDF/XML through `urn:rdf:transrept`, the resource
/// that does that for everyone. The readouts are live, so the answer is too.
fn graph_endpoint() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(
        "inspect-graph",
        |inv: &Invocation<'_>| -> InvokeFuture<'_> {
            Box::pin(async move {
                let name = inv
                    .bindings
                    .get("view")
                    .ok_or_else(|| Error::MissingArgument("view".to_string()))?;
                let view = View::parse(name).ok_or_else(|| {
                    Error::NotFound(format!(
                        "{}: not a view (cache, threads, topology, catalog, entry:{{id}}, \
                     thread:{{name}})",
                        inv.request.target.as_str()
                    ))
                })?;
                let readout =
                    Request::new(Verb::Source, Iri::parse(view.source()).expect("an IRI"))
                        .with_arg("as", ArgRef::Inline(TURTLE.as_bytes().to_vec()));
                let turtle = inv.issue(readout).await?;
                let turtle = String::from_utf8(turtle.bytes)
                    .map_err(|_| Error::Endpoint("the readout is not UTF-8".to_string()))?;
                let enriched = enrich(&turtle, &view)?;
                let transrept = Request::new(
                    Verb::Source,
                    Iri::parse("urn:rdf:transrept").expect("an IRI"),
                )
                .with_arg("content", ArgRef::Inline(enriched.into_bytes()))
                .with_arg("as", ArgRef::Inline(RDF_XML.as_bytes().to_vec()));
                inv.issue(transrept).await
            })
        },
    )
    .with_description(
        Description::new("inspect-graph")
            .title("A kernel readout, ready for a stylesheet")
            .summary(
                "The Turtle face of one of the kernel's own readouts (the cache, the cut \
                 threads, the topology, the catalog) read under the caller's capability, with \
                 the links a page needs added as data, as RDF/XML. A detail (entry:{id}, \
                 thread:{name}) keeps only its focus and what is linked to it.",
            )
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .requires(CAP_INSPECT)
            .input(
                ArgSpec::new("view")
                    .summary("cache, threads, topology, catalog, entry:{id} or thread:{name}")
                    .class(XSD_STRING)
                    .binding(),
            )
            .output(RDF_XML),
    )
}

// --- titles ------------------------------------------------------------------------------------

/// `title:{name}`: each page's title, as text (the shell escapes it).
fn titles_endpoint() -> FnEndpoint {
    FnEndpoint::new("inspect-title", |inv: &Invocation<'_>| {
        let name = inv.bindings.get("name").unwrap_or_default();
        let title = match name {
            "cache" => "Cache".to_string(),
            "threads" => "Golden threads".to_string(),
            "topology" => "Topology".to_string(),
            "catalog" => "Catalog".to_string(),
            "scheduler" => "Scheduler".to_string(),
            "note" => "A note to watch".to_string(),
            "planets-json" => "Planets, from JSON".to_string(),
            "planets-csv" => "Planets, from CSV".to_string(),
            other => match View::parse(other) {
                Some(View::Entry(id)) => format!("Cache entry {id}"),
                Some(View::Thread(name)) => format!("Golden thread {name}"),
                _ => {
                    return Err(Error::NotFound(format!(
                        "{}: no such page",
                        inv.request.target.as_str()
                    )))
                }
            },
        };
        Ok(Representation::new(
            ReprType::new(TEXT),
            format!("{title} · Inspector").into_bytes(),
        )
        .cacheable())
    })
    .with_description(
        Description::new("inspect-title")
            .title("An inspector page's title")
            .summary("The title of one inspector page, as text.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("name")
                    .summary("the page")
                    .class(XSD_STRING)
                    .binding(),
            )
            .output(TEXT),
    )
}

// --- the one atom ------------------------------------------------------------------------------

/// `demo:note`: a note, the one thing in the inspector that holds state, so there is something
/// to watch: write it and every view cached over it is cut. Source reads it (cacheable: it
/// hangs from its own name, which a Sink cuts); Sink replaces it with `content`.
fn note_endpoint(note: Arc<Mutex<String>>) -> FnEndpoint {
    FnEndpoint::new("inspect-note", move |inv: &Invocation<'_>| {
        let mut held = note
            .lock()
            .map_err(|_| Error::Endpoint("the note is poisoned".into()))?;
        if inv.request.verb == Verb::Sink {
            // A browser form posts `content=…` urlencoded; a REPL or a script sinks the text.
            let raw = inv.inline_str("content")?;
            let form = inv
                .inline_str("content-type")
                .ok()
                .is_some_and(|t| t.starts_with("application/x-www-form-urlencoded"));
            let decoded;
            let text = if form {
                decoded = form_urlencoded::parse(raw.as_bytes())
                    .find(|(k, _)| k == "content")
                    .map(|(_, v)| v.into_owned())
                    .ok_or_else(|| Error::MissingArgument("content".to_string()))?;
                decoded.as_str()
            } else {
                raw
            };
            if text.len() > 4096 {
                return Err(Error::InvalidArgument {
                    name: "content".to_string(),
                    detail: format!("{} bytes; a note holds at most 4096", text.len()),
                });
            }
            *held = text.to_string();
            return Ok(Representation::new(
                ReprType::new(TEXT),
                b"written".to_vec(),
            ));
        }
        Ok(Representation::new(ReprType::new(TEXT), held.clone().into_bytes()).cacheable())
    })
    .with_description(
        Description::new("inspect-note")
            .title("A note to watch")
            .summary(
                "A short text you can write. Reading it is cached; writing it cuts its thread, \
                 so every view over it recomputes, and the inspector's cache view shows the cut.",
            )
            .action(ActionSpec::new(Verb::Source).output(TEXT))
            .action(
                ActionSpec::new(Verb::Sink)
                    .requires(CAP_NOTE)
                    .input(
                        ArgSpec::new("content")
                            .summary("the note's new text, at most 4096 bytes")
                            .class(XSD_STRING),
                    )
                    .output(TEXT),
            )
            .verb(Verb::Meta),
    )
}

// --- the space ---------------------------------------------------------------------------------

fn template(pattern: &str) -> UriTemplate {
    UriTemplate::parse(pattern).expect("a valid URI template")
}

fn composed(template_iri: &str, name: &str) -> ikigai_fn::ComposeOver {
    ikigai_fn::compose_over(Iri::parse(template_iri).expect("an IRI")).named(name)
}

/// The inspector's own names: the artifacts, the graph, the fragments, the titles, the note and
/// its card. Pair it with [`ikigai_ui::space`], [`ikigai_ui::pages`]`(`[`FAMILY`]`, `[`BASE`]`)`,
/// `urn:xslt:transform` and `urn:rdf:transrept`; [`kernel`] does.
pub fn space() -> EndpointSpace {
    let mut space = EndpointSpace::new();
    for a in &ARTIFACTS {
        space = space.bind(Exact::new(a.iri), artifact_endpoint(a));
    }
    // A detail's stylesheet is named after the view, in either spelling (see the crate doc).
    for (kind, iri) in [
        ("entry", "urn:iki:tutorial:inspect:xsl:entry"),
        ("thread", "urn:iki:tutorial:inspect:xsl:thread"),
    ] {
        for sep in [":", "%3A"] {
            space = space.bind(
                template(&format!("{BASE}xsl:{kind}{sep}{{key}}")),
                artifact_endpoint(by_name(iri)),
            );
        }
    }
    // The data views, before `fragment:{view}`, so `fragment:planets-json` is theirs.
    let space = data::space(space);
    // ANCHOR: views
    space
        .bind(template(&format!("{BASE}graph:{{view}}")), graph_endpoint())
        .bind(
            template(&format!("{BASE}title:{{name}}")),
            titles_endpoint(),
        )
        .bind(
            Exact::new(format!("{BASE}fragment:scheduler")),
            composed(
                "urn:iki:tutorial:inspect:template:scheduler",
                "inspect-fragment-scheduler",
            ),
        )
        .bind(
            Exact::new(format!("{BASE}fragment:note")),
            composed(
                "urn:iki:tutorial:inspect:template:note",
                "inspect-fragment-note",
            ),
        )
        .bind(
            template(&format!("{BASE}fragment:{{view}}")),
            composed("urn:iki:tutorial:inspect:template:view", "inspect-fragment"),
        )
        // ANCHOR_END: views
        .bind(
            Exact::new(format!("{BASE}demo:note")),
            note_endpoint(Arc::new(Mutex::new(
                "Write me, then watch the cache.".to_string(),
            ))),
        )
        .bind(
            Exact::new(format!("{BASE}demo:card")),
            composed(
                "urn:iki:tutorial:inspect:template:card",
                "inspect-demo-card",
            ),
        )
}

// ANCHOR: kernel
/// A kernel that serves the inspector: ikigai-ui's shared artifacts and the inspector's pages,
/// its own names, XSLT and RDF transreption, with the Turtle Meta renderer (the catalog view
/// needs it). `extra` comes FIRST, so a host's directory binding at [`DIR`] (or anything else it
/// adds) is consulted before the rest.
pub fn kernel(extra: Vec<Arc<dyn Space>>) -> Kernel {
    let mut layers = extra;
    layers.push(Arc::new(ikigai_ui::space()));
    layers.push(Arc::new(
        ikigai_ui::pages(FAMILY, BASE).expect("a valid family and base"),
    ));
    layers.push(Arc::new(space()));
    layers.push(Arc::new(ikigai_xslt::space()));
    layers.push(Arc::new(ikigai_rdf::space()));
    layers.push(Arc::new(ikigai_json::space()));
    Kernel::with_meta_renderer(Arc::new(Fallback::new(layers)), Arc::new(TurtleRenderer))
}
// ANCHOR_END: kernel

/// The scopes a local viewer of the inspector holds: inspect the kernel, hear its cuts (the push
/// stream, UI conventions D9) and write the note. The kernel enforces each one; a host that
/// grants less shows less.
pub const VIEWER_SCOPES: [&str; 3] = [CAP_INSPECT, ikigai_core::CAP_LISTEN, CAP_NOTE];
