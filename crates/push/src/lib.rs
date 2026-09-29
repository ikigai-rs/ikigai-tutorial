//! The code taught by the spreadsheet arc's part VII: **push**, the host's half of it.
//!
//! Part V's page found out about a write it did not make by asking again. This crate is what a
//! host needs to be *told* instead, and it is small, because the kernel already does the hard
//! part: since ikigai-core 0.1.82, [`Kernel::listen`] queues every cut of a golden thread a
//! listener asked about, with the cached answers the cut invalidated. What is left for the
//! host is three things, designed as names before code:
//!
//! * `urn:iki:tutorial:push:listener:{name}` — a **cut listener**, as a resource. `Sink` a
//!   prefix to it and it listens for cuts of every thread under that prefix; `Source` takes
//!   what it has heard since it was last read; `Delete` stops it. Listening is an authority,
//!   so every verb needs `urn:cap:kernel:listen`, declared, and the kernel refuses a
//!   registration without it too. The state is the kernel's queue; this crate holds only the
//!   registration, by name.
//! * `urn:iki:tutorial:push:carries?target=…` — what a read of `target` **carries**: its
//!   golden threads, and when it expires. A question about another read, answered by making
//!   the read. It is what the host consults to decide which views a cut makes stale.
//! * [`Drawn`] — the host's memory of the views it has handed out: for each, the threads its
//!   answer carried and its deadline. Not a resource: it is the host's own state, one per
//!   page here, one per session on a server. A cut of thread `T` makes stale exactly the
//!   views whose threads include `T`, and the earliest deadline is when to look again,
//!   because **time does not cut**: an answer that expires is never announced.
//!
//! The sheet never learns that anybody is listening. Nothing in `crates/spreadsheet` changed
//! for push but one template, the page that asks each cell to listen.

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, OnceLock, Weak};

use ikigai_core::{
    ActionSpec, ArgSpec, AsyncFnEndpoint, CutBatch, CutListener, Description, EndpointSpace, Error,
    Expiry, FnEndpoint, Invocation, InvokeFuture, Iri, Kernel, ListenSpec, ReprType,
    Representation, Result, UriTemplate, Verb, CAP_LISTEN,
};

// ANCHOR: names
/// A cut listener, by name: `Sink` a prefix to listen, `Source` to take what it heard,
/// `Delete` to stop.
pub const LISTENER: &str = "urn:iki:tutorial:push:listener:{name}";

/// What a read of `target` carries: its golden threads and its deadline.
pub const CARRIES: &str = "urn:iki:tutorial:push:carries";
// ANCHOR_END: names

const TEXT_PLAIN_UTF8: &str = "text/plain;charset=utf-8";
const XSD_STRING: &str = "http://www.w3.org/2001/XMLSchema#string";
const XSD_ANY_URI: &str = "http://www.w3.org/2001/XMLSchema#anyURI";

fn text(body: String) -> Representation {
    Representation::new(
        ReprType::new("text/plain").with_param("charset", "utf-8"),
        body.into_bytes(),
    )
}

/// `urn:iki:tutorial:push:listener:{name}` for `name`.
pub fn listener_name(name: &str) -> String {
    LISTENER.replace("{name}", name)
}

// ANCHOR: listening
/// The host's registrations: each listener it holds, by the name it was registered under, and
/// a handle on the kernel that registers them.
///
/// The handle is attached after the kernel is built ([`Listening::attach`]), because the kernel
/// is built over the space that binds the listener: the two would otherwise need each other
/// first. It is weak, so the space does not keep its own kernel alive.
#[derive(Default)]
pub struct Listening {
    kernel: OnceLock<Weak<Kernel>>,
    listeners: Mutex<BTreeMap<String, Arc<CutListener>>>,
}

impl Listening {
    /// Give the listener endpoint the kernel it registers with. Once: a second call is ignored.
    pub fn attach(&self, kernel: &Arc<Kernel>) {
        let _ = self.kernel.set(Arc::downgrade(kernel));
    }

    /// The listener registered as `name`, for a host that awaits it ([`CutListener::wait`])
    /// rather than reading the resource.
    pub fn listener(&self, name: &str) -> Option<Arc<CutListener>> {
        self.lock().get(name).cloned()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<String, Arc<CutListener>>> {
        self.listeners.lock().expect("the listeners' lock")
    }

    fn kernel(&self) -> Result<Arc<Kernel>> {
        self.kernel
            .get()
            .and_then(Weak::upgrade)
            .ok_or_else(|| Error::Endpoint("this host has not attached its kernel".to_string()))
    }
}
// ANCHOR_END: listening

// ANCHOR: listener
/// `push-listener`: a cut listener, as a resource.
///
/// * `Sink` registers a listener named `{name}` for every thread under the prefixes in
///   `content` (one per line), under the request's own capability, replacing any listener of
///   that name. The kernel decides what it may hear: under the root authority, every cut
///   there; under any other, only a cut of a name some read made under that same capability
///   rests on.
/// * `Source` takes what it has heard since it was last read ([`render`]). Reading takes: a
///   second read answers only what came after the first. Never cached.
/// * `Delete` stops it.
pub fn listener(listening: Arc<Listening>) -> FnEndpoint {
    FnEndpoint::new("push-listener", move |inv: &Invocation<'_>| {
        let name = inv
            .bindings
            .get("name")
            .ok_or_else(|| Error::MissingArgument("name".to_string()))?
            .to_string();
        match inv.request.verb {
            Verb::Sink => {
                let prefixes: Vec<&str> = inv
                    .inline_str("content")?
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty())
                    .collect();
                if prefixes.is_empty() {
                    return Err(Error::InvalidArgument {
                        name: "content".to_string(),
                        detail: "no prefix: say which threads to listen for, e.g. \
                                 urn:iki:tutorial:sheet:"
                            .to_string(),
                    });
                }
                let spec = prefixes
                    .iter()
                    .fold(ListenSpec::new(), |spec, prefix| spec.prefix(*prefix));
                let registered = listening.kernel()?.listen(spec, inv.capability)?;
                listening.lock().insert(name, Arc::new(registered));
                let hears = if inv.capability.is_root() {
                    "as root: every cut there"
                } else {
                    "under this capability: a cut there only when a read made under it rests on it"
                };
                Ok(text(format!(
                    "listening for cuts under {}, {hears}\n",
                    prefixes.join(" and ")
                )))
            }
            Verb::Delete => match listening.lock().remove(&name) {
                Some(_) => Ok(text(format!("stopped listening as `{name}`\n"))),
                None => Err(Error::NotFound(format!("nothing is listening as `{name}`"))),
            },
            _ => match listening.listener(&name) {
                Some(heard) => Ok(text(render(&heard.drain()))),
                None => Err(Error::NotFound(format!(
                    "nothing is listening as `{name}`: sink a prefix to it first"
                ))),
            },
        }
    })
    .with_description(
        Description::new("push-listener")
            .title("Cut listener")
            .summary(
                "A golden-thread listener, as a resource: sink a prefix to listen, source to \
                 take what it heard, delete to stop.",
            )
            .verb(Verb::Meta)
            .action(
                ActionSpec::new(Verb::Source)
                    .summary("take the cuts heard since the last read; never cached")
                    .input(name_input())
                    .requires(CAP_LISTEN)
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Sink)
                    .summary("listen for cuts of every thread under these prefixes")
                    .input(name_input())
                    .input(
                        ArgSpec::new("content")
                            .summary("the prefixes to listen under, one per line")
                            .class(XSD_STRING),
                    )
                    .requires(CAP_LISTEN)
                    .output(TEXT_PLAIN_UTF8),
            )
            .action(
                ActionSpec::new(Verb::Delete)
                    .summary("stop listening")
                    .input(name_input())
                    .requires(CAP_LISTEN)
                    .output(TEXT_PLAIN_UTF8),
            ),
    )
}
// ANCHOR_END: listener

fn name_input() -> ArgSpec {
    ArgSpec::new("name")
        .summary("the listener's name, e.g. mine")
        .class(XSD_STRING)
        .binding()
}

// ANCHOR: render
/// A batch of cuts as the listener's `Source` answers it: each cut's thread, then the cached
/// answers it invalidated. A drop is said, never hidden: the history above it is a prefix.
pub fn render(batch: &CutBatch) -> String {
    let mut out = match batch.events.len() {
        0 if batch.dropped == 0 => return "heard nothing\n".to_string(),
        1 => "heard 1 cut\n".to_string(),
        n => format!("heard {n} cuts\n"),
    };
    for event in &batch.events {
        out.push_str(&format!("cut {}\n", event.thread.as_str()));
        for target in &event.invalidated {
            out.push_str(&format!("  invalidated {target}\n"));
        }
        if event.invalidated_more > 0 {
            out.push_str(&format!("  and {} more\n", event.invalidated_more));
        }
    }
    if batch.dropped > 0 {
        out.push_str(&format!(
            "dropped {}: the queue was full, so nothing after the last cut above is known\n",
            batch.dropped
        ));
    }
    out
}
// ANCHOR_END: render

// ANCHOR: carried
/// What one answer carries that a host needs in order to keep a copy of it honest: the golden
/// threads it hangs from (a composite's include every part's), and when it expires.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Carried {
    /// The threads, by name.
    pub threads: BTreeSet<String>,
    /// When it expires.
    pub expiry: Expiry,
}

impl Carried {
    /// What `answer` carries.
    pub fn of(answer: &Representation) -> Self {
        Carried {
            threads: answer
                .threads()
                .iter()
                .map(|t| t.as_str().to_string())
                .collect(),
            expiry: answer.expiry,
        }
    }

    /// Its deadline in milliseconds since 1970, if it has one. `Never` has none (only a cut
    /// changes it), and neither has `Always` (it is stale at once, and a timer cannot help).
    pub fn deadline(&self) -> Option<u64> {
        match self.expiry {
            Expiry::At(time) => Some(time.as_millis()),
            _ => None,
        }
    }
}
// ANCHOR_END: carried

/// An expiry, in words: what `carries` prints after `expires `.
///
/// ```
/// use ikigai_core::{Expiry, Time};
///
/// // 2026-09-29T14:06Z, in milliseconds since 1970.
/// let minute = Time::from_millis(1_790_690_760_000);
/// assert_eq!(push::expiry_text(Expiry::At(minute)), "at 2026-09-29T14:06Z");
/// assert_eq!(push::expiry_text(Expiry::Never), "never: only a cut changes it");
/// assert_eq!(push::expiry_text(Expiry::Always), "at once: it is never cached");
/// ```
pub fn expiry_text(expiry: Expiry) -> String {
    match expiry {
        Expiry::Never => "never: only a cut changes it".to_string(),
        Expiry::Always => "at once: it is never cached".to_string(),
        Expiry::At(time) => {
            let millis = time.as_millis();
            let minute = time_resource::instant_text((millis / 60_000) as i64);
            match millis % 60_000 {
                0 => format!("at {minute}"),
                rest => format!("at {minute}, plus {rest} ms"),
            }
        }
    }
}

// ANCHOR: carries
/// `push-carries`: what a read of `target` carries. Makes the read, and answers its deadline
/// and its threads, one per line. Never cached: it is a question about a read made now.
pub fn carries() -> AsyncFnEndpoint {
    AsyncFnEndpoint::new("push-carries", |inv: &Invocation<'_>| -> InvokeFuture<'_> {
        Box::pin(async move {
            let named = inv.inline_str("target")?.trim();
            let target = Iri::parse(named).map_err(|e| Error::InvalidArgument {
                name: "target".to_string(),
                detail: format!("`{named}` is not a name: {e}"),
            })?;
            let carried = Carried::of(&inv.source(&target).await?);
            let mut out = format!(
                "{named}\nexpires {}\nhangs from {} threads\n",
                expiry_text(carried.expiry),
                carried.threads.len()
            );
            for thread in &carried.threads {
                out.push_str(&format!("  {thread}\n"));
            }
            Ok(text(out))
        })
    })
    .with_description(
        Description::new("push-carries")
            .title("What a read carries")
            .summary("Read a resource, and answer the golden threads and the deadline it carries.")
            .verb(Verb::Source)
            .verb(Verb::Meta)
            .input(
                ArgSpec::new("target")
                    .summary("the name to read")
                    .class(XSD_ANY_URI),
            )
            .output(TEXT_PLAIN_UTF8),
    )
}
// ANCHOR_END: carries

// ANCHOR: drawn
/// The host's memory of the views it has handed out: for each view's name, what its last
/// answer carried. One per page here; one per session on a server.
#[derive(Debug, Default)]
pub struct Drawn {
    views: BTreeMap<String, Carried>,
}

impl Drawn {
    /// The host drew `view` from `answer`: remember what the answer carried, replacing
    /// whatever the view's last answer did.
    pub fn drew(&mut self, view: impl Into<String>, answer: &Representation) {
        self.views.insert(view.into(), Carried::of(answer));
    }

    /// The views a batch of cuts makes stale: each view whose threads include a thread that
    /// was cut. No dependents index: the answer already named everything it rests on. A batch
    /// that DROPPED cuts makes every view stale, since what was missed is unknown.
    pub fn stale(&self, batch: &CutBatch) -> Vec<String> {
        if batch.dropped > 0 {
            return self.views.keys().cloned().collect();
        }
        self.views
            .iter()
            .filter(|(_, carried)| {
                batch
                    .events
                    .iter()
                    .any(|event| carried.threads.contains(event.thread.as_str()))
            })
            .map(|(view, _)| view.clone())
            .collect()
    }

    /// The earliest deadline among the views, in milliseconds since 1970: when to set the one
    /// timer. A cut is announced; an expiry is not, so the host has to know when to look.
    pub fn deadline(&self) -> Option<u64> {
        self.views.values().filter_map(Carried::deadline).min()
    }

    /// The views whose deadline is at or before `now` (milliseconds since 1970).
    pub fn expired(&self, now: u64) -> Vec<String> {
        self.views
            .iter()
            .filter(|(_, carried)| carried.deadline().is_some_and(|at| at <= now))
            .map(|(view, _)| view.clone())
            .collect()
    }

    /// What the view's last answer carried, if the host has drawn it.
    pub fn carried(&self, view: &str) -> Option<&Carried> {
        self.views.get(view)
    }

    /// How many views the host remembers.
    pub fn len(&self) -> usize {
        self.views.len()
    }

    /// Whether the host remembers none.
    pub fn is_empty(&self) -> bool {
        self.views.is_empty()
    }
}
// ANCHOR_END: drawn

/// Every name push adds: the listener and what a read carries.
pub fn space(listening: Arc<Listening>) -> EndpointSpace {
    EndpointSpace::new()
        .bind(template(LISTENER), listener(listening))
        .bind(template(CARRIES), carries())
}

fn template(source: &str) -> UriTemplate {
    UriTemplate::parse(source).expect("a constant template parses")
}
