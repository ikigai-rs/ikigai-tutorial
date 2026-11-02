//! **What a rendered cell is styled by.**
//!
//! A cell that carries `data-render='html'` (`js/run.js`, the one script every book loads) shows
//! the view its command composes as a page, not as its source text. The view is put in a shadow
//! root of its own, so the book's styles do not reach into it and its markup does not reach out,
//! and that root is styled by ikigai-ui's token stylesheet (`urn:iki:ui:tokens.css`, UI
//! conventions D12): the same palettes and the same shell rules an ikigai-ui page links.
//!
//! The stylesheet is written for a document: its palettes hang from `:root` and `data-theme` on
//! `<html>`, and its page colors from `body`. In a shadow root neither exists, so [`styles`]
//! rewrites those selectors to the root's host (`:host`, `:host([data-theme="dark"])`), and
//! `js/run.js` sets `data-theme` on the host from the book's theme. Nothing else in the sheet
//! changes: a color it does not name is not written here either.
//!
//! The page kernels (crates/book-wasm, crates/ui-book-wasm) export it as `viewStyles`. It is
//! composed by a kernel of its own, over ikigai-ui's space alone, so asking for it never adds an
//! entry to the cache the page's cells share.

#![forbid(unsafe_code)]

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{Capability, Iri, Kernel, Request, Verb};

/// ikigai-ui's token stylesheet, with its document selectors moved to a shadow root's host.
///
/// ```
/// let css = book_view::styles();
/// assert!(css.contains(":host([data-theme=\"dark\"]) {"));
/// assert!(!css.contains(":root"));
/// ```
pub fn styles() -> String {
    let kernel = Kernel::new(Arc::new(ikigai_ui::space()));
    let sheet = block_on(kernel.issue(
        Request::new(
            Verb::Source,
            Iri::parse(ikigai_ui::TOKENS_CSS).expect("an IRI"),
        ),
        &Capability::root(),
    ))
    .expect("ikigai-ui composes its own token stylesheet");
    for_host(&String::from_utf8(sheet.bytes).expect("the stylesheet is UTF-8"))
}

/// A document's stylesheet, for a shadow root: `:root` and its conditions become `:host` with
/// the same conditions (`:root:not([data-theme])` is `:host(:not([data-theme]))`), and a rule
/// for `body` becomes one for `:host`.
pub fn for_host(css: &str) -> String {
    let mut out = String::with_capacity(css.len());
    let mut rest = css;
    while let Some(at) = rest.find(":root") {
        out.push_str(&rest[..at]);
        let after = &rest[at + ":root".len()..];
        // The conditions on :root run to the end of its compound selector: whitespace, a comma
        // or the block, outside brackets and parentheses.
        let mut depth = 0usize;
        let mut end = after.len();
        for (i, c) in after.char_indices() {
            match c {
                '[' | '(' => depth += 1,
                ']' | ')' => depth = depth.saturating_sub(1),
                c if depth == 0 && (c.is_whitespace() || c == ',' || c == '{') => {
                    end = i;
                    break;
                }
                _ => {}
            }
        }
        let conditions = &after[..end];
        if conditions.is_empty() {
            out.push_str(":host");
        } else {
            out.push_str(&format!(":host({conditions})"));
        }
        rest = &after[end..];
    }
    out.push_str(rest);
    out.lines()
        .map(|line| match line.strip_prefix("body ") {
            Some(rule) if rule.starts_with('{') => format!(":host {rule}"),
            _ => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_document_selector_moves_to_the_host() {
        let css = styles();
        assert!(!css.contains(":root"), "{css}");
        assert!(!css.lines().any(|l| l.starts_with("body ")), "{css}");
        for want in [
            ":host {\n  color-scheme: light;",
            ":host(:not([data-theme])), :host([data-theme=\"auto\"]) {",
            ":host([data-theme=\"light\"]) {",
            ":host([data-theme=\"dark\"]) {",
            ":host([data-theme=\"high-contrast\"]) {",
            ":host { margin: 0; background: var(--bg); color: var(--fg); }",
            "a { color: var(--accent); }",
        ] {
            assert!(css.contains(want), "missing {want:?} in\n{css}");
        }
    }

    #[test]
    fn every_palette_is_in_it_whole() {
        let css = styles();
        for palette in ikigai_ui::palette::PALETTES {
            assert!(css.contains(&palette.declarations()), "{}", palette.name);
        }
    }

    #[test]
    fn conditions_keep_their_brackets() {
        assert_eq!(
            for_host(":root:not([data-theme]), :root[data-theme=\"a b\"] {}\nbody { x: y; }"),
            ":host(:not([data-theme])), :host([data-theme=\"a b\"]) {}\n:host { x: y; }"
        );
    }
}
