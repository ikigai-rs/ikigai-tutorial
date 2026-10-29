//! The quakes context and the quakes vocabulary agree (`books/ikigai/ns/`).
//!
//! The context is what maps USGS's feed onto names; the vocabulary is what those names mean,
//! published at `https://ikigai-rs.dev/ns/quakes`. A term the context maps into our namespace
//! and the vocabulary does not define is a name nobody can look up, and a term the vocabulary
//! defines and no context maps is one nothing uses. Both fail here. A term the context maps
//! outside our namespace must be in a vocabulary the ontology says it reuses.

use std::collections::{BTreeMap, BTreeSet};

use oxrdf::Term;
use oxttl::TurtleParser;
use serde_json::Value;

const NS: &str = "https://ikigai-rs.dev/ns/quakes#";
const ONTOLOGY: &str = "https://ikigai-rs.dev/ns/quakes";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";
const RDFS_LABEL: &str = "http://www.w3.org/2000/01/rdf-schema#label";
const RDFS_COMMENT: &str = "http://www.w3.org/2000/01/rdf-schema#comment";
const RDFS_SEE_ALSO: &str = "http://www.w3.org/2000/01/rdf-schema#seeAlso";
const OWL_ONTOLOGY: &str = "http://www.w3.org/2002/07/owl#Ontology";
const OWL_VERSION: &str = "http://www.w3.org/2002/07/owl#versionInfo";

/// The vocabularies the context may reuse: each namespace, and the document the ontology
/// must name in `rdfs:seeAlso` for it.
const REUSED: [(&str, &str); 2] = [
    (
        "https://purl.org/geojson/vocab#",
        "https://purl.org/geojson/vocab",
    ),
    ("https://schema.org/", "https://schema.org/"),
];

/// The context's own definitions: `@context`'s object.
fn context() -> serde_json::Map<String, Value> {
    let doc: Value = serde_json::from_str(quakes::CONTEXT_TEXT).expect("the context is JSON");
    doc["@context"]
        .as_object()
        .expect("the context document holds one context object")
        .clone()
}

/// Every term the context maps to an IRI, by term: prefixes and keyword aliases left out,
/// compact IRIs expanded.
fn mapped_terms() -> BTreeMap<String, String> {
    let context = context();
    let prefixes: BTreeMap<&str, &str> = context
        .iter()
        .filter_map(|(term, value)| {
            let iri = value.as_str()?;
            (iri.ends_with('#') || iri.ends_with('/')).then_some((term.as_str(), iri))
        })
        .collect();
    let expand = |iri: &str| -> String {
        match iri.split_once(':') {
            Some((prefix, rest)) if prefixes.contains_key(prefix) => {
                format!("{}{rest}", prefixes[prefix])
            }
            _ => iri.to_string(),
        }
    };
    context
        .iter()
        .filter(|(term, _)| !term.starts_with('@') && !prefixes.contains_key(term.as_str()))
        .filter_map(|(term, value)| {
            let iri = match value {
                Value::String(iri) => iri.as_str(),
                Value::Object(definition) => definition["@id"].as_str().expect("an @id"),
                other => panic!("term {term}: a definition this test does not read: {other}"),
            };
            // `"id": "@id"`, `"type": "@type"`, `"properties": "@nest"`: keyword aliases.
            (!iri.starts_with('@')).then(|| (term.clone(), expand(iri)))
        })
        .collect()
}

/// The vocabulary's triples, as (subject, predicate, object) strings, IRIs bare.
fn vocabulary() -> Vec<(String, String, String)> {
    TurtleParser::new()
        .for_slice(quakes::VOCABULARY_TEXT.as_bytes())
        .map(|triple| {
            let triple = triple.expect("the vocabulary is Turtle");
            let object = match &triple.object {
                Term::NamedNode(node) => node.as_str().to_string(),
                Term::Literal(literal) => literal.value().to_string(),
                other => other.to_string(),
            };
            (
                triple
                    .subject
                    .to_string()
                    .trim_matches(['<', '>'])
                    .to_string(),
                triple.predicate.as_str().to_string(),
                object,
            )
        })
        .collect()
}

fn has(triples: &[(String, String, String)], s: &str, p: &str) -> bool {
    triples.iter().any(|(ts, tp, _)| ts == s && tp == p)
}

#[test]
fn every_term_the_context_maps_into_our_namespace_is_defined_and_every_defined_term_is_mapped() {
    let triples = vocabulary();
    let mapped: BTreeSet<String> = mapped_terms()
        .into_values()
        .filter(|iri| iri.starts_with(NS))
        .collect();
    let defined: BTreeSet<String> = triples
        .iter()
        .filter(|(s, _, _)| s.starts_with(NS))
        .map(|(s, _, _)| s.clone())
        .collect();
    assert_eq!(
        mapped, defined,
        "the context maps (left) and the vocabulary defines (right) different terms"
    );
    for term in &defined {
        for property in [RDF_TYPE, RDFS_LABEL, RDFS_COMMENT] {
            assert!(has(&triples, term, property), "{term} has no <{property}>");
        }
    }
}

#[test]
fn every_term_the_context_maps_elsewhere_is_in_a_vocabulary_the_ontology_says_it_reuses() {
    let triples = vocabulary();
    for (term, iri) in mapped_terms() {
        if iri.starts_with(NS) {
            continue;
        }
        let (_, document) = REUSED
            .iter()
            .find(|(namespace, _)| iri.starts_with(namespace))
            .unwrap_or_else(|| panic!("`{term}` maps to {iri}, in no vocabulary this one reuses"));
        assert!(
            triples
                .iter()
                .any(|(s, p, o)| s == ONTOLOGY && p == RDFS_SEE_ALSO && o == document),
            "`{term}` reuses {iri}, and the ontology does not name {document} in rdfs:seeAlso"
        );
    }
}

#[test]
fn the_vocabulary_is_a_versioned_ontology() {
    let triples = vocabulary();
    assert!(
        triples
            .iter()
            .any(|(s, p, o)| s == ONTOLOGY && p == RDF_TYPE && o == OWL_ONTOLOGY),
        "<{ONTOLOGY}> is not an owl:Ontology"
    );
    assert!(has(&triples, ONTOLOGY, OWL_VERSION), "no owl:versionInfo");
}

/// The earthquakes keep USGS's names, and the context mints none of its own: no `@vocab`,
/// which would give every key it does not map a term in some namespace.
#[test]
fn the_context_names_each_earthquake_by_its_usgs_page_and_has_no_vocab() {
    let context = context();
    assert_eq!(
        context["@base"],
        "https://earthquake.usgs.gov/earthquakes/eventpage/"
    );
    assert!(!context.contains_key("@vocab"));
    assert_eq!(context["id"], "@id");
}

/// The chapter's shell steps write the same template and the same question the page binds.
#[test]
fn the_chapter_writes_the_template_and_the_question_the_page_holds() {
    let chapter = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../books/ikigai/src/explore/names.md"
    ))
    .expect("the chapter is readable");
    let template = format!(
        "ikigai> sink urn:file:graph.txt {}\n",
        quakes::GRAPH_TEMPLATE_TEXT
    );
    assert!(
        chapter.contains(&template),
        "the chapter does not write {template}"
    );
    let (_, by_magtype) = quakes::QUESTIONS[2];
    let question = format!("ikigai> sink urn:file:queries/by-magtype.rq {by_magtype}\n");
    assert!(
        chapter.contains(&question),
        "the chapter does not write {question}"
    );
}
