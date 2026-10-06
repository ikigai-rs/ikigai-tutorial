//! **A ledger of your own** — the code The gonk Book's first chapter teaches.
//!
//! gonk is a server, but the work ledger it serves is two published modules, and this crate
//! is what it takes to have that ledger WITHOUT the server: the store the ledger is written
//! in, the ledger beside it, and a clock, in a kernel you own. Nothing here is an endpoint.
//!
//! ## The resource map, as built
//!
//! Done the resource-first way (name the resources, write code only for what holds state,
//! compose the rest), and the honest result is that this crate holds no state at all:
//!
//! * **Sets.** The ledgers (`urn:iki:ledger:ledgers`); in each, its items
//!   (`urn:iki:ledger:{ledger}:items`). Both are the ledger module's names.
//! * **Atoms that hold state.** One named graph per ledger, `urn:iki:ledger:graph:{ledger}`,
//!   and its graveyard `…:deleted` — held by `ikigai-store`'s dataset, written only through
//!   its narrow doors (`urn:iki:store:graph-update`). The clock is the other input, and it
//!   is the host's to give.
//! * **Compositions.** Every ledger action — append, comment, link, claim, close, label,
//!   defer, delete, purge — is `ikigai-ledger`'s SPARQL over that graph, issued back
//!   through the kernel.
//! * **Views.** `items`, `item:{id}` and `next` in `text/plain` and `text/turtle`; `next`
//!   is a view whose ranking policy is a seam (`urn:iki:ledger:policy:{name}`).
//!
//! So the code this crate owns is [`space`] (a `Fallback` of two published spaces),
//! [`kernel`] (that space, a Meta renderer and a clock) and [`ask`] (one request, issued).
//! Resource ratio: zero endpoints written, fourteen ledger resources and thirteen store
//! resources composed.
//!
//! Read the chapter: `mdbook serve books/gonk --open`.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Clock, Error, Fallback, Iri, Kernel, Request, Result, Space, Time, Verb,
};
use ikigai_store::DurableStore;

// ANCHOR: space
/// The ledger and the store it is written in, as one space.
///
/// Order matters only for names both would claim, and they claim none in common: the store
/// owns `urn:iki:store:*`, the ledger `urn:iki:ledger:*`. What matters is that both are
/// HERE — the ledger owns no bytes, so a kernel with the ledger and no store refuses every
/// ledger name, naming `urn:iki:store:*` as what to bind.
pub fn space() -> Fallback {
    let store = DurableStore::in_memory().expect("an in-memory store opens");
    Fallback::new(vec![
        Arc::new(ikigai_store::space(store)) as Arc<dyn Space>,
        Arc::new(ikigai_ledger::space()) as Arc<dyn Space>,
    ])
}
// ANCHOR_END: space

// ANCHOR: kernel
/// The host: [`space`], a Meta renderer (so the ledger can describe itself), and a clock.
///
/// ⚠ The clock is not optional. Every item, comment, claim and close is stamped, and the
/// ledger refuses a write it cannot stamp rather than making an unstamped one.
pub fn kernel() -> Kernel {
    kernel_with_clock(Arc::new(StoryClock::default()))
}

/// [`kernel`], with a clock of the caller's choosing — a host's real clock, say.
pub fn kernel_with_clock(clock: Arc<dyn Clock>) -> Kernel {
    Kernel::with_meta_renderer(Arc::new(space()), Arc::new(ikigai_vocab::TurtleRenderer))
        .with_clock(clock)
}
// ANCHOR_END: kernel

// ANCHOR: ask
/// Issue one request and read the answer as text: the whole of "using a resource" from Rust.
///
/// Every argument goes inline, by name; a Sink's body is the argument called `content`, the
/// name the REPL and gonk's HTTP door both put a piped value or a request body under.
pub fn ask(kernel: &Kernel, verb: Verb, iri: &str, args: &[(&str, &str)]) -> Result<String> {
    ask_as(kernel, &Capability::root(), verb, iri, args)
}

/// [`ask`], under a capability other than root — what a caller holding a grant can do.
pub fn ask_as(
    kernel: &Kernel,
    capability: &Capability,
    verb: Verb,
    iri: &str,
    args: &[(&str, &str)],
) -> Result<String> {
    let iri = Iri::parse(iri).map_err(|e| Error::Endpoint(format!("not an IRI: {e}")))?;
    let request = args
        .iter()
        .fold(Request::new(verb, iri), |request, (name, value)| {
            request.with_arg(*name, ArgRef::Inline(value.as_bytes().to_vec()))
        });
    let answer = block_on(kernel.issue(request, capability))?;
    Ok(String::from_utf8_lossy(&answer.bytes).into_owned())
}
// ANCHOR_END: ask

// ANCHOR: grant
/// What a caller needs to read and write ONE ledger, and nothing else.
///
/// Two halves, and the ledger checks both: its own grant for the ledger by name, and the
/// store's grant for that ledger's graph (the ledger asks the store under the CALLER's
/// capability, never its own). This is the list `ikigai-gonk grants <ledger> write` prints.
pub fn write_grant(ledger: &str) -> Capability {
    let graph = format!("urn:iki:ledger:graph:{ledger}");
    Capability::scoped([
        format!("urn:cap:ledger:read:{ledger}"),
        format!("urn:cap:ledger:write:{ledger}"),
        format!("urn:cap:store:read:graph:{graph}"),
        format!("urn:cap:store:write:graph:{graph}"),
    ])
}
// ANCHOR_END: grant

/// The instant the book's stories start: 2026-10-06T09:00:00Z, in milliseconds.
pub const STORY_START: u64 = 1_791_277_200_000;

/// A clock that advances one second each time it is read, from [`STORY_START`].
///
/// Advancing rather than stopped, because the default ranking breaks ties on recency: with
/// a stopped clock every item would be filed "at the same instant". Deterministic rather
/// than the wall clock, so the times this book prints are the times a reader's run prints.
pub struct StoryClock(AtomicU64);

impl Default for StoryClock {
    fn default() -> Self {
        StoryClock(AtomicU64::new(STORY_START))
    }
}

impl Clock for StoryClock {
    fn now(&self) -> Time {
        Time::from_millis(self.0.fetch_add(1_000, Ordering::SeqCst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_filed_item_is_listed_and_ranked() {
        let kernel = kernel();
        let filed = ask(
            &kernel,
            Verb::Sink,
            "urn:iki:ledger:append",
            &[("content", "Write the gonk book")],
        )
        .unwrap();
        assert!(
            filed.starts_with("#1 urn:iki:ledger:default:item:"),
            "{filed}"
        );
        let items = ask(&kernel, Verb::Source, "urn:iki:ledger:items", &[]).unwrap();
        assert!(
            items.contains("#1  open    p-  Write the gonk book"),
            "{items}"
        );
        let next = ask(&kernel, Verb::Source, "urn:iki:ledger:next", &[]).unwrap();
        assert!(next.contains("ready: 1"), "{next}");
    }

    #[test]
    fn the_grant_reaches_its_ledger_and_no_other() {
        let kernel = kernel();
        let grant = write_grant("book");
        ask_as(
            &kernel,
            &grant,
            Verb::Sink,
            "urn:iki:ledger:book:append",
            &[("content", "Mine")],
        )
        .unwrap();
        let refused = ask_as(
            &kernel,
            &grant,
            Verb::Sink,
            "urn:iki:ledger:append",
            &[("content", "Not mine")],
        );
        assert!(matches!(refused, Err(Error::Denied(_))), "{refused:?}");
    }

    #[test]
    fn without_the_store_the_ledger_says_what_to_bind() {
        let kernel = Kernel::new(Arc::new(ikigai_ledger::space()))
            .with_clock(Arc::new(StoryClock::default()));
        let answer = ask(&kernel, Verb::Source, "urn:iki:ledger:items", &[]);
        let message = format!("{answer:?}");
        assert!(message.contains("urn:iki:store"), "{message}");
    }
}
