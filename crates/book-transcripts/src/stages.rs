//! The stages viewer's native check, for any book: a chapter's committed stages held to what
//! its in-page kernel answers.
//!
//! `js/stages.js` (the contract is `docs/stages.md`) shows a picture that a chapter's steps
//! change as numbered tabs, one per stage, each with the picture and the words as they were.
//! The stages are committed: the pictures as files beside the chapter, the words and the facts
//! as text in each stage's result block. Nothing in a browser checks that they are what the
//! kernel says, so a native test does, and this module is the part of that test that is the
//! same for every book: reading the viewer's markup, reading the viewer's lines after a step
//! the way the script does, and walking a chapter's steps to see which make a stage.
//!
//! It knows nothing about any kernel. A book's test hands it a function that runs one line in
//! the viewer's chain on a page and says what came back ([`Reply`]); `crates/book-wasm` and
//! `crates/architecture-book-wasm` each run their own kernel behind that function.
//!
//! The committed pictures are written by the code, never by hand: run the test with
//! `BOOK_PICTURES_WRITE=1` and every picture that differs is written where the chapter keeps
//! it. Without it, a picture that differs is written beside the build and the failure names the
//! `cp` that commits it. The result blocks are printed, never written: they sit in prose.

use std::path::Path;

/// What one line answered, as the page's kernel says it: `kind` is `output` or `error` (or
/// another word the engine uses), `text` what it printed, `cache` the cache's verdict.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Reply {
    pub kind: String,
    pub text: String,
    pub cache: String,
}

impl Reply {
    /// What the line said, in words: an error as `error: …`, as `js/stages.js` says it.
    pub fn said(&self) -> String {
        if self.kind == "error" {
            format!("error: {}", self.text)
        } else {
            self.text.clone()
        }
    }

    /// What a cell prints for it, as `js/run.js` renders a reply: the text, then the cache
    /// verdict in brackets; an error as `error: …`.
    pub fn render(&self) -> String {
        let mut out = if self.kind == "error" {
            format!("error: {}\n", self.text)
        } else {
            let mut text = self.text.clone();
            if !text.ends_with('\n') {
                text.push('\n');
            }
            text
        };
        if !self.cache.is_empty() {
            out.push_str(&format!("[{}]\n", self.cache));
        }
        out
    }
}

/// The HTML escapes a page's markup uses, undone.
pub fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#32;", " ")
        .replace("&amp;", "&")
}

/// `text` escaped for HTML text and a double-quoted attribute.
pub fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// The value of `attr='…'` (single-quoted, as the books write a line) in `tag`.
pub fn attribute(tag: &str, attr: &str) -> Option<String> {
    let open = format!("{attr}='");
    let at = tag.find(&open)? + open.len();
    let end = tag[at..].find('\'')?;
    Some(unescape(&tag[at..at + end]))
}

/// The value of `attr="…"` (double-quoted) in `tag`.
pub fn quoted(tag: &str, attr: &str) -> Option<String> {
    let open = format!("{attr}=\"");
    let i = tag.find(&open)? + open.len();
    Some(unescape(&tag[i..i + tag[i..].find('"')?]))
}

/// A line's non-blank lines, trimmed: what `js/stages.js` compares.
pub fn lines(text: &str) -> Vec<&str> {
    text.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect()
}

/// One fact a viewer keeps current above its tabs (`ul.ikigai-stages-facts > li`).
#[derive(Clone, Debug)]
pub struct Fact {
    /// `data-label`, said before the value.
    pub label: Option<String>,
    /// `data-cmd`: the line whose answer holds it.
    pub line: String,
    /// `data-line`: which line of the answer (1-based).
    pub nth: usize,
    /// `data-none`: what to say when the answer has no such line.
    pub none: String,
}

/// One stage a viewer shows: where its section is on the page, its tab's word, its markup.
#[derive(Clone, Debug)]
pub struct StageMarkup {
    pub at: usize,
    pub end: usize,
    pub label: String,
    pub markup: String,
}

/// One control a viewer offers (`ul.ikigai-stages-controls > li`): its button's name and lines.
#[derive(Clone, Debug)]
pub struct Control {
    pub label: String,
    pub cmd: String,
}

/// A stages viewer a page declares (`div.ikigai-stages`): the chain its steps run in, the line
/// that draws its picture, the line that says it in words, the alt text its pictures start
/// with, its facts, its controls and its chapter's stages.
#[derive(Clone, Debug)]
pub struct Viewer {
    pub chain: Option<String>,
    pub picture: String,
    pub words: String,
    /// `data-before`: a line whose answer, read after each step, is handed to the next read
    /// of the picture as `before="…"`, so the picture can mark what changed since.
    pub before: Option<String>,
    pub alt: String,
    pub facts: Vec<Fact>,
    /// The facts list as the page shows it before any step.
    pub facts_shown: Vec<String>,
    pub controls: Vec<Control>,
    /// Where the viewer starts and ends on the page.
    pub at: usize,
    pub end: usize,
    pub stages: Vec<StageMarkup>,
}

/// The items of the first `<ul class="{class}">` in `within`: each tag and its text.
fn items<'a>(within: &'a str, class: &str) -> Vec<(&'a str, String)> {
    let open = format!("<ul class=\"{class}\"");
    let Some(at) = within.find(&open) else {
        return Vec::new();
    };
    let list = &within[at..at + within[at..].find("</ul>").expect("a closed list")];
    let mut found = Vec::new();
    let mut rest = list;
    while let Some(li) = rest.find("<li ") {
        let item = &rest[li..];
        // The tag ends at the first `>` outside a quoted value: a line may hold one.
        let mut quote = None;
        let tag_end = item
            .char_indices()
            .find(|&(_, c)| {
                match (quote, c) {
                    (None, '\'' | '"') => quote = Some(c),
                    (Some(q), c) if c == q => quote = None,
                    (None, '>') => return true,
                    _ => {}
                }
                false
            })
            .map(|(i, _)| i)
            .expect("a closed tag");
        let text_end = item.find("</li>").expect("a closed item");
        found.push((&item[..tag_end], unescape(&item[tag_end + 1..text_end])));
        rest = &item[text_end..];
    }
    found
}

/// The viewers a page declares, in page order.
pub fn viewers(page: &str) -> Vec<Viewer> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = page[from..].find("<div class=\"ikigai-stages\"") {
        let at = from + at;
        let tag = &page[at..at + page[at..].find("\">").expect("a closed tag") + 1];
        // The viewer's end: its last stage's end. A stage is a section that follows the one
        // before with nothing but whitespace between them.
        let mut stages = Vec::new();
        let mut cursor = at;
        let close = "</section>";
        let mut end = at;
        while let Some(s) = page[cursor..].find("<section class=\"ikigai-stage\"") {
            let s_at = cursor + s;
            if end > at && !page[end..s_at].trim().is_empty() {
                break;
            }
            let s_end = s_at + page[s_at..].find(close).expect("a closed stage") + close.len();
            let s_tag = &page[s_at..s_at + page[s_at..].find('>').expect("a closed tag")];
            stages.push(StageMarkup {
                at: s_at,
                end: s_end,
                label: quoted(s_tag, "data-label").expect("a stage names its tab"),
                markup: page[s_at..s_end].to_string(),
            });
            end = s_end;
            cursor = s_end;
        }
        let head = &page[at..stages.first().map_or(end, |s| s.at)];
        let (facts, facts_shown) = items(head, "ikigai-stages-facts")
            .into_iter()
            .map(|(tag, text)| {
                (
                    Fact {
                        label: attribute(tag, "data-label"),
                        line: attribute(tag, "data-cmd").expect("a fact names its line"),
                        nth: attribute(tag, "data-line")
                            .map_or(1, |n| n.parse().expect("a line number")),
                        none: attribute(tag, "data-none").unwrap_or_default(),
                    },
                    text,
                )
            })
            .unzip();
        let controls = items(head, "ikigai-stages-controls")
            .into_iter()
            .map(|(tag, text)| Control {
                label: text.trim().to_string(),
                cmd: attribute(tag, "data-cmd").expect("a control names its lines"),
            })
            .collect();
        found.push(Viewer {
            chain: attribute(tag, "data-chain"),
            picture: attribute(tag, "data-picture").expect("a viewer names its picture"),
            words: attribute(tag, "data-words").expect("a viewer names its words"),
            before: attribute(tag, "data-before"),
            alt: quoted(tag, "data-alt").expect("a viewer says what its picture draws"),
            facts,
            facts_shown,
            controls,
            at,
            end,
            stages,
        });
        from = end.max(at + 1);
    }
    found
}

/// What a viewer reads after a step, in the order `js/stages.js` reads it.
#[derive(Clone, Debug)]
pub struct Snapshot {
    /// (scheme, SVG) per scheme the picture line asks for.
    pub pictures: Vec<(Option<&'static str>, String)>,
    /// The cache's verdict on the first picture read.
    pub cache: String,
    pub words: String,
    /// What `data-before` answered, when the viewer names one.
    pub before: Option<String>,
    pub facts: Vec<String>,
}

/// `text` as one quoted value of a line, the way the engine reads one: `\` and `"` escaped.
pub fn quote(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

/// Two answers in words are the same when their lines are, trimmed, blank lines dropped.
pub fn same_words(a: &str, b: &str) -> bool {
    lines(a) == lines(b)
}

impl Snapshot {
    /// What the page has seen before any step: the first stage's words, and no picture.
    pub fn start(viewer: &Viewer) -> Snapshot {
        Snapshot {
            pictures: Vec::new(),
            cache: String::new(),
            words: words_in(&viewer.stages[0].markup),
            before: None,
            facts: Vec::new(),
        }
    }

    /// Whether this is a new stage after `seen`: the words differ, or what `data-before`
    /// answers does, or (without `data-before`, whose picture marks changes and so differs
    /// whenever its `before` does) the picture does.
    pub fn moved_from(&self, seen: &Snapshot) -> bool {
        if !same_words(&self.words, &seen.words) {
            return true;
        }
        match (&seen.before, &self.before) {
            (Some(was), Some(now)) => was != now,
            (None, Some(_)) => false,
            _ => !seen.pictures.is_empty() && seen.pictures[0].1 != self.pictures[0].1,
        }
    }
}

/// The words a stage's markup commits: its `pre.ikigai-stage-words`.
pub fn words_in(markup: &str) -> String {
    let open = "<pre class=\"ikigai-stage-words\">";
    markup
        .find(open)
        .map(|i| {
            let body = &markup[i + open.len()..];
            unescape(&body[..body.find("</pre>").expect("closed")])
        })
        .unwrap_or_default()
}

impl Viewer {
    /// The schemes the picture line is asked in: `light` and `dark` when it holds `{scheme}`.
    pub fn schemes(&self) -> Vec<Option<&'static str>> {
        if self.picture.contains("{scheme}") {
            vec![Some("light"), Some("dark")]
        } else {
            vec![None]
        }
    }

    /// The picture line in `scheme`, with `before="…"` when the viewer names `data-before`
    /// and there is an earlier answer of it to hand on.
    pub fn picture_line(&self, scheme: Option<&str>, before: Option<&str>) -> String {
        let line = scheme.map_or(self.picture.clone(), |s| {
            self.picture.replace("{scheme}", s)
        });
        match (&self.before, before) {
            (Some(_), Some(earlier)) => format!("{line} before={}", quote(earlier)),
            _ => line,
        }
    }

    /// Read the viewer's stage as `js/stages.js` reads it after a step: the words, then
    /// `data-before`, then each fact, then the picture in each scheme, marked against `before`
    /// when the viewer names `data-before`. `eval` runs one line in the viewer's chain.
    pub fn read(&self, eval: &dyn Fn(&str) -> Reply, before: Option<&str>) -> Snapshot {
        let mut now = self.read_state(eval);
        self.read_pictures(eval, before, &mut now);
        now
    }

    /// The words, what `data-before` answers and the facts, with no picture yet.
    fn read_state(&self, eval: &dyn Fn(&str) -> Reply) -> Snapshot {
        let words = eval(&self.words).said();
        let before = self.before.as_ref().map(|line| {
            let r = eval(line);
            assert_eq!(r.kind, "output", "`{line}` answered {r:?}");
            r.text
        });
        let facts = self
            .facts
            .iter()
            .map(|fact| {
                let answer = eval(&fact.line).said();
                let value = lines(&answer)
                    .get(fact.nth - 1)
                    .map_or(fact.none.clone(), |l| l.to_string());
                fact.label
                    .as_ref()
                    .map_or(value.clone(), |l| format!("{l}: {value}"))
            })
            .collect();
        Snapshot {
            pictures: Vec::new(),
            cache: String::new(),
            words,
            before,
            facts,
        }
    }

    /// The picture in each scheme, marked against `before`, into `now`.
    fn read_pictures(
        &self,
        eval: &dyn Fn(&str) -> Reply,
        before: Option<&str>,
        now: &mut Snapshot,
    ) {
        for scheme in self.schemes() {
            let line = self.picture_line(scheme, before);
            let r = eval(&line);
            assert_eq!(r.kind, "output", "`{line}` drew no picture: {r:?}");
            if now.cache.is_empty() {
                now.cache = r.cache.clone();
            }
            now.pictures.push((scheme, r.text));
        }
    }

    /// A reader of this viewer, as the page is one: what it last saw, starting from the
    /// first stage's committed words.
    pub fn reading(&self) -> Reading<'_> {
        Reading {
            viewer: self,
            seen: Snapshot::start(self),
            drawn_with: None,
        }
    }

    /// The file stage `number`'s picture in `scheme` is committed as.
    pub fn picture_file(stem: &str, number: usize, scheme: Option<&str>) -> String {
        match scheme {
            Some(s) => format!("{stem}-{number}-{s}.svg"),
            None => format!("{stem}-{number}.svg"),
        }
    }

    /// The result block stage `number`'s section must hold, as the kernel answers it: the
    /// facts, the pictures in each scheme (files named for the stage, `{stem}-{number}-{scheme}`),
    /// each with the stage's text alternative, and the words. `js/stages.js` adds what made the
    /// stage and what changed.
    pub fn result_block(&self, number: usize, stem: &str, now: &Snapshot) -> String {
        let words = now.words.trim_end_matches('\n');
        let alt = escape(&format!(
            "{} At this stage: {}.",
            self.alt,
            lines(words).join("; ")
        ));
        let pictures: String = now
            .pictures
            .iter()
            .map(|(scheme, _)| {
                let file = Self::picture_file(stem, number, *scheme);
                let class =
                    scheme.map_or(String::new(), |s| format!(" class=\"ikigai-scheme-{s}\""));
                format!("<a{class} href=\"{file}\"><img src=\"{file}\" alt=\"{alt}\"></a>")
            })
            .collect();
        let facts_line = if now.facts.is_empty() {
            String::new()
        } else {
            format!(
                "<p class=\"ikigai-stage-facts\">{}</p>\n",
                escape(&now.facts.join(" · "))
            )
        };
        format!(
            "<div class=\"ikigai-stage-result\">\n{facts_line}<p class=\"ikigai-stage-picture\">{pictures}</p>\n<pre class=\"ikigai-stage-words\">{}</pre>\n</div>",
            escape(words)
        )
    }
}

/// The viewer's reads after each step, as `js/stages.js` makes them.
///
/// With `data-before`, a step that makes a stage draws its picture marked against the stage
/// before (what `data-before` answered at the last read), and a step that changes nothing
/// draws it exactly as the current stage was drawn, so that read is the same request, and a
/// step that cut nothing finds it in the cache.
pub struct Reading<'v> {
    viewer: &'v Viewer,
    /// What the last read saw.
    pub seen: Snapshot,
    /// What the current stage's picture was marked against.
    drawn_with: Option<String>,
}

impl Reading<'_> {
    /// Read the viewer now, and say whether that is a new stage after the last read.
    pub fn step(&mut self, eval: &dyn Fn(&str) -> Reply) -> (Snapshot, bool) {
        let viewer = self.viewer;
        let mut now = viewer.read_state(eval);
        let moved = if viewer.before.is_some() {
            let moved = !same_words(&now.words, &self.seen.words)
                || (self.seen.before.is_some() && self.seen.before != now.before);
            if moved {
                self.drawn_with = self.seen.before.clone();
            }
            viewer.read_pictures(eval, self.drawn_with.as_deref(), &mut now);
            moved
        } else {
            viewer.read_pictures(eval, None, &mut now);
            now.moved_from(&self.seen)
        };
        self.seen = now.clone();
        (now, moved)
    }
}

/// One step of a chapter, in page order: where its markup starts on the page, the chain it
/// runs in, and its lines. A cell, as `js/run.js` places it.
#[derive(Clone, Debug)]
pub struct Step {
    pub at: usize,
    pub chain: Option<String>,
    pub cmd: String,
}

/// Where a viewer's committed pictures live, and what they are called.
pub struct Pictures<'a> {
    /// The directory beside the chapter that holds them.
    pub dir: &'a Path,
    /// Their stem: stage `n` in scheme `s` is `{stem}-{n}-{s}.svg`.
    pub stem: &'a str,
    /// Where a picture that differs is written when the committed ones are not rewritten
    /// (the test's `CARGO_TARGET_TMPDIR`).
    pub out: &'a Path,
}

/// The chapter's stages ARE its walkthrough, as the kernel answers it.
///
/// `fresh` reads a line on a fresh page in the viewer's chain: stage 1 is that page read before
/// any step, and the facts list must show its facts. Then every step runs in page order on ONE
/// page (`run` runs a step there and returns what it printed; `eval` reads a line on that same
/// page in the viewer's chain), and after each step in the viewer's chain the viewer's lines
/// are read as `js/stages.js` reads them. A step that moves the words or the picture makes the
/// next stage and must sit in that stage's section; a step that moves nothing must sit in some
/// stage when it is inside the viewer. Each stage's section must hold the result block the
/// kernel answers, and its committed pictures must be the kernel's own, byte for byte.
///
/// Returns the problems, empty when the chapter is true, and the printed output of each step.
pub fn walkthrough(
    viewer: &Viewer,
    steps: &[Step],
    fresh: &dyn Fn(&str) -> Reply,
    run: &dyn Fn(&Step) -> String,
    eval: &dyn Fn(&str) -> Reply,
    pictures: &Pictures,
) -> (Vec<String>, Vec<String>) {
    let write = std::env::var_os("BOOK_PICTURES_WRITE").is_some();
    let mut problems = Vec::new();
    let mut to_copy = Vec::new();
    let mut check = |number: usize, now: &Snapshot, problems: &mut Vec<String>| {
        let Some(stage) = viewer.stages.get(number - 1) else {
            problems.push(format!("stage {number} has no section in the chapter"));
            return;
        };
        let block = viewer.result_block(number, pictures.stem, now);
        if !stage.markup.contains(&block) {
            problems.push(format!(
                "stage {number} ({}): its section does not hold the result block the kernel answers:\n{block}",
                stage.label
            ));
        }
        for (scheme, svg) in &now.pictures {
            let file = Viewer::picture_file(pictures.stem, number, *scheme);
            let committed = pictures.dir.join(&file);
            if std::fs::read_to_string(&committed).ok().as_deref() == Some(svg.as_str()) {
                continue;
            }
            if write {
                std::fs::write(&committed, svg).expect("write the picture the kernel draws");
                println!("wrote {}", committed.display());
            } else {
                std::fs::write(pictures.out.join(&file), svg)
                    .expect("write the picture the kernel draws");
                to_copy.push(file);
            }
        }
    };

    let start = viewer.read(fresh, None);
    check(1, &start, &mut problems);
    if viewer.facts_shown != start.facts {
        problems.push(format!(
            "the facts list before any step shows {:?}; the kernel says {:?}",
            viewer.facts_shown, start.facts
        ));
    }

    let mut printed = Vec::new();
    // The committed stages are marked against the stage before; stage 1's board is the
    // fresh page's. (In the page, the first step's stage has no earlier read to be marked
    // against: the viewer reads nothing on load.)
    let mut reading = viewer.reading();
    reading.seen.before = start.before.clone();
    let mut number = 1;
    for (i, step) in steps.iter().enumerate() {
        let out = run(step);
        printed.push(out.clone());
        if step.chain != viewer.chain {
            continue;
        }
        let (now, moved) = reading.step(eval);
        let within = viewer
            .stages
            .iter()
            .position(|s| s.at < step.at && step.at < s.end);
        if !moved {
            if within.is_none() && viewer.at < step.at && step.at < viewer.end {
                problems.push(format!("step {i} runs in the viewer outside every stage"));
            }
            continue;
        }
        number += 1;
        if within != Some(number - 1) {
            problems.push(format!(
                "step {i} ({}) makes stage {number}, but sits in stage {:?}; it printed {out}",
                step.cmd,
                within.map(|w| w + 1)
            ));
        }
        check(number, &now, &mut problems);
    }
    if number != viewer.stages.len() {
        problems.push(format!(
            "the walkthrough makes {number} stages, and the chapter has {}",
            viewer.stages.len()
        ));
    }
    if !to_copy.is_empty() {
        problems.push(format!(
            "these committed pictures are not what the kernel draws. If the change is intended, \
             rerun with BOOK_PICTURES_WRITE=1, or:  (cd {} && cp {} {}/)",
            pictures.out.display(),
            to_copy.join(" "),
            pictures.dir.display()
        ));
    }
    (problems, printed)
}

#[cfg(test)]
mod tests {
    use super::*;

    const PAGE: &str = "intro\n\
<div class=\"ikigai-stages\" data-chain='live:w' data-picture='source urn:p scheme={scheme}'\n\
     data-words='source urn:w' data-alt=\"A picture.\">\n\
<ul class=\"ikigai-stages-controls\" hidden>\n\
<li data-cmd='sink urn:m a>b'>Set it</li>\n\
</ul>\n\
<ul class=\"ikigai-stages-facts\">\n\
<li data-cmd='source urn:f' data-line='2' data-label='f' data-none='none yet'>f: none yet</li>\n\
</ul>\n\
<section class=\"ikigai-stage\" data-label=\"one\">\n\nA\n\n</section>\n\
<section class=\"ikigai-stage\" data-label=\"two\">\n\nB\n\n</section>\n\
</div>\n\
<section class=\"ikigai-stage\" data-label=\"not the viewer's\"></section>\n";

    #[test]
    fn a_viewer_is_read_from_its_markup() {
        let found = viewers(PAGE);
        assert_eq!(found.len(), 1);
        let v = &found[0];
        assert_eq!(v.chain.as_deref(), Some("live:w"));
        assert_eq!(v.schemes(), [Some("light"), Some("dark")]);
        assert_eq!(v.alt, "A picture.");
        assert_eq!(v.controls.len(), 1);
        assert_eq!(v.controls[0].label, "Set it");
        assert_eq!(v.controls[0].cmd, "sink urn:m a>b");
        assert_eq!(v.facts.len(), 1);
        assert_eq!(v.facts[0].nth, 2);
        assert_eq!(v.facts_shown, ["f: none yet"]);
        let labels: Vec<&str> = v.stages.iter().map(|s| s.label.as_str()).collect();
        assert_eq!(labels, ["one", "two"]);
    }

    #[test]
    fn a_result_block_escapes_what_it_quotes() {
        let v = &viewers(PAGE)[0];
        let now = Snapshot {
            pictures: vec![
                (Some("light"), String::new()),
                (Some("dark"), String::new()),
            ],
            cache: String::new(),
            words: "a <b> & \"c\"\n".to_string(),
            before: None,
            facts: vec!["f: x".to_string()],
        };
        let block = v.result_block(2, "s", &now);
        assert!(block
            .contains("<pre class=\"ikigai-stage-words\">a &lt;b&gt; &amp; &quot;c&quot;</pre>"));
        assert!(block.contains("href=\"s-2-light.svg\""));
        assert!(
            block.contains("alt=\"A picture. At this stage: a &lt;b&gt; &amp; &quot;c&quot;.\"")
        );
        assert!(block.contains("<p class=\"ikigai-stage-facts\">f: x</p>"));
        assert_eq!(unescape(&escape("<&>\"")), "<&>\"");
    }

    #[test]
    fn a_picture_line_carries_the_earlier_answer_quoted() {
        let page = PAGE.replace(
            "data-words='source urn:w'",
            "data-words='source urn:w' data-before='source urn:b as=text/turtle'",
        );
        let v = &viewers(&page)[0];
        assert_eq!(v.before.as_deref(), Some("source urn:b as=text/turtle"));
        assert_eq!(
            v.picture_line(Some("dark"), Some("<a> <b> \"c\\d\" .\n")),
            "source urn:p scheme=dark before=\"<a> <b> \\\"c\\\\d\\\" .\n\""
        );
        assert_eq!(
            v.picture_line(Some("light"), None),
            "source urn:p scheme=light"
        );
        // Without data-before, an earlier answer is never handed on.
        let plain = &viewers(PAGE)[0];
        assert_eq!(
            plain.picture_line(None, Some("x")),
            "source urn:p scheme={scheme}"
        );
    }

    #[test]
    fn a_reply_renders_as_a_cell_prints_it() {
        let ok = Reply {
            kind: "output".into(),
            text: "x".into(),
            cache: "computed".into(),
        };
        assert_eq!(ok.render(), "x\n[computed]\n");
        let no = Reply {
            kind: "error".into(),
            text: "nope".into(),
            cache: String::new(),
        };
        assert_eq!(no.render(), "error: nope\n");
        assert_eq!(no.said(), "error: nope");
    }
}
