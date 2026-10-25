//! Every runnable cell and every harness control of The ikigai Architecture, replayed natively
//! against the kernel the page runs (`architecture_book_wasm`), and the harness's committed
//! stages (their pictures in both schemes, their words and facts) held to what the kernel says.
//!
//! A cell runs the way `js/run.js` runs it: in page order, line by line, on one fresh page,
//! in the chain its markup names (`data-harness='overlays'` is `harness:overlays`), and its
//! output must match the page's "Expected output" (`…` matching what the kernel mints, as in a
//! transcript). A control runs the way `js/stages.js` runs it, and after every cell or control
//! in a stages viewer's chain the viewer's lines are read, as `js/stages.js` reads them
//! (`book_transcripts::stages`, which every book's stages test shares).

use architecture_book_wasm::{eval_on, Page};
use book_transcripts::stages::{self, attribute, unescape, Pictures, Reply, Snapshot, Step};
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

fn lines(cmd: &str) -> Vec<&str> {
    stages::lines(cmd)
}

/// One line on `page` in `chain`, as the kernel answers it.
fn reply(page: &Page, chain: Option<&str>, line: &str) -> Reply {
    let json: serde_json::Value =
        serde_json::from_str(&block_on(eval_on(page, chain, line))).expect("the reply is JSON");
    let text = |key: &str| json[key].as_str().unwrap_or_default().to_string();
    Reply {
        kind: text("kind"),
        text: text("text"),
        cache: text("cache"),
    }
}

/// Run `cmd`'s lines in `chain` on `page`, as the page does, and print what it would show.
fn run(page: &Page, chain: Option<&str>, cmd: &str) -> String {
    lines(cmd)
        .into_iter()
        .map(|line| reply(page, chain, line).render())
        .collect()
}

/// Every cell of a page, in page order on one fresh page, against the output the page shows.
fn run_page_in_order(file: &str, expected_cells: usize) {
    let text = page(file);
    let cells = cells(&text);
    let viewers = stages::viewers(&text);
    assert_eq!(cells.len(), expected_cells, "{file}'s cells: {cells:#?}");
    let page = Page::default();
    let mut misses = Vec::new();
    for (i, cell) in cells.iter().enumerate() {
        let printed = run(&page, cell.chain.as_deref(), &cell.cmd);
        // As `js/stages.js` does after every step in a viewer's chain: read its lines.
        for viewer in viewers.iter().filter(|v| v.chain == cell.chain) {
            viewer.read(&|line| reply(&page, viewer.chain.as_deref(), line));
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
    let viewer = &stages::viewers(&page("overlays/overlays.md"))[0];
    let labels: Vec<&str> = viewer.controls.iter().map(|c| c.label.as_str()).collect();
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
    for control in &viewer.controls {
        let page = Page::default();
        for line in lines(&control.cmd) {
            let reply = reply(&page, viewer.chain.as_deref(), line);
            assert_eq!(
                reply.kind, "output",
                "{}: `{line}` answered {reply:?}",
                control.label
            );
        }
    }
}

/// The controls, pressed in the order a reader trying the breaker presses them: what each
/// answers, and what the board says after, as `js/stages.js` reads them.
#[test]
fn the_controls_trip_the_breaker_and_close_it_again() {
    let chapter = page("overlays/overlays.md");
    let viewer = &stages::viewers(&chapter)[0];
    let chain = viewer.chain.as_deref();
    let page = Page::default();
    let seen = std::cell::RefCell::new(Snapshot::start(viewer));
    let press = |label: &str| {
        let control = viewer
            .controls
            .iter()
            .find(|c| c.label == label)
            .unwrap_or_else(|| panic!("no control `{label}`"));
        let printed = run(&page, chain, &control.cmd);
        let now = viewer.read(&|line| reply(&page, chain, line));
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

/// The chapter's stages ARE the walkthrough, as the kernel answers it
/// ([`stages::walkthrough`]): the page's cells run in page order on one fresh page, every cell
/// that moves the words or the picture makes the next stage and sits in that stage's section,
/// each stage's section holds the result block the kernel answers, and its committed pictures
/// are the kernel's own, byte for byte, in each scheme. `BOOK_PICTURES_WRITE=1` rewrites the
/// pictures that differ; a result block that differs is printed.
#[test]
fn the_chapters_stages_are_the_walkthrough_as_the_kernel_draws_it() {
    let chapter = page("overlays/overlays.md");
    let viewers = stages::viewers(&chapter);
    assert_eq!(viewers.len(), 1, "the chapter's viewers: {viewers:#?}");
    let viewer = &viewers[0];
    assert_eq!(
        viewer.alt,
        resilience_harness::PICTURE_ALT,
        "data-alt is not PICTURE_ALT"
    );
    let chain = viewer.chain.as_deref();
    let steps: Vec<Step> = cells(&chapter)
        .into_iter()
        .map(|c| Step {
            at: c.at,
            chain: c.chain,
            cmd: c.cmd,
        })
        .collect();
    let dir = std::path::PathBuf::from(format!(
        "{}/../../books/architecture/src/overlays",
        env!("CARGO_MANIFEST_DIR")
    ));
    let out = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR"));
    let fresh = Page::default();
    let page = Page::default();
    let (problems, _) = stages::walkthrough(
        viewer,
        &steps,
        &|line| reply(&fresh, chain, line),
        &|step| run(&page, step.chain.as_deref(), &step.cmd),
        &|line| reply(&page, chain, line),
        &Pictures {
            dir: &dir,
            stem: "harness-stage",
            out: &out,
        },
    );
    assert!(problems.is_empty(), "{}", problems.join("\n\n"));
}
