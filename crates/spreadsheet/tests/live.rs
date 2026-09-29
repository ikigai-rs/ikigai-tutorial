//! Parts IV and V, end to end through a real kernel: cells that read the time, and cells that
//! read a feed something else writes.
//!
//! ```text
//! cargo test -p spreadsheet --test live
//! ```
//!
//! The clock is always one the test holds (the time chapter's `ManualClock`), so "cached at
//! 14:05:59.999, computed at 14:06:00.000" is a fact these tests check, not a race.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Clock, Error, Expiry, Fallback, Iri, Kernel, Request, Space, Time, Verb,
};
use ikigai_vocab::TurtleRenderer;
use spreadsheet::{FeedStore, InputStore};
use time_resource::{millis_at, ManualClock};

fn request(verb: Verb, name: &str) -> Request {
    Request::new(verb, Iri::parse(name).expect("a name"))
}

fn source(kernel: &Kernel, name: &str) -> Result<String, Error> {
    block_on(kernel.issue(request(Verb::Source, name), &Capability::root()))
        .map(|r| String::from_utf8_lossy(&r.bytes).into_owned())
}

fn expiry(kernel: &Kernel, name: &str) -> Expiry {
    block_on(kernel.issue(request(Verb::Source, name), &Capability::root()))
        .expect("an answer")
        .expiry
}

fn sink(kernel: &Kernel, name: &str, content: &str) -> Result<String, Error> {
    let sink =
        request(Verb::Sink, name).with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()));
    block_on(kernel.issue(sink, &Capability::root()))
        .map(|r| String::from_utf8_lossy(&r.bytes).into_owned())
}

fn value(kernel: &Kernel, cell: &str) -> String {
    source(kernel, &format!("urn:iki:tutorial:sheet:cell:{cell}"))
        .unwrap_or_else(|e| panic!("cell {cell}: {e}"))
}

fn type_into(kernel: &Kernel, cell: &str, typed: &str) {
    sink(
        kernel,
        &format!("urn:iki:tutorial:sheet:input:{cell}"),
        typed,
    )
    .unwrap_or_else(|e| panic!("typing {typed} into {cell}: {e}"));
}

fn cached(kernel: &Kernel, name: &str) -> bool {
    kernel.is_cached(&request(Verb::Source, name), &Capability::root())
}

fn value_cached(kernel: &Kernel, cell: &str) -> bool {
    cached(kernel, &format!("urn:iki:tutorial:sheet:cell:{cell}"))
}

/// Which of the sheet's twenty-four values are not cached right now: what the page's cache
/// viewer says.
fn uncached(kernel: &Kernel) -> Vec<String> {
    spreadsheet::CellRef::all()
        .map(|cell| cell.to_string())
        .filter(|cell| !value_cached(kernel, cell))
        .collect()
}

/// A host for the sheet: its names, the time chapter's, and compose, under `clock` if there
/// is one — the page's kernel, in miniature.
fn host(clock: Option<Arc<dyn Clock>>) -> Kernel {
    let space = Fallback::new(vec![
        Arc::new(spreadsheet::space_with(
            Arc::new(InputStore::default()),
            Arc::new(FeedStore::default()),
        )) as Arc<dyn Space>,
        Arc::new(time_resource::space()),
        Arc::new(ikigai_fn::space()),
    ]);
    let kernel = Kernel::with_meta_renderer(Arc::new(space), Arc::new(TurtleRenderer));
    match clock {
        Some(clock) => kernel.with_clock(clock),
        None => kernel,
    }
}

fn clocked(at: u64) -> (Kernel, Arc<ManualClock>) {
    let clock = Arc::new(ManualClock::at(at));
    (host(Some(Arc::clone(&clock) as Arc<dyn Clock>)), clock)
}

// ANCHOR: minute
#[test]
fn a_time_cell_goes_on_the_minute_and_takes_every_cell_that_reads_it_along() {
    let (kernel, clock) = clocked(millis_at("2026-09-29T14:05Z", 20));
    type_into(&kernel, "A1", "=NOW()");
    type_into(&kernel, "A2", "=TODAY()");
    type_into(&kernel, "B1", "=A1");
    type_into(&kernel, "C1", "=6*7");
    assert_eq!(value(&kernel, "A1"), "2026-09-29T14:05Z");
    assert_eq!(value(&kernel, "B1"), "2026-09-29T14:05Z");
    assert_eq!(value(&kernel, "A2"), "2026-09-29");
    assert_eq!(value(&kernel, "C1"), "42");

    // B1's formula says nothing about time. Its value has the time's deadline all the same;
    // C1's, which read no time, has none.
    let next_minute = Expiry::At(Time::from_millis(millis_at("2026-09-29T14:06Z", 0)));
    assert_eq!(
        expiry(&kernel, "urn:iki:tutorial:sheet:cell:B1"),
        next_minute
    );
    assert_eq!(
        expiry(&kernel, "urn:iki:tutorial:sheet:cell:C1"),
        Expiry::Never
    );

    // The grid reads every value, so it is cached until the minute too.
    source(&kernel, spreadsheet::VIEW_GRID).unwrap();
    assert_eq!(expiry(&kernel, spreadsheet::VIEW_GRID), next_minute);
    assert!(uncached(&kernel).is_empty());

    // The minute turns: nobody wrote anything, and exactly the two values that read the
    // time are gone. The date and the arithmetic stay.
    clock.set(millis_at("2026-09-29T14:06Z", 0));
    assert_eq!(uncached(&kernel), ["A1", "B1"]);
    assert!(!cached(&kernel, spreadsheet::VIEW_GRID));
    source(&kernel, spreadsheet::VIEW_GRID).unwrap();
    assert!(uncached(&kernel).is_empty());
    assert_eq!(value(&kernel, "B1"), "2026-09-29T14:06Z");

    // Midnight: the date goes too.
    clock.set(millis_at("2026-09-30T00:00Z", 5));
    assert_eq!(uncached(&kernel), ["A1", "B1", "A2"]);
    assert_eq!(value(&kernel, "A2"), "2026-09-30");
}
// ANCHOR_END: minute

#[test]
fn a_kernel_with_no_clock_says_the_time_is_not_available_and_keeps_saying_it() {
    let kernel = host(None);
    type_into(&kernel, "A1", "=NOW()");
    type_into(&kernel, "A2", "=TODAY()");
    assert_eq!(value(&kernel, "A1"), "#N/A");
    assert_eq!(value(&kernel, "A2"), "#N/A");
    // A kernel's clock is fixed when it is built, so the answer is too.
    assert!(value_cached(&kernel, "A1"));
    assert_eq!(
        expiry(&kernel, "urn:iki:tutorial:sheet:cell:A1"),
        Expiry::Never
    );
}

/// A clock that moves on one millisecond every time anybody reads it, so a date and a time
/// read in one resolution fall at different instants.
struct Ticking(AtomicU64);

impl Clock for Ticking {
    fn now(&self) -> Time {
        Time::from_millis(self.0.fetch_add(1, Ordering::SeqCst))
    }
}

// ANCHOR: torn
/// `NOW()` reads the date and the time, and across midnight it never tears them: it reads the
/// time chapter's `instant`, which is one reading, not two.
#[test]
fn now_is_never_a_day_out_across_midnight() {
    let midnight = millis_at("2026-09-30T00:00Z", 0);
    for before in 1..200 {
        let kernel = host(Some(Arc::new(Ticking(AtomicU64::new(midnight - before)))));
        type_into(&kernel, "A1", "=NOW()");
        let now = value(&kernel, "A1");
        assert!(
            now == "2026-09-29T23:59Z" || now == "2026-09-30T00:00Z",
            "{before}ms before midnight: {now}"
        );
    }
}
// ANCHOR_END: torn

// ANCHOR: feed
#[test]
fn a_write_to_a_feed_recomputes_exactly_the_cells_that_read_it() {
    let kernel = host(None);
    let acme = "urn:iki:tutorial:sheet:feed:acme";
    sink(&kernel, acme, "100").unwrap();
    type_into(&kernel, "A1", "=FEED(acme)");
    type_into(&kernel, "A2", "10");
    type_into(&kernel, "A3", "=A1*A2");
    type_into(&kernel, "C1", "=6*7");
    source(&kernel, spreadsheet::VIEW_GRID).unwrap();
    assert_eq!(value(&kernel, "A3"), "1000");
    assert!(uncached(&kernel).is_empty());

    // Something that is not the sheet writes the feed: the price, and what reads it, go.
    sink(&kernel, acme, "101.5").unwrap();
    assert_eq!(uncached(&kernel), ["A1", "A3"]);
    assert_eq!(value(&kernel, "A3"), "1015");

    // The market moves it: the same write, from an endpoint.
    let moved = sink(&kernel, "urn:iki:tutorial:sheet:tick:acme", "").unwrap();
    assert_eq!(source(&kernel, acme).unwrap(), moved);
    assert_eq!(uncached(&kernel), ["A1", "A3"]);
    assert_eq!(value(&kernel, "A1"), moved);
}
// ANCHOR_END: feed

// ANCHOR: unwritten
/// A feed nothing has written to is `#N/A`, cached — and the first write cuts it, because the
/// fallback caught the ATOM's own NotFound, so it hangs from the feed's thread.
#[test]
fn an_unwritten_feed_is_not_available_until_its_first_write() {
    let kernel = host(None);
    type_into(&kernel, "B1", "=FEED(globex)");
    assert_eq!(value(&kernel, "B1"), "#N/A");
    assert!(value_cached(&kernel, "B1"));
    sink(&kernel, "urn:iki:tutorial:sheet:feed:globex", "55").unwrap();
    assert!(!value_cached(&kernel, "B1"));
    assert_eq!(value(&kernel, "B1"), "55");
}
// ANCHOR_END: unwritten

#[test]
fn a_feed_has_one_spelling_and_the_market_moves_only_prices() {
    let kernel = host(None);
    for refused in ["Acme", "2x", "ac-me"] {
        match source(&kernel, &format!("urn:iki:tutorial:sheet:feed:{refused}")) {
            Err(Error::InvalidArgument { name, .. }) => assert_eq!(name, "name", "{refused}"),
            other => panic!("{refused}: {other:?}"),
        }
    }
    assert!(matches!(
        source(&kernel, "urn:iki:tutorial:sheet:feed:acme"),
        Err(Error::NotFound(_))
    ));
    // An unwritten feed opens at 100 and moves from there.
    let opened = sink(&kernel, "urn:iki:tutorial:sheet:tick:acme", "").unwrap();
    let cents = (opened.parse::<f64>().unwrap() * 100.0).round() as u64;
    assert_eq!(cents, spreadsheet::next_price(10_000));

    sink(&kernel, "urn:iki:tutorial:sheet:feed:acme", "halted").unwrap();
    assert!(matches!(
        sink(&kernel, "urn:iki:tutorial:sheet:tick:acme", ""),
        Err(Error::Conflict(_))
    ));
    // A feed may hold text; a formula that does arithmetic on it says so.
    type_into(&kernel, "A1", "=FEED(acme)+1");
    assert_eq!(value(&kernel, "A1"), "#VALUE");
    // Reading the market is refused: it is a write.
    assert!(source(&kernel, "urn:iki:tutorial:sheet:tick:acme").is_err());
}

// ANCHOR: poll
/// The page's poll, as a test: the grid asked every two seconds while the market writes the
/// feed every three. A poll between writes is answered from the cache; a poll after one
/// computes the feed's dependents and nothing else.
#[test]
fn a_polled_grid_computes_only_after_a_write_and_only_what_it_cut() {
    let kernel = host(None);
    type_into(&kernel, "A1", "=FEED(acme)");
    type_into(&kernel, "A2", "10");
    type_into(&kernel, "A3", "=A1*A2");
    type_into(&kernel, "B1", "=SUM(C1:C6)");
    type_into(&kernel, "C1", "=6*7");
    let mut polls = Vec::new();
    for second in 0..12 {
        if second % 3 == 0 {
            sink(&kernel, "urn:iki:tutorial:sheet:tick:acme", "").unwrap();
        }
        if second % 2 == 0 {
            let was_cached = cached(&kernel, spreadsheet::VIEW_GRID);
            let recomputed = uncached(&kernel);
            source(&kernel, spreadsheet::VIEW_GRID).unwrap();
            polls.push((second, was_cached, recomputed));
        }
    }
    let wrote = ["A1".to_string(), "A3".to_string()];
    for (second, was_cached, recomputed) in &polls[1..] {
        // The market wrote since the last poll (at 3 and 9, for the polls at 4 and 10; at 6,
        // for the poll at 6 itself) exactly when the poll computed, and then only its cells.
        let wrote_since = [4, 6, 10].contains(second);
        assert_eq!(*was_cached, !wrote_since, "the poll at {second}s");
        if wrote_since {
            assert_eq!(recomputed, &wrote, "the poll at {second}s");
        } else {
            assert!(recomputed.is_empty(), "the poll at {second}s");
        }
    }
}
// ANCHOR_END: poll
