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
//!
//! It also answers what a browse root sends a model ("Ways of working", ledger #1435), so a scratch
//! gonk can explain, review and judge through the same mount: `urn:llm:coder:ask` (gonk's default
//! provider for files, reviews and the judge) is bound to the same endpoint, `urn:llm:coder:model`
//! names the stub, and an ask is told apart by its SYSTEM prompt, as `crates/browse-host`'s
//! in-process stub tells them apart: a review gets [`REVIEW`], a judge's check gets [`JUDGE`], an
//! explanation gets [`EXPLANATION`], and anything else (drafting sends no system prompt this stub
//! recognizes) gets a SPARQL draft as before.

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

/// The stub's explanation, whatever it is asked to explain (browse-host's, word for word).
pub const EXPLANATION: &str = "Defines `initials`, which joins the first letter of each \
space-separated word of a name into one string.";

/// The stub's review of `src/lib.rs` (browse-host's `demo` file), in the grammar browse parses:
/// three lines per finding. Each QUOTE is a line of that file, so both findings anchor.
pub const REVIEW: &str = "\
QUOTE: name.split(' ').map(|word| &word[..1]).collect()
SEVERITY: major
NOTE: Slicing the first byte panics on an empty word, so a name with two spaces in a row, or an empty name, crashes the caller.
QUOTE: pub fn initials(name: &str) -> String {
SEVERITY: minor
NOTE: Returning a String allocates on every call; return a &str instead.
";

/// The stub judge's answer to any claim: browse's four questions and a verdict, in the format
/// its judge prompt asks for. The answers are the ones that make browse's rule say `confirmed`.
pub const JUDGE: &str = "\
CODE: yes - the code shown slices the first byte of every word, empty or not.
DISCLOSED: no - no comment at the site says an empty word is excluded.
OCCURS: yes - a name with two spaces in a row splits into an empty word.
TEST: no - this is library code.
VERDICT: confirmed
";

/// What a model is asked to do, told apart by the system prompt browse sends with each ask.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// A review pass over a file (browse's review system prompt).
    Review,
    /// The judge checking one finding (browse's judge system prompt).
    Judge,
    /// An explanation of a file or a directory (browse's explain system prompt).
    Explain,
    /// Anything else: a SPARQL draft, by the ask the prompt carries.
    Draft,
}

/// The kind of an ask, by its system prompt. The phrases are browse 0.18's own prompts'.
pub fn kind_of(system: &str) -> Kind {
    if system.contains("reviewing a colleague's file") {
        Kind::Review
    } else if system.contains("You check one claim a code reviewer made") {
        Kind::Judge
    } else if system.contains("explanation layer of a repository browser") {
        Kind::Explain
    } else {
        Kind::Draft
    }
}

/// The `Path:` a review prompt names, for the log line.
fn path_of(prompt: &str) -> &str {
    prompt
        .lines()
        .find_map(|l| l.strip_prefix("Path: "))
        .and_then(|p| p.split_whitespace().next())
        .unwrap_or("")
}

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
        let arg = |name: &str| match inv.request.args.get(name) {
            Some(ArgRef::Inline(bytes)) => String::from_utf8_lossy(bytes).into_owned(),
            _ => String::new(),
        };
        let prompt = arg("prompt");
        let system = arg("system");
        let mut out = std::io::stdout().lock();
        let kind = kind_of(&system);
        let text = match kind {
            Kind::Review => {
                let _ = writeln!(out, "model-stub: ask {n}: review {}", path_of(&prompt));
                REVIEW.to_string()
            }
            Kind::Judge => {
                let _ = writeln!(out, "model-stub: ask {n}: judge one finding");
                JUDGE.to_string()
            }
            Kind::Explain => {
                let _ = writeln!(out, "model-stub: ask {n}: explain");
                EXPLANATION.to_string()
            }
            Kind::Draft => {
                let draft = draft_for(&prompt);
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
                format!("Here is the query.\n```sparql\n{draft}\n```\n")
            }
        };
        let _ = out.flush();
        // What `ikigai-llm` does: the JSON envelope for `as=application/json` (a review asks for
        // it, to read `finish_reason`), plain text otherwise (the judge reads the raw answer).
        // A draft keeps the envelope whatever it asks, as this stub always answered drafts.
        if kind != Kind::Draft && arg("as") != "application/json" {
            return Ok(Representation::new(
                ReprType::new("text/plain").with_param("charset", "utf-8"),
                text.into_bytes(),
            ));
        }
        let envelope = serde_json::json!({
            "text": text,
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
                .summary("A canned answer: a review, a judge check or an explanation by the system prompt, otherwise a SPARQL draft by the ask.")
                .input(ArgSpec::new("prompt").class(string))
                .input(ArgSpec::new("system").class(string).optional())
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

/// `urn:llm:coder:model`: the model's identity, which browse folds into every version tag.
struct ModelId;

#[async_trait]
impl Endpoint for ModelId {
    async fn invoke(&self, _inv: &Invocation<'_>) -> ikigai_core::Result<Representation> {
        Ok(Representation::new(
            ReprType::new("text/plain").with_param("charset", "utf-8"),
            MODEL.as_bytes().to_vec(),
        ))
    }

    fn name(&self) -> &str {
        "model-stub-model"
    }

    fn describe(&self) -> Description {
        Description::new("model-stub-model")
            .verb(Verb::Source)
            .summary("The stub's model name.")
            .output("text/plain")
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
        .bind(
            Exact::new("urn:llm:ask"),
            Ask {
                calls: calls.clone(),
            },
        )
        .bind(Exact::new("urn:llm:coder:ask"), Ask { calls })
        .bind(Exact::new("urn:llm:coder:model"), ModelId)
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

    #[test]
    fn a_system_prompt_names_the_kind_of_ask() {
        assert_eq!(
            kind_of("You are an experienced engineer reviewing a colleague's file. …"),
            Kind::Review
        );
        assert_eq!(
            kind_of("You check one claim a code reviewer made about a file. …"),
            Kind::Judge
        );
        assert_eq!(
            kind_of("You are the explanation layer of a repository browser. …"),
            Kind::Explain
        );
        assert_eq!(kind_of(""), Kind::Draft);
        assert_eq!(
            path_of("Repository: demo\nPath: src/lib.rs\n\n```"),
            "src/lib.rs"
        );
    }
}
