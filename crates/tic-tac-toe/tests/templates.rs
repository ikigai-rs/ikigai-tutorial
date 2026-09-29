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
    let name = |iri: &str| Iri::parse(iri).expect("a name");
    let space = ikigai_fn::space()
        .bind(Exact::new("urn:t:mark"), fixed(r#"<b>"&'$a{urn:t:secret}"#))
        .bind(Exact::new("urn:t:html"), fixed("<i>ok</i>"))
        .bind(Exact::new("urn:t:dash"), fixed(" -\n"))
        .bind(Exact::new("urn:t:on"), fixed(" On\n"))
        .bind(Exact::new("urn:t:empty"), fixed(""))
        .bind(Exact::new("urn:t:inner"), fixed("[$h{{x}}]"))
        .bind(Exact::new("urn:t:note"), fixed("($h{{message}})"))
        // Two views, bound as the game binds its own: a template at a name.
        .bind(
            at("urn:t:view:inner:{x}"),
            ikigai_fn::compose_over(name("urn:t:inner")),
        )
        .bind(
            Exact::new("urn:t:view:note"),
            ikigai_fn::compose_over(name("urn:t:note")),
        )
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
        let (template, after) = match rest.split_once('\t') {
            Some((template, after)) => (template, Some(after)),
            None => (rest, None),
        };
        if kind == "fill" {
            assert!(after.is_some(), "a `fill` case has a tab: {line:?}");
        }
        cases.push((kind, unescaped(template), after.map(unescaped)));
    }
    assert!(cases.len() >= 38, "{} cases", cases.len());
    let templates = Arc::new(cases.iter().map(|(_, t, _)| t.clone()).collect::<Vec<_>>());
    let kernel = kernel(templates);
    for (n, (kind, template, expected)) in cases.iter().enumerate() {
        let got = fill(&kernel, n);
        match (*kind, expected.as_deref()) {
            ("fill", Some(expected)) => {
                assert_eq!(
                    got.as_deref().map_err(ToString::to_string),
                    Ok(expected),
                    "{template:?}"
                );
            }
            ("refuse", class) => {
                let Err(error) = got else {
                    panic!("should be refused: {template:?} gave {got:?}");
                };
                // `malformed`: the template itself is refused — compose's own error, which
                // it raises before a level's markers resolve. `failed`: a marker's request
                // or argument failed, and the fill fails with THAT error.
                let malformed =
                    matches!(&error, Error::Endpoint(message) if message.starts_with("compose:"));
                match class {
                    None => {}
                    Some("malformed") => assert!(malformed, "{template:?}: {error}"),
                    Some("failed") => assert!(!malformed, "{template:?}: {error}"),
                    Some(other) => {
                        panic!("a refusal's class is `malformed` or `failed`: {other:?}")
                    }
                }
            }
            _ => unreachable!("a `fill` case has its expected text"),
        }
    }
}

/// A case as the README writes it, with each `\u{…}` the code point it names — so a case can
/// hold a character that does not show (U+3000, U+0085, U+FEFF). No other `\` is special.
fn unescaped(case: &str) -> String {
    let mut out = String::with_capacity(case.len());
    let mut rest = case;
    while let Some(at) = rest.find("\\u{") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 3..];
        let close = after.find('}').expect("a `\\u{` closes");
        let code = u32::from_str_radix(&after[..close], 16).expect("hex digits");
        out.push(char::from_u32(code).expect("a code point"));
        rest = &after[close + 1..];
    }
    out.push_str(rest);
    out
}
