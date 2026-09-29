//! The claims the chapter makes about time, each one run against a clock the test holds.
//!
//! ```text
//! cargo test -p time-resource --test time
//! ```
//!
//! Never the wall clock: a minute boundary happens exactly where a test puts it, so "cached
//! at 14:05:59.999, computed at 14:06:00.000" is a fact these tests check, not a race they
//! hope to win.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Clock, Error, Expiry, Iri, Kernel, Request, Result, Time, Verb,
};
use time_resource::{
    event_name, kernel_with_clock, millis_at, space, until_name, ManualClock, INSTANT, NOW, TODAY,
    VIEW_CLOCK,
};

fn iri(name: &str) -> Iri {
    Iri::parse(name).expect("a valid IRI")
}

fn source(kernel: &Kernel, name: &str) -> Result<String> {
    let repr = block_on(kernel.issue(Request::new(Verb::Source, iri(name)), &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn expiry(kernel: &Kernel, name: &str) -> Expiry {
    block_on(kernel.issue(Request::new(Verb::Source, iri(name)), &Capability::root()))
        .expect("an answer")
        .expiry
}

fn sink(kernel: &Kernel, name: &str, content: &str) -> Result<String> {
    let request = Request::new(Verb::Sink, iri(name))
        .with_arg("content", ArgRef::Inline(content.as_bytes().to_vec()));
    let repr = block_on(kernel.issue(request, &Capability::root()))?;
    Ok(String::from_utf8_lossy(&repr.bytes).into_owned())
}

fn cached(kernel: &Kernel, name: &str) -> bool {
    kernel.is_cached(&Request::new(Verb::Source, iri(name)), &Capability::root())
}

/// A host with the chapter's names and a clock the test holds, stopped at `at`.
fn host(at: u64) -> (Kernel, Arc<ManualClock>) {
    let clock = Arc::new(ManualClock::at(at));
    (
        kernel_with_clock(Arc::clone(&clock) as Arc<dyn Clock>),
        clock,
    )
}

#[test]
fn a_kernel_without_a_clock_says_so() {
    let kernel = Kernel::new(Arc::new(space()));
    for name in [NOW, TODAY] {
        match source(&kernel, name) {
            Err(Error::Endpoint(why)) => assert!(why.contains("no clock"), "{why}"),
            other => panic!("{name}: {other:?}"),
        }
    }
}

// ANCHOR: minute
#[test]
fn now_is_cached_until_the_minute_turns_and_not_a_millisecond_longer() {
    let (kernel, clock) = host(millis_at("2026-09-29T14:05Z", 20));

    assert_eq!(source(&kernel, NOW).unwrap(), "14:05");
    assert_eq!(
        expiry(&kernel, NOW),
        Expiry::At(Time::from_millis(millis_at("2026-09-29T14:06Z", 0)))
    );

    clock.set(millis_at("2026-09-29T14:05Z", 59) + 999);
    assert!(cached(&kernel, NOW), "one millisecond before the minute");

    clock.advance(1);
    assert!(!cached(&kernel, NOW), "on the minute");
    assert_eq!(source(&kernel, NOW).unwrap(), "14:06");
}
// ANCHOR_END: minute

#[test]
fn today_is_cached_until_midnight() {
    let (kernel, clock) = host(millis_at("2026-09-29T23:58Z", 0));
    assert_eq!(source(&kernel, TODAY).unwrap(), "2026-09-29");
    clock.set(millis_at("2026-09-29T23:59Z", 59) + 999);
    assert!(cached(&kernel, TODAY));
    assert!(!cached(&kernel, NOW), "now expired a minute ago");
    clock.advance(1);
    assert!(!cached(&kernel, TODAY));
    assert_eq!(source(&kernel, TODAY).unwrap(), "2026-09-30");
}

// ANCHOR: poll
/// The page's poll, as a test: the clock view asked once a second for a minute. The cache
/// answers 59 of the 60; the one computation is the first, and the next one waits for the
/// minute, however often anybody asks.
#[test]
fn a_poll_every_second_computes_once_a_minute() {
    let (kernel, clock) = host(millis_at("2026-09-29T14:05Z", 0));
    let mut computed = 0;
    for _ in 0..60 {
        if !cached(&kernel, VIEW_CLOCK) {
            computed += 1;
        }
        source(&kernel, VIEW_CLOCK).unwrap();
        clock.advance(1000);
    }
    assert_eq!(computed, 1);
    assert!(
        !cached(&kernel, VIEW_CLOCK),
        "14:06:00, and the minute has turned"
    );
    assert_eq!(
        source(&kernel, VIEW_CLOCK).unwrap(),
        "<time datetime=\"14:06\">14:06</time>"
    );
}
// ANCHOR_END: poll

// ANCHOR: countdown
#[test]
fn a_countdown_is_exactly_as_cacheable_as_now() {
    let (kernel, clock) = host(millis_at("2026-09-29T14:06Z", 10));
    let (event, until) = (event_name("tea"), until_name("tea"));

    assert_eq!(sink(&kernel, &event, "+90").unwrap(), "2026-09-29T15:36Z");
    assert_eq!(source(&kernel, &until).unwrap(), "90");

    // It declared no deadline. It has now's.
    assert_eq!(expiry(&kernel, &until), expiry(&kernel, NOW));
    assert!(matches!(expiry(&kernel, &until), Expiry::At(_)));

    // The minute turns: the countdown goes with now, and comes back one less.
    clock.set(millis_at("2026-09-29T14:07Z", 0));
    assert!(!cached(&kernel, &until));
    assert_eq!(source(&kernel, &until).unwrap(), "89");
    assert!(cached(&kernel, &until));

    // The event moves: its golden thread goes, and the countdown with it, mid-minute.
    sink(&kernel, &event, "2026-09-29T14:37Z").unwrap();
    assert!(!cached(&kernel, &until));
    assert!(
        cached(&kernel, NOW),
        "moving the event does not touch the clock"
    );
    assert_eq!(source(&kernel, &until).unwrap(), "30");
}
// ANCHOR_END: countdown

#[test]
fn an_event_is_an_atom_with_one_spelling() {
    let (kernel, clock) = host(millis_at("2026-09-29T14:06Z", 10));
    let event = event_name("launch");

    assert!(matches!(source(&kernel, &event), Err(Error::NotFound(_))));
    assert!(matches!(
        source(&kernel, &until_name("launch")),
        Err(Error::NotFound(_))
    ));

    assert_eq!(
        sink(&kernel, &event, "2026-09-29T14:00Z").unwrap(),
        "2026-09-29T14:00Z"
    );
    assert_eq!(source(&kernel, &until_name("launch")).unwrap(), "-6");

    for refused in [
        "16:00",
        "2026-09-29T16:00",
        "2026-02-30T16:00Z",
        "+090",
        "+",
    ] {
        match sink(&kernel, &event, refused) {
            Err(Error::InvalidArgument { name, .. }) => assert_eq!(name, "content", "{refused}"),
            other => panic!("{refused}: {other:?}"),
        }
    }
    for refused in ["Tea", "2tea", "te_a"] {
        match source(&kernel, &event_name(refused)) {
            Err(Error::InvalidArgument { name, .. }) => assert_eq!(name, "name", "{refused}"),
            other => panic!("{refused}: {other:?}"),
        }
    }

    // `+N` is read from the kernel's clock at the moment of the Sink, and stored.
    clock.set(millis_at("2027-01-01T00:00Z", 0));
    assert_eq!(sink(&kernel, &event, "+1").unwrap(), "2027-01-01T00:01Z");
    let delete = Request::new(Verb::Delete, iri(&event));
    block_on(kernel.issue(delete, &Capability::root())).unwrap();
    assert!(matches!(source(&kernel, &event), Err(Error::NotFound(_))));
}

/// A clock that moves on by one millisecond every time anybody reads it — the kernel's
/// cache included — so a read of `today` and a read of `now` in one resolution fall at
/// different instants, as they do in a browser.
struct Ticking(AtomicU64);

impl Clock for Ticking {
    fn now(&self) -> Time {
        Time::from_millis(self.0.fetch_add(1, Ordering::SeqCst))
    }
}

// ANCHOR: midnight
/// Across midnight, the date and the time are read at different instants, and a countdown
/// built from a date on one side and a time on the other is a day out. Start just before
/// midnight, many times over, so that some resolution straddles it.
#[test]
fn a_countdown_read_across_midnight_is_never_a_day_out() {
    let midnight = millis_at("2026-09-30T00:00Z", 0);
    for before in 1..200 {
        let clock = Arc::new(Ticking(AtomicU64::new(midnight - before)));
        let kernel = kernel_with_clock(clock);
        sink(&kernel, &event_name("tea"), "2026-09-30T00:10Z").unwrap();
        let minutes: i64 = source(&kernel, &until_name("tea"))
            .unwrap()
            .parse()
            .unwrap();
        assert!(
            minutes == 10 || minutes == 11,
            "{before}ms before midnight: {minutes}"
        );
    }
}
// ANCHOR_END: midnight

/// `instant` is the guarded reading with a name: across midnight it is never a torn pair,
/// and it is cached exactly as long as `now`.
#[test]
fn the_instant_is_one_reading_and_as_cacheable_as_now() {
    let midnight = millis_at("2026-09-30T00:00Z", 0);
    for before in 1..200 {
        let clock = Arc::new(Ticking(AtomicU64::new(midnight - before)));
        let kernel = kernel_with_clock(clock);
        let instant = source(&kernel, INSTANT).unwrap();
        assert!(
            instant == "2026-09-29T23:59Z" || instant == "2026-09-30T00:00Z",
            "{before}ms before midnight: {instant}"
        );
    }
    let (kernel, clock) = host(millis_at("2026-09-29T14:05Z", 20));
    assert_eq!(source(&kernel, INSTANT).unwrap(), "2026-09-29T14:05Z");
    assert_eq!(expiry(&kernel, INSTANT), expiry(&kernel, NOW));
    clock.set(millis_at("2026-09-29T14:06Z", 0));
    assert!(!cached(&kernel, INSTANT));
    assert_eq!(source(&kernel, INSTANT).unwrap(), "2026-09-29T14:06Z");
}

/// What the guard prevents, and what it does not need to: a composite that reads `today` and
/// then `now` with no second look at the date, read across midnight, answers a day out — and
/// is never served again, because the date it read expired at the midnight it straddled, and
/// a composite is no fresher than what it read. A torn reading is wrong once, not for a minute.
#[test]
fn an_unguarded_reading_across_midnight_is_wrong_once_and_never_cached() {
    use ikigai_core::{
        AsyncFnEndpoint, EndpointSpace, Fallback, Invocation, InvokeFuture, Representation, Space,
        UriTemplate,
    };
    use ikigai_vocab::TurtleRenderer;

    let torn =
        || {
            AsyncFnEndpoint::new("torn", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
                Box::pin(async move {
                    let date = inv.source(&iri(TODAY)).await?.bytes;
                    let time = inv.source(&iri(NOW)).await?.bytes;
                    let text = format!(
                        "{}T{}Z",
                        String::from_utf8_lossy(&date),
                        String::from_utf8_lossy(&time)
                    );
                    Ok(Representation::new(
                        ikigai_core::ReprType::new("text/plain"),
                        text.into_bytes(),
                    )
                    .cacheable())
                })
            })
        };
    let midnight = millis_at("2026-09-30T00:00Z", 0);
    let mut torn_seen = 0;
    for before in 1..200 {
        let clock = Arc::new(Ticking(AtomicU64::new(midnight - before)));
        let space = Fallback::new(vec![
            Arc::new(space()) as Arc<dyn Space>,
            Arc::new(
                EndpointSpace::new().bind(UriTemplate::parse("urn:test:torn").unwrap(), torn()),
            ),
        ]);
        let kernel =
            Kernel::with_meta_renderer(Arc::new(space), Arc::new(TurtleRenderer)).with_clock(clock);
        let first = source(&kernel, "urn:test:torn").unwrap();
        if first == "2026-09-29T00:00Z" {
            torn_seen += 1;
            assert!(!cached(&kernel, "urn:test:torn"), "a torn answer was kept");
            assert_eq!(
                source(&kernel, "urn:test:torn").unwrap(),
                "2026-09-30T00:00Z"
            );
        }
    }
    assert!(torn_seen > 0, "no resolution straddled midnight");
}

#[test]
fn the_clock_view_is_the_template_composed() {
    let (kernel, _clock) = host(millis_at("2026-09-29T09:41Z", 0));
    assert_eq!(
        source(&kernel, "urn:iki:tutorial:time:template:clock").unwrap(),
        "<time datetime=\"$a{urn:iki:tutorial:time:now}\">$a{urn:iki:tutorial:time:now}</time>"
    );
    assert_eq!(
        source(&kernel, VIEW_CLOCK).unwrap(),
        "<time datetime=\"09:41\">09:41</time>"
    );
    // Compose declares itself cacheable and knows nothing about time; the view has now's
    // deadline all the same.
    assert_eq!(expiry(&kernel, VIEW_CLOCK), expiry(&kernel, NOW));
}
