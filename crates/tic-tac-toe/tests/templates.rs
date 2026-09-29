//! The template language as the README states it for a host that fills the templates itself
//! (the Python and Deno apps): every case in the README's `template-cases` block, run
//! through `ikigai-fn`'s own compose, so the spec those fillers are written from and the
//! language the Rust views use cannot drift apart.
//!
//! ```text
//! cargo test -p tic-tac-toe --test templates
//! ```

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Error, Exact, FnEndpoint, Invocation, Iri, Kernel, ReprType,
    Representation, Request, Result, UriTemplate, Verb,
};

/// A fixed text answer, cacheable, as the game's own resources are.
fn fixed(text: &'static str) -> FnEndpoint {
    FnEndpoint::new("fixed", move |_: &Invocation<'_>| {
        Ok(Representation::new(ReprType::new("text/html"), text.as_bytes().to_vec()).cacheable())
    })
}

/// The README's world: the resources its cases are filled over, each case's template at
/// `urn:t:case:{n}`, and `ikigai-fn` (for `compose` and `conditional`).
fn kernel(cases: Arc<Vec<String>>) -> Kernel {
    let at = |pattern: &str| UriTemplate::parse(pattern).expect("a valid template");
    let space = ikigai_fn::space()
        .bind(Exact::new("urn:t:mark"), fixed(r#"<b>"&'$a{urn:t:secret}"#))
        .bind(Exact::new("urn:t:html"), fixed("<i>ok</i>"))
        .bind(Exact::new("urn:t:dash"), fixed(" -\n"))
        .bind(Exact::new("urn:t:inner"), fixed("[$h{{x}}]"))
        .bind(
            at("urn:t:cell:{x}:{y}"),
            FnEndpoint::new("cell", |inv: &Invocation<'_>| {
                let get = |name: &str| inv.bindings.get(name).unwrap_or_default().to_string();
                let text = format!("{}.{}", get("x"), get("y"));
                Ok(Representation::new(
                    ReprType::new("text/plain"),
                    text.into_bytes(),
                ))
            }),
        )
        .bind(
            at("urn:t:case:{n}"),
            FnEndpoint::new("case", move |inv: &Invocation<'_>| {
                let n: usize = inv
                    .bindings
                    .get("n")
                    .and_then(|n| n.parse().ok())
                    .ok_or_else(|| Error::MissingArgument("n".to_string()))?;
                Ok(Representation::new(
                    ReprType::new("text/html"),
                    cases[n].clone().into_bytes(),
                ))
            }),
        );
    Kernel::new(Arc::new(space))
}

/// Fill case `n` with the README's arguments.
fn fill(kernel: &Kernel, n: usize) -> Result<String> {
    let arg = |text: &str| ArgRef::Inline(text.as_bytes().to_vec());
    let request = Request::new(
        Verb::Source,
        Iri::parse("urn:iki:fn:compose").expect("a name"),
    )
    .with_arg("src", arg(&format!("urn:t:case:{n}")))
    .with_arg("x", arg("1"))
    .with_arg("y", arg("-2"))
    .with_arg("message", arg("it's <b>"));
    let repr = block_on(kernel.issue(request, &Capability::root()))?;
    Ok(String::from_utf8(repr.bytes).expect("UTF-8"))
}

#[test]
fn the_template_language_cases_in_the_readme_hold() {
    const README: &str = include_str!("../README.md");
    let block = README
        .split_once("<!-- template-cases: begin -->\n```text\n")
        .and_then(|(_, rest)| rest.split_once("```\n<!-- template-cases: end -->"))
        .expect("the README has a template-cases block")
        .0;
    let mut cases: Vec<(&str, String, Option<String>)> = Vec::new();
    for line in block.lines() {
        let kind = ["fill", "refuse"]
            .into_iter()
            .find(|kind| line.starts_with(kind))
            .unwrap_or_else(|| panic!("a case is `fill` or `refuse`: {line:?}"));
        let rest = line[kind.len()..].trim_start();
        match kind {
            "fill" => {
                let (template, filled) = rest.split_once('\t').expect("a tab");
                cases.push((kind, template.to_string(), Some(filled.to_string())));
            }
            _ => cases.push((kind, rest.to_string(), None)),
        }
    }
    assert!(cases.len() >= 20, "{} cases", cases.len());
    let templates = Arc::new(cases.iter().map(|(_, t, _)| t.clone()).collect::<Vec<_>>());
    let kernel = kernel(templates);
    for (n, (kind, template, expected)) in cases.iter().enumerate() {
        let got = fill(&kernel, n);
        match (kind, expected) {
            (&"fill", Some(expected)) => {
                assert_eq!(
                    got.as_deref().map_err(ToString::to_string),
                    Ok(expected.as_str()),
                    "{template:?}"
                );
            }
            _ => assert!(got.is_err(), "should be refused: {template:?} gave {got:?}"),
        }
    }
}
