//! A deterministic model at `urn:llm:ask`, for every gate.
//!
//! The same shape as `ikigai-nl`'s own test stub (its `tests/common`): it speaks
//! `ikigai-llm`'s JSON envelope (`{"text", "model", "finish_reason", "usage"}`), answers each
//! ask with a canned draft per attempt (the nth prompt for an ask gets the nth draft, and the
//! last one repeats), fences the draft the way a model would, and records every prompt it was
//! sent. Like the real door it declares `urn:cap:net:*`, so a caller who could not reach a
//! model cannot reach this one either. Nothing here touches a network.

use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use ikigai_core::{
    ArgRef, ArgSpec, Description, Endpoint, EndpointSpace, Error, Exact, Invocation, ReprType,
    Representation, Verb,
};

/// The model name the stub answers under.
pub const MODEL: &str = "stub";

/// The stub's script, and what it was sent.
#[derive(Default)]
pub struct Stub {
    /// For an ask (found as a substring of the prompt), the draft each attempt answers.
    pub drafts: Vec<(String, Vec<String>)>,
    /// Every prompt, in order.
    pub prompts: Vec<String>,
}

impl Stub {
    /// A stub answering each `(ask, drafts)` with its drafts, in order.
    pub fn new(drafts: &[(&str, &[&str])]) -> Stub {
        Stub {
            drafts: drafts
                .iter()
                .map(|(ask, d)| (ask.to_string(), d.iter().map(|s| s.to_string()).collect()))
                .collect(),
            prompts: Vec::new(),
        }
    }
}

struct Ask {
    stub: Arc<Mutex<Stub>>,
}

#[async_trait]
impl Endpoint for Ask {
    async fn invoke(&self, inv: &Invocation<'_>) -> ikigai_core::Result<Representation> {
        let prompt = match inv.request.args.get("prompt") {
            Some(ArgRef::Inline(bytes)) => String::from_utf8_lossy(bytes).into_owned(),
            _ => return Err(Error::MissingArgument("prompt".into())),
        };
        let mut stub = self.stub.lock().expect("the stub");
        let draft = match stub
            .drafts
            .iter()
            .find(|(ask, _)| prompt.contains(ask.as_str()))
        {
            Some((ask, drafts)) => {
                let seen = stub
                    .prompts
                    .iter()
                    .filter(|p| p.contains(ask.as_str()))
                    .count();
                drafts[seen.min(drafts.len() - 1)].clone()
            }
            None => "SELECT * WHERE { ?s ?p ?o } LIMIT 1".to_string(),
        };
        stub.prompts.push(prompt);
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
        "llm-stub"
    }

    fn describe(&self) -> Description {
        let string = "http://www.w3.org/2001/XMLSchema#string";
        Description::new("llm-stub").verb(Verb::Meta).action(
            ikigai_core::ActionSpec::new(Verb::Source)
                .summary("A canned draft, for the book's gates.")
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

/// `urn:llm:ask`, answered by `stub`.
pub fn space(stub: Arc<Mutex<Stub>>) -> EndpointSpace {
    EndpointSpace::new().bind(Exact::new("urn:llm:ask"), Ask { stub })
}
