//! The code taught by "Who a script runs as".
//!
//! A run of a script holds the RUNNER's capability, narrowed to
//!
//! ```text
//! { declared } ∩ { what the publisher held at publish } ∩ { the host's ceiling for this script }
//!   + every exclusion the publisher or the ceiling carries
//! ```
//!
//! For a SPARQL script "declared" is not written by its author: it is DERIVED from the graphs
//! the text names. The publisher's half is taken when they publish, and a publish that would
//! give the script more than its publisher holds is refused. The ceiling is the host's, read
//! here from text in the format of `<config home>/script-authority/{name}`, one file per
//! script; a script with no file touches nothing.
//!
//! The people are capabilities a door would mint: [`alice`] publishes, [`bob`] runs and holds
//! more than she does, [`carol`] holds less.

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use ikigai_core::{Capability, Kernel};
use ikigai_script::authority::{Ceiling, CeilingPolicy};
use script_host::{READ_ITEMS, READ_PAYROLL, WRITE_ITEMS};

// ANCHOR: ceilings
/// The host's ceiling files, by script name, as they would sit under
/// `<config home>/script-authority/`. One `urn:cap:` scope per line, `#` comments, a
/// trailing `*` for a family, a lone `*` for no ceiling.
pub const CEILINGS: &[(&str, &str)] = &[
    (
        "open-items",
        "# the items, and nothing else\nurn:cap:store:read:graph:urn:iki:tutorial:items\n",
    ),
    (
        "graphs",
        "# any graph the publisher and the runner both hold\nurn:cap:store:read:graph:*\n",
    ),
    ("every-graph", "urn:cap:store:read:graph:*\n"),
    ("team-open", "*\n"),
    (
        "team-a-open",
        "urn:cap:store:read:graph:urn:iki:tutorial:items\n",
    ),
];

/// The host's ceiling files, held where the host can edit them (a stand-in for the directory).
#[derive(Clone)]
pub struct Ceilings(Arc<Mutex<BTreeMap<String, String>>>);

impl Ceilings {
    /// The files in [`CEILINGS`].
    pub fn new() -> Ceilings {
        let files = CEILINGS
            .iter()
            .map(|(name, text)| (name.to_string(), text.to_string()))
            .collect();
        Ceilings(Arc::new(Mutex::new(files)))
    }

    /// Replace one script's file. ⚠ The library cannot see this happen: the host must also cut
    /// `urn:script:{name}:authority`, or a result cached under the old ceiling is served on.
    pub fn set(&self, name: &str, text: &str) {
        self.0
            .lock()
            .expect("the ceiling files")
            .insert(name.to_string(), text.to_string());
    }

    /// The policy the host hands the script space: the script's file, parsed, and **nothing**
    /// for a script the host has no file for, so a script nobody configured can run nothing at
    /// all. Asked on every run, so a changed file takes effect at the next one that runs.
    pub fn policy(&self) -> CeilingPolicy {
        let files = Arc::clone(&self.0);
        Arc::new(move |name: &str| {
            files
                .lock()
                .expect("the ceiling files")
                .get(name)
                .and_then(|text| Ceiling::parse(text).ok())
                .unwrap_or_else(Ceiling::nothing)
        })
    }
}

impl Default for Ceilings {
    fn default() -> Self {
        Ceilings::new()
    }
}
// ANCHOR_END: ceilings

/// The previous chapter's kernel, under the ceiling files in `ceilings`.
pub fn kernel(ceilings: &Ceilings) -> Arc<Kernel> {
    script_host::kernel_with(ceilings.policy())
}

// ANCHOR: people
/// Publishes. Holds the items graph (read and write), and writes the `open-items` and
/// `graphs` scripts and every script in the `team` namespace except `team-payroll`.
pub fn alice() -> Capability {
    person(
        "alice",
        &[
            "urn:cap:script:write:open-items",
            "urn:cap:script:write:graphs",
            "urn:cap:script:write:team-*",
            "urn:cap:script:write:-team-payroll",
            "urn:cap:script:read:open-items",
            READ_ITEMS,
            WRITE_ITEMS,
        ],
    )
}

/// Runs. Holds MORE data than Alice (the payroll graph too), and may run every script in this
/// chapter and every one in the `team` namespace.
pub fn bob() -> Capability {
    person(
        "bob",
        &[
            "urn:cap:script:run:open-items",
            "urn:cap:script:run:graphs",
            "urn:cap:script:run:every-graph",
            "urn:cap:script:run:orphan",
            "urn:cap:script:run:team-*",
            READ_ITEMS,
            READ_PAYROLL,
        ],
    )
}

/// Holds less: she may write `salaries` and run `open-items`, and read no graph at all.
pub fn carol() -> Capability {
    person(
        "carol",
        &[
            "urn:cap:script:write:salaries",
            "urn:cap:script:run:open-items",
        ],
    )
}
// ANCHOR_END: people

fn person(name: &str, scopes: &[&str]) -> Capability {
    Capability::scoped(scopes.iter().copied())
        .with_principal(&format!("urn:iki:tutorial:person:{name}"))
        .expect("a principal IRI")
}

// ANCHOR: graphs
/// How many triples each graph a run can see holds. `GRAPH ?g` fixes no graph in the text, so
/// the script declares the FAMILY `urn:cap:store:read:graph:*`, and each run reads the graphs
/// its own narrowed capability holds: never more.
pub const GRAPHS: &str = r#"# How many triples each graph this run may read holds.
SELECT ?g (COUNT(*) AS ?triples) WHERE { GRAPH ?g { ?s ?p ?o } }
GROUP BY ?g ORDER BY ?g"#;
// ANCHOR_END: graphs

/// The salaries, by person: a query over the payroll graph.
pub const SALARIES: &str = r#"# Every salary.
SELECT ?who ?salary WHERE { GRAPH <urn:iki:tutorial:payroll> { ?who <urn:iki:tutorial:salary> ?salary } }"#;
