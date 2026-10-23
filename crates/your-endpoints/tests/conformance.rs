//! Hold this crate's endpoint to the recipe taught in "Contribute a module".

use std::sync::Arc;

use ikigai_conformance::{Check, Fixture, Suite};
use ikigai_core::{FailureClass, Kernel, Verb};

#[test]
fn word_count_conforms() {
    let kernel = Kernel::new(Arc::new(your_endpoints::space()));
    let report = Suite::new()
        .own("word-count")
        .check_failures()
        .pure("word-count")
        .fails_under(
            Fixture::new("word-count", Verb::Source).arg_bytes("in", vec![0xff]),
            FailureClass::Permanent,
        )
        .run_blocking(&kernel);
    report.assert_clean();
    assert_eq!(&*report.walked, ["word-count"]);
    // These belong to the spaces we chain onto. A new endpoint must be accounted for,
    // rather than silently joining the inherited list and escaping the suite.
    assert_eq!(
        report.declared.inherited,
        [
            "toUpper",
            "reverseList",
            "compose",
            "conditional",
            // ikigai-fn 0.6.0's collection fill, inherited through Part I's space.
            "each",
            "wrap",
            "split",
            "greet",
            "echo",
            "camel-case",
            "title",
            "camel-title",
        ]
    );
    assert!(report.checks.contains(Check::Recompute));
    assert!(report.checks.contains(Check::Failures));
    assert_eq!(report.recomputed().len(), 1);
    assert_eq!(report.recomputed()[0].recomputations, 24);
    assert_eq!(report.failures_seen().len(), 1);
    assert_eq!(
        report.failures_seen()[0].declared.as_deref(),
        Some(&[FailureClass::Permanent][..])
    );
    assert_eq!(report.fails_under_probes().len(), 1);
    assert_eq!(
        report.fails_under_probes()[0].got,
        Some(FailureClass::Permanent)
    );
}
