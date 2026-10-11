//! A model that is not one, for The gonk Book's "Scripts and sentences in gonk".
//!
//! gonk drafts through whatever its `gonk.mount` line mounts at `urn:llm:`, and a transcript
//! must never reach a real model or the network. So the book's check starts this peer on a Unix
//! socket and points a scratch gonk at it, exactly as an operator points gonk at a real model
//! host: the mount's own checks (its net host, its contracts) are in the path, and only the
//! answer is canned. It binds what `ikigai-llm` binds that drafting reaches:
//!
//! - `urn:llm:ask`, declaring `urn:cap:net:*` as the real door does, answering each prompt by
//!   the ASK it carries (the text after the prompt's `The ask:` line), in `ikigai-llm`'s JSON
//!   envelope, with the draft fenced the way a model fences it;
//! - `urn:llm:config`, capability-free, naming the stub as the default backend.
//!
//! Every ask is counted and logged on standard output, so a page can show that a refused draft
//! never reached the model. The shape is gonk's own test stub's (`tests/nl_1138.rs`) and the
//! ikigai Book's (`crates/sentence-draft/src/stub.rs`).

use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use async_trait::async_trait;
use ikigai_core::{
    ActionSpec, ArgRef, ArgSpec, Description, Endpoint, EndpointSpace, Exact, Invocation, Kernel,
    ReprType, Representation, Verb,
};

/// The model name every answer carries.
pub const MODEL: &str = "stub";

/// The live demo's ask (gonk, 2026-10-11), word for word.
pub const ASK_P2: &str =
    "the number and title of every open item with priority 2 or higher (0 is highest), ordered by number";

/// What a 70B model drafted for [`ASK_P2`] in that demo, on gonk's default ledger: the status
/// written as the string `"open"` where the ledger stores the IRI `ledger:open` (so nothing
/// matches), `FROM` paired with `GRAPH` (harmless here: a script's graph set is both its default
/// and its named graphs), and the priority bound named `limit`.
pub const P2_DRAFT: &str = "# @param limit xsd:integer optional default 2
SELECT ?number ?title
FROM <urn:iki:ledger:graph:default>
WHERE {
  GRAPH <urn:iki:ledger:graph:default> {
    ?item a <https://ikigai-rs.dev/ns/ledger#Item> ;
      <https://ikigai-rs.dev/ns/ledger#status> \"open\" ;
      <https://ikigai-rs.dev/ns/ledger#priority> ?priority ;
      <https://ikigai-rs.dev/ns/ledger#number> ?number ;
      <http://purl.org/dc/terms/title> ?title .
    FILTER (?priority <= ?limit)
  }
}
ORDER BY ?number";

/// An ask whose first draft the check refuses and whose repair it passes.
pub const ASK_TITLES: &str = "the titles of the open items";

/// The first draft for [`ASK_TITLES`]: `ledger:state` is no predicate the ledger graph uses.
pub const TITLES_FIRST: &str = "PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?title WHERE {
  GRAPH <urn:iki:ledger:graph:default> { ?item ledger:state ledger:open ; dcterms:title ?title }
} ORDER BY ?title";

/// Its repair, once the prompt carries what the check refused.
pub const TITLES_REPAIRED: &str = "PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?title WHERE {
  GRAPH <urn:iki:ledger:graph:default> { ?item ledger:status ledger:open ; dcterms:title ?title }
} ORDER BY ?title";

/// The first page's ask ("Your first saved query"), which drafts right the first time.
pub const ASK_OPEN_COUNT: &str = "How many open items are there?";

/// The draft for [`ASK_OPEN_COUNT`]: a count of the open items, the query the page's reader
/// wrote by hand a step earlier.
pub const OPEN_COUNT: &str = "PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
SELECT (COUNT(?item) AS ?count) WHERE {
  GRAPH <urn:iki:ledger:graph:default> { ?item ledger:status ledger:open }
}";

/// Any other ask: every item's number and title.
pub const ANYTHING: &str = "PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title WHERE {
  GRAPH <urn:iki:ledger:graph:default> { ?item ledger:number ?number ; dcterms:title ?title }
} ORDER BY ?number";

/// The ask a drafting prompt carries: between its `The ask:` line and the grounding after it.
pub fn ask_of(prompt: &str) -> &str {
    prompt
        .split_once("The ask:\n")
        .map(|(_, rest)| rest)
        .and_then(|rest| rest.split("\n\n").next())
        .unwrap_or("")
        .trim()
}

/// The draft for a prompt: by its ask, and whether it is a repair (`Your previous draft was:`).
pub fn draft_for(prompt: &str) -> &'static str {
    let ask = ask_of(prompt);
    let repairing = prompt.contains("Your previous draft was:");
    if ask == ASK_P2 {
        P2_DRAFT
    } else if ask == ASK_OPEN_COUNT {
        OPEN_COUNT
    } else if ask == ASK_TITLES {
        if repairing {
            TITLES_REPAIRED
        } else {
            TITLES_FIRST
        }
    } else {
        ANYTHING
    }
}

struct Ask {
    calls: Arc<AtomicUsize>,
}

#[async_trait]
impl Endpoint for Ask {
    async fn invoke(&self, inv: &Invocation<'_>) -> ikigai_core::Result<Representation> {
        let n = self.calls.fetch_add(1, Ordering::SeqCst) + 1;
        let prompt = match inv.request.args.get("prompt") {
            Some(ArgRef::Inline(bytes)) => String::from_utf8_lossy(bytes).into_owned(),
            _ => String::new(),
        };
        let draft = draft_for(&prompt);
        let mut out = std::io::stdout().lock();
        let _ = writeln!(
            out,
            "model-stub: ask {n}{}: {}",
            if prompt.contains("Your previous draft was:") {
                " (a repair)"
            } else {
                ""
            },
            ask_of(&prompt)
        );
        let _ = out.flush();
        let envelope = serde_json::json!({
            "text": format!("Here is the query.\n```sparql\n{draft}\n```\n"),
            "model": MODEL,
            "finish_reason": "stop",
            "usage": null,
        });
        Ok(Representation::new(
            ReprType::new("application/json"),
            serde_json::to_vec(&envelope).expect("JSON"),
        ))
    }

    fn name(&self) -> &str {
        "model-stub-ask"
    }

    fn describe(&self) -> Description {
        let string = "http://www.w3.org/2001/XMLSchema#string";
        Description::new("model-stub-ask").verb(Verb::Meta).action(
            ActionSpec::new(Verb::Source)
                .summary("A canned SPARQL draft, by the ask the prompt carries.")
                .input(ArgSpec::new("prompt").class(string))
                .input(
                    ArgSpec::new("temperature")
                        .class("http://www.w3.org/2001/XMLSchema#decimal")
                        .optional(),
                )
                .input(
                    ArgSpec::new("max_tokens")
                        .class("http://www.w3.org/2001/XMLSchema#positiveInteger")
                        .optional(),
                )
                .input(ArgSpec::new("as").class(string).optional())
                .output("application/json")
                .requires("urn:cap:net:*"),
        )
    }
}

struct Config;

#[async_trait]
impl Endpoint for Config {
    async fn invoke(&self, _inv: &Invocation<'_>) -> ikigai_core::Result<Representation> {
        Ok(Representation::new(
            ReprType::new("application/json"),
            format!(r#"{{"default":"{MODEL}","providers":{{}}}}"#).into_bytes(),
        )
        .cacheable())
    }

    fn name(&self) -> &str {
        "model-stub-config"
    }

    fn describe(&self) -> Description {
        Description::new("model-stub-config")
            .verb(Verb::Source)
            .summary("The stub as the default backend.")
    }
}

/// The peer's kernel: `urn:llm:ask` and `urn:llm:config`, with a Meta renderer so a mount reads
/// its contracts. `calls` counts the asks.
pub fn kernel(calls: Arc<AtomicUsize>) -> Kernel {
    let space = EndpointSpace::new()
        .bind(Exact::new("urn:llm:ask"), Ask { calls })
        .bind(Exact::new("urn:llm:config"), Config);
    Kernel::with_meta_renderer(Arc::new(space), Arc::new(ikigai_vocab::TurtleRenderer))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prompt(ask: &str, repair: bool) -> String {
        let head = if repair {
            "Your previous draft was:\n…\n\n"
        } else {
            ""
        };
        format!("{head}Write one SPARQL query.\n\nThe ask:\n{ask}\n\nWhat this person may read: …")
    }

    #[test]
    fn each_ask_gets_its_draft_and_a_repair_its_repair() {
        assert_eq!(draft_for(&prompt(ASK_P2, false)), P2_DRAFT);
        assert_eq!(draft_for(&prompt(ASK_TITLES, false)), TITLES_FIRST);
        assert_eq!(draft_for(&prompt(ASK_TITLES, true)), TITLES_REPAIRED);
        assert_eq!(draft_for(&prompt(ASK_OPEN_COUNT, false)), OPEN_COUNT);
        assert_eq!(draft_for(&prompt("something else", false)), ANYTHING);
        assert_eq!(draft_for("no ask at all"), ANYTHING);
    }
}
