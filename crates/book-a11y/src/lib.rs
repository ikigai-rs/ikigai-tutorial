//! Measure the contrast of the palette the book is actually served in.
//!
//! `mdbook` is sound by default in most of the ways that matter — `<html lang>`, a
//! `<main>`, a labelled `<nav>`, labelled theme buttons, no image without `alt` — and none
//! of that needed doing here. Colour is the exception, because a theme is a palette
//! somebody chose by eye and **nothing in a static site generator measures one**.
//!
//! ## Why this reads generated CSS rather than a table of colours
//!
//! The obvious version of this check is a list of hex pairs in a Rust file. That version
//! is worse than no check: mdbook ships the palette, mdbook can change it in a release,
//! and a table copied out of it goes stale silently while the test stays green — a gate
//! that covers less than it looks like it does.
//!
//! So this reads `book/css/variables-*.css`, which is what mdbook just generated, and then
//! the book's own `additional-css` files **in the order `book.toml` lists them**, because
//! that is the order the cascade applies and a repair is only real if it lands last. The
//! cost is that it can only run after `mdbook build`, which is why it is a binary invoked
//! by `scripts/test-books.sh` rather than a `cargo test`.
//!
//! ## Two floors, because WCAG has two
//!
//! Body text is measured against **4.5:1** (WCAG 2.2 SC 1.4.3, AA). A non-text UI
//! component — an icon, a control's border — is measured against **3:1** (SC 1.4.11).
//! Holding an icon to the text floor would be inventing a rule; holding text to the icon
//! floor would be missing the point.

use std::collections::BTreeMap;

use ikigai_a11y::color::{ratio, Rgba};

/// The themes mdbook ships, as the class each one is served under.
///
/// A reader picks any of these from the theme menu, so the default being sound is not the
/// same as the book being sound.
pub const THEMES: &[&str] = &[".light", ".rust", ".coal", ".navy", ".ayu"];

/// One thing the palette has to get right, and why it is that thing.
pub struct Pair {
    /// The foreground custom property, e.g. `--links`.
    pub foreground: &'static str,
    /// The background it lands on.
    pub background: &'static str,
    /// The ratio it has to clear.
    pub floor: f64,
    /// What a reader is looking at when this pair is on screen.
    pub what: &'static str,
}

/// AA for text (SC 1.4.3).
const TEXT: f64 = 4.5;
/// AA for a non-text UI component (SC 1.4.11).
const COMPONENT: f64 = 3.0;

/// The pairs a reader of *this* book actually meets.
///
/// Deliberately not every combination in the stylesheet: a pair nobody sees is noise, and
/// a gate with noise in it gets muted. Blockquote alert colours (`--blockquote-tip` and
/// friends) are absent because this book writes plain `>` blockquotes rather than GitHub
/// alerts — add them here on the day it uses one.
pub const PAIRS: &[Pair] = &[
    Pair {
        foreground: "--fg",
        background: "--bg",
        floor: TEXT,
        what: "body text",
    },
    Pair {
        foreground: "--links",
        background: "--bg",
        floor: TEXT,
        what: "a link in the text — every cross-reference in the book",
    },
    Pair {
        foreground: "--inline-code-color",
        background: "--bg",
        floor: TEXT,
        what: "`inline code`, which this book uses in nearly every sentence",
    },
    Pair {
        foreground: "--fg",
        background: "--quote-bg",
        floor: TEXT,
        what: "text in a blockquote — every ⚠ trap in the book",
    },
    Pair {
        foreground: "--links",
        background: "--quote-bg",
        floor: TEXT,
        what: "a link inside a blockquote",
    },
    Pair {
        foreground: "--inline-code-color",
        background: "--quote-bg",
        floor: TEXT,
        what: "inline code inside a blockquote",
    },
    Pair {
        foreground: "--fg",
        background: "--table-header-bg",
        floor: TEXT,
        what: "a table header",
    },
    Pair {
        foreground: "--fg",
        background: "--table-alternate-bg",
        floor: TEXT,
        what: "every other table row",
    },
    Pair {
        foreground: "--sidebar-fg",
        background: "--sidebar-bg",
        floor: TEXT,
        what: "a chapter title in the sidebar",
    },
    Pair {
        foreground: "--sidebar-active",
        background: "--sidebar-bg",
        floor: TEXT,
        what: "the chapter you are on, in the sidebar",
    },
    Pair {
        foreground: "--sidebar-non-existant",
        background: "--sidebar-bg",
        floor: TEXT,
        what: "a draft chapter in the sidebar",
    },
    Pair {
        foreground: "--searchbar-fg",
        background: "--searchbar-bg",
        floor: TEXT,
        what: "what you type into the search box",
    },
    Pair {
        foreground: "--searchresults-header-fg",
        background: "--bg",
        floor: TEXT,
        what: "the search results header",
    },
    Pair {
        foreground: "--icons",
        background: "--bg",
        floor: COMPONENT,
        what: "the theme and search buttons (a UI component, so 3:1)",
    },
];

/// A pair that does not clear its floor.
#[derive(Debug, Clone)]
pub struct Finding {
    /// The theme class, e.g. `.rust`.
    pub theme: String,
    /// The foreground property.
    pub foreground: String,
    /// The background property.
    pub background: String,
    /// The colours those resolved to, as CSS.
    pub colors: (String, String),
    /// The measured ratio.
    pub ratio: f64,
    /// The floor it had to clear.
    pub floor: f64,
    /// What a reader is looking at.
    pub what: String,
}

impl Finding {
    /// One line, naming the fix as well as the fault.
    pub fn report(&self) -> String {
        format!(
            "{} {} on {} — {:.2}:1, needs {:.1}:1 ({} on {}): {}",
            self.theme,
            self.foreground,
            self.background,
            self.ratio,
            self.floor,
            self.colors.0,
            self.colors.1,
            self.what
        )
    }
}

/// Every custom property declared for a theme, later files winning — the cascade, for the
/// one case this needs: whole declarations replacing earlier ones.
pub type Palette = BTreeMap<String, BTreeMap<String, String>>;

/// Parse `--name: value;` declarations, per selector, out of one stylesheet.
///
/// A deliberately small parser. It understands selector blocks and custom properties and
/// nothing else, because that is all a palette is; anything richer would be a CSS engine,
/// and a wrong CSS engine measures a page nobody is served.
pub fn parse_variables(css: &str) -> Palette {
    // Comments come out first, and before anything else looks at the text: a `/* Themes */`
    // sitting above `.ayu {` otherwise becomes part of the selector, and that theme
    // silently drops out of the survey while every other one is measured. Which is the
    // failure mode this whole crate exists to avoid — a gate quietly covering less than it
    // appears to.
    let css = strip_comments(css);
    let mut palette: Palette = BTreeMap::new();
    let mut chars = css.chars().peekable();
    let mut selector = String::new();

    while let Some(c) = chars.next() {
        match c {
            '{' => {
                let mut body = String::new();
                let mut depth = 1;
                for c in chars.by_ref() {
                    match c {
                        '{' => {
                            depth += 1;
                            body.push(c);
                        }
                        '}' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                            body.push(c);
                        }
                        _ => body.push(c),
                    }
                }
                let selectors = selector.trim().to_string();
                selector.clear();
                // `@media (...) { .theme { … } }` — recurse rather than guess.
                if selectors.starts_with('@') {
                    for (key, values) in parse_variables(&body) {
                        palette.entry(key).or_default().extend(values);
                    }
                    continue;
                }
                let declarations = parse_declarations(&body);
                for one in selectors.split(',') {
                    let one = one.trim();
                    if one.is_empty() {
                        continue;
                    }
                    palette
                        .entry(one.to_string())
                        .or_default()
                        .extend(declarations.clone());
                }
            }
            '}' => selector.clear(),
            _ => selector.push(c),
        }
    }
    palette
}

fn parse_declarations(body: &str) -> BTreeMap<String, String> {
    let mut out = BTreeMap::new();
    for declaration in strip_comments(body).split(';') {
        let Some((name, value)) = declaration.split_once(':') else {
            continue;
        };
        let name = name.trim();
        if !name.starts_with("--") {
            continue;
        }
        out.insert(name.to_string(), value.trim().to_string());
    }
    out
}

fn strip_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("/*") {
        out.push_str(&rest[..start]);
        match rest[start..].find("*/") {
            Some(end) => rest = &rest[start + end + 2..],
            None => return out,
        }
    }
    out.push_str(rest);
    out
}

/// Merge stylesheets in cascade order: what a later file declares wins.
pub fn merge(sheets: &[String]) -> Palette {
    let mut merged: Palette = BTreeMap::new();
    for sheet in sheets {
        for (selector, values) in parse_variables(sheet) {
            merged.entry(selector).or_default().extend(values);
        }
    }
    merged
}

/// Resolve one property to a colour, following `var(--other)` within the same theme.
pub fn color(values: &BTreeMap<String, String>, name: &str) -> Option<Rgba> {
    let mut name = name.to_string();
    // A short chain, and a bound on it: a cycle in a stylesheet must not hang the gate.
    for _ in 0..8 {
        let raw = values.get(&name)?.trim().to_string();
        if let Some(inner) = raw.strip_prefix("var(").and_then(|r| r.strip_suffix(')')) {
            name = inner.split(',').next().unwrap_or(inner).trim().to_string();
            continue;
        }
        return parse_color(&raw);
    }
    None
}

/// Parse the colour notations mdbook's own themes are written in.
///
/// `hsl()` is here because mdbook writes several of its grounds that way — `--bg` in the
/// `rust` theme is `hsl(60, 9%, 87%)` — and `ikigai-a11y` reads hex only, which is the
/// notation its own producers emit.
pub fn parse_color(value: &str) -> Option<Rgba> {
    let value = value.trim();
    if let Some(rest) = value.strip_prefix("hsl(").and_then(|v| v.strip_suffix(')')) {
        let parts: Vec<&str> = rest
            .split(&[',', ' '][..])
            .filter(|p| !p.is_empty())
            .collect();
        if parts.len() < 3 {
            return None;
        }
        let h: f64 = parts[0].parse().ok()?;
        let s: f64 = parts[1].trim_end_matches('%').parse::<f64>().ok()? / 100.0;
        let l: f64 = parts[2].trim_end_matches('%').parse::<f64>().ok()? / 100.0;
        return Some(from_hsl(h, s, l));
    }
    Rgba::parse(value).ok()
}

/// CSS `hsl()` to sRGB, per the CSS Color specification.
fn from_hsl(h: f64, s: f64, l: f64) -> Rgba {
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let sector = (h.rem_euclid(360.0)) / 60.0;
    let x = c * (1.0 - ((sector % 2.0) - 1.0).abs());
    let (r, g, b) = match sector as u8 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };
    let m = l - c / 2.0;
    let byte = |v: f64| ((v + m).clamp(0.0, 1.0) * 255.0).round() as u8;
    Rgba::rgb(byte(r), byte(g), byte(b))
}

/// Measure every pair in every theme, and report the ones below their floor.
pub fn check(palette: &Palette) -> Vec<Finding> {
    let mut findings = Vec::new();
    for theme in THEMES {
        let Some(values) = palette.get(*theme) else {
            continue;
        };
        for pair in PAIRS {
            let (Some(fg), Some(bg)) = (
                color(values, pair.foreground),
                color(values, pair.background),
            ) else {
                continue;
            };
            // A translucent foreground is what the eye sees over its own ground.
            let fg = fg.over(bg);
            let measured = ratio(fg, bg);
            if measured < pair.floor {
                findings.push(Finding {
                    theme: (*theme).to_string(),
                    foreground: pair.foreground.to_string(),
                    background: pair.background.to_string(),
                    colors: (fg.to_css(), bg.to_css()),
                    ratio: measured,
                    floor: pair.floor,
                    what: pair.what.to_string(),
                });
            }
        }
    }
    findings
}

/// The nearest colour along the black/white axis that would clear a floor.
///
/// Not a design opinion — a starting point with a number attached. It moves the colour
/// toward black on a light ground and toward white on a dark one, in one-percent steps,
/// and stops at the first value that clears; the human picking the final colour still has
/// to look at it, and the gate is what says whether the choice worked.
pub fn nearest_passing(foreground: Rgba, background: Rgba, floor: f64) -> Option<Rgba> {
    let toward_white = background.luminance() < 0.5;
    for step in 0..=100 {
        let k = step as f64 / 100.0;
        let mix = |v: u8| {
            let v = v as f64;
            let target = if toward_white { 255.0 } else { 0.0 };
            (v + (target - v) * k).round() as u8
        };
        let candidate = Rgba::rgb(mix(foreground.r), mix(foreground.g), mix(foreground.b));
        if ratio(candidate, background) >= floor {
            return Some(candidate);
        }
    }
    None
}

/// For one failing pair, the two single-property repairs that would clear it.
///
/// Both ends are offered because which one is right is a judgement about the page: a
/// foreground shared by a dozen passing pairs is the expensive one to move, and sometimes
/// the ground is simply too close to the text.
pub fn suggest(finding: &Finding) -> (Option<String>, Option<String>) {
    let (Some(fg), Some(bg)) = (
        parse_color(&finding.colors.0),
        parse_color(&finding.colors.1),
    ) else {
        return (None, None);
    };
    let foreground = nearest_passing(fg, bg, finding.floor).map(|c| c.to_css());
    let background = nearest_passing(bg, fg, finding.floor).map(|c| c.to_css());
    (foreground, background)
}

/// Every pair measured, whether it passes or not — what the survey prints.
pub fn measure_all(palette: &Palette) -> Vec<(String, &'static Pair, Option<f64>)> {
    let mut out = Vec::new();
    for theme in THEMES {
        let Some(values) = palette.get(*theme) else {
            continue;
        };
        for pair in PAIRS {
            let measured = match (
                color(values, pair.foreground),
                color(values, pair.background),
            ) {
                (Some(fg), Some(bg)) => Some(ratio(fg.over(bg), bg)),
                _ => None,
            };
            out.push(((*theme).to_string(), pair, measured));
        }
    }
    out
}

/// What a built page must still contain for the skip link to be a skip link.
///
/// The link is JavaScript reaching into markup this repository does not own, so the thing
/// that can rot silently is the id it aims at: mdbook 0.5 renamed that element (`#content`
/// became `#mdbook-content`), and a skip link pointing at nothing looks exactly like a
/// working one — it takes focus, it looks focused, and it goes nowhere.
pub fn skip_link_faults(html: &str) -> Result<(), Vec<String>> {
    let mut faults = Vec::new();
    if !html.contains("id=\"mdbook-content\"") {
        faults.push(
            "`id=\"mdbook-content\"` — the element js/a11y.js aims the skip link at. mdbook \
             has renamed it before; find the new id and change it in both js/a11y.js and \
             the `#mdbook-content:focus` rule in css/a11y.css"
                .to_string(),
        );
    }
    if !html.contains("a11y") {
        faults.push(
            "any reference to the a11y assets — `additional-css` and `additional-js` in \
             book.toml are what put them on the page"
                .to_string(),
        );
    }
    if faults.is_empty() {
        Ok(())
    } else {
        Err(faults)
    }
}

/// The `additional-css` entries `book.toml` lists, in order.
///
/// A three-line scan rather than a TOML dependency: the value is a single-line array of
/// quoted paths, and the alternative is a parser for a file format this crate otherwise
/// has no interest in.
pub fn additional_css(book_toml: &str) -> Vec<String> {
    for line in book_toml.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("additional-css") else {
            continue;
        };
        let Some(rest) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        let rest = rest.trim();
        let inner = rest.trim_start_matches('[').trim_end_matches(']');
        return inner
            .split(',')
            .map(|entry| entry.trim().trim_matches(['"', '\'']).to_string())
            .filter(|entry| !entry.is_empty())
            .collect();
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hsl_is_read_the_way_a_browser_reads_it() {
        // mdbook writes the `rust` theme's ground this way, and it is the same colour as
        // that theme's `--theme-popup-bg`, which is written as hex — so the two notations
        // agreeing is a real check rather than a restatement of the formula.
        assert_eq!(
            parse_color("hsl(60, 9%, 87%)").unwrap(),
            Rgba::parse("#e1e1db").unwrap()
        );
    }

    #[test]
    fn a_later_stylesheet_wins_because_the_cascade_says_so() {
        let base = ".rust { --links: #2b79a2; --fg: #262625; }".to_string();
        let over = ".rust { --links: #1e5571; }".to_string();
        let palette = merge(&[base, over]);
        let values = &palette[".rust"];
        assert_eq!(values["--links"], "#1e5571");
        assert_eq!(values["--fg"], "#262625", "an untouched property survives");
    }

    #[test]
    fn a_var_reference_is_followed() {
        let palette =
            parse_variables(".x { --sidebar-fg: #c8c9db; --scrollbar: var(--sidebar-fg); }");
        let values = &palette[".x"];
        assert_eq!(
            color(values, "--scrollbar").unwrap(),
            Rgba::parse("#c8c9db").unwrap()
        );
    }

    #[test]
    fn a_comment_is_not_a_declaration() {
        let palette = parse_variables(".x { /* --links: #000; */ --links: #fff; }");
        assert_eq!(palette[".x"]["--links"], "#fff");
    }

    #[test]
    fn a_comment_above_a_selector_does_not_swallow_it() {
        // mdbook writes `/* Themes */` above `.ayu {`, and gluing the two together drops
        // that theme out of the survey without failing anything.
        let palette = parse_variables("/* Themes */\n\n.ayu { --bg: #0f1419; }");
        assert!(palette.contains_key(".ayu"), "{:?}", palette.keys());
    }

    #[test]
    fn a_grouped_selector_declares_for_each_of_its_parts() {
        let palette = parse_variables(".light, html:not(.js) { --bg: #fff; }");
        assert_eq!(palette[".light"]["--bg"], "#fff");
        assert_eq!(palette["html:not(.js)"]["--bg"], "#fff");
    }

    #[test]
    fn a_failing_pair_is_found_and_a_passing_one_is_not() {
        // #777 on white is 4.48:1 — below the text floor by two hundredths, which is the
        // kind of miss an eye cannot make and this can.
        let palette = parse_variables(".rust { --fg: #777777; --bg: #ffffff; }");
        let findings = check(&palette);
        assert_eq!(findings.len(), 1, "{findings:?}");
        assert_eq!(findings[0].foreground, "--fg");
        assert!(findings[0].ratio < 4.5);

        let palette = parse_variables(".rust { --fg: #767676; --bg: #ffffff; }");
        assert!(check(&palette).is_empty(), "4.54:1 clears the floor");
    }

    #[test]
    fn a_suggestion_actually_clears_the_floor() {
        let ground = Rgba::parse("#e1e1db").unwrap();
        let link = Rgba::parse("#2b79a2").unwrap();
        assert!(ratio(link, ground) < 4.5, "the starting point fails");
        let fixed = nearest_passing(link, ground, 4.5).expect("a darker blue exists");
        assert!(
            ratio(fixed, ground) >= 4.5,
            "{} is still short",
            fixed.to_css()
        );
    }

    #[test]
    fn a_suggestion_on_a_dark_ground_goes_the_other_way() {
        let ground = Rgba::parse("#131516").unwrap();
        let link = Rgba::parse("#2b79a2").unwrap();
        let fixed = nearest_passing(link, ground, 4.5).expect("a lighter blue exists");
        assert!(fixed.luminance() > link.luminance(), "it lightened");
        assert!(ratio(fixed, ground) >= 4.5);
    }

    #[test]
    fn a_page_that_lost_the_skip_link_target_is_a_fault() {
        let good = "<div id=\"mdbook-content\"><main></main></div><script src=\"js/a11y.js\">";
        assert!(skip_link_faults(good).is_ok());

        let renamed = "<div id=\"content\"><main></main></div><script src=\"js/a11y.js\">";
        let faults = skip_link_faults(renamed).expect_err("the target id is gone");
        assert_eq!(faults.len(), 1);
        assert!(faults[0].contains("mdbook-content"));

        let unwired = "<div id=\"mdbook-content\"><main></main></div>";
        assert!(
            skip_link_faults(unwired).is_err(),
            "nothing loads the assets"
        );
    }

    #[test]
    fn the_additional_css_list_is_read_in_order() {
        assert_eq!(
            additional_css("additional-css = [\"css/a11y.css\", \"css/other.css\"]\n"),
            vec!["css/a11y.css".to_string(), "css/other.css".to_string()]
        );
        assert!(additional_css("additional-css = []\n").is_empty());
    }
}
