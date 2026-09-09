//! Structural accessibility checks over the built HTML — the half of the gate a parser
//! can decide.
//!
//! The contrast half (`lib.rs`) measures colours; this half reads the pages `mdbook`
//! wrote and refuses the faults that a static document can prove: a missing `lang`, a
//! page without exactly one `h1` in its content, a heading level that skips, an image
//! with no `alt`, a link with no text or with text that says nothing out of context, a
//! duplicate `id`, an ARIA reference to an `id` that does not exist, a table with no
//! header cell, a button or an input with no accessible name, a runnable cell that is
//! not the shape `js/run.js` expects. Each fault names the WCAG 2.2 success criterion it
//! fails, so the message is the citation.
//!
//! ## What this parser is, and is not
//!
//! A tolerant tokenizer over the HTML mdbook 0.5 emits: tags with attributes (quoted,
//! single-quoted or bare), text, comments, `<script>`/`<style>` bodies skipped whole. It
//! does **not** build a tree, does not implement the HTML5 error-recovery algorithm, and
//! does not know about implied end tags — so it tracks nesting only for the few elements
//! the checks need (`main`, `a`, `button`, `table`, `label`). That is enough for a
//! generator's output, which is well-formed by construction; it would be the wrong tool
//! for the open web, and it says so here rather than pretending.
//!
//! Two things it cannot see, stated so nobody trusts it for them. Anything JavaScript
//! adds after load — mdbook's copy-to-clipboard buttons, the cells `js/run.js` builds,
//! the two repairs `js/a11y.js` makes to the search box — is invisible to a static read.
//! The second instrument, `a11y/axe.mjs` (axe-core over a jsdom of the same pages, with
//! the book's scripts run), is what sees those; `pages.yml` runs both. And no parser can
//! judge reading order, plain language, or whether a link's purpose is clear beyond the
//! generic phrases refused below — those stay with a human reviewer.

use std::collections::{BTreeMap, BTreeSet};

/// One fault, with the success criterion it fails.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fault {
    /// WCAG 2.2 success criterion, e.g. `1.1.1`.
    pub criterion: &'static str,
    /// Its name, e.g. `Non-text Content`.
    pub name: &'static str,
    /// What was found, specific enough to fix.
    pub detail: String,
}

impl Fault {
    fn new(criterion: &'static str, name: &'static str, detail: impl Into<String>) -> Self {
        Fault {
            criterion,
            name,
            detail: detail.into(),
        }
    }

    /// One line: `SC 1.1.1 Non-text Content — <detail>`.
    pub fn report(&self) -> String {
        format!("SC {} {} — {}", self.criterion, self.name, self.detail)
    }
}

/// A start tag, as the tokenizer saw it.
#[derive(Debug, Clone)]
struct Tag {
    name: String,
    attrs: BTreeMap<String, String>,
    /// `<br/>`, `<img … />`.
    self_closing: bool,
}

#[derive(Debug, Clone)]
enum Token {
    Start(Tag),
    End(String),
    Text(String),
}

/// Elements HTML defines as void — they never have an end tag.
const VOID: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "source", "track",
    "wbr",
];

/// Tokenize `html`. Comments, doctype, `<script>` and `<style>` bodies are skipped.
fn tokenize(html: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut rest = html;
    let mut text = String::new();
    while let Some(lt) = rest.find('<') {
        text.push_str(&rest[..lt]);
        rest = &rest[lt..];
        if rest.starts_with("<!--") {
            let end = rest.find("-->").map(|e| e + 3).unwrap_or(rest.len());
            rest = &rest[end..];
            continue;
        }
        if rest.starts_with("<!") || rest.starts_with("<?") {
            let end = rest.find('>').map(|e| e + 1).unwrap_or(rest.len());
            rest = &rest[end..];
            continue;
        }
        let Some(gt) = find_tag_end(rest) else {
            break;
        };
        let inner = &rest[1..gt];
        rest = &rest[gt + 1..];
        if !text.is_empty() {
            tokens.push(Token::Text(std::mem::take(&mut text)));
        }
        if let Some(name) = inner.strip_prefix('/') {
            tokens.push(Token::End(name.trim().to_ascii_lowercase()));
            continue;
        }
        let tag = parse_tag(inner);
        let name = tag.name.clone();
        tokens.push(Token::Start(tag));
        // Raw text elements: skip to the matching end tag without tokenizing the body.
        if name == "script" || name == "style" {
            let close = format!("</{name}");
            let lower = rest.to_ascii_lowercase();
            if let Some(at) = lower.find(&close) {
                let after = rest[at..]
                    .find('>')
                    .map(|e| at + e + 1)
                    .unwrap_or(rest.len());
                rest = &rest[after..];
                tokens.push(Token::End(name));
            } else {
                rest = "";
            }
        }
    }
    text.push_str(rest);
    if !text.trim().is_empty() {
        tokens.push(Token::Text(text));
    }
    tokens
}

/// The index of the `>` that closes the tag starting at `rest[0]`, honouring quotes.
fn find_tag_end(rest: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (i, c) in rest.char_indices().skip(1) {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (Some(_), _) => {}
            (None, '"') | (None, '\'') => quote = Some(c),
            (None, '>') => return Some(i),
            _ => {}
        }
    }
    None
}

fn parse_tag(inner: &str) -> Tag {
    let inner = inner.trim();
    let (inner, self_closing) = match inner.strip_suffix('/') {
        Some(stripped) => (stripped.trim(), true),
        None => (inner, false),
    };
    let name_end = inner
        .find(|c: char| c.is_whitespace())
        .unwrap_or(inner.len());
    let name = inner[..name_end].to_ascii_lowercase();
    let mut attrs = BTreeMap::new();
    let mut rest = inner[name_end..].trim_start();
    while !rest.is_empty() {
        let key_end = rest
            .find(|c: char| c.is_whitespace() || c == '=')
            .unwrap_or(rest.len());
        let key = rest[..key_end].to_ascii_lowercase();
        rest = rest[key_end..].trim_start();
        let mut value = String::new();
        if let Some(after_eq) = rest.strip_prefix('=') {
            let after_eq = after_eq.trim_start();
            if let Some(q) = after_eq.chars().next().filter(|c| *c == '"' || *c == '\'') {
                let body = &after_eq[1..];
                let end = body.find(q).unwrap_or(body.len());
                value = body[..end].to_string();
                rest = body[end..].strip_prefix(q).unwrap_or("").trim_start();
            } else {
                let end = after_eq
                    .find(|c: char| c.is_whitespace())
                    .unwrap_or(after_eq.len());
                value = after_eq[..end].to_string();
                rest = after_eq[end..].trim_start();
            }
        }
        if !key.is_empty() {
            attrs.insert(key, value);
        }
    }
    Tag {
        name,
        attrs,
        self_closing,
    }
}

/// Decode the few entities that matter for reading text out of a page.
fn decode(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&nbsp;", " ")
}

/// Link text that names nothing: refused outright (SC 2.4.4). The list is short on
/// purpose — a longer one becomes a style guide, and the point is the words that a
/// screen reader's links list turns into a row of identical entries.
const GENERIC_LINK_TEXT: &[&str] = &["here", "click here", "this", "link", "more", "read more"];

/// Check one built page. `label` names it in reports; `pages_titles` is for the
/// uniqueness check across the book (pass `None` to skip it).
pub fn check_page(label: &str, html: &str) -> Vec<Fault> {
    let tokens = tokenize(html);
    let mut faults = Vec::new();

    // ── html lang (SC 3.1.1) ────────────────────────────────────────────────
    let html_tag = tokens.iter().find_map(|t| match t {
        Token::Start(tag) if tag.name == "html" => Some(tag),
        _ => None,
    });
    match html_tag.and_then(|t| t.attrs.get("lang")) {
        Some(lang) if !lang.trim().is_empty() => {}
        _ => faults.push(Fault::new(
            "3.1.1",
            "Language of Page",
            format!("{label}: <html> has no non-empty lang attribute"),
        )),
    }

    // ── title (SC 2.4.2) ────────────────────────────────────────────────────
    if title_of(&tokens).is_none() {
        faults.push(Fault::new(
            "2.4.2",
            "Page Titled",
            format!("{label}: no non-empty <title>"),
        ));
    }

    // One pass over the token stream, tracking the little nesting the checks need.
    let mut in_main = false;
    let mut h1_in_main = 0usize;
    let mut last_level = 0u8;
    let mut ids: BTreeMap<String, usize> = BTreeMap::new();
    let mut references: Vec<(String, String, String)> = Vec::new(); // (attr, id, element)
    let mut label_for: BTreeSet<String> = BTreeSet::new();
    let mut link: Option<(Tag, String)> = None; // open <a> and its collected text
    let mut button: Option<(Tag, String)> = None;
    let mut table: Option<bool> = None; // open <table>: has a <th> yet?
    let mut cell: Option<Tag> = None; // open <div class="ikigai-run">
    let mut cell_has_expected = false;
    let mut cell_depth = 0usize;
    let mut inputs: Vec<Tag> = Vec::new();

    for token in &tokens {
        match token {
            Token::Start(tag) => {
                if let Some(id) = tag.attrs.get("id") {
                    *ids.entry(id.clone()).or_default() += 1;
                }
                for attr in ["aria-controls", "aria-describedby", "aria-labelledby"] {
                    if let Some(value) = tag.attrs.get(attr) {
                        for id in value.split_whitespace() {
                            references.push((attr.to_string(), id.to_string(), tag.name.clone()));
                        }
                    }
                }
                match tag.name.as_str() {
                    "main" => in_main = true,
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" if in_main => {
                        let level = tag.name.as_bytes()[1] - b'0';
                        if level == 1 {
                            h1_in_main += 1;
                        }
                        if last_level != 0 && level > last_level + 1 {
                            faults.push(Fault::new(
                                "1.3.1",
                                "Info and Relationships",
                                format!(
                                    "{label}: heading level skips from h{last_level} to h{level} \
                                     — a screen reader's heading list loses the structure"
                                ),
                            ));
                        }
                        last_level = level;
                    }
                    "img" => {
                        if !tag.attrs.contains_key("alt") {
                            faults.push(Fault::new(
                                "1.1.1",
                                "Non-text Content",
                                format!(
                                    "{label}: <img src=\"{}\"> has no alt attribute (empty alt \
                                     marks a decorative image; absent alt marks nothing)",
                                    tag.attrs.get("src").cloned().unwrap_or_default()
                                ),
                            ));
                        }
                    }
                    "a" => {
                        if tag.attrs.contains_key("href") {
                            link = Some((tag.clone(), String::new()));
                        }
                    }
                    "button" => button = Some((tag.clone(), String::new())),
                    "table" => table = Some(false),
                    "th" => {
                        if let Some(has) = table.as_mut() {
                            *has = true;
                        }
                    }
                    "label" => {
                        if let Some(target) = tag.attrs.get("for") {
                            label_for.insert(target.clone());
                        }
                    }
                    "input" => {
                        if tag.attrs.get("type").map(String::as_str) != Some("hidden") {
                            inputs.push(tag.clone());
                        }
                    }
                    "div"
                        if tag
                            .attrs
                            .get("class")
                            .is_some_and(|c| c.split_whitespace().any(|c| c == "ikigai-run")) =>
                    {
                        cell = Some(tag.clone());
                        cell_has_expected = false;
                        cell_depth = 0;
                    }
                    "div" if cell.is_some() => cell_depth += 1,
                    "pre"
                        if cell.is_some()
                            && tag.attrs.get("class").is_some_and(|c| {
                                c.split_whitespace().any(|c| c == "ikigai-run-expected")
                            }) =>
                    {
                        cell_has_expected = true;
                    }
                    _ => {}
                }
                // Alt text inside a link or button counts toward its name.
                if tag.name == "img" {
                    if let Some(alt) = tag.attrs.get("alt") {
                        if let Some((_, text)) = link.as_mut() {
                            text.push_str(alt);
                        }
                        if let Some((_, text)) = button.as_mut() {
                            text.push_str(alt);
                        }
                    }
                }
                if tag.self_closing || VOID.contains(&tag.name.as_str()) {
                    continue;
                }
            }
            Token::End(name) => match name.as_str() {
                "main" => in_main = false,
                "a" => {
                    if let Some((tag, text)) = link.take() {
                        let text = decode(text.trim());
                        let named = !text.is_empty()
                            || tag
                                .attrs
                                .get("aria-label")
                                .is_some_and(|l| !l.trim().is_empty())
                            || tag.attrs.contains_key("aria-labelledby")
                            || tag.attrs.get("title").is_some_and(|l| !l.trim().is_empty());
                        if !named {
                            faults.push(Fault::new(
                                "2.4.4",
                                "Link Purpose (In Context)",
                                format!(
                                    "{label}: <a href=\"{}\"> has no text, aria-label or title — it \
                                     reads as \"link\" and nothing else",
                                    tag.attrs.get("href").cloned().unwrap_or_default()
                                ),
                            ));
                        } else if GENERIC_LINK_TEXT
                            .contains(&text.to_ascii_lowercase().trim_end_matches('.'))
                        {
                            faults.push(Fault::new(
                                "2.4.4",
                                "Link Purpose (In Context)",
                                format!(
                                    "{label}: link text \"{text}\" (to {}) names nothing out of \
                                     context — say what it points at",
                                    tag.attrs.get("href").cloned().unwrap_or_default()
                                ),
                            ));
                        }
                    }
                }
                "button" => {
                    if let Some((tag, text)) = button.take() {
                        let named = !decode(text.trim()).is_empty()
                            || tag
                                .attrs
                                .get("aria-label")
                                .is_some_and(|l| !l.trim().is_empty())
                            || tag.attrs.contains_key("aria-labelledby")
                            || tag.attrs.get("title").is_some_and(|l| !l.trim().is_empty());
                        if !named {
                            faults.push(Fault::new(
                                "4.1.2",
                                "Name, Role, Value",
                                format!(
                                    "{label}: <button id=\"{}\"> has no accessible name",
                                    tag.attrs.get("id").cloned().unwrap_or_default()
                                ),
                            ));
                        }
                    }
                }
                "table" => {
                    if table.take() == Some(false) {
                        faults.push(Fault::new(
                            "1.3.1",
                            "Info and Relationships",
                            format!("{label}: a <table> has no <th> — its header row is not one"),
                        ));
                    }
                }
                "div" if cell.is_some() => {
                    if cell_depth == 0 {
                        let tag = cell.take().expect("open cell");
                        if !tag
                            .attrs
                            .get("data-cmd")
                            .is_some_and(|c| !c.trim().is_empty())
                        {
                            faults.push(Fault::new(
                                "4.1.2",
                                "Name, Role, Value",
                                format!(
                                    "{label}: a <div class=\"ikigai-run\"> has no data-cmd, so \
                                     js/run.js builds a Run button that runs nothing"
                                ),
                            ));
                        }
                        if !cell_has_expected {
                            faults.push(Fault::new(
                                "4.1.3",
                                "Status Messages",
                                format!(
                                    "{label}: a runnable cell has no <pre \
                                     class=\"ikigai-run-expected\"> — with the wasm absent its \
                                     status region would announce nothing"
                                ),
                            ));
                        }
                    } else {
                        cell_depth -= 1;
                    }
                }
                _ => {}
            },
            Token::Text(text) => {
                if let Some((_, collected)) = link.as_mut() {
                    collected.push_str(text);
                }
                if let Some((_, collected)) = button.as_mut() {
                    collected.push_str(text);
                }
            }
        }
    }

    // ── one h1 in the content (SC 1.3.1 / 2.4.6) ────────────────────────────
    //
    // Counted inside <main> only: mdbook puts the book's title in an <h1> in the menu
    // bar of every page, which is upstream's template and deliberately left alone
    // (README, "Left alone, deliberately"). The chapter's own title is the one that has
    // to be exactly one.
    if h1_in_main != 1 {
        faults.push(Fault::new(
            "1.3.1",
            "Info and Relationships",
            format!("{label}: {h1_in_main} <h1> elements inside <main>; a chapter has exactly one"),
        ));
    }

    // ── duplicate ids (SC 1.3.1 / 4.1.2 — every ARIA relationship rides on ids) ──
    for (id, count) in &ids {
        if *count > 1 {
            faults.push(Fault::new(
                "4.1.2",
                "Name, Role, Value",
                format!("{label}: id=\"{id}\" appears {count} times — an aria-* reference to it is ambiguous"),
            ));
        }
    }

    // ── dangling ARIA references (SC 1.3.1) ─────────────────────────────────
    for (attr, id, element) in &references {
        if !ids.contains_key(id) {
            faults.push(Fault::new(
                "1.3.1",
                "Info and Relationships",
                format!("{label}: <{element} {attr}=\"{id}\"> names an id that is not on the page"),
            ));
        }
    }

    // ── inputs with a name (SC 4.1.2 / 3.3.2) ───────────────────────────────
    for input in &inputs {
        let id = input.attrs.get("id").cloned().unwrap_or_default();
        let named = input
            .attrs
            .get("aria-label")
            .is_some_and(|l| !l.trim().is_empty())
            || input.attrs.contains_key("aria-labelledby")
            || (!id.is_empty() && label_for.contains(&id))
            || input
                .attrs
                .get("title")
                .is_some_and(|l| !l.trim().is_empty());
        if !named {
            faults.push(Fault::new(
                "4.1.2",
                "Name, Role, Value",
                format!(
                    "{label}: <input id=\"{id}\"> has no label, aria-label or aria-labelledby \
                     (a placeholder is not a name)"
                ),
            ));
        }
    }

    faults
}

/// The page's `<title>` text, if non-empty.
fn title_of(tokens: &[Token]) -> Option<String> {
    let mut in_title = false;
    for token in tokens {
        match token {
            Token::Start(tag) if tag.name == "title" => in_title = true,
            Token::Text(text) if in_title => {
                let text = decode(text.trim());
                return if text.is_empty() { None } else { Some(text) };
            }
            Token::End(name) if name == "title" => return None,
            _ => {}
        }
    }
    None
}

/// The `<title>` of a page as a string — for the cross-page uniqueness check.
pub fn page_title(html: &str) -> Option<String> {
    title_of(&tokenize(html))
}

/// Faults across pages: two pages with the same title (SC 2.4.2 asks that a title
/// describe the page, and two identical titles describe neither).
pub fn check_titles(titles: &[(String, Option<String>)]) -> Vec<Fault> {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (label, title) in titles {
        if let Some(title) = title {
            seen.entry(title.clone()).or_default().push(label.clone());
        }
    }
    seen.into_iter()
        .filter(|(_, pages)| pages.len() > 1)
        .map(|(title, pages)| {
            Fault::new(
                "2.4.2",
                "Page Titled",
                format!("title \"{title}\" is shared by {}", pages.join(", ")),
            )
        })
        .collect()
}

/// The repairs `js/a11y.js` promises to make at load, each keyed on the thing it
/// repairs. The static checks above report the search box's missing name and dangling
/// description as faults of the page mdbook wrote — which they are — and this is how the
/// gate knows the book answers them: the script that is wired into every page must still
/// contain each repair, by name. Delete the repair and the gate goes red, which is the
/// point; the static fault is then not a false alarm but a regression.
pub const A11Y_JS_REPAIRS: &[(&str, &str)] = &[
    ("mdbook-searchbar", "setAttribute(\"aria-label\""),
    ("aria-describedby", "setAttribute(\"aria-describedby\""),
];

/// Which of a page's faults `js/a11y.js` repairs at load, given the script's source.
/// Returns the faults that remain.
pub fn without_scripted_repairs(faults: Vec<Fault>, a11y_js: &str) -> Vec<Fault> {
    let repairs_present = A11Y_JS_REPAIRS
        .iter()
        .all(|(_, needle)| a11y_js.contains(needle));
    if !repairs_present {
        return faults;
    }
    faults
        .into_iter()
        .filter(|fault| {
            let search_name =
                fault.criterion == "4.1.2" && fault.detail.contains("mdbook-searchbar");
            let search_description = fault.criterion == "1.3.1"
                && fault
                    .detail
                    .contains("aria-describedby=\"searchresults-header\"");
            !(search_name || search_description)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"<!DOCTYPE html><html lang="en"><head><title>A page - Book</title></head>
<body><h1 class="menu-title">Book</h1><main><h1 id="a">A page</h1><h2>Part</h2><h3>Detail</h3>
<a href="x.html">the x chapter</a><img src="p.png" alt="a plot"><table><tr><th>k</th></tr></table>
<button type="button" aria-label="Change theme"></button><label for="q">Search</label><input id="q">
<div class="ikigai-run" data-cmd='source urn:x'><pre class="ikigai-run-expected">x</pre></div>
</main></body></html>"#;

    #[test]
    fn a_sound_page_has_no_faults() {
        assert_eq!(check_page("good.html", GOOD), vec![]);
    }

    #[test]
    fn a_missing_lang_is_a_fault_of_3_1_1() {
        let html = GOOD.replace(" lang=\"en\"", "");
        let faults = check_page("p", &html);
        assert!(faults.iter().any(|f| f.criterion == "3.1.1"), "{faults:?}");
    }

    #[test]
    fn a_skipped_heading_level_is_a_fault_of_1_3_1() {
        let html = GOOD.replace("<h2>Part</h2>", "");
        let faults = check_page("p", &html);
        assert!(
            faults
                .iter()
                .any(|f| f.criterion == "1.3.1" && f.detail.contains("h1 to h3")),
            "{faults:?}"
        );
    }

    #[test]
    fn the_menu_bar_h1_outside_main_does_not_count() {
        // Two <h1> on the page, one inside <main>: exactly what mdbook emits, and sound.
        assert!(check_page("p", GOOD).is_empty());
        let two = GOOD.replace("<h2>Part</h2>", "<h1>Another</h1>");
        let faults = check_page("p", &two);
        assert!(
            faults.iter().any(|f| f.detail.contains("2 <h1>")),
            "{faults:?}"
        );
    }

    #[test]
    fn an_image_without_alt_is_a_fault_but_an_empty_alt_is_decorative() {
        let missing = GOOD.replace(" alt=\"a plot\"", "");
        assert!(check_page("p", &missing)
            .iter()
            .any(|f| f.criterion == "1.1.1"));
        let decorative = GOOD.replace("alt=\"a plot\"", "alt=\"\"");
        assert!(check_page("p", &decorative).is_empty());
    }

    #[test]
    fn empty_and_generic_link_text_are_faults_of_2_4_4() {
        let empty = GOOD.replace(">the x chapter<", "><");
        assert!(check_page("p", &empty)
            .iter()
            .any(|f| f.criterion == "2.4.4"));
        let here = GOOD.replace("the x chapter", "here");
        assert!(check_page("p", &here)
            .iter()
            .any(|f| f.detail.contains("\"here\"")));
        // An image's alt is the link's text.
        let image_link = GOOD.replace(
            ">the x chapter<",
            "><img src=\"i.png\" alt=\"the x chapter\"><",
        );
        assert!(check_page("p", &image_link).is_empty());
    }

    #[test]
    fn duplicate_ids_and_dangling_aria_references_are_faults() {
        let dup = GOOD.replace("<h2>Part</h2>", "<h2 id=\"a\">Part</h2>");
        assert!(check_page("p", &dup)
            .iter()
            .any(|f| f.detail.contains("id=\"a\" appears 2")));
        let dangling = GOOD.replace(
            "<input id=\"q\">",
            "<input id=\"q\" aria-describedby=\"nowhere\">",
        );
        assert!(check_page("p", &dangling)
            .iter()
            .any(|f| f.detail.contains("nowhere")));
    }

    #[test]
    fn a_table_without_a_header_cell_is_a_fault() {
        let html = GOOD.replace("<th>k</th>", "<td>k</td>");
        assert!(check_page("p", &html)
            .iter()
            .any(|f| f.detail.contains("<table>")));
    }

    #[test]
    fn unnamed_buttons_and_inputs_are_faults_of_4_1_2() {
        let button = GOOD.replace(" aria-label=\"Change theme\"", "");
        assert!(check_page("p", &button)
            .iter()
            .any(|f| f.detail.contains("<button")));
        let input = GOOD.replace("<label for=\"q\">Search</label>", "");
        assert!(check_page("p", &input)
            .iter()
            .any(|f| f.detail.contains("<input id=\"q\"")));
        // A placeholder does not name it.
        let placeholder = input.replace(
            "<input id=\"q\">",
            "<input id=\"q\" placeholder=\"Search\">",
        );
        assert!(check_page("p", &placeholder)
            .iter()
            .any(|f| f.detail.contains("placeholder")));
    }

    #[test]
    fn a_cell_without_its_expected_output_is_a_fault_of_4_1_3() {
        let html = GOOD.replace("<pre class=\"ikigai-run-expected\">x</pre>", "");
        assert!(check_page("p", &html)
            .iter()
            .any(|f| f.criterion == "4.1.3"));
        let no_cmd = GOOD.replace(" data-cmd='source urn:x'", "");
        assert!(check_page("p", &no_cmd)
            .iter()
            .any(|f| f.detail.contains("data-cmd")));
    }

    #[test]
    fn two_pages_with_one_title_are_a_fault_of_2_4_2() {
        let faults = check_titles(&[
            ("a.html".into(), Some("Same".into())),
            ("b.html".into(), Some("Same".into())),
            ("c.html".into(), Some("Other".into())),
        ]);
        assert_eq!(faults.len(), 1);
        assert!(faults[0].detail.contains("a.html, b.html"));
    }

    #[test]
    fn the_scripted_repairs_are_credited_only_when_the_script_still_makes_them() {
        let page = GOOD
            .replace("<label for=\"q\">Search</label><input id=\"q\">",
                     "<input id=\"mdbook-searchbar\" placeholder=\"Search\" aria-describedby=\"searchresults-header\">");
        let faults = check_page("p", &page);
        assert_eq!(faults.len(), 2, "{faults:?}");
        let js =
            "input.setAttribute(\"aria-label\", x); input.setAttribute(\"aria-describedby\", y);";
        assert!(without_scripted_repairs(faults.clone(), js).is_empty());
        assert_eq!(without_scripted_repairs(faults, "nothing here").len(), 2);
    }

    #[test]
    fn scripts_and_comments_are_not_content() {
        let html = GOOD.replace(
            "<main>",
            "<main><!-- <img src=\"c\"> --><script>var s = '<img src=\"s\">';</script>",
        );
        assert!(check_page("p", &html).is_empty());
    }
}
