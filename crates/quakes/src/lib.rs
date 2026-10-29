//! The recorded window the ikigai Book's part "Exploring without Rust" runs on.
//!
//! | name | kind | what it is |
//! |---|---|---|
//! | `urn:file:quakes-2024-01-01.geojson` | atom | one week of USGS earthquakes, M4.5 and up, as GeoJSON |
//! | `urn:quakes:template:headline` | atom | a one-line template whose `$j{…}` markers ask the week two questions |
//! | `urn:file:quakes.context.jsonld` | atom | the JSON-LD context that maps the feed onto the quakes vocabulary (`books/ikigai/ns/`) |
//! | `urn:quakes:template:graph` | atom | a one-line template that splices the context into the week: the week as JSON-LD |
//! | `urn:quakes:graph` | compose | that template, filled: the graph "Giving the data names" asks its questions of |
//! | `urn:quakes:question:{count,biggest,by-magtype}` | atom | the chapter's three SPARQL questions, as `queries/*.rq` holds them |
//! | `urn:file:quakes-2024-01-08.geojson` | atom | the week after, recorded the same way: 105 events |
//! | `urn:quakes:template:graph:2024-01-08`, `urn:quakes:graph:2024-01-08` | atom, compose | the week after as JSON-LD, under the same context |
//! | `urn:quakes:question:{strong,strongest-first,week-after}` | atom | "Composites, one layer at a time": a filter, a sort and a join |
//! | `urn:quakes:strong` | layer | the `strong` question asked of `urn:quakes:graph`: the filter's answer, a graph at a name |
//!
//! The part works at a shell, where the CLI's file workspace (`ikigai --files <dir>`) serves a
//! copy of the same file under the same name, and in the page, where this crate serves it. So a
//! cell and a command name one resource. The page has no file system: the window is compiled in
//! with `include_bytes!`, it answers only `Source` and `Meta`, and it never changes, so its
//! answer is cacheable and hangs from the thread of its own name.
//!
//! Everything a chapter asks of it (a count, the largest, a grouping) is a composition of
//! published resources over these atoms: `urn:json:query` (ikigai-json) for jq, ikigai-fn's
//! `$j{…}` and `$r{…}` for a template, `urn:sparql:select` (ikigai-sparql) over the graph. Our
//! own names live under `urn:quakes:`, and our own terms under `https://ikigai-rs.dev/ns/quakes#`
//! (`books/ikigai/ns/quakes.ttl`), never under USGS's domain. USGS's own names (each event's
//! page) stay theirs: the context makes them the earthquakes' IRIs.
//!
//! The context is a second `urn:file:` name, for the same reason as the window: at a shell the
//! reader downloads the same file into the same workspace. `urn:quakes:graph` is the one name
//! here with no file behind it at a shell, where the reader keeps the filled template in a file
//! of their own (`quakes.jsonld`): a request with arguments cannot be named as one resource
//! (named requests, ledger #2447), and `graph=` takes a name.

use ikigai_core::{
    ActionSpec, ArgRef, AsyncFnEndpoint, Description, EndpointSpace, Exact, FnEndpoint, Invocation,
    Iri, ReprType, Representation, Request, Verb,
};

// ANCHOR: names
/// The recorded window's name: the same `urn:file:` name the CLI's file workspace gives the
/// copy a reader saves there.
pub const WINDOW: &str = "urn:file:quakes-2024-01-01.geojson";

/// GeoJSON's registered media type (RFC 7946). A `+json` type, so `urn:json:query` reads it as
/// JSON without a transreptor.
pub const GEOJSON: &str = "application/geo+json";
// ANCHOR_END: names

/// The window's bytes, exactly as USGS served them (see this crate's README for the source URL,
/// the recording date and the digest).
pub const WINDOW_BYTES: &[u8] = include_bytes!("../quakes-2024-01-01.geojson");

/// How many events the window holds: what `.features|length` answers.
pub const EVENTS: usize = 178;

/// The headline template "Reshaping JSON with jq" fills with `urn:iki:fn:compose`. A name of our
/// own, under `urn:quakes:`: at a shell, the reader writes the same text to a file of their own.
pub const HEADLINE: &str = "urn:quakes:template:headline";

/// The headline's text, as `templates/headline.txt` holds it.
pub const HEADLINE_TEXT: &str = include_str!("../templates/headline.txt");

/// The context's name: the `urn:file:` name the CLI's file workspace gives the copy a reader
/// downloads beside the window.
pub const CONTEXT: &str = "urn:file:quakes.context.jsonld";

/// JSON-LD's registered media type.
pub const JSONLD: &str = "application/ld+json";

/// The context's bytes: the file `https://ikigai-rs.dev/ns/quakes.context.jsonld` serves.
pub const CONTEXT_TEXT: &str = include_str!("../../../books/ikigai/ns/quakes.context.jsonld");

/// The vocabulary the context maps onto: the file `https://ikigai-rs.dev/ns/quakes` serves.
pub const VOCABULARY_TEXT: &str = include_str!("../../../books/ikigai/ns/quakes.ttl");

/// The template that splices the context into the window. A name of our own, under
/// `urn:quakes:`: at a shell, the reader writes the same text to a file of their own.
pub const GRAPH_TEMPLATE: &str = "urn:quakes:template:graph";

/// The graph template's text, as `templates/graph.txt` holds it.
pub const GRAPH_TEMPLATE_TEXT: &str = include_str!("../templates/graph.txt");

/// The graph template, filled: the window as JSON-LD, which `urn:sparql:select` loads by name.
pub const GRAPH: &str = "urn:quakes:graph";

/// The second recorded window: the week after [`WINDOW`], under the same kind of name.
pub const WINDOW_2: &str = "urn:file:quakes-2024-01-08.geojson";

/// The second window's bytes, exactly as USGS served them (see this crate's README).
pub const WINDOW_2_BYTES: &[u8] = include_bytes!("../quakes-2024-01-08.geojson");

/// How many events the second window holds.
pub const EVENTS_2: usize = 105;

/// The graph template for [`WINDOW_2`]: [`GRAPH_TEMPLATE`]'s text with the other week in it.
pub const GRAPH_TEMPLATE_2: &str = "urn:quakes:template:graph:2024-01-08";

/// The second graph template's text, as `templates/graph-2024-01-08.txt` holds it.
pub const GRAPH_TEMPLATE_2_TEXT: &str = include_str!("../templates/graph-2024-01-08.txt");

/// The second graph template, filled: the week after as JSON-LD.
pub const GRAPH_2: &str = "urn:quakes:graph:2024-01-08";

/// The filter's answer at a name: the `strong` question asked of [`GRAPH`], a graph that the
/// next layer reads by this name ("Composites, one layer at a time").
pub const STRONG: &str = "urn:quakes:strong";

/// The question [`STRONG`] asks.
pub const STRONG_QUESTION: &str = "urn:quakes:question:strong";

/// The SPARQL questions "Giving the data names" and "Composites, one layer at a time" ask:
/// each name, and its text as `queries/<name>.rq` holds it.
pub const QUESTIONS: [(&str, &str); 6] = [
    (
        "urn:quakes:question:count",
        include_str!("../queries/count.rq"),
    ),
    (
        "urn:quakes:question:biggest",
        include_str!("../queries/biggest.rq"),
    ),
    (
        "urn:quakes:question:by-magtype",
        include_str!("../queries/by-magtype.rq"),
    ),
    (STRONG_QUESTION, include_str!("../queries/strong.rq")),
    (
        "urn:quakes:question:strongest-first",
        include_str!("../queries/strongest-first.rq"),
    ),
    (
        "urn:quakes:question:week-after",
        include_str!("../queries/week-after.rq"),
    ),
];

/// SPARQL's registered media type for a query.
pub const SPARQL_QUERY: &str = "application/sparql-query";

// ANCHOR: space
/// The window, bound at [`WINDOW`]: an atom that answers its bytes, cacheably; the headline
/// template, at [`HEADLINE`]; the context, the graph template, the graph and the questions.
pub fn space() -> EndpointSpace {
    let mut space = EndpointSpace::new()
        .bind(Exact::new(WINDOW), window())
        .bind(Exact::new(HEADLINE), headline())
        .bind(Exact::new(CONTEXT), context())
        .bind(Exact::new(GRAPH_TEMPLATE), graph_template())
        .bind(Exact::new(GRAPH), graph())
        .bind(Exact::new(WINDOW_2), window_2())
        .bind(Exact::new(GRAPH_TEMPLATE_2), graph_template_2())
        .bind(Exact::new(GRAPH_2), graph_2())
        .bind(
            Exact::new(STRONG),
            layer("quakes-strong", STRONG_QUESTION, GRAPH),
        );
    for (name, text) in QUESTIONS {
        space = space.bind(Exact::new(name), question(name, text));
    }
    space
}
// ANCHOR_END: space

/// The atom behind [`WINDOW`].
pub fn window() -> FnEndpoint {
    FnEndpoint::new("quakes-window", |_: &Invocation<'_>| {
        Ok(Representation::new(ReprType::new(GEOJSON), WINDOW_BYTES.to_vec()).cacheable())
    })
    .with_description(
        Description::new("quakes-window")
            .title("A recorded week of earthquakes")
            .summary(
                "USGS earthquakes of M4.5 and up from 2024-01-01 to 2024-01-08, 178 events, as \
                 GeoJSON, recorded once and never changed: the window \"Exploring without \
                 Rust\" runs on.",
            )
            .action(ActionSpec::new(Verb::Source).output(GEOJSON))
            .verb(Verb::Meta),
    )
}

/// The atom behind [`WINDOW_2`].
pub fn window_2() -> FnEndpoint {
    FnEndpoint::new("quakes-window-2024-01-08", |_: &Invocation<'_>| {
        Ok(Representation::new(ReprType::new(GEOJSON), WINDOW_2_BYTES.to_vec()).cacheable())
    })
    .with_description(
        Description::new("quakes-window-2024-01-08")
            .title("The week after, recorded")
            .summary(
                "USGS earthquakes of M4.5 and up from 2024-01-08 to 2024-01-15, 105 events, as \
                 GeoJSON, recorded once and never changed: the second window \"Composites, one \
                 layer at a time\" joins.",
            )
            .action(ActionSpec::new(Verb::Source).output(GEOJSON))
            .verb(Verb::Meta),
    )
}

/// The atom behind [`HEADLINE`]: the template as it is written, markers and all.
pub fn headline() -> FnEndpoint {
    FnEndpoint::new("quakes-headline", |_: &Invocation<'_>| {
        Ok(Representation::new(
            ReprType::new("text/plain"),
            HEADLINE_TEXT.as_bytes().to_vec(),
        )
        .cacheable())
    })
    .with_description(
        Description::new("quakes-headline")
            .title("The week's headline, as a template")
            .summary(
                "One line with two `$j{…}` markers over the recorded week: how many \
                 earthquakes, and the largest. Fill it with urn:iki:fn:compose.",
            )
            .action(ActionSpec::new(Verb::Source).output("text/plain"))
            .verb(Verb::Meta),
    )
}

/// An atom that answers `text` as `media`, cacheably, described by `title` and `summary`.
fn atom(
    id: &str,
    media: &'static str,
    text: &'static str,
    title: &str,
    summary: &str,
) -> FnEndpoint {
    FnEndpoint::new(id, move |_: &Invocation<'_>| {
        Ok(Representation::new(ReprType::new(media), text.as_bytes().to_vec()).cacheable())
    })
    .with_description(
        Description::new(id)
            .title(title)
            .summary(summary)
            .action(ActionSpec::new(Verb::Source).output(media))
            .verb(Verb::Meta),
    )
}

/// The atom behind [`CONTEXT`].
pub fn context() -> FnEndpoint {
    atom(
        "quakes-context",
        JSONLD,
        CONTEXT_TEXT,
        "The quakes context",
        "A JSON-LD context that maps USGS's earthquake GeoJSON onto the quakes vocabulary          (https://ikigai-rs.dev/ns/quakes#), the GeoJSON-LD vocabulary and schema.org, and          names each earthquake by its USGS event page.",
    )
}

/// The atom behind [`GRAPH_TEMPLATE`]: the template as it is written, markers and all. Typed
/// [`JSONLD`], because `compose` answers in its template's type and the filled template is
/// JSON-LD, which is what `urn:sparql:select` must be told to load it as. At a shell the same
/// text sits in a `.txt` file and the filled copy is written to `quakes.jsonld`, whose name
/// gives it the type.
pub fn graph_template() -> FnEndpoint {
    atom(
        "quakes-graph-template",
        JSONLD,
        GRAPH_TEMPLATE_TEXT,
        "The week as JSON-LD, as a template",
        "One line with two `$r{…}` markers: the context, picked out of its file by jq, and the          recorded week. Fill it with urn:iki:fn:compose.",
    )
}

/// The compose behind [`GRAPH`]: [`GRAPH_TEMPLATE`], filled whenever it is read, so it hangs
/// from the context's thread and the window's.
pub fn graph() -> ikigai_fn::ComposeOver {
    ikigai_fn::compose_over(Iri::parse(GRAPH_TEMPLATE).expect("a valid IRI")).named("quakes-graph")
}

/// The atom behind [`GRAPH_TEMPLATE_2`], typed as [`graph_template`] is.
pub fn graph_template_2() -> FnEndpoint {
    atom(
        "quakes-graph-template-2024-01-08",
        JSONLD,
        GRAPH_TEMPLATE_2_TEXT,
        "The week after as JSON-LD, as a template",
        "The graph template with the second recorded week in it. Fill it with \
         urn:iki:fn:compose.",
    )
}

/// The compose behind [`GRAPH_2`].
pub fn graph_2() -> ikigai_fn::ComposeOver {
    ikigai_fn::compose_over(Iri::parse(GRAPH_TEMPLATE_2).expect("a valid IRI"))
        .named("quakes-graph-2024-01-08")
}

/// A layer: the CONSTRUCT question at `question`, asked of the graph at `over`, answered as
/// Turtle. Both are read through the kernel whenever the layer is, so the answer is cached
/// as the SPARQL answer is and hangs from every thread that answer hangs from: the question's,
/// the graph's, and through the graph the context's and the window's.
///
/// It is the page's stand-in for what the CLI cannot do yet: name a request with arguments
/// (named requests, ledger #2447). At a shell, the reader writes the same answer to a file.
pub fn layer(id: &'static str, question: &'static str, over: &'static str) -> AsyncFnEndpoint {
    AsyncFnEndpoint::new(id, move |inv| {
        Box::pin(async move {
            let text = inv
                .source(&Iri::parse(question).expect("a valid IRI"))
                .await?;
            let request = Request::new(
                Verb::Source,
                Iri::parse("urn:sparql:construct").expect("a valid IRI"),
            )
            .with_arg("query", ArgRef::Inline(text.bytes.to_vec()))
            .with_arg("graph", ArgRef::Inline(over.as_bytes().to_vec()))
            .with_arg("as", ArgRef::Inline(b"text/turtle".to_vec()));
            inv.issue(request).await
        })
    })
    .with_description(
        Description::new(id)
            .title("A layer: a question's answer at a name")
            .summary(format!(
                "The CONSTRUCT question {question}, asked of {over} and answered as Turtle: a \
                 graph the next layer reads by this name."
            ))
            .action(ActionSpec::new(Verb::Source).output("text/turtle"))
            .verb(Verb::Meta),
    )
}

/// The atom behind one of [`QUESTIONS`].
fn question(name: &str, text: &'static str) -> FnEndpoint {
    let short = name.rsplit(':').next().expect("a name has a last segment");
    atom(
        &format!("quakes-question-{short}"),
        SPARQL_QUERY,
        text,
        &format!("The {short} question"),
        "A SPARQL query over the week as a graph (urn:quakes:graph). Pipe it into          urn:sparql:select.",
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use ikigai_core::{Capability, Iri, Kernel, Request};
    use std::sync::Arc;

    fn read() -> Representation {
        let kernel = Kernel::new(Arc::new(space()));
        let request = Request::new(Verb::Source, Iri::parse(WINDOW).expect("an IRI"));
        block_on(kernel.issue(request, &Capability::root())).expect("the window resolves")
    }

    #[test]
    fn the_window_answers_its_committed_bytes_as_geojson() {
        let answer = read();
        assert_eq!(answer.repr_type.media_type, GEOJSON);
        assert_eq!(answer.bytes, WINDOW_BYTES);
    }

    #[test]
    fn the_window_is_the_week_the_readme_says() {
        let doc: serde_json::Value = serde_json::from_slice(WINDOW_BYTES).expect("JSON");
        assert_eq!(doc["features"].as_array().expect("features").len(), EVENTS);
        assert_eq!(doc["metadata"]["count"], EVENTS);
        assert_eq!(
            doc["metadata"]["url"],
            "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&starttime=2024-01-01&endtime=2024-01-08&minmagnitude=4.5"
        );
        let biggest = doc["features"]
            .as_array()
            .expect("features")
            .iter()
            .max_by(|a, b| {
                let mag = |f: &serde_json::Value| f["properties"]["mag"].as_f64().expect("a mag");
                mag(a).total_cmp(&mag(b))
            })
            .expect("one event at least");
        assert_eq!(biggest["id"], "us6000m0xl");
        assert_eq!(biggest["properties"]["mag"], 7.5);
    }

    #[test]
    fn the_second_window_is_the_week_after_as_the_readme_says() {
        let doc: serde_json::Value = serde_json::from_slice(WINDOW_2_BYTES).expect("JSON");
        assert_eq!(
            doc["features"].as_array().expect("features").len(),
            EVENTS_2
        );
        assert_eq!(doc["metadata"]["count"], EVENTS_2);
        assert_eq!(
            doc["metadata"]["url"],
            "https://earthquake.usgs.gov/fdsnws/event/1/query?format=geojson&starttime=2024-01-08&endtime=2024-01-15&minmagnitude=4.5"
        );
        // The second graph template is the first with the other week in it.
        assert_eq!(
            GRAPH_TEMPLATE_2_TEXT,
            GRAPH_TEMPLATE_TEXT.replace(WINDOW, WINDOW_2)
        );
    }
}
