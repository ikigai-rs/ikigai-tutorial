//! The code taught by **Book 1 — Getting Started**.
//!
//! The book is the prose; this crate is the code, and the book pulls these snippets in
//! by anchor rather than copying them. That way there is exactly one `toCamel` and it is
//! the one that compiles — a book that paraphrases its own example is a book that will
//! eventually be wrong about it.
//!
//! Read the book: `mdbook serve books/getting-started --open`

use ikigai_core::{
    ArgSpec, Description, EndpointSpace, Exact, FnEndpoint, Invocation, ReprType, Representation,
    Result, Verb,
};

/// `text/plain; charset=utf-8` as a [`ReprType`].
///
/// A *local* helper. `ikigai-fn` defines an identical pair privately; this crate repeats
/// it rather than depending on somebody else's internals.
fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

/// The same media type as a string, for [`Description::output`].
const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";

// ANCHOR: impl
/// Camel-case the whitespace-separated words of the `in` argument.
///
/// The first word is left exactly as given and each subsequent word has its first
/// character upper-cased, so `"resource oriented computing"` becomes
/// `"resourceOrientedComputing"`.
///
/// ⚠ Note what it deliberately does *not* do: it never lower-cases anything. `"Hello
/// WORLD"` becomes `"HelloWORLD"`, because the input's own casing is treated as
/// meaningful rather than as noise to normalize away.
pub fn to_camel_impl(inv: &Invocation<'_>) -> Result<Representation> {
    let input = inv.inline_str("in")?;
    let mut words = input.split_whitespace();
    let mut output = words.next().unwrap_or_default().to_string();

    for word in words {
        let mut characters = word.chars();
        if let Some(first) = characters.next() {
            output.extend(first.to_uppercase());
            output.push_str(characters.as_str());
        }
    }

    Ok(Representation::new(text_plain_utf8(), output.into_bytes()).cacheable())
}
// ANCHOR_END: impl

// ANCHOR: endpoint
/// `toCamel`: camel-cases the UTF-8 string in the `in` argument.
pub fn to_camel() -> FnEndpoint {
    FnEndpoint::new("toCamel", to_camel_impl).with_description(
        Description::new("toCamel")
            .title("Camel-case")
            .summary("Camel-cases the UTF-8 text supplied in the `in` argument.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("in")
                    .summary("the text to camel-case")
                    .class("http://www.w3.org/2001/XMLSchema#string"),
            )
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: endpoint

// ANCHOR: space
/// This book's space: the built-in function library, plus `urn:fn:toCamel`.
pub fn space() -> EndpointSpace {
    ikigai_fn::space().bind(Exact::new("urn:fn:toCamel"), to_camel())
}
// ANCHOR_END: space

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request};
    use std::sync::Arc;

    fn camel(input: &str) -> String {
        let kernel = Kernel::new(Arc::new(space()));
        let request = Request::new(Verb::Source, Iri::parse("urn:fn:toCamel").expect("iri"))
            .with_arg("in", ArgRef::Inline(input.as_bytes().to_vec()));
        let repr = block_on(kernel.issue(request, &Capability::root())).expect("resolves");
        String::from_utf8(repr.bytes).expect("utf-8")
    }

    #[test]
    fn joins_words_on_the_camel_hump() {
        assert_eq!(
            camel("resource oriented computing"),
            "resourceOrientedComputing"
        );
    }

    #[test]
    fn the_first_word_keeps_its_given_case() {
        // Documented behaviour, not an accident: input casing is meaningful.
        assert_eq!(camel("Hello WORLD"), "HelloWORLD");
    }

    #[test]
    fn empty_input_is_empty_output_not_an_error() {
        assert_eq!(camel("   "), "");
    }

    #[test]
    fn the_endpoint_describes_itself_under_the_name_it_is_bound_at() {
        use ikigai_core::Endpoint;
        assert_eq!(to_camel().describe().id, "toCamel");
    }
}
