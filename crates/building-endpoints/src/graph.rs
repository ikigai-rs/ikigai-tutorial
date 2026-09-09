//! **The graph face** — the catalog is RDF, so it is queryable.
//!
//! Nothing here is an endpoint. It is the *use* of two that already exist:
//! `urn:kernel:catalog`, which is one Turtle graph over every binding, and
//! `urn:sparql:select` from the published `ikigai-sparql` crate, which resolves the
//! graph named in its `graph` argument **through the kernel** and runs a query over it.
//! The result is a resolution like any other — a representation with a type you chose.

use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request, Result, Verb};

// ANCHOR: query
/// The `ik:id` of every endpoint in `kernel`'s catalog that takes an argument of RDF
/// class `class` — "what can I do with a string?", asked of the graph.
///
/// `graph=urn:kernel:catalog` is the whole trick: the SPARQL endpoint does not hold a
/// store, it *resolves* the graph you name. Any resource that answers as Turtle would do.
///
/// The property path `ik:input | ik:action/ik:input` matters. A single-verb endpoint's
/// inputs hang off the endpoint node; an endpoint with per-verb `ActionSpec`s declares
/// each verb's inputs on that verb's *action* node. Ask only the first and every
/// multi-verb endpoint is invisible to the question.
pub async fn endpoints_taking(kernel: &Kernel, class: &str) -> Result<Vec<String>> {
    let query = format!(
        "PREFIX ik: <https://ikigai-rs.dev/ns#>
         SELECT DISTINCT ?id WHERE {{
           ?endpoint a ik:Endpoint ; ik:id ?id ; (ik:input | ik:action/ik:input) ?input .
           ?input ik:class <{class}> .
         }} ORDER BY ?id"
    );
    let request = Request::new(
        Verb::Source,
        Iri::parse("urn:sparql:select").expect("a constant IRI"),
    )
    .with_arg("graph", ArgRef::Inline(b"urn:kernel:catalog".to_vec()))
    .with_arg("query", ArgRef::Inline(query.into_bytes()))
    .with_arg("as", ArgRef::Inline(b"text/csv".to_vec()));

    let repr = kernel.issue(request, &Capability::root()).await?;
    let csv = String::from_utf8_lossy(&repr.bytes);
    // One column, so each line after the header is an id.
    Ok(csv
        .lines()
        .skip(1)
        .map(|line| line.trim().to_string())
        .collect())
}
// ANCHOR_END: query

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_in;
    use futures::executor::block_on;

    // ANCHOR: select
    /// One SELECT over the reader's own endpoints.
    #[test]
    fn the_endpoints_that_take_a_string_are_the_ones_that_declared_it() {
        let kernel = kernel_in(None);
        let ids = block_on(endpoints_taking(
            &kernel,
            "http://www.w3.org/2001/XMLSchema#string",
        ))
        .expect("the catalog is a graph and the graph answers");

        // Declared `.class(xsd:string)` on an input, and found by that declaration.
        assert!(ids.contains(&"camel-case".to_string()), "{ids:?}");
        assert!(ids.contains(&"title".to_string()), "{ids:?}");
        // Declared a different class, and correctly absent.
        assert!(!ids.contains(&"counter".to_string()), "{ids:?}");
        // Declared no class at all: an input with no `class` is invisible to this
        // question, which is the argument for declaring one.
        assert!(!ids.contains(&"markdown".to_string()), "{ids:?}");
    }
    // ANCHOR_END: select

    #[test]
    fn the_answer_comes_back_in_the_type_asked_for() {
        let kernel = kernel_in(None);
        let request = Request::new(Verb::Source, Iri::parse("urn:sparql:select").expect("iri"))
            .with_arg("graph", ArgRef::Inline(b"urn:kernel:catalog".to_vec()))
            .with_arg(
                "query",
                ArgRef::Inline(b"SELECT (COUNT(*) AS ?n) WHERE { ?s ?p ?o }".to_vec()),
            )
            .with_arg("as", ArgRef::Inline(b"text/csv".to_vec()));
        let repr = block_on(kernel.issue(request, &Capability::root())).expect("resolves");
        // (`ikigai-sparql` writes the charset into the media type string itself rather
        // than as a parameter, so this is a prefix check, not an equality.)
        assert!(
            repr.repr_type.media_type.starts_with("text/csv"),
            "{}",
            repr.repr_type.media_type
        );
        let n: usize = String::from_utf8_lossy(&repr.bytes)
            .lines()
            .nth(1)
            .and_then(|line| line.trim().parse().ok())
            .expect("a count");
        assert!(
            n > 50,
            "the catalog plus the vocabulary is a real graph: {n} triples"
        );
    }
}
