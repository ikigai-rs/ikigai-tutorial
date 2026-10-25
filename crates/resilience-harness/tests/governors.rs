//! Failover and the bulkhead, through the harness's kernel: the route moves to the backup and
//! back, and a burst of calls parks behind the bulkhead's two slots and moves only when the
//! harness clock does, each call answering at its true time.

use futures::executor::block_on;
use ikigai_core::Verb;
use resilience_harness::{Harness, BURST, CLOCK, MODE, SERVICE};

fn ask(h: &Harness, verb: Verb, name: &str, content: Option<&str>) -> String {
    block_on(h.ask(verb, name, content)).unwrap_or_else(|e| format!("error: {e}"))
}

fn board(h: &Harness, governor: &str) -> String {
    ask(
        h,
        Verb::Source,
        &format!("urn:throttle:state:{governor}"),
        None,
    )
}

#[test]
fn the_failover_moves_to_the_backup_and_back() {
    let h = Harness::new();
    assert!(
        !board(&h, "failover").contains("route"),
        "{}",
        board(&h, "failover")
    );
    ask(&h, Verb::Sink, MODE, Some("fail"));
    assert_eq!(
        ask(&h, Verb::Source, SERVICE, None),
        "ok from the backup (backup call 1)"
    );
    assert!(
        board(&h, "failover").contains("last call answered at candidate 1"),
        "{}",
        board(&h, "failover")
    );
    ask(&h, Verb::Sink, MODE, Some("ok"));
    assert_eq!(ask(&h, Verb::Source, SERVICE, None), "ok (call 2)");
    assert!(
        board(&h, "failover").contains("last call answered at candidate 0"),
        "{}",
        board(&h, "failover")
    );
    // A slow primary is cut off by its timeout, which is transient: the backup answers.
    ask(&h, Verb::Sink, MODE, Some("slow"));
    assert_eq!(
        ask(&h, Verb::Source, SERVICE, None),
        "ok from the backup (backup call 2)"
    );
    assert_eq!(h.clock().elapsed().as_secs(), 2);
}

#[test]
fn a_burst_parks_behind_the_bulkhead_and_moves_with_the_clock() {
    let h = Harness::new();
    ask(&h, Verb::Sink, MODE, Some("slow"));
    assert_eq!(
        ask(&h, Verb::Sink, BURST, Some("5")),
        "started 5 calls to urn:iki:tutorial:harness:work at once: none answered and 5 calls \
         still in flight"
    );
    let bulkhead = || board(&h, "bulkhead");
    assert!(
        bulkhead().contains("2 of 2 running, 3 parked"),
        "{}",
        bulkhead()
    );
    assert_eq!(h.clock().elapsed().as_secs(), 0, "a burst moves no time");
    assert_eq!(
        ask(&h, Verb::Sink, CLOCK, Some("10s")),
        "the harness clock moved 10s: it reads 10s; in the background, 2 calls answered and \
         3 calls still in flight"
    );
    assert!(
        bulkhead().contains("2 of 2 running, 1 parked"),
        "{}",
        bulkhead()
    );
    // Five seconds is not enough for the next two, which started at 10s.
    assert!(ask(&h, Verb::Sink, CLOCK, Some("5s"))
        .ends_with("none answered and 3 calls still in flight"));
    assert!(ask(&h, Verb::Sink, CLOCK, Some("5s"))
        .ends_with("2 calls answered and 1 call still in flight"));
    assert!(
        bulkhead().contains("1 of 2 running, 0 parked"),
        "{}",
        bulkhead()
    );
    assert!(
        ask(&h, Verb::Sink, CLOCK, Some("10s")).ends_with("1 call answered and none is in flight")
    );
    assert!(
        bulkhead().contains("0 of 2 running, 0 parked"),
        "{}",
        bulkhead()
    );
    assert_eq!(h.clock().elapsed().as_secs(), 30);
    assert_eq!(h.state().calls(), 5);
    // Nothing in flight: the clock says only that it moved.
    assert_eq!(
        ask(&h, Verb::Sink, CLOCK, Some("1s")),
        "the harness clock moved 1s: it reads 31s"
    );
    // In `ok` mode a burst answers at once.
    ask(&h, Verb::Sink, MODE, Some("ok"));
    assert!(
        ask(&h, Verb::Sink, BURST, Some("3")).ends_with("3 calls answered and none is in flight")
    );
    assert!(ask(&h, Verb::Sink, BURST, Some("11")).starts_with("error: invalid argument"));
}
