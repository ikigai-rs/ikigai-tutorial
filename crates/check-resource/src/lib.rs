//! The code taught by the book's chapter **Checking a resource**.
//!
//! Any resource can be checked, because transreption gives it a Turtle face. The host here
//! holds two players of tic-tac-toe, each answering N-Triples and readable only with a
//! scope, a transreptor from N-Triples to Turtle, and the closed SHACL shapes a player must
//! fit. Beside them it mounts two published spaces:
//!
//! * `ikigai_toolbox::space()`: the RDF Toolbox's names. `urn:toolbox:input:{iri}` is a
//!   resource's Turtle face as text, read under the CALLER's capability, and
//!   `urn:toolbox:check:{iri}` is the page that checks it. The page and its WebAssembly
//!   engine are a build a host binds; this crate binds none, so it uses the input and not
//!   the page.
//! * `ikigai_shacl::space()`: `urn:shacl:validate`, the shapes check on the host, which
//!   stands in here for the engine the page runs in a browser.
//!
//! [`check`] composes the two the way the page does: both inputs read as Turtle faces under
//! one caller's capability, then the data checked against the shapes.
//!
//! Read the book: `mdbook serve books/ikigai --open`

use std::sync::Arc;

use ikigai_core::{
    ArgRef, ArgSpec, Capability, Description, EndpointSpace, Error, Exact, Fallback, FnEndpoint,
    Invocation, Iri, Kernel, ReprType, Representation, Request, Result, Space, Verb,
};
use ikigai_shacl::Report;

/// The player who plays X.
pub const PLAYER_X: &str = "urn:iki:tutorial:check:player:x";
/// The player who plays O.
pub const PLAYER_O: &str = "urn:iki:tutorial:check:player:o";
/// The shapes a player must fit.
pub const SHAPES: &str = "urn:iki:tutorial:check:shapes";
/// The transreptor from N-Triples to Turtle.
pub const TO_TURTLE: &str = "urn:iki:tutorial:check:to-turtle";
/// The scope reading a player requires.
pub const READ: &str = "urn:cap:tutorial:players:read";

const N_TRIPLES: &str = "application/n-triples";
const TURTLE: &str = "text/turtle";

// ANCHOR: shapes
/// One closed node shape: a player has exactly one name and exactly one mark, X or O,
/// and nothing else but its type.
pub const PLAYER_SHAPES: &str = r#"@prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix sh:  <http://www.w3.org/ns/shacl#> .
@prefix xsd: <http://www.w3.org/2001/XMLSchema#> .
@prefix ttt: <http://example.org/ttt#> .

ttt:PlayerShape a sh:NodeShape ;
    sh:targetClass ttt:Player ;
    sh:closed true ;
    sh:ignoredProperties ( rdf:type ) ;
    sh:property [ sh:path ttt:name ; sh:datatype xsd:string ; sh:minCount 1 ; sh:maxCount 1 ] ;
    sh:property [ sh:path ttt:mark ; sh:in ( "X" "O" ) ; sh:minCount 1 ; sh:maxCount 1 ] .
"#;
// ANCHOR_END: shapes

// ANCHOR: players
/// The two players, as N-Triples. O carries one property the shapes do not name.
const X_TRIPLES: &str = r#"<urn:iki:tutorial:check:player:x> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/ttt#Player> .
<urn:iki:tutorial:check:player:x> <http://example.org/ttt#name> "Ada" .
<urn:iki:tutorial:check:player:x> <http://example.org/ttt#mark> "X" .
"#;
const O_TRIPLES: &str = r#"<urn:iki:tutorial:check:player:o> <http://www.w3.org/1999/02/22-rdf-syntax-ns#type> <http://example.org/ttt#Player> .
<urn:iki:tutorial:check:player:o> <http://example.org/ttt#name> "Grace" .
<urn:iki:tutorial:check:player:o> <http://example.org/ttt#mark> "O" .
<urn:iki:tutorial:check:player:o> <http://example.org/ttt#wins> "3" .
"#;

/// A player: a graph in N-Triples, readable only by a caller holding [`READ`]. Nothing in
/// it knows Turtle exists.
fn player(id: &'static str, triples: &'static str) -> FnEndpoint {
    FnEndpoint::new(id, move |_: &Invocation<'_>| {
        Ok(Representation::new(ReprType::new(N_TRIPLES), triples.as_bytes().to_vec()).cacheable())
    })
    .with_description(
        Description::new(id)
            .title("A player")
            .summary("One player of tic-tac-toe, as an RDF graph in N-Triples.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(N_TRIPLES)
            .requires(READ),
    )
}
// ANCHOR_END: players

// ANCHOR: to_turtle
/// A transreptor from N-Triples to Turtle: the same triples, written with prefixes.
/// Lossless (every triple survives), so it is declared with no `.lossy()`.
pub fn to_turtle() -> FnEndpoint {
    FnEndpoint::new("to-turtle", |inv: &Invocation<'_>| {
        let content = inv.inline_str("content")?;
        let invalid = |detail: String| Error::InvalidArgument {
            name: "content".into(),
            detail,
        };
        let mut writer = oxttl::TurtleSerializer::new()
            .with_prefix("ttt", "http://example.org/ttt#")
            .map_err(|e| Error::Endpoint(e.to_string()))?
            .for_writer(Vec::new());
        for triple in oxttl::NTriplesParser::new().for_slice(content.as_bytes()) {
            let triple = triple.map_err(|e| invalid(e.to_string()))?;
            writer
                .serialize_triple(&triple)
                .map_err(|e| Error::Endpoint(e.to_string()))?;
        }
        let turtle = writer
            .finish()
            .map_err(|e| Error::Endpoint(e.to_string()))?;
        Ok(Representation::new(ReprType::new(TURTLE), turtle).cacheable())
    })
    .with_description(
        Description::new("to-turtle")
            .title("N-Triples to Turtle")
            .summary("The same triples, written as Turtle.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(ArgSpec::new("content").summary("the N-Triples document"))
            .input(
                ArgSpec::new("as")
                    .summary("the target type; only text/turtle")
                    .optional(),
            )
            .output(TURTLE)
            .transreptor([N_TRIPLES], [TURTLE]),
    )
}
// ANCHOR_END: to_turtle

/// The shapes, as a resource anyone may read.
fn shapes() -> FnEndpoint {
    FnEndpoint::new("player-shapes", |_: &Invocation<'_>| {
        Ok(
            Representation::new(ReprType::new(TURTLE), PLAYER_SHAPES.as_bytes().to_vec())
                .cacheable(),
        )
    })
    .with_description(
        Description::new("player-shapes")
            .title("The player shapes")
            .summary("A closed SHACL node shape for a player of tic-tac-toe.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TURTLE),
    )
}

// ANCHOR: space
/// The chapter's host: its own doors, then the toolbox's names, then the shapes check.
pub fn space() -> Fallback {
    let ours = EndpointSpace::new()
        .bind(Exact::new(PLAYER_X), player("player-x", X_TRIPLES))
        .bind(Exact::new(PLAYER_O), player("player-o", O_TRIPLES))
        .bind(Exact::new(SHAPES), shapes())
        .bind(Exact::new(TO_TURTLE), to_turtle());
    Fallback::new(vec![
        Arc::new(ours) as Arc<dyn Space>,
        Arc::new(ikigai_toolbox::space()) as Arc<dyn Space>,
        Arc::new(ikigai_shacl::space()) as Arc<dyn Space>,
    ])
}
// ANCHOR_END: space

/// A kernel over [`space`].
pub fn kernel() -> Kernel {
    Kernel::new(Arc::new(space()))
}

// ANCHOR: face
/// A resource's Turtle face, as the toolbox page reads it: `urn:toolbox:input:{iri}`,
/// issued under the caller's own capability.
pub async fn turtle_face(kernel: &Kernel, iri: &str, caller: &Capability) -> Result<String> {
    let input = Iri::parse(format!("urn:toolbox:input:{iri}"))
        .map_err(|e| Error::Endpoint(format!("`{iri}` makes no input name: {e}")))?;
    let face = kernel
        .issue(Request::new(Verb::Source, input), caller)
        .await?;
    String::from_utf8(face.bytes).map_err(|e| Error::Endpoint(format!("not UTF-8: {e}")))
}
// ANCHOR_END: face

// ANCHOR: check
/// CHECK `iri` against the shapes at `shapes`: both read as Turtle faces under `caller`'s
/// capability, the way the check page reads its two inputs, then `urn:shacl:validate`.
pub async fn check(
    kernel: &Kernel,
    iri: &str,
    shapes: &str,
    caller: &Capability,
) -> Result<Report> {
    let data = turtle_face(kernel, iri, caller).await?;
    let shapes = turtle_face(kernel, shapes, caller).await?;
    let validate = Request::new(
        Verb::Source,
        Iri::parse("urn:shacl:validate").expect("an IRI"),
    )
    .with_arg("data", ArgRef::Inline(data.into_bytes()))
    .with_arg("shapes", ArgRef::Inline(shapes.into_bytes()))
    .with_arg("as", ArgRef::Inline(b"application/json".to_vec()));
    let report = kernel.issue(validate, caller).await?;
    serde_json::from_slice(&report.bytes).map_err(|e| Error::Endpoint(format!("report: {e}")))
}
// ANCHOR_END: check
