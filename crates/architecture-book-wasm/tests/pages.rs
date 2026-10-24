//! Every runnable cell and every harness control of The ikigai Architecture, replayed natively
//! against the kernel the page runs (`architecture_book_wasm`), and the harness's committed
//! starting picture held to what the harness draws.
//!
//! A cell runs the way `js/run.js` runs it: in page order, line by line, on one fresh page,
//! in the chain its markup names (`data-harness='overlays'` is `harness:overlays`), and its
//! output must match the page's "Expected output" (`…` matching what the kernel mints, as in a
//! transcript). A control runs the way `js/harness.js` runs it.

use architecture_book_wasm::{eval_on, Page};
use book_transcripts::{matches, normalize};
use futures::executor::block_on;

/// One runnable cell: the chain its markup names, its line(s), and its expected output.
#[derive(Debug)]
struct Cell {
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
    let mut rest = page;
    while let Some(at) = rest.find("<div class=\"ikigai-run\"") {
        let after = &rest[at..];
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
            chain,
            cmd,
            expected: unescape(&body[..close]),
        });
        rest = &body[close..];
    }
    found
}

/// The controls each harness figure on a page offers, in page order.
fn controls(page: &str) -> Vec<Control> {
    let mut found = Vec::new();
    let mut rest = page;
    while let Some(at) = rest.find("<figure class=\"ikigai-harness\"") {
        let figure = &rest[at..rest[at..]
            .find("</figure>")
            .map(|e| at + e)
            .expect("closed")];
        let chain = format!(
            "harness:{}",
            attribute(figure, "data-harness").expect("a harness figure names its harness")
        );
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
    let cells = cells(&page(file));
    assert_eq!(cells.len(), expected_cells, "{file}'s cells: {cells:#?}");
    let page = Page::default();
    let mut misses = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        let printed = run(&page, cell.chain.as_deref(), &cell.cmd);
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
    let controls = controls(&page("overlays/overlays.md"));
    let page = Page::default();
    let press = |label: &str| {
        let control = controls
            .iter()
            .find(|c| c.label == label)
            .unwrap_or_else(|| panic!("no control `{label}`"));
        run(&page, Some(&control.chain), &control.cmd)
    };
    let board = || {
        let host = page.harness("overlays").unwrap();
        block_on(host.harness().words()).unwrap()
    };

    press("Set the upstream to fail");
    let ten = press("Call ten times");
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

    press("Wait 5 seconds");
    assert!(board().contains(": half-open,"), "{}", board());
    press("Set the upstream to ok");
    assert_eq!(press("Call once"), "ok (call 4)\n[uncacheable]\n");
    assert!(board().contains(": closed, 0 consecutive"), "{}", board());
}

/// The picture on the page before anything runs IS the harness as it starts, byte for byte,
/// and its `alt` is the one `js/harness.js` keeps giving it. On a mismatch the picture the
/// harness draws is written beside the build, and the failure names the command that commits it.
#[test]
fn the_committed_picture_is_the_harness_as_it_starts() {
    let drawn = block_on(resilience_harness::Harness::new().picture()).unwrap();
    let file = "harness-start.svg";
    let committed = format!(
        "{}/../../books/architecture/src/overlays/{file}",
        env!("CARGO_MANIFEST_DIR")
    );
    if std::fs::read_to_string(&committed).ok().as_deref() != Some(drawn.as_str()) {
        let out = format!("{}/{file}", env!("CARGO_TARGET_TMPDIR"));
        std::fs::write(&out, &drawn).expect("write the picture the harness draws");
        panic!("the committed {file} is not what the harness draws.\nIf the change is intended:  cp {out} {committed}");
    }
    let chapter = page("overlays/overlays.md");
    let alt = format!("alt=\"{}\"", resilience_harness::PICTURE_ALT);
    assert!(
        chapter.contains(&alt),
        "the committed picture's alt is not PICTURE_ALT"
    );
}
