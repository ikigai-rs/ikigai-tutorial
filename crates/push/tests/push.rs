//! Part VII, end to end through a real kernel: the sheet, drawn view by view, and a host that
//! is told when to draw a view again instead of asking.
//!
//! ```text
//! cargo test -p push --test push
//! ```
//!
//! The clock is one the test holds (the time chapter's `ManualClock`), so "the deadline is
//! 14:06:00" is a fact these tests check, not a race.

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Clock, Error, Fallback, Iri, Kernel, ListenSpec, Request, Space, Verb,
};
use ikigai_vocab::TurtleRenderer;
use push::{Drawn, Listening};
use spreadsheet::{CellRef, FeedStore, InputStore};
use time_resource::{millis_at, ManualClock};

const PREFIX: &str = "urn:iki:tutorial:sheet:";

fn request(verb: Verb, name: &str) -> Request {
    Request::new(verb, Iri::parse(name).expect("a name"))
}

fn text(result: Result<ikigai_core::Representation, Error>) -> Result<String, Error> {
    result.map(|r| String::from_utf8_lossy(&r.bytes).into_owned())
}

fn sink_as(kernel: &Kernel, cap: &Capability, name: &str, content: &str) -> Result<String, Error> {
    let sink =
        request(Verb::Sink, name).with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()));
    text(block_on(kernel.issue(sink, cap)))
}

fn sink(kernel: &Kernel, name: &str, content: &str) -> Result<String, Error> {
    sink_as(kernel, &Capability::root(), name, content)
}

fn source_as(kernel: &Kernel, cap: &Capability, name: &str) -> Result<String, Error> {
    text(block_on(kernel.issue(request(Verb::Source, name), cap)))
}

fn type_into(kernel: &Kernel, cell: &str, typed: &str) {
    sink(kernel, &format!("{PREFIX}input:{cell}"), typed)
        .unwrap_or_else(|e| panic!("typing {typed} into {cell}: {e}"));
}

/// The page's kernel, in miniature: the sheet, the time chapter's names, compose, and push,
/// with the listener given the kernel it registers with.
fn host(clock: Arc<ManualClock>) -> Arc<Kernel> {
    let listening = Arc::new(Listening::default());
    let space = Fallback::new(vec![
        Arc::new(spreadsheet::space_with(
            Arc::new(InputStore::default()),
            Arc::new(FeedStore::default()),
        )) as Arc<dyn Space>,
        Arc::new(time_resource::space()),
        Arc::new(ikigai_fn::space()),
        Arc::new(push::space(Arc::clone(&listening))),
    ]);
    let kernel = Arc::new(
        Kernel::with_meta_renderer(Arc::new(space), Arc::new(TurtleRenderer))
            .with_clock(clock as Arc<dyn Clock>),
    );
    listening.attach(&kernel);
    kernel
}

fn view(cell: &str) -> String {
    format!("{PREFIX}view:cell:{cell}")
}

/// Draw every cell's view, one by one, the way the push page does, and remember each answer.
fn draw_all(kernel: &Kernel, drawn: &mut Drawn) {
    for cell in CellRef::all() {
        draw(kernel, drawn, &view(&cell.to_string()));
    }
}

fn draw(kernel: &Kernel, drawn: &mut Drawn, name: &str) {
    let answer = block_on(kernel.issue(request(Verb::Source, name), &Capability::root()))
        .unwrap_or_else(|e| panic!("{name}: {e}"));
    drawn.drew(name, &answer);
}

fn the_market(kernel: &Kernel) {
    sink(kernel, &format!("{PREFIX}feed:acme"), "100").expect("the feed");
    type_into(kernel, "A1", "=FEED(acme)");
    type_into(kernel, "A2", "10");
    type_into(kernel, "A3", "=A1*A2");
    type_into(kernel, "C1", "=6*7");
}

// ANCHOR: authority
#[test]
fn listening_is_an_authority() {
    let kernel = host(Arc::new(ManualClock::at(0)));
    let mine = push::listener_name("mine");
    let reader = Capability::root().attenuate(["urn:cap:iki:tutorial:sheet:scenario:bob"]);
    match sink_as(&kernel, &reader, &mine, PREFIX) {
        Err(Error::Denied(why)) => assert!(why.contains("urn:cap:kernel:listen"), "{why}"),
        other => panic!("a reader without the capability registered a listener: {other:?}"),
    }
    let said = sink(&kernel, &mine, PREFIX).expect("root may listen");
    assert_eq!(
        said,
        "listening for cuts under urn:iki:tutorial:sheet:, as root: every cut there\n"
    );
    assert_eq!(
        source_as(&kernel, &Capability::root(), &mine).unwrap(),
        "heard nothing\n"
    );
}
// ANCHOR_END: authority

// ANCHOR: feed_write
#[test]
fn a_feed_write_is_one_cut_and_makes_exactly_its_readers_stale() {
    let clock = Arc::new(ManualClock::at(millis_at("2026-09-29T14:05Z", 20)));
    let kernel = host(clock);
    the_market(&kernel);
    let mut drawn = Drawn::default();
    draw_all(&kernel, &mut drawn);
    assert_eq!(drawn.len(), 24);

    let listener = kernel
        .listen(ListenSpec::new().prefix(PREFIX), &Capability::root())
        .expect("root may listen");
    sink(&kernel, &format!("{PREFIX}feed:acme"), "101.5").expect("the price moves");

    let batch = listener.drain();
    assert_eq!(batch.dropped, 0);
    assert_eq!(batch.events.len(), 1, "one write, one cut: {batch:?}");
    assert_eq!(
        batch.events[0].thread.as_str(),
        "urn:iki:tutorial:sheet:feed:acme"
    );
    // A1 reads the feed and A3 reads A1: two of the twenty-four views, and no others.
    assert_eq!(drawn.stale(&batch), [view("A1"), view("A3")]);
}
// ANCHOR_END: feed_write

// ANCHOR: minute
#[test]
fn time_does_not_cut_so_the_host_keeps_a_deadline() {
    let clock = Arc::new(ManualClock::at(millis_at("2026-09-29T14:05Z", 20)));
    let kernel = host(Arc::clone(&clock));
    the_market(&kernel);
    type_into(&kernel, "D1", "=NOW()");
    let mut drawn = Drawn::default();
    draw_all(&kernel, &mut drawn);
    let listener = kernel
        .listen(ListenSpec::new().prefix(PREFIX), &Capability::root())
        .expect("root may listen");

    // One view has a deadline, the next minute; every other one lasts until a cut.
    assert_eq!(drawn.deadline(), Some(millis_at("2026-09-29T14:06Z", 0)));
    assert!(drawn.expired(millis_at("2026-09-29T14:05Z", 59)).is_empty());

    clock.set(millis_at("2026-09-29T14:06Z", 0));
    assert!(
        listener.drain().is_empty(),
        "the minute turned, and nobody cut anything"
    );
    assert_eq!(
        drawn.expired(millis_at("2026-09-29T14:06Z", 0)),
        [view("D1")]
    );

    // Drawn again, the view carries the next deadline.
    draw(&kernel, &mut drawn, &view("D1"));
    assert_eq!(drawn.deadline(), Some(millis_at("2026-09-29T14:07Z", 0)));
}
// ANCHOR_END: minute

// ANCHOR: scenario
#[test]
fn a_scenario_edit_is_heard_by_the_shared_sheet_too() {
    let clock = Arc::new(ManualClock::at(millis_at("2026-09-29T14:05Z", 20)));
    let kernel = host(clock);
    the_market(&kernel);
    let mut drawn = Drawn::default();
    draw_all(&kernel, &mut drawn);
    let listener = kernel
        .listen(ListenSpec::new().prefix(PREFIX), &Capability::root())
        .expect("root may listen");

    // Alice overrides A2 in her scenario. The shared sheet does not change...
    let alice = spreadsheet::scenario("alice", Arc::default()).expect("a scenario");
    let write = request(Verb::Sink, &format!("{PREFIX}input:A2"))
        .with_arg("content", ArgRef::Inline(b"12".to_vec()));
    block_on(kernel.issue_in(write, &Capability::root(), alice)).expect("her override");

    // ...but a thread is a name, not a name in a space (ledger item 581): the shared views
    // that read A2 are stale too, and are drawn again to show the same values.
    let batch = listener.drain();
    let cut: Vec<&str> = batch.events.iter().map(|e| e.thread.as_str()).collect();
    assert_eq!(cut, ["urn:iki:tutorial:sheet:input:A2"]);
    assert_eq!(drawn.stale(&batch), [view("A2"), view("A3")]);
    for cell in ["A2", "A3"] {
        let before = source_as(&kernel, &Capability::root(), &view(cell)).unwrap();
        draw(&kernel, &mut drawn, &view(cell));
        assert_eq!(
            source_as(&kernel, &Capability::root(), &view(cell)).unwrap(),
            before
        );
    }
}
// ANCHOR_END: scenario

// ANCHOR: narrow
#[test]
fn a_listener_under_a_narrow_capability_hears_only_what_its_own_reads_rest_on() {
    let clock = Arc::new(ManualClock::at(millis_at("2026-09-29T14:05Z", 20)));
    let kernel = host(clock);
    the_market(&kernel);
    let session = Capability::root().attenuate(["urn:cap:kernel:listen"]);
    let mine = push::listener_name("session");
    let said = sink_as(&kernel, &session, &mine, PREFIX).expect("the session may listen");
    assert!(said.contains("under this capability"), "{said}");

    // The session reads A3. Somebody else writes the feed A3 rests on, and a cell it never read.
    source_as(&kernel, &session, &format!("{PREFIX}cell:A3")).expect("A3");
    sink(&kernel, &format!("{PREFIX}feed:acme"), "102").unwrap();
    type_into(&kernel, "D6", "hello");

    let heard = source_as(&kernel, &session, &mine).unwrap();
    assert!(
        heard.starts_with("heard 1 cut\ncut urn:iki:tutorial:sheet:feed:acme\n"),
        "{heard}"
    );
    assert!(!heard.contains("D6"), "{heard}");
}
// ANCHOR_END: narrow

// ANCHOR: dropped
#[test]
fn a_full_queue_counts_a_drop_and_the_host_draws_everything_again() {
    let clock = Arc::new(ManualClock::at(millis_at("2026-09-29T14:05Z", 20)));
    let kernel = host(clock);
    the_market(&kernel);
    let mut drawn = Drawn::default();
    draw_all(&kernel, &mut drawn);
    let listener = kernel
        .listen(
            ListenSpec::new().prefix(PREFIX).capacity(1),
            &Capability::root(),
        )
        .expect("root may listen");
    type_into(&kernel, "D5", "1");
    type_into(&kernel, "D6", "2");
    let batch = listener.drain();
    assert_eq!((batch.events.len(), batch.dropped), (1, 1));
    assert_eq!(drawn.stale(&batch).len(), 24);
    assert!(push::render(&batch).ends_with(
        "dropped 1: the queue was full, so nothing after the last cut above is known\n"
    ));
}
// ANCHOR_END: dropped
