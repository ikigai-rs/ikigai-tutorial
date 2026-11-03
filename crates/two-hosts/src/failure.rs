//! Who may repeat a failure, and for how long: the code taught by "Failure has a class".
//!
//! [`mounts`](crate::mounts) shows the one rule a degrading mount needs from a failure's
//! class. This module is the rest of it: reading a class, declaring the classes an action
//! can fail with, and the two things a run carries from the door that minted it, a retry
//! budget and a deadline, including across a socket.

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use ikigai_core::{
    Description, EndpointSpace, Error, Exact, FailureClass, FnEndpoint, Invocation, Kernel,
    ReprType, Representation, SystemClock, Verb,
};

// ANCHOR: who_may_repeat
/// What a client does with a failure it received. It reads the error it got, not a
/// declaration, and a class it has never heard of repeats nothing: `FailureClass` is
/// `#[non_exhaustive]`, and the wildcard arm is that rule.
pub fn who_may_repeat(class: FailureClass) -> &'static str {
    match class {
        FailureClass::Transient => "an overlay holding retry budget",
        FailureClass::RetryAfter => "an overlay holding budget and time, else the caller",
        FailureClass::OverBudget => "only a holder of a larger bound, with a new request",
        FailureClass::Refused => "the holder of the credential, once, after renewing it",
        FailureClass::Denied | FailureClass::Permanent => "nobody",
        _ => "nobody automatically",
    }
}
// ANCHOR_END: who_may_repeat

// ANCHOR: declared_feed
/// A feed that says how it fails: the origin may limit it, or may be down. Its first
/// `failures` reads answer `error`, and every read after that answers `fresh`.
///
/// The declaration changes nothing at run time. The error returned is the error produced,
/// whatever this says.
pub fn feed(
    failures: u32,
    error: impl Fn() -> Error + Send + Sync + 'static,
) -> (FnEndpoint, Arc<AtomicU32>) {
    let reads = Arc::new(AtomicU32::new(0));
    let counter = Arc::clone(&reads);
    let endpoint = FnEndpoint::new("feed", move |_: &Invocation<'_>| {
        if counter.fetch_add(1, Ordering::SeqCst) < failures {
            return Err(error());
        }
        Ok(Representation::new(
            ReprType::new("text/plain"),
            b"fresh".to_vec(),
        ))
    })
    .with_description(
        Description::new("feed")
            .verb(Verb::Source)
            .fails(FailureClass::RetryAfter)
            .fails(FailureClass::Transient),
    );
    (endpoint, reads)
}
// ANCHOR_END: declared_feed

/// The name [`feed`] and [`feed_space`] answer at.
pub const FEED: &str = "urn:iki:tutorial:feed";

/// [`feed`], bound at [`FEED`]; the handle counts its reads.
pub fn feed_space(
    failures: u32,
    error: impl Fn() -> Error + Send + Sync + 'static,
) -> (EndpointSpace, Arc<AtomicU32>) {
    let (endpoint, reads) = feed(failures, error);
    (EndpointSpace::new().bind(Exact::new(FEED), endpoint), reads)
}

/// The name [`run_report`] answers at.
pub const RUN_REPORT: &str = "urn:iki:tutorial:run-report";

// ANCHOR: run_report
/// What a run looks like from inside it: the retries its pool has left and the
/// milliseconds its deadline has left, `-` for either one it does not have.
pub fn run_report() -> FnEndpoint {
    FnEndpoint::new("run-report", |inv: &Invocation<'_>| {
        let retries = inv.retry_budget().map(|pool| pool.remaining().to_string());
        let left = inv.time_left().map(|left| left.as_millis().to_string());
        let report = format!(
            "retries={} left={}",
            retries.as_deref().unwrap_or("-"),
            left.as_deref().unwrap_or("-")
        );
        Ok(Representation::new(
            ReprType::new("text/plain"),
            report.into_bytes(),
        ))
    })
}

/// The kernel on the far side of the socket. It sets its own policy, a base of 0 and a
/// ceiling of 3, and a clock to measure a deadline against. Without a clock `time_left`
/// has nothing to subtract from, and answers `None`.
pub fn run_report_kernel() -> Kernel {
    let space = EndpointSpace::new().bind(Exact::new(RUN_REPORT), run_report());
    Kernel::new(Arc::new(space))
        .with_retry_budget(0, 3)
        .with_clock(Arc::new(SystemClock))
}
// ANCHOR_END: run_report

/// A pair of a run-report's two numbers, for a test or a listing to compare.
pub fn parse_run_report(bytes: &[u8]) -> (Option<u32>, Option<u64>) {
    let text = String::from_utf8_lossy(bytes);
    let field = |name: &str| {
        text.split_whitespace()
            .find_map(|word| word.strip_prefix(name))
            .map(str::to_string)
    };
    (
        field("retries=").and_then(|n| n.parse().ok()),
        field("left=").and_then(|n| n.parse().ok()),
    )
}

/// Dial a server that is starting on another thread, for up to ten seconds. The first dial
/// legitimately races the bind.
#[cfg(unix)]
pub fn dial_when_up(path: &std::path::Path) -> ikigai_ipc::IpcResolver {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    loop {
        match ikigai_ipc::connect(path) {
            Ok(resolver) => return resolver,
            Err(_) if std::time::Instant::now() < deadline => {
                std::thread::sleep(std::time::Duration::from_millis(20));
            }
            Err(error) => panic!("the server never came up on {}: {error}", path.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mounts::prefer;
    use futures::executor::block_on;
    use ikigai_core::{
        cap_retry, Capability, Clock, FixedClock, Iri, IssueContext, Request, RetryLimits, Space,
        TraceEvent, Tracer, UNDECLARED_FAILURE_NOTE,
    };
    use ikigai_resolve::{MountedRemote, Resolver};
    use ikigai_throttle::{Failover, Retry, Timer};
    use std::path::PathBuf;
    use std::sync::Mutex;
    use std::time::Duration;

    fn read(iri: &str) -> Request {
        Request::new(Verb::Source, Iri::parse(iri).expect("iri"))
    }

    fn socket_path(label: &str) -> PathBuf {
        std::env::temp_dir().join(format!("ikigai-fail-{label}-{}.sock", std::process::id()))
    }

    fn unavailable() -> Error {
        Error::Unavailable("the origin is down".into())
    }

    /// Every variant, its class, whether the older predicate says transient, and what the
    /// HTTP edge answers. One table, so the chapter's table is a claim this test checks.
    #[test]
    fn every_error_has_a_class_and_an_edge_status() {
        let verb_refused = Error::Unsupported {
            verb: Some(Verb::Sink),
            detail: "append-only".into(),
        };
        let no_facility = Error::Unsupported {
            verb: None,
            detail: "no clock".into(),
        };
        let cases = [
            (Error::Timeout("slow".into()), "transient", true, 504),
            (unavailable(), "transient", true, 503),
            (
                Error::RetryAfter {
                    delay: Some(Duration::from_millis(1_500)),
                    detail: "limited".into(),
                },
                "retry-after",
                false,
                429,
            ),
            (
                Error::OverBudget {
                    detail: "past 5000 ms".into(),
                },
                "over-budget",
                false,
                422,
            ),
            (Error::Denied("no grant".into()), "denied", false, 403),
            (Error::Refused("401".into()), "refused", false, 502),
            (Error::NotFound("gone".into()), "permanent", false, 404),
            (Error::Conflict("taken".into()), "permanent", false, 409),
            (verb_refused, "permanent", false, 405),
            (no_facility, "permanent", false, 501),
            (Error::MissingArgument("in".into()), "permanent", false, 400),
            (
                Error::InvalidArgument {
                    name: "in".into(),
                    detail: "not a number".into(),
                },
                "permanent",
                false,
                400,
            ),
            (
                Error::Unresolved(Iri::parse(FEED).expect("iri")),
                "permanent",
                false,
                404,
            ),
            (
                Error::DepthExceeded {
                    depth: 65,
                    target: Iri::parse(FEED).expect("iri"),
                },
                "permanent",
                false,
                500,
            ),
            (Error::Endpoint("boom".into()), "unknown", false, 500),
        ];
        for (error, class, transient, status) in cases {
            assert_eq!(error.class().word(), class, "{error:?}");
            assert_eq!(error.is_transient(), transient, "{error:?}");
            assert_eq!(ikigai_web::error_status(&error), status, "{error:?}");
        }
    }

    #[test]
    fn a_class_nobody_named_repeats_nothing() {
        assert_eq!(who_may_repeat(FailureClass::Denied), "nobody");
        assert_eq!(
            who_may_repeat(FailureClass::Unknown),
            "nobody automatically"
        );
    }

    /// Delivered unchanged, and noted only on a trace.
    #[test]
    fn a_failure_the_declaration_does_not_admit_is_noted_and_not_rewritten() {
        struct Collect(Mutex<Vec<TraceEvent>>);
        impl Tracer for Collect {
            fn record(&self, event: TraceEvent) {
                self.0.lock().unwrap().push(event);
            }
        }

        let (space, _) = feed_space(9, || Error::Endpoint("boom".into()));
        let kernel = Kernel::new(Arc::new(space));
        let tracer = Arc::new(Collect(Mutex::new(Vec::new())));
        let error = block_on(kernel.issue_traced(read(FEED), &Capability::root(), tracer.clone()))
            .expect_err("the feed failed");

        assert_eq!(error, Error::Endpoint("boom".into()));
        let events = tracer.0.lock().unwrap();
        assert!(events[0]
            .notes
            .contains(&(UNDECLARED_FAILURE_NOTE.to_string(), "unknown".to_string())));
    }

    #[test]
    fn a_declared_failure_is_published_in_the_contract() {
        let (space, _) = feed_space(0, unavailable);
        let kernel =
            Kernel::with_meta_renderer(Arc::new(space), Arc::new(ikigai_vocab::TurtleRenderer));
        let meta = Request::new(Verb::Meta, Iri::parse(FEED).expect("iri"));
        let answer = block_on(kernel.issue(meta, &Capability::root())).expect("meta");
        let graph = String::from_utf8_lossy(&answer.bytes);
        assert!(graph.contains("ik:mayFail"), "{graph}");
        assert!(graph.contains("ik:RetryAfterFailure"), "{graph}");
        assert!(graph.contains("ik:TransientFailure"), "{graph}");
    }

    /// Two layers of three attempts, under one pool: one first try and two re-issues, never
    /// three times three.
    #[test]
    fn nested_retries_spend_one_pool() {
        let (space, reads) = feed_space(99, unavailable);
        let kernel =
            Kernel::new(Arc::new(Retry::new(Retry::new(space, 3), 3))).with_retry_budget(0, 2);

        block_on(kernel.issue(read(FEED), &Capability::root())).expect_err("still down");

        assert_eq!(reads.load(Ordering::SeqCst), 3);
    }

    /// With no retry budget configured, the same layers multiply: this is the thing the
    /// budget exists to stop, and what every kernel that configures nothing still does.
    #[test]
    fn with_no_budget_configured_the_layers_multiply() {
        let (space, reads) = feed_space(99, unavailable);
        let kernel = Kernel::new(Arc::new(Retry::new(Retry::new(space, 3), 3)));

        block_on(kernel.issue(read(FEED), &Capability::root())).expect_err("still down");

        assert_eq!(reads.load(Ordering::SeqCst), 9);
    }

    /// The grant is monotone: attenuation can only remove it, and removing it lowers the
    /// budget to the host's base. A narrower NUMBER is not an attenuation of a wider one.
    #[test]
    fn a_retry_grant_attenuates_as_a_store_budget_does() {
        let limits = RetryLimits::new(0, 3);
        let holder = Capability::scoped([cap_retry(2), "urn:cap:read".to_string()]);
        assert_eq!(limits.for_capability(&holder), 2);
        assert_eq!(
            limits.for_capability(&Capability::scoped([cap_retry(9)])),
            3
        );
        assert_eq!(
            limits.for_capability(&holder.attenuate(["urn:cap:read"])),
            0
        );
        assert_eq!(limits.for_capability(&holder.attenuate([cap_retry(1)])), 0);
        assert_eq!(
            limits.for_capability(&Capability::root().attenuate([cap_retry(1)])),
            1
        );
    }

    /// A failover's advance is a re-issue, so it is the caller's to spend: the same pair of
    /// spaces falls through for a holder of the grant and returns the error for a caller
    /// without one.
    #[test]
    fn failing_over_spends_the_callers_authority() {
        let answer = |cap: &Capability| {
            let (down, _) = feed_space(99, unavailable);
            let (up, _) = feed_space(0, unavailable);
            let pair = Failover::new(vec![Arc::new(down) as Arc<dyn Space>, Arc::new(up)]);
            let kernel = Kernel::new(Arc::new(pair)).with_retry_budget(0, 3);
            block_on(kernel.issue(read(FEED), cap))
        };

        let holder = answer(&Capability::scoped([cap_retry(1)])).expect("one advance is allowed");
        assert_eq!(holder.bytes, b"fresh".to_vec());
        let error = answer(&Capability::scoped(["urn:cap:read"])).expect_err("no advance");
        assert_eq!(error.class(), FailureClass::Transient);
    }

    /// Records what it was asked to wait, and returns at once.
    #[derive(Default)]
    struct Recording(Mutex<Vec<Duration>>);

    #[async_trait::async_trait]
    impl Timer for Recording {
        async fn sleep(&self, delay: Duration) {
            self.0.lock().unwrap().push(delay);
        }
    }

    /// A stated delay is waited out only when it fits what the run has left. One that does
    /// not fit comes straight back, with nothing slept.
    #[test]
    fn a_retry_waits_only_for_a_delay_that_fits_the_deadline() {
        for (not_after, expect_answer, slept) in [(1_250, true, 1), (1_050, false, 0)] {
            let (space, _) = feed_space(1, || Error::RetryAfter {
                delay: Some(Duration::from_millis(100)),
                detail: "limited".into(),
            });
            let timer = Arc::new(Recording::default());
            let kernel = Kernel::new(Arc::new(Retry::new(space, 2).with_timer(timer.clone())))
                .with_clock(Arc::new(FixedClock::at(1_000)));
            let run = IssueContext::new().with_not_after(ikigai_core::Time::from_millis(not_after));

            let answer = block_on(kernel.issue_with_context(read(FEED), &Capability::root(), run));

            assert_eq!(answer.is_ok(), expect_answer, "{answer:?}");
            assert_eq!(timer.0.lock().unwrap().len(), slept);
        }
    }

    /// A deadline only ever shortens on the way down.
    #[test]
    fn a_deadline_shortens_and_never_extends() {
        let run = IssueContext::new()
            .with_not_after(ikigai_core::Time::from_millis(5_000))
            .with_not_after(ikigai_core::Time::from_millis(9_000));
        assert_eq!(run.not_after(), Some(ikigai_core::Time::from_millis(5_000)));
    }

    /// The whole road: a door's run, a mount, a socket, and an endpoint on the far side that
    /// reads what its run has left, on its own clock.
    #[cfg(unix)]
    #[test]
    fn a_run_crosses_a_mount_narrowed_by_the_peers_policy() {
        let path = socket_path("run");
        let _ = std::fs::remove_file(&path);
        let server = path.clone();
        std::thread::spawn(move || ikigai_ipc::serve(run_report_kernel(), &server));

        let peer: Arc<dyn Resolver> = Arc::new(dial_when_up(&path));
        let mount = MountedRemote::overriding(peer, RUN_REPORT, "test://peer");
        let here = Kernel::new(Arc::new(mount))
            .with_retry_budget(0, 9)
            .with_clock(Arc::new(SystemClock));
        let not_after = SystemClock.now().plus_millis(30_000);

        // The door asks for 30 s and 8 retries, and the peer's ceiling is 3.
        let run = IssueContext::new()
            .with_not_after(not_after)
            .with_retry_limit(8);
        let answer = block_on(here.issue_with_context(read(RUN_REPORT), &Capability::root(), run))
            .expect("the peer answers");
        let (retries, left) = parse_run_report(&answer.bytes);
        assert_eq!(retries, Some(3), "the peer's own ceiling still holds");
        let left = left.expect("the deadline crossed");
        assert!((20_000..=30_000).contains(&left), "{left} ms left");

        // A narrower run stays narrow on the far side.
        let run = IssueContext::new().with_retry_limit(1);
        let answer = block_on(here.issue_with_context(read(RUN_REPORT), &Capability::root(), run))
            .expect("the peer answers");
        assert_eq!(parse_run_report(&answer.bytes), (Some(1), None));
        let _ = std::fs::remove_file(&path);
    }

    /// The classes cross the socket typed, which is what lets a prefer-mount decide over a
    /// real peer: a stated delay falls through, a refusal does not.
    #[cfg(unix)]
    #[test]
    fn a_prefer_mount_decides_by_a_class_that_crossed_a_socket() {
        type Case = (&'static str, fn() -> Error, Option<FailureClass>);
        let cases: [Case; 2] = [
            (
                "limited",
                || Error::RetryAfter {
                    delay: Some(Duration::from_secs(30)),
                    detail: "limited".into(),
                },
                None,
            ),
            (
                "refused",
                || Error::Refused("the origin answered 401".into()),
                Some(FailureClass::Refused),
            ),
        ];
        for (label, error, expected) in cases {
            let path = socket_path(label);
            let _ = std::fs::remove_file(&path);
            let server = path.clone();
            std::thread::spawn(move || {
                let (space, _) = feed_space(99, error);
                ikigai_ipc::serve(Kernel::new(Arc::new(space)), &server)
            });
            let peer: Arc<dyn Resolver> = Arc::new(dial_when_up(&path));
            let (local, _) = feed_space(0, unavailable);

            let mount = prefer(FEED, "test://peer", peer, Arc::new(local));
            let answer = block_on(Kernel::new(mount).issue(read(FEED), &Capability::root()));

            match expected {
                None => assert_eq!(answer.expect("falls through").bytes, b"fresh".to_vec()),
                Some(class) => assert_eq!(answer.expect_err("propagates").class(), class),
            }
            let _ = std::fs::remove_file(&path);
        }
    }

    /// `RetryAfter` keeps its delay across the socket.
    #[cfg(unix)]
    #[test]
    fn a_stated_delay_survives_the_wire() {
        let path = socket_path("delay");
        let _ = std::fs::remove_file(&path);
        let server = path.clone();
        std::thread::spawn(move || {
            let (space, _) = feed_space(99, || Error::RetryAfter {
                delay: Some(Duration::from_millis(1_500)),
                detail: "limited".into(),
            });
            ikigai_ipc::serve(Kernel::new(Arc::new(space)), &server)
        });

        let error = dial_when_up(&path)
            .issue(read(FEED))
            .expect_err("the peer is limited");

        assert_eq!(error.class(), FailureClass::RetryAfter);
        assert_eq!(error.retry_after_millis(), Some(1_500));
        let _ = std::fs::remove_file(&path);
    }
}
