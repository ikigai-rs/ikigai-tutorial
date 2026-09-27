//! **Transreption** — a transreptor of your own, and the kernel finding it.
//!
//! A transreptor is an endpoint that converts one representation into another. What makes
//! it more than a function is that it is *declared*: `Description::transreptor(from, to)`
//! tells the kernel which conversions it performs, and from then on a request for
//! `as=text/markdown` anywhere the kernel knows how to reach Turtle can be answered by
//! routing through it — without the requester, or the endpoint being described, knowing
//! this transreptor exists.

use std::collections::BTreeMap;

use ikigai_core::{
    ArgSpec, Description, Error, FnEndpoint, Invocation, ReprType, Representation, Result, Verb,
};
use oxrdf::{NamedOrBlankNode, Term, Triple};

/// `text/markdown; charset=utf-8`.
fn text_markdown_utf8() -> ReprType {
    ReprType::new("text/markdown").with_param("charset", "utf-8")
}

const IK: &str = "https://ikigai-rs.dev/ns#";
const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

// ANCHOR: impl
/// Render the ikigai description vocabulary as Markdown: one section per endpoint, its
/// summary, and a bullet per declared input.
///
/// The input is parsed as RDF — real triples, not lines — so this works on any Turtle
/// carrying the vocabulary, whether it came from one `Meta` or a whole catalog. The
/// `content` argument is the document; a transreptor reads its input from there because
/// that is where the kernel puts it when it drives a transreption step.
pub fn markdown_impl(inv: &Invocation<'_>) -> Result<Representation> {
    let turtle = inv.inline_arg("content")?;
    let triples: Vec<Triple> = oxttl::TurtleParser::new()
        .for_slice(turtle)
        .collect::<std::result::Result<_, _>>()
        .map_err(|e| Error::Endpoint(format!("not Turtle: {e}")))?;

    // Every triple, grouped by subject and then by predicate — stable order, so the
    // output is diffable and a test can assert on it.
    let mut by_subject: BTreeMap<String, BTreeMap<String, Vec<Term>>> = BTreeMap::new();
    for triple in triples {
        let subject = match triple.subject {
            NamedOrBlankNode::NamedNode(node) => node.as_str().to_string(),
            other => other.to_string(),
        };
        by_subject
            .entry(subject)
            .or_default()
            .entry(triple.predicate.as_str().to_string())
            .or_default()
            .push(triple.object);
    }

    let literal = |node: &BTreeMap<String, Vec<Term>>, property: &str| -> Option<String> {
        node.get(&format!("{IK}{property}"))
            .and_then(|terms| terms.first())
            .and_then(|term| match term {
                Term::Literal(literal) => Some(literal.value().to_string()),
                _ => None,
            })
    };

    let mut out = String::new();
    for (iri, node) in &by_subject {
        let is_endpoint = node
            .get(RDF_TYPE)
            .is_some_and(|types| types.iter().any(|t| t.to_string().contains("ns#Endpoint")));
        if !is_endpoint {
            continue;
        }
        let id = literal(node, "id").unwrap_or_else(|| iri.clone());
        let title = literal(node, "title").unwrap_or_else(|| id.clone());
        out.push_str(&format!("## {title} (`{id}`)\n\n"));
        if let Some(summary) = literal(node, "summary") {
            out.push_str(&format!("{summary}\n\n"));
        }
        // Inputs are nodes of their own; follow the `ik:input` edges to them.
        let inputs = node.get(&format!("{IK}input")).into_iter().flatten();
        let mut listed = false;
        for input in inputs {
            let Term::NamedNode(input_iri) = input else {
                continue;
            };
            let Some(input_node) = by_subject.get(input_iri.as_str()) else {
                continue;
            };
            let name = literal(input_node, "inputName").unwrap_or_default();
            let required = literal(input_node, "required").as_deref() == Some("true");
            let summary = literal(input_node, "summary").unwrap_or_default();
            out.push_str(&format!(
                "- `{name}`{} — {summary}\n",
                if required { " (required)" } else { "" }
            ));
            listed = true;
        }
        if listed {
            out.push('\n');
        }
    }

    Ok(Representation::new(text_markdown_utf8(), out.into_bytes()).cacheable())
}
// ANCHOR_END: impl

// ANCHOR: endpoint
/// `markdown`: a transreptor from `text/turtle` to `text/markdown` — declared **lossy**.
///
/// The description is the interesting half. `.transreptor(from, to)` is what makes the
/// kernel *select* this endpoint when something asks for Markdown and has Turtle; the
/// two inputs are `content` and `as`, and both are what the kernel supplies when it drives
/// a step — an auto-invocable transreptor is one whose required inputs are exactly
/// those. Declare a third required input and it is still a transreptor, but the kernel
/// will not pick it up on its own.
///
/// `.lossy()` says what kind of conversion this is. A transreption proper changes form
/// without changing what is represented — the Turtle could be recovered from the output.
/// This one cannot be undone: it keeps a title, a summary and the inputs and drops the
/// verbs, the capabilities, the output type and everything else, so it is a *rendering*
/// of the description, a projection. Saying so is what lets the kernel refuse to route a
/// `Meta` through it silently — the caller has to consent with `lossy=allow` — while a
/// transreptor that says nothing claims, by default, to be lossless.
pub fn markdown() -> FnEndpoint {
    FnEndpoint::new("markdown", markdown_impl).with_description(
        Description::new("markdown")
            .title("Turtle to Markdown")
            .summary("Renders the ikigai description vocabulary as Markdown.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(ArgSpec::new("content").summary("the Turtle document to render"))
            .input(
                ArgSpec::new("as")
                    .summary("the target type; only text/markdown")
                    .optional(),
            )
            .output("text/markdown;charset=utf-8")
            .transreptor(["text/turtle"], ["text/markdown"])
            .lossy(),
    )
}
// ANCHOR_END: endpoint

#[cfg(test)]
mod tests {
    use crate::kernel_in;
    use futures::executor::block_on;
    use ikigai_core::{ArgRef, Capability, Error, Iri, Request, TransreptionPolicy, Verb};

    // ANCHOR: selected
    /// The kernel finds the transreptor by its declaration, not by its name — and reads
    /// the whole declaration: a lossy edge is planned only with consent.
    #[test]
    fn the_kernel_selects_the_transreptor_from_its_declaration() {
        let kernel = kernel_in(None);

        // The default policy plans through lossless edges only, and this one is lossy.
        assert!(kernel
            .select_transreptor("text/turtle", "text/markdown")
            .is_none());

        // With consent the route is one hop, and the step reports what it declared.
        let plan = kernel
            .select_transreptor_with(
                "text/turtle",
                "text/markdown",
                &TransreptionPolicy::allow_lossy(),
            )
            .expect("a route from Turtle to Markdown exists, with consent");
        assert_eq!(plan.len(), 1, "one hop");
        assert_eq!(plan[0].endpoint, "urn:iki:tutorial:markdown");
        assert_eq!(plan[0].to, "text/markdown");
        assert!(!plan[0].lossless);

        // ...and no route to a type nothing produces, consent or not.
        assert!(kernel
            .select_transreptor_with(
                "text/turtle",
                "application/pdf",
                &TransreptionPolicy::allow_lossy()
            )
            .is_none());
    }
    // ANCHOR_END: selected

    // ANCHOR: meta_as
    /// `Meta … as=text/markdown lossy=allow` on an endpoint that knows nothing about
    /// Markdown. The renderer emits Turtle; the kernel finds the route; the answer is
    /// Markdown. The consent is there because the route is a projection — see the
    /// refusal test below for what happens without it.
    #[test]
    fn meta_reaches_markdown_through_the_transreptor() {
        let kernel = kernel_in(None);
        let request = Request::new(
            Verb::Meta,
            Iri::parse("urn:iki:tutorial:camel-case").expect("iri"),
        )
        .with_arg("as", ArgRef::Inline(b"text/markdown".to_vec()))
        .with_arg("lossy", ArgRef::Inline(b"allow".to_vec()));

        let repr = block_on(kernel.issue(request, &Capability::root())).expect("resolves");
        let markdown = String::from_utf8(repr.bytes).expect("utf-8");

        assert_eq!(repr.repr_type.media_type, "text/markdown");
        assert!(
            markdown.starts_with("## Camel-case (`camel-case`)\n"),
            "{markdown}"
        );
        assert!(
            markdown.contains("- `in` (required) — the text to camel-case"),
            "{markdown}"
        );
    }
    // ANCHOR_END: meta_as

    /// Without consent the lossy route is refused — typed, naming the step and the
    /// consent — rather than taken silently or substituted with Turtle. The type asked
    /// for IS reachable, so it is not `NotFound`; no authority is at issue, so it is not
    /// `Denied`; the arguments are well-formed, so it is not `InvalidArgument`.
    #[test]
    fn meta_refuses_the_lossy_route_without_consent() {
        let kernel = kernel_in(None);
        let request = Request::new(
            Verb::Meta,
            Iri::parse("urn:iki:tutorial:camel-case").expect("iri"),
        )
        .with_arg("as", ArgRef::Inline(b"text/markdown".to_vec()));

        let error = block_on(kernel.issue(request, &Capability::root())).unwrap_err();
        assert!(matches!(error, Error::Endpoint(_)), "{error:?}");
        let message = error.to_string();
        assert!(message.contains("`urn:iki:tutorial:markdown`"), "{message}");
        assert!(message.contains("lossy=allow"), "{message}");
    }

    #[test]
    fn a_document_that_is_not_turtle_is_refused_not_rendered() {
        let kernel = kernel_in(None);
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:markdown").expect("iri"),
        )
        .with_arg("content", ArgRef::Inline(b"this is not turtle }{".to_vec()));
        let error = block_on(kernel.issue(request, &Capability::root())).unwrap_err();
        assert!(error.to_string().contains("not Turtle"), "{error}");
    }
}
