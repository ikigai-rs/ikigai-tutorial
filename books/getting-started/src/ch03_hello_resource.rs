//! # 3 · Hello, resource
//!
//! The smallest useful thing: an endpoint that camel-cases text. It is this book's
//! "hello world", and it is a real endpoint — the code below is what the crate compiles,
//! not a paraphrase of it.
//!
//! ## The implementation
//!
//! ```
//! # use ikigai_core::{Invocation, Representation, Result, ReprType};
//! # fn text_plain_utf8() -> ReprType { ReprType::new("text/plain").with_param("charset", "utf-8") }
//! fn to_camel_impl(inv: &Invocation<'_>) -> Result<Representation> {
//!     let input = inv.inline_str("in")?;
//!     let mut words = input.split_whitespace();
//!     let mut output = words.next().unwrap_or_default().to_string();
//!
//!     for word in words {
//!         let mut characters = word.chars();
//!         if let Some(first) = characters.next() {
//!             output.extend(first.to_uppercase());
//!             output.push_str(characters.as_str());
//!         }
//!     }
//!
//!     Ok(Representation::new(text_plain_utf8(), output.into_bytes()).cacheable())
//! }
//! ```
//!
//! Four things in there are worth slowing down for.
//!
//! ## `inv.inline_str("in")`
//!
//! Arguments arrive on the [`Invocation`], by name, and they may
//! be *inline bytes* or *a reference to another resource*. `inline_str` says "give me this
//! argument as a UTF-8 string, and fail cleanly if it is not there or is not text."
//!
//! The reference case is the interesting one, and you get it for free: a caller can pass
//! `in=urn:something:else` and the kernel resolves that first. Your endpoint does not
//! change. This is why pipes work — `|` is not a shell feature bolted on, it is one
//! resolution's output becoming another's argument.
//!
//! ## `Representation::new(text_plain_utf8(), …)`
//!
//! You return bytes *and their type*. Always. The type is not decoration: it is what lets
//! a transreptor find a route from what you produced to what somebody asked for.
//!
//! ## `.cacheable()`
//!
//! You are asserting **this is a pure function of its declared inputs**. Same arguments,
//! same answer, forever. The kernel may then cache it and hand out the cached
//! representation under a golden thread.
//!
//! Get this wrong in the optimistic direction and you have a bug that is very hard to
//! see: stale answers that look plausible. The rule of thumb is the honest one — *when in
//! doubt, do not cache*. `toCamel` is genuinely pure, so it says so.
//!
//! ## `char::to_uppercase` returns an iterator
//!
//! Not a `char`. Some characters upper-case to more than one — German ß becomes SS — so
//! the API cannot pretend otherwise. `output.extend(first.to_uppercase())` is the
//! Unicode-correct form; `push` would not compile, which is the type system doing you a
//! favor.
//!
//! ## Try it
//!
//! ```
//! use getting_started::to_camel;
//! use ikigai_core::Endpoint;
//!
//! // Every endpoint knows its own name.
//! assert_eq!(to_camel().describe().id, "toCamel");
//! ```
//!
//! Next: [`super::ch04_self_description`] — because an endpoint that works is only half
//! of one.

use ikigai_core::{
    ArgSpec, Description, FnEndpoint, Invocation, ReprType, Representation, Result, Verb,
};

/// `text/plain; charset=utf-8` as a [`ReprType`].
///
/// Note this is a *local* helper. `ikigai-fn` defines an identical pair privately, and
/// this book deliberately repeats it rather than reaching for something that is not
/// public API — a small honest duplication beats a dependency on somebody's internals.
fn text_plain_utf8() -> ReprType {
    ReprType::new("text/plain").with_param("charset", "utf-8")
}

/// The same media type as a string, for [`Description::output`].
const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";

/// Camel-case the whitespace-separated words of the `in` argument.
///
/// The first word is left exactly as given and each subsequent word has its first
/// character upper-cased — so `"resource oriented computing"` becomes
/// `"resourceOrientedComputing"`.
///
/// ⚠ Note what it deliberately does *not* do: it never lower-cases anything. `"Hello
/// WORLD"` becomes `"HelloWORLD"`, because the input's own casing is treated as
/// meaningful rather than as noise to be normalized. If you want strict lowerCamelCase,
/// that is a different function — and a good first exercise.
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

/// `toCamel`: camel-cases the UTF-8 string in the `in` argument.
///
/// The [`Description`] is not paperwork — see [chapter 4](super::ch04_self_description).
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
