//! The code taught by **Part I — Getting started**.
//!
//! The book is the prose; this crate is the code, and the book pulls these snippets in
//! by anchor rather than copying them. That way there is exactly one `camel-case`, and it
//! is the one that compiles — a book that paraphrases its own example is a book that will
//! eventually be wrong about it.
//!
//! Three endpoints live here. `camel-case` is the book's hello-world. `title` and
//! `camel-title` exist for "What resolution buys you": a mutable resource and a composite
//! derived from it, which is the smallest pair that can show a cache hit, a golden thread
//! being cut, a traced sub-resolution and a catalog with more than one entry in it. The
//! demonstrations themselves are `tests/payoff.rs`.
//!
//! Read the book: `mdbook serve books/ikigai --open`

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use ikigai_core::{
    ActionSpec, ArgSpec, AsyncFnEndpoint, Description, EndpointSpace, Exact, FnEndpoint,
    Invocation, InvokeFuture, Iri, Kernel, ReprType, Representation, Result, Verb,
};
use ikigai_vocab::TurtleRenderer;

/// `text/plain; charset=utf-8` as a [`ReprType`].
///
/// A *local* helper. `ikigai-fn` defines an identical pair privately; this crate repeats
/// it rather than depending on somebody else's internals.
fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

/// The same media type as a string, for [`Description::output`].
const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";

/// `xsd:string`, the class every text argument in this crate declares.
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";

/// The name `title` is bound at — and therefore, by convention, the name of the golden
/// thread that tracks its state. The two are the same string on purpose: the kernel cuts
/// the thread named after a `Sink`'s target, so a `Source` that declares this thread is
/// invalidated by exactly the write that changes it.
pub const TITLE: &str = "urn:iki:tutorial:title";

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
pub fn camel_case_impl(inv: &Invocation<'_>) -> Result<Representation> {
    let input = inv.inline_str("in")?;
    Ok(Representation::new(text_plain_utf8(), camel(input).into_bytes()).cacheable())
}

/// The pure function under the endpoint, so `camel-title` can reuse it without a
/// second resolution.
pub fn camel(input: &str) -> String {
    let mut words = input.split_whitespace();
    let mut output = words.next().unwrap_or_default().to_string();

    for word in words {
        let mut characters = word.chars();
        if let Some(first) = characters.next() {
            output.extend(first.to_uppercase());
            output.push_str(characters.as_str());
        }
    }
    output
}
// ANCHOR_END: impl

// ANCHOR: endpoint
/// `camel-case`: camel-cases the UTF-8 string in the `in` argument.
pub fn camel_case() -> FnEndpoint {
    FnEndpoint::new("camel-case", camel_case_impl).with_description(
        Description::new("camel-case")
            .title("Camel-case")
            .summary("Camel-cases the UTF-8 text supplied in the `in` argument.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("in")
                    .summary("the text to camel-case")
                    .class(XSD_STRING),
            )
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: endpoint

// ANCHOR: title
/// What `title` is *about*: a string, and a count of how often it has been read.
///
/// The state lives outside the endpoint on purpose. An endpoint over shared state is the
/// normal shape — a store handle, a connection pool, a file — and keeping the handle
/// where a test can hold it is what lets the test *prove* a cached answer was served
/// without this code running, rather than merely returning the same bytes, which a
/// recompute would also do.
#[derive(Debug)]
pub struct TitleState {
    text: Mutex<String>,
    /// How many times the `Source` arm has actually run.
    pub reads: AtomicUsize,
}

impl Default for TitleState {
    fn default() -> Self {
        TitleState {
            text: Mutex::new("resource oriented computing".to_string()),
            reads: AtomicUsize::new(0),
        }
    }
}

/// `title`: a string the host holds in memory. `Source` reads it, `Sink` replaces it.
///
/// The `Source` representation is `.cacheable()` *and* `.depends_on(TITLE)`. Together
/// those say: cache this, and treat it as valid until the thread named `TITLE` is cut.
/// The kernel cuts that thread itself after every successful `Sink` to this name — so
/// a write invalidates the cached read, and everything derived from it, with no code
/// here doing the invalidating.
pub fn title(state: Arc<TitleState>) -> FnEndpoint {
    FnEndpoint::new("title", move |inv: &Invocation<'_>| {
        let mut current = state.text.lock().expect("title lock");
        match inv.request.verb {
            Verb::Sink => {
                *current = inv.inline_str("content")?.to_string();
                Ok(Representation::new(text_plain_utf8(), b"ok".to_vec()))
            }
            _ => {
                state.reads.fetch_add(1, Ordering::SeqCst);
                Ok(
                    Representation::new(text_plain_utf8(), current.clone().into_bytes())
                        .cacheable()
                        .depends_on(TITLE),
                )
            }
        }
    })
    .with_description(
        Description::new("title")
            .title("Title")
            .summary("A string the host holds in memory: read it, or replace it.")
            .verb(Verb::Meta)
            // Two verbs with two different contracts, so each gets its own action.
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("the current title")
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("replace the title")
                    .input(
                        ArgSpec::new("content")
                            .summary("the new title")
                            .class(XSD_STRING),
                    ),
            ),
    )
}
// ANCHOR_END: title

// ANCHOR: camel_title
/// `camel-title`: the camel-cased form of whatever `title` currently says.
///
/// This endpoint takes no arguments. It *resolves* `urn:iki:tutorial:title` through the
/// kernel — `inv.source(..)` — and camel-cases the answer. That one call is what makes it
/// a composite: the kernel records the sub-resolution as a dependency, so this result
/// inherits `title`'s golden thread and is invalidated when `title` is written, even
/// though nothing here names the thread.
///
/// `inv.source` is async, which is why this is an [`AsyncFnEndpoint`] rather than the
/// [`FnEndpoint`] the other two are.
pub fn camel_title() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("camel-title", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let source = inv
                .source(&Iri::parse(TITLE).expect("a constant IRI"))
                .await?;
            let text = String::from_utf8_lossy(&source.bytes);
            Ok(Representation::new(text_plain_utf8(), camel(&text).into_bytes()).cacheable())
        })
    })
    .with_description(
        Description::new("camel-title")
            .title("Camel-cased title")
            .summary("The camel-cased form of whatever `title` currently says.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: camel_title

// ANCHOR: space
/// This book's space: the built-in function library, plus the three endpoints above
/// under a prefix this book owns.
pub fn space() -> EndpointSpace {
    space_over(Arc::default())
}
// ANCHOR_END: space

/// [`space`], with `title` reading and writing a state handle the caller keeps — so a
/// test can watch the read counter. Every other binding is identical.
pub fn space_over(state: Arc<TitleState>) -> EndpointSpace {
    ikigai_fn::space()
        .bind(Exact::new("urn:iki:tutorial:camel-case"), camel_case())
        .bind(Exact::new(TITLE), title(state))
        .bind(Exact::new("urn:iki:tutorial:camel-title"), camel_title())
}

// ANCHOR: kernel
/// The tutorial host: a kernel over [`space`], with a Meta renderer.
///
/// `Kernel::new` would resolve every `Source` here just as well. What it could not do is
/// answer `Meta` or `urn:kernel:catalog`, because rendering a `Description` into bytes is
/// a projection the kernel deliberately does not own — `ikigai-core` has no RDF in it.
/// `ikigai-vocab`'s `TurtleRenderer` is that projection, and injecting it is the whole
/// difference between a host that can describe itself and one that cannot.
pub fn kernel() -> Kernel {
    Kernel::with_meta_renderer(Arc::new(space()), Arc::new(TurtleRenderer))
}
// ANCHOR_END: kernel

/// [`kernel`] over [`space_over`]: the same host, with `title`'s state in your hand.
pub fn kernel_over(state: Arc<TitleState>) -> Kernel {
    Kernel::with_meta_renderer(Arc::new(space_over(state)), Arc::new(TurtleRenderer))
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures::executor::block_on;
    use ikigai_core::{ArgRef, Capability, Request};

    fn camel(input: &str) -> String {
        let kernel = kernel();
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:camel-case").expect("iri"),
        )
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
        assert_eq!(camel_case().describe().id, "camel-case");
    }

    #[test]
    fn title_declares_one_action_per_verb_and_the_sink_names_its_input() {
        use ikigai_core::Endpoint;
        let actions = title(Arc::default()).describe().action_specs();
        let verbs: Vec<Verb> = actions.iter().map(|a| a.verb).collect();
        assert_eq!(verbs, vec![Verb::Source, Verb::Sink]);
        let sink = actions
            .iter()
            .find(|a| a.verb == Verb::Sink)
            .expect("a Sink action");
        assert_eq!(sink.inputs[0].name, "content");
    }
}
