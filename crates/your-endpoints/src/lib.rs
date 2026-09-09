//! **Your crate.** The book's exercises are done here, and nowhere else.
//!
//! Every listing in the book is included from `crates/hello-camel`, `crates/loadable-module`
//! or `crates/two-hosts` — the files the book *quotes*. Editing one of those changes what
//! the book says, which is why the exercises used to be worse than they looked: doing
//! exercise 1 rewrote the chapter that set it. Nothing quotes this crate, and a test in
//! `crates/book-urns` keeps it that way. Break anything in here you like.
//!
//! What is here to start with:
//!
//! * one worked endpoint, `word-count`, in the shape the book teaches — a function, a
//!   description, a binding;
//! * [`space`], which chains onto Part I's space so `urn:iki:tutorial:camel-case` and
//!   `urn:iki:tutorial:title` resolve from your host too;
//! * [`kernel`], the host, with the Meta renderer that lets it describe itself;
//! * `module.rs`, a module and a host for Part II's exercises;
//! * `tests/beyond.rs`, the helpers for Part III's exercises;
//! * the `text_of` test helper — the four lines every exercise's test starts from.
//!
//! The names are under `urn:iki:tutorial:yours:`. That prefix is a stand-in for one you
//! own; the day this is not a tutorial, rename it, and notice that nothing else has to
//! change — which is the lesson of "Bind into a namespace you own".
//!
//! ```text
//! cargo test -p your-endpoints
//! cargo run  -p your-endpoints -- "a few words here"
//! cargo run  -p your-endpoints -- --catalog
//! ```

pub mod module;

use std::sync::Arc;

use ikigai_core::{
    ArgSpec, Description, EndpointSpace, Exact, FnEndpoint, Invocation, Kernel, ReprType,
    Representation, Result, Verb,
};
use ikigai_vocab::TurtleRenderer;

/// `text/plain; charset=utf-8` as a [`ReprType`].
pub fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

/// The same media type as a string, for [`Description::output`].
pub const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";

/// `xsd:string`, the class a text argument declares.
pub const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// The prefix your names live under. See the crate docs for why it is this one.
pub const PREFIX: &str = "urn:iki:tutorial:yours:";

/// Count the whitespace-separated words in the `in` argument.
///
/// Pure — same input, same answer — so the representation says `.cacheable()`.
pub fn word_count_impl(inv: &Invocation<'_>) -> Result<Representation> {
    let input = inv.inline_str("in")?;
    let count = input.split_whitespace().count();
    Ok(Representation::new(text_plain_utf8(), count.to_string().into_bytes()).cacheable())
}

/// `word-count`: the number of words in the `in` argument.
pub fn word_count() -> FnEndpoint {
    FnEndpoint::new("word-count", word_count_impl).with_description(
        Description::new("word-count")
            .title("Word count")
            .summary("Counts the whitespace-separated words in the `in` argument.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("in")
                    .summary("the text to count")
                    .class(XSD_STRING),
            )
            .output(TEXT_PLAIN_UTF8),
    )
}

/// Your space: Part I's bindings, plus yours. Add a line per endpoint.
pub fn space() -> EndpointSpace {
    hello_camel::space().bind(
        Exact::new("urn:iki:tutorial:yours:word-count"),
        word_count(),
    )
    // .bind(Exact::new("urn:iki:tutorial:yours:your-thing"), your_thing())
}

/// Your host: a kernel over [`module::host_space`] — which is [`space`] plus the Part II
/// module — with the renderer that lets it answer `Meta` and `urn:kernel:catalog`.
pub fn kernel() -> Kernel {
    Kernel::with_meta_renderer(Arc::new(module::host_space()), Arc::new(TurtleRenderer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use ikigai_core::{ArgRef, Capability, Iri, Request};

    /// Resolve `name` with `in=input` through your host and hand back the text. Copy it,
    /// change the name, and you have a way to ask any endpoint a question.
    fn text_of(name: &str, input: &str) -> String {
        let kernel = kernel();
        let request = Request::new(Verb::Source, Iri::parse(name).expect("iri"))
            .with_arg("in", ArgRef::Inline(input.as_bytes().to_vec()));
        let repr = block_on(kernel.issue(request, &Capability::root())).expect("resolves");
        String::from_utf8(repr.bytes).expect("utf-8")
    }

    #[test]
    fn word_count_counts_words() {
        assert_eq!(
            text_of("urn:iki:tutorial:yours:word-count", "a few words here"),
            "4"
        );
    }

    #[test]
    fn empty_input_is_zero_not_an_error() {
        assert_eq!(text_of("urn:iki:tutorial:yours:word-count", "   "), "0");
    }

    #[test]
    fn part_one_is_reachable_from_your_host() {
        // Chained onto, not copied: the book's endpoint answers under your kernel.
        assert_eq!(
            text_of("urn:iki:tutorial:camel-case", "resource oriented computing"),
            "resourceOrientedComputing"
        );
    }

    #[test]
    fn your_endpoint_describes_itself() {
        use ikigai_core::Endpoint;
        let description = word_count().describe();
        assert_eq!(description.id, "word-count");
        assert_eq!(description.inputs[0].name, "in");
    }
}
