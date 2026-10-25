//! Every runnable cell and every harness control of The ikigai Architecture, replayed natively
//! against the kernel the page runs (`architecture_book_wasm`), and the harness's committed
//! stages (their pictures in both schemes, their words and facts) held to what the kernel says.
//!
//! A cell runs the way `js/run.js` runs it: in page order, line by line, on one fresh page,
//! in the chain its markup names (`data-harness='overlays'` is `harness:overlays`), and its
//! output must match the page's "Expected output" (`…` matching what the kernel mints, as in a
//! transcript). A control runs the way `js/stages.js` runs it, and after every cell or control
//! in a stages viewer's chain the viewer's lines are read, as `js/stages.js` reads them.

use architecture_book_wasm::{eval_on, Page};
use book_transcripts::{matches, normalize};
use futures::executor::block_on;

/// One runnable cell: the chain its markup names, its line(s), and its expected output.
#[derive(Debug)]
struct Cell {
    /// Where the cell's markup starts in the page.
    at: usize,
    chain: Option<String>,
    cmd: String,
    expected: String,
}

/// One harness control: its harness, its label (the button's name) and its lines.
#[derive(Debug)]
struct Control {
    chain: String,
    label: String,
    cmd: String,
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&#32;", " ")
        .replace("&amp;", "&")
}

/// The value of `attr='…'` in `tag`, if it is there.
fn attribute(tag: &str, attr: &str) -> Option<String> {
    let open = format!("{attr}='");
    let at = tag.find(&open)? + open.len();
    let end = tag[at..].find('\'')?;
    Some(unescape(&tag[at..at + end]))
}

fn page(file: &str) -> String {
    let path = format!(
        "{}/../../books/architecture/src/{file}",
        env!("CARGO_MANIFEST_DIR")
    );
    std::fs::read_to_string(&path).expect("the page is readable")
}

/// The cells a page places, in page order.
fn cells(page: &str) -> Vec<Cell> {
    let mut found = Vec::new();
    let mut from_start = 0;
    while let Some(at) = page[from_start..].find("<div class=\"ikigai-run\"") {
        let at = from_start + at;
        let after = &page[at..];
        let cmd_at = after.find("data-cmd='").expect("a cell has a data-cmd");
        let chain = attribute(&after[..cmd_at], "data-harness").map(|h| format!("harness:{h}"));
        let cmd = attribute(&after[cmd_at..], "data-cmd").expect("a closed data-cmd");
        let open = "<pre class=\"ikigai-run-expected\">";
        let from = after
            .find(open)
            .expect("every cell shows its expected output");
        let body = &after[from + open.len()..];
        let close = body.find("</pre>").expect("a closed expected output");
        found.push(Cell {
            at,
            chain,
            cmd,
            expected: unescape(&body[..close]),
        });
        from_start = at + from + open.len() + close;
    }
    found
}

/// The controls each harness figure on a page offers, in page order.
fn controls(page: &str) -> Vec<Control> {
    let mut found = Vec::new();
    let mut rest = page;
    while let Some(at) = rest.find("<div class=\"ikigai-stages\"") {
        let figure = &rest[at..rest[at..]
            .find("</ul>")
            .map(|e| at + e)
            .expect("a harness's controls are a closed list")];
        let chain =
            attribute(figure, "data-chain").expect("a viewer with controls names its chain");
        let mut items = figure;
        while let Some(li) = items.find("<li data-cmd='") {
            let item = &items[li..];
            let cmd = attribute(item, "data-cmd").expect("a closed data-cmd");
            let label_at = item.find("'>").expect("the label follows") + 2;
            let label_end = item.find("</li>").expect("a closed item");
            found.push(Control {
                chain: chain.clone(),
                label: unescape(&item[label_at..label_end]),
                cmd,
            });
            items = &item[label_end..];
        }
        rest = &rest[at + figure.len()..];
    }
    found
}

fn lines(cmd: &str) -> Vec<&str> {
    cmd.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect()
}

/// What a cell prints, as `js/run.js` renders a reply: the text, then the cache verdict in
/// brackets; an error as `error: …`.
fn render(json: &str) -> String {
    let reply: serde_json::Value = serde_json::from_str(json).expect("the reply is JSON");
    let mut out = String::new();
    if reply["kind"] == "error" {
        out.push_str(&format!(
            "error: {}\n",
            reply["text"].as_str().unwrap_or_default()
        ));
    } else {
        out.push_str(reply["text"].as_str().unwrap_or_default());
        if !out.ends_with('\n') {
            out.push('\n');
        }
    }
    if let Some(cache) = reply["cache"].as_str().filter(|c| !c.is_empty()) {
        out.push_str(&format!("[{cache}]\n"));
    }
    out
}

/// Run `cmd`'s lines in `chain` on `page`, as the page does, and print what it would show.
fn run(page: &Page, chain: Option<&str>, cmd: &str) -> String {
    lines(cmd)
        .into_iter()
        .map(|line| render(&block_on(eval_on(page, chain, line))))
        .collect()
}

/// Every cell of a page, in page order on one fresh page, against the output the page shows.
fn run_page_in_order(file: &str, expected_cells: usize) {
    let text = page(file);
    let cells = cells(&text);
    let viewers = viewers(&text);
    assert_eq!(cells.len(), expected_cells, "{file}'s cells: {cells:#?}");
    let page = Page::default();
    let mut misses = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        let printed = run(&page, cell.chain.as_deref(), &cell.cmd);
        // As `js/stages.js` does after every step in a viewer's chain: read its lines.
        for viewer in viewers.iter().filter(|v| v.chain == cell.chain) {
            read(&page, viewer);
        }
        let expected = normalize(cell.expected.lines());
        let actual = normalize(printed.lines());
        if !matches(&expected, &actual) {
            misses.push(format!(
                "cell {i} ({:?}): {}\n  the page says:\n    {}\n  the kernel printed:\n    {}",
                cell.chain,
                cell.cmd,
                expected.join("\n    "),
                actual.join("\n    ")
            ));
        }
    }
    assert!(misses.is_empty(), "{file}:\n{}", misses.join("\n"));
}

#[test]
fn spaces_that_wrap_spaces_answers_as_the_page_says_in_page_order() {
    run_page_in_order("overlays/overlays.md", 11);
}

#[test]
fn golden_threads_answers_as_the_page_says_in_page_order() {
    run_page_in_order("economics/economics.md", 6);
}

/// Each control, pressed first on a fresh page, runs every one of its lines in its harness:
/// nothing it sends is a name the harness does not bind or a line the engine cannot read.
#[test]
fn every_harness_control_runs_in_its_harness() {
    let controls = controls(&page("overlays/overlays.md"));
    let labels: Vec<&str> = controls.iter().map(|c| c.label.as_str()).collect();
    assert_eq!(
        labels,
        [
            "Set the upstream to ok",
            "Set the upstream to slow",
            "Set the upstream to fail",
            "Set the upstream to limited",
            "Call once",
            "Call ten times",
            "Wait 5 seconds",
        ]
    );
    for control in &controls {
        let page = Page::default();
        for line in lines(&control.cmd) {
            let reply: serde_json::Value =
                serde_json::from_str(&block_on(eval_on(&page, Some(&control.chain), line)))
                    .unwrap();
            assert_eq!(
                reply["kind"], "output",
                "{}: `{line}` answered {reply}",
                control.label
            );
        }
    }
}

/// The controls, pressed in the order a reader trying the breaker presses them: what each
/// answers, and what the board says after, as `js/harness.js` reads them.
#[test]
fn the_controls_trip_the_breaker_and_close_it_again() {
    let chapter = page("overlays/overlays.md");
    let controls = controls(&chapter);
    let viewer = &viewers(&chapter)[0];
    let page = Page::default();
    let seen = std::cell::RefCell::new(Snapshot::start(viewer));
    let press = |label: &str| {
        let control = controls
            .iter()
            .find(|c| c.label == label)
            .unwrap_or_else(|| panic!("no control `{label}`"));
        let printed = run(&page, Some(&control.chain), &control.cmd);
        let now = read(&page, viewer);
        let moved = now.moved_from(&seen.borrow());
        let verdict = now.cache.clone();
        *seen.borrow_mut() = now;
        (printed, moved, verdict)
    };
    let board = || {
        let host = page.harness("overlays").unwrap();
        block_on(host.harness().words()).unwrap()
    };

    let (_, moved, verdict) = press("Set the upstream to fail");
    assert!(!moved, "setting a mode changes nothing the picture shows");
    assert_eq!(
        verdict, "computed",
        "the first read of the picture draws it"
    );
    let (ten, moved, _) = press("Call ten times");
    let reached: Vec<&str> = ten.lines().filter(|l| l.contains("(call ")).collect();
    assert_eq!(
        reached,
        ["error: unavailable: upstream failed (call 3)"],
        "of ten calls, only the first reached the stub (three times): {ten}"
    );
    assert_eq!(
        ten.matches("error: retry after 5000 ms: circuit open")
            .count(),
        9,
        "{ten}"
    );
    assert!(board().contains(": open, 3 consecutive"), "{}", board());
    assert!(moved, "the circuit opened: a new stage");

    let (_, moved, _) = press("Wait 5 seconds");
    assert!(board().contains(": half-open,"), "{}", board());
    assert!(moved, "the cooldown ran out: a new stage");
    press("Set the upstream to ok");
    let (once, moved, _) = press("Call once");
    assert_eq!(once, "ok (call 4)\n[uncacheable]\n");
    assert!(board().contains(": closed, 0 consecutive"), "{}", board());
    assert!(moved, "the probe closed the circuit: a new stage");
    let (_, moved, verdict) = press("Call once");
    assert!(
        !moved,
        "a call that succeeds on a closed circuit moves nothing"
    );
    assert_eq!(
        verdict, "cached",
        "and cuts nothing, so the picture is served from the cache"
    );
}

/// A stages viewer a page declares (`div.ikigai-stages`, `js/stages.js`): the chain its steps
/// run in, the line that draws its picture, the line that says it in words, its facts, the
/// alt text its pictures start with, and its chapter's stages.
#[derive(Debug)]
struct Viewer {
    chain: Option<String>,
    picture: String,
    words: String,
    alt: String,
    /// (label, line, which line of the answer, what to say when there is no such line) per fact.
    facts: Vec<(Option<String>, String, usize, String)>,
    /// The facts list as the page shows it before any step.
    facts_shown: Vec<String>,
    at: usize,
    end: usize,
    stages: Vec<StageMarkup>,
}

/// One stage a viewer shows: its tab's word, and its markup.
#[derive(Debug)]
struct StageMarkup {
    at: usize,
    end: usize,
    label: String,
    markup: String,
}

/// The value of `attr="…"` (double-quoted) in `tag`.
fn quoted(tag: &str, attr: &str) -> Option<String> {
    let open = format!("{attr}=\"");
    let i = tag.find(&open)? + open.len();
    Some(unescape(&tag[i..i + tag[i..].find('"')?]))
}

/// The viewers a page declares, in page order.
fn viewers(page: &str) -> Vec<Viewer> {
    let mut found = Vec::new();
    let mut from = 0;
    while let Some(at) = page[from..].find("<div class=\"ikigai-stages\"") {
        let at = from + at;
        let tag = &page[at..at + page[at..].find("\">").expect("a closed tag") + 1];
        // The viewer's end: the last stage's end, then its closing div.
        let mut stages = Vec::new();
        let mut cursor = at;
        let close = "</section>";
        let mut end = at;
        while let Some(s) = page[cursor..].find("<section class=\"ikigai-stage\"") {
            let s_at = cursor + s;
            let between = &page[end.max(at)..s_at];
            if end > at && !between.trim().is_empty() {
                break;
            }
            let s_end = s_at + page[s_at..].find(close).expect("a closed stage") + close.len();
            let s_tag = &page[s_at..s_at + page[s_at..].find('>').unwrap()];
            stages.push(StageMarkup {
                at: s_at,
                end: s_end,
                label: quoted(s_tag, "data-label").expect("a stage names its tab"),
                markup: page[s_at..s_end].to_string(),
            });
            end = s_end;
            cursor = s_end;
        }
        let facts_at = page[at..end].find("<ul class=\"ikigai-stages-facts\">");
        let mut facts = Vec::new();
        let mut facts_shown = Vec::new();
        if let Some(f) = facts_at {
            let list = &page[at + f..at + f + page[at + f..].find("</ul>").unwrap()];
            let mut rest = list;
            while let Some(li) = rest.find("<li ") {
                let item = &rest[li..];
                let tag_end = item.find('>').unwrap();
                let tag = &item[..tag_end];
                let text_end = item.find("</li>").unwrap();
                facts.push((
                    attribute(tag, "data-label"),
                    attribute(tag, "data-cmd").expect("a fact names its line"),
                    attribute(tag, "data-line").map_or(1, |n| n.parse().expect("a line number")),
                    attribute(tag, "data-none").unwrap_or_default(),
                ));
                facts_shown.push(unescape(&item[tag_end + 1..text_end]));
                rest = &item[text_end..];
            }
        }
        found.push(Viewer {
            chain: attribute(tag, "data-chain"),
            picture: attribute(tag, "data-picture").expect("a viewer names its picture"),
            words: attribute(tag, "data-words").expect("a viewer names its words"),
            alt: quoted(tag, "data-alt").expect("a viewer says what its picture draws"),
            facts,
            facts_shown,
            at,
            end,
            stages,
        });
        from = end;
    }
    found
}

/// What a viewer reads after a step, in the order `js/stages.js` reads it.
#[derive(Clone, Debug)]
struct Snapshot {
    /// (scheme, SVG) per scheme the picture line asks for.
    pictures: Vec<(Option<&'static str>, String)>,
    cache: String,
    words: String,
    facts: Vec<String>,
}

fn same_words(a: &str, b: &str) -> bool {
    let lines = |t: &str| -> Vec<String> {
        t.lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(String::from)
            .collect()
    };
    lines(a) == lines(b)
}

impl Snapshot {
    /// What the page has seen before any step: the first stage's words, and no picture.
    fn start(viewer: &Viewer) -> Snapshot {
        Snapshot {
            pictures: Vec::new(),
            cache: String::new(),
            words: words_in(&viewer.stages[0].markup),
            facts: Vec::new(),
        }
    }

    /// Whether this is a new stage after `seen`: the words differ, or the picture does.
    fn moved_from(&self, seen: &Snapshot) -> bool {
        !same_words(&self.words, &seen.words)
            || (!seen.pictures.is_empty() && seen.pictures[0].1 != self.pictures[0].1)
    }
}

fn reply(page: &Page, chain: Option<&str>, line: &str) -> serde_json::Value {
    serde_json::from_str(&block_on(eval_on(page, chain, line))).expect("the reply is JSON")
}

fn said(reply: &serde_json::Value) -> String {
    let text = reply["text"].as_str().unwrap_or_default();
    if reply["kind"] == "error" {
        format!("error: {text}")
    } else {
        text.to_string()
    }
}

/// Read the viewer's stage as `js/stages.js` reads it after a step.
fn read(page: &Page, viewer: &Viewer) -> Snapshot {
    let chain = viewer.chain.as_deref();
    let schemes: Vec<Option<&'static str>> = if viewer.picture.contains("{scheme}") {
        vec![Some("light"), Some("dark")]
    } else {
        vec![None]
    };
    let mut pictures = Vec::new();
    let mut cache = String::new();
    for scheme in schemes {
        let line = scheme.map_or(viewer.picture.clone(), |s| {
            viewer.picture.replace("{scheme}", s)
        });
        let r = reply(page, chain, &line);
        assert_eq!(r["kind"], "output", "`{line}` drew no picture: {r}");
        if cache.is_empty() {
            cache = r["cache"].as_str().unwrap_or_default().to_string();
        }
        pictures.push((scheme, r["text"].as_str().unwrap().to_string()));
    }
    let words = said(&reply(page, chain, &viewer.words));
    let facts = viewer
        .facts
        .iter()
        .map(|(label, line, n, none)| {
            let answer = said(&reply(page, chain, line));
            let value = answer
                .lines()
                .map(str::trim)
                .filter(|l| !l.is_empty())
                .nth(n - 1)
                .map_or(none.clone(), str::to_string);
            label
                .as_ref()
                .map_or(value.clone(), |l| format!("{l}: {value}"))
        })
        .collect();
    Snapshot {
        pictures,
        cache,
        words,
        facts,
    }
}

/// The words a stage's markup commits: its `pre.ikigai-stage-words`.
fn words_in(markup: &str) -> String {
    let open = "<pre class=\"ikigai-stage-words\">";
    markup
        .find(open)
        .map(|i| {
            let body = &markup[i + open.len()..];
            unescape(&body[..body.find("</pre>").expect("closed")])
        })
        .unwrap_or_default()
}

/// The result block a stage's markup must hold, as the kernel answers it: the facts, the
/// pictures in each scheme (files named for the stage), each with the stage's text alternative,
/// and the words. `js/stages.js` adds what made the stage and what changed.
fn result_block(viewer: &Viewer, number: usize, stem: &str, now: &Snapshot) -> String {
    let words = now.words.trim_end_matches('\n');
    let lines: Vec<&str> = words
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .collect();
    let alt = format!("{} At this stage: {}.", viewer.alt, lines.join("; "));
    let facts = now.facts.join(" · ");
    for text in [&alt, &facts, &words.to_string()] {
        assert!(
            !text.contains(['&', '<', '>', '"']),
            "a stage's words need escaping, which this test does not do: {text}"
        );
    }
    let pictures: String = now
        .pictures
        .iter()
        .map(|(scheme, _)| {
            let file = match scheme {
                Some(s) => format!("{stem}-{number}-{s}.svg"),
                None => format!("{stem}-{number}.svg"),
            };
            let class = scheme.map_or(String::new(), |s| format!(" class=\"ikigai-scheme-{s}\""));
            format!("<a{class} href=\"{file}\"><img src=\"{file}\" alt=\"{alt}\"></a>")
        })
        .collect();
    let facts_line = if now.facts.is_empty() {
        String::new()
    } else {
        format!("<p class=\"ikigai-stage-facts\">{facts}</p>\n")
    };
    format!(
        "<div class=\"ikigai-stage-result\">\n{facts_line}<p class=\"ikigai-stage-picture\">{pictures}</p>\n<pre class=\"ikigai-stage-words\">{words}</pre>\n</div>"
    )
}

/// The chapter's stages ARE the walkthrough, as the kernel answers it. The page's cells run in
/// page order on one fresh page, and after each that runs in the viewer's chain the viewer's
/// lines are read as `js/stages.js` reads them. Every cell that moves the words or the picture
/// makes the next stage, and must sit in that stage's section; a cell that moves nothing must
/// sit in some stage. Each stage's section holds the result block the kernel answers (its facts,
/// its pictures with their text alternative, its words), and its committed pictures are the
/// kernel's own, byte for byte, in each scheme. The first stage is a fresh page's, read before any
/// step, and the facts list shows its facts. On a mismatch the pictures are written beside the
/// build, with the command that commits them, and each expected result block is printed.
#[test]
fn the_chapters_stages_are_the_walkthrough_as_the_kernel_draws_it() {
    const STEM: &str = "harness-stage";
    let chapter = page("overlays/overlays.md");
    let cells = cells(&chapter);
    let viewers = viewers(&chapter);
    assert_eq!(viewers.len(), 1, "the chapter's viewers: {viewers:#?}");
    let viewer = &viewers[0];
    assert_eq!(
        viewer.alt,
        resilience_harness::PICTURE_ALT,
        "data-alt is not PICTURE_ALT"
    );
    let dir = format!(
        "{}/../../books/architecture/src/overlays",
        env!("CARGO_MANIFEST_DIR")
    );
    let out = env!("CARGO_TARGET_TMPDIR");
    let mut problems = Vec::new();
    let mut to_copy = Vec::new();

    let mut check = |number: usize, now: &Snapshot, problems: &mut Vec<String>| {
        let Some(stage) = viewer.stages.get(number - 1) else {
            problems.push(format!("stage {number} has no section in the chapter"));
            return;
        };
        let block = result_block(viewer, number, STEM, now);
        if !stage.markup.contains(&block) {
            problems.push(format!(
                "stage {number} ({}): its section does not hold the result block the kernel answers:\n{block}",
                stage.label
            ));
        }
        for (scheme, svg) in &now.pictures {
            let file = match scheme {
                Some(s) => format!("{STEM}-{number}-{s}.svg"),
                None => format!("{STEM}-{number}.svg"),
            };
            if std::fs::read_to_string(format!("{dir}/{file}"))
                .ok()
                .as_deref()
                != Some(svg.as_str())
            {
                std::fs::write(format!("{out}/{file}"), svg)
                    .expect("write the picture the kernel draws");
                to_copy.push(file);
            }
        }
    };

    // Stage 1: a fresh page, read before any step.
    let fresh = Page::default();
    let start = read(&fresh, viewer);
    check(1, &start, &mut problems);
    if viewer.facts_shown != start.facts {
        problems.push(format!(
            "the facts list before any step shows {:?}; the kernel says {:?}",
            viewer.facts_shown, start.facts
        ));
    }

    let page = Page::default();
    let mut seen = Snapshot::start(viewer);
    let mut number = 1;
    for (i, cell) in cells.iter().enumerate() {
        let printed = run(&page, cell.chain.as_deref(), &cell.cmd);
        if cell.chain != viewer.chain {
            continue;
        }
        let now = read(&page, viewer);
        let within = viewer
            .stages
            .iter()
            .position(|s| s.at < cell.at && cell.at < s.end);
        let moved = now.moved_from(&seen);
        seen = now.clone();
        if !moved {
            if within.is_none() && viewer.at < cell.at && cell.at < viewer.end {
                problems.push(format!("cell {i} runs in the viewer outside every stage"));
            }
            continue;
        }
        number += 1;
        if within != Some(number - 1) {
            problems.push(format!(
                "cell {i} ({}) makes stage {number}, but sits in stage {:?}; it printed {printed}",
                cell.cmd,
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
            "these committed pictures are not what the kernel draws.\nIf the change is intended:  (cd {out} && cp {} {dir}/)",
            to_copy.join(" ")
        ));
    }
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}
