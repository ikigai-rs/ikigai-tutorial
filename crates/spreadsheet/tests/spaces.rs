//! Part VI, end to end through a real kernel: the sheet in a person's scenario, the sheet as of
//! an instant, and the two stacked, in both orders.
//!
//! ```text
//! cargo test -p spreadsheet --test spaces
//! ```
//!
//! Every corridor here is built the way the page builds it: a scenario with
//! `spreadsheet::scenario`, an instant with `Scope::with_named_at` over the sheet's and the time
//! chapter's doors (what the engine's `as-of=` does with the doors a host hands it), and the two
//! combined with `Scope::stack`.

use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Clock, Error, Expiry, Fallback, FixedClock, Iri, Kernel, Request, Scope,
    Space, Verb,
};
use ikigai_vocab::TurtleRenderer;
use spreadsheet::{FeedStore, InputStore, OverrideStore};
use time_resource::{millis_at, ManualClock};

/// The page's kernel, in miniature: the sheet over a store the test keeps, the time chapter's
/// names, and the function library, on a clock the test holds.
struct Host {
    kernel: Kernel,
    inputs: Arc<InputStore>,
    clock: Arc<ManualClock>,
}

impl Host {
    fn at(instant: &str) -> Host {
        let inputs = Arc::new(InputStore::default());
        let space = Fallback::new(vec![
            Arc::new(spreadsheet::space_with(
                Arc::clone(&inputs),
                Arc::new(FeedStore::default()),
            )) as Arc<dyn Space>,
            Arc::new(time_resource::space()),
            Arc::new(ikigai_fn::space()),
        ]);
        let clock = Arc::new(ManualClock::at(millis_at(instant, 0)));
        let kernel = Kernel::with_meta_renderer(Arc::new(space), Arc::new(TurtleRenderer))
            .with_clock(Arc::clone(&clock) as Arc<dyn Clock>);
        Host {
            kernel,
            inputs,
            clock,
        }
    }

    // ANCHOR: as_of_scope
    /// The sheet as of `instant`: a temporal corridor named for it, binding the sheet's doors
    /// and the time chapter's, with the clock derived from it. What `as-of=` builds.
    fn as_of(&self, instant: &str) -> Scope {
        let doors = Fallback::new(vec![
            Arc::new(spreadsheet::as_of_doors(Arc::clone(&self.inputs))) as Arc<dyn Space>,
            Arc::new(time_resource::as_of_doors()),
        ]);
        Scope::empty().with_named_at(
            Iri::parse(format!("urn:ctx:time:{instant}")).unwrap(),
            Arc::new(doors),
            Arc::new(FixedClock::at(millis_at(instant, 0))),
        )
    }
    // ANCHOR_END: as_of_scope

    fn issue(&self, request: Request, cap: &Capability, scope: &Scope) -> Result<String, Error> {
        block_on(self.kernel.issue_in(request, cap, scope.clone()))
            .map(|r| String::from_utf8_lossy(&r.bytes).into_owned())
    }

    fn value_as(&self, cap: &Capability, scope: &Scope, cell: &str) -> Result<String, Error> {
        self.issue(request(Verb::Source, &cell_name(cell)), cap, scope)
    }

    fn value(&self, scope: &Scope, cell: &str) -> String {
        self.value_as(&Capability::root(), scope, cell)
            .unwrap_or_else(|e| panic!("{cell} in `{scope}`: {e}"))
    }

    fn type_into(&self, scope: &Scope, cell: &str, typed: &str) -> Result<String, Error> {
        let sink = request(Verb::Sink, &format!("urn:iki:tutorial:sheet:input:{cell}"))
            .with_arg("content", ArgRef::Inline(typed.as_bytes().to_vec()));
        self.issue(sink, &Capability::root(), scope)
    }

    fn cached(&self, scope: &Scope, cell: &str) -> bool {
        self.kernel.is_cached_in(
            &request(Verb::Source, &cell_name(cell)),
            &Capability::root(),
            scope,
        )
    }
}

fn request(verb: Verb, name: &str) -> Request {
    Request::new(verb, Iri::parse(name).expect("a name"))
}

fn cell_name(cell: &str) -> String {
    format!("urn:iki:tutorial:sheet:cell:{cell}")
}

/// A price, a quantity, their product, a cost and a profit, typed on Tuesday; the price rises
/// a week later.
fn the_week(host: &Host) -> Scope {
    let root = Scope::empty();
    for (cell, typed) in [
        ("A1", "100"),
        ("A2", "10"),
        ("A3", "=A1*A2"),
        ("B1", "600"),
        ("B2", "=A3-B1"),
        ("C1", "=NOW()"),
    ] {
        host.type_into(&root, cell, typed).unwrap();
    }
    host.clock.set(millis_at("2026-09-29T14:05Z", 0));
    host.type_into(&root, "A1", "110").unwrap();
    root
}

// ANCHOR: as_of_test
#[test]
fn the_sheet_as_of_an_instant_is_the_sheet_as_it_was_and_it_caches() {
    let host = Host::at("2026-09-22T12:00Z");
    let root = the_week(&host);
    assert_eq!(host.value(&root, "B2"), "500");

    let tuesday = host.as_of("2026-09-22T18:00Z");
    assert_eq!(host.value(&tuesday, "B2"), "400", "A1 was 100 then");
    assert!(host.cached(&tuesday, "B2"), "a then does not change");
    // Before anything was typed, the sheet was empty.
    assert_eq!(host.value(&host.as_of("2026-09-22T11:00Z"), "B2"), "");

    // Inside the corridor there is one now: NOW() reads the pinned instant, and caches.
    assert_eq!(host.value(&tuesday, "C1"), "2026-09-22T18:00Z");
    let read = block_on(host.kernel.issue_in(
        request(Verb::Source, &cell_name("C1")),
        &Capability::root(),
        tuesday.clone(),
    ))
    .unwrap();
    assert_eq!(read.expiry, Expiry::Never);
    assert_eq!(host.value(&root, "C1"), "2026-09-29T14:05Z");

    // The past is read-only.
    assert!(matches!(
        host.type_into(&tuesday, "A1", "90"),
        Err(Error::Conflict(_))
    ));
}
// ANCHOR_END: as_of_test

// ANCHOR: scenario_test
#[test]
fn a_scenario_overrides_its_cells_and_the_shared_sheet_shows_through_the_rest() {
    let host = Host::at("2026-09-22T12:00Z");
    let root = the_week(&host);
    let alice = spreadsheet::scenario("alice", Arc::new(OverrideStore::default())).unwrap();

    assert_eq!(
        host.value(&alice, "B2"),
        "500",
        "nothing overridden: the shared sheet"
    );
    assert_eq!(host.type_into(&alice, "A2", "12").unwrap(), "12");
    assert_eq!(host.value(&alice, "B2"), "720", "110 × 12 − 600");
    assert_eq!(
        host.value(&root, "B2"),
        "500",
        "the shared sheet is untouched"
    );

    // Two spaces, two cache entries for one name.
    assert!(host.cached(&root, "B2") && host.cached(&alice, "B2"));

    // A shared edit recomputes both.
    host.type_into(&root, "B1", "650").unwrap();
    assert!(!host.cached(&root, "B2") && !host.cached(&alice, "B2"));
    assert_eq!(host.value(&alice, "B2"), "670");
    assert_eq!(host.value(&root, "B2"), "450");

    // Dropping the override lets the shared cell show through again.
    let drop = request(Verb::Delete, "urn:iki:tutorial:sheet:input:A2");
    host.issue(drop, &Capability::root(), &alice).unwrap();
    assert_eq!(host.value(&alice, "B2"), "450");
}
// ANCHOR_END: scenario_test

/// ⚠ A scenario edit cuts the SHARED sheet's values too. A golden thread is a name, and an
/// override's write cuts the thread named `input:A2`, which the shared B2 hangs from as well
/// (ledger item 581). Sound, since it only over-invalidates: the shared value is computed again
/// and comes out the same. When this test fails, threads have learned which corridor answered,
/// and part VI's claim "a scenario edit recomputes only its own" has become true.
#[test]
fn a_scenario_edit_also_cuts_the_shared_values_because_a_thread_is_a_name() {
    let host = Host::at("2026-09-22T12:00Z");
    let root = the_week(&host);
    let alice = spreadsheet::scenario("alice", Arc::new(OverrideStore::default())).unwrap();
    host.value(&root, "B2");
    host.type_into(&alice, "A2", "12").unwrap();
    assert!(!host.cached(&root, "B2"), "cut by alice's write");
    assert_eq!(
        host.value(&root, "B2"),
        "500",
        "and computed again to the same value"
    );
}

// ANCHOR: stack_test
#[test]
fn stacked_corridors_answer_in_order_innermost_first() {
    let host = Host::at("2026-09-22T12:00Z");
    let root = the_week(&host);
    let alice = spreadsheet::scenario("alice", Arc::new(OverrideStore::default())).unwrap();
    host.type_into(&alice, "A2", "15").unwrap();
    let tuesday = host.as_of("2026-09-22T18:00Z");

    // The scenario innermost: alice's override over Tuesday's sheet, 100 × 15 − 600.
    let what_if_then = tuesday.clone().stack(&alice);
    assert_eq!(host.value(&what_if_then, "B2"), "900");
    // The instant innermost: its doors answer every input, alice's included. Tuesday's sheet.
    let then_in_alice = alice.clone().stack(&tuesday);
    assert_eq!(host.value(&then_in_alice, "B2"), "400");
    // Two orders, two chains, two cache entries.
    assert_ne!(what_if_then.fingerprint(), then_in_alice.fingerprint());
    // And writes go where the innermost corridor says: to alice, or refused by the past.
    assert_eq!(host.type_into(&what_if_then, "A1", "95").unwrap(), "95");
    assert_eq!(host.value(&what_if_then, "B2"), "825");
    assert!(matches!(
        host.type_into(&then_in_alice, "A1", "95"),
        Err(Error::Conflict(_))
    ));
    assert_eq!(
        host.value(&root, "B2"),
        "500",
        "and the shared sheet never moved"
    );
}
// ANCHOR_END: stack_test

// ANCHOR: cap_test
#[test]
fn a_scenario_is_readable_only_with_its_owners_capability() {
    let host = Host::at("2026-09-22T12:00Z");
    the_week(&host);
    let alice = spreadsheet::scenario("alice", Arc::new(OverrideStore::default())).unwrap();
    host.type_into(&alice, "A2", "12").unwrap();

    let as_alice = Capability::scoped([spreadsheet::scenario_cap("alice")]);
    let as_bob = Capability::scoped([spreadsheet::scenario_cap("bob")]);
    assert_eq!(host.value_as(&as_alice, &alice, "B2").unwrap(), "720");
    assert!(
        matches!(host.value_as(&as_bob, &alice, "B2"), Err(Error::Denied(_))),
        "bob holds alice's corridor and not her capability"
    );
    // The cells alice did not override are the shared sheet's, and need nothing.
    assert_eq!(host.value_as(&as_bob, &alice, "B1").unwrap(), "600");
}
// ANCHOR_END: cap_test

#[test]
fn a_scenario_has_one_owner_spelled_one_way() {
    for refused in ["Alice", "", "a b", "1x"] {
        assert!(
            spreadsheet::scenario(refused, Arc::default()).is_err(),
            "{refused:?}"
        );
    }
}
