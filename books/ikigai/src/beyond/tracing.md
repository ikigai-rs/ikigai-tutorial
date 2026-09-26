# Tracing a resolution

`trace` has appeared twice already as a command — in the [front door](../front-door/cli.md)
and in [What resolution buys you](../getting-started/payoff.md), where `issue_traced` drew a
tree with one child. This chapter makes it the subject, because three chapters of
plumbing have just given a resolution more ways to go somewhere you did not expect: a
socket, a clamp, a mount that fell back. The trace is how you find out what actually
happened, and — the part worth a chapter — it is where the kernel records the facts that
name something which *never ran*.

## An event per invocation

While a tracer is installed, the kernel hands it one `TraceEvent` per invocation of the
resolution being traced. The fields are the questions you would ask afterwards:

| field | what it answers |
|---|---|
| `target` | which name — the *canonical* one, after any rewrite |
| `span`, `parent` | where in the tree; `(span, parent)` pairs rebuild it |
| `cache_hit` | served, or computed |
| `started`, `ended` | when, if the kernel has a clock; `None` if it does not |
| `thread` | which worker ran it |
| `capability` | the authority it ran under — `None` for root, the scopes otherwise |
| `notes` | facts attached to this hop: by the endpoint, or by the kernel |

Nothing is recorded when no tracer is installed; an untraced resolution pays one atomic
load. The events are plain, serializable data, which is why they can cross a wire.

## Two ways to install one, and they are exclusive

`Kernel::set_tracer` installs a tracer on the kernel. Every resolution after it records
there until `clear_tracer` — one collector for the whole process, which is what the
REPL's `trace` does around exactly one call. `Kernel::issue_traced(request, cap, tracer)`
records **this resolution** into `tracer` and nothing else, with its own span numbering
starting at 0.

The two are exclusive, not additive. A resolution routed through `issue_traced` does
not land in the global collector, and a host that sets one *and* traces some calls
per-request will not see those calls globally. That is deliberate: the alternative is a
double write, and the per-call form exists for a server tracing one connection's call
while other tenants' resolutions run beside it — with a single global slot their events
would interleave into whichever collector was installed last. The wire servers use the
per-call form for that reason. It is a contract you would otherwise discover by reading
the kernel, which is not where a host author looks, so it is written here.

## What a denial looks like

The kernel enforces a declared `requires` before dispatch, so a refused endpoint is never
entered — and nothing inside it can report the refusal. The kernel records it instead:
an event whose `notes` carry `("denied", <the scope the caller lacked>)`, with
`started` and `ended` both `None` and `cache_hit` false, because nothing ran. `counter`
from [Multi-verb endpoints](../building/multi-verb.md) declares a write scope on its
`Sink`; here a narrow caller tries to write, through `issue_traced`:

```rust
# extern crate building_endpoints;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::{Arc, Mutex};
use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Error, Iri, Request, TraceEvent, Tracer, Verb, DENIED_NOTE,
};

#[derive(Default)]
struct Recorder(Mutex<Vec<TraceEvent>>);
impl Tracer for Recorder {
    fn record(&self, event: TraceEvent) {
        self.0.lock().unwrap().push(event);
    }
}

let kernel = building_endpoints::kernel_in(None);
let reader = Capability::root().attenuate(["urn:cap:kernel:inspect"]);
let write = Request::new(Verb::Sink, Iri::parse("urn:iki:tutorial:counter").unwrap())
    .with_arg("content", ArgRef::Inline(b"3".to_vec()));

let recorder = Arc::new(Recorder::default());
let err = block_on(kernel.issue_traced(write, &reader, recorder.clone())).unwrap_err();
assert!(matches!(err, Error::Denied(_)));

// One event, for something that never ran.
let events = recorder.0.lock().unwrap();
assert_eq!(events.len(), 1);
let denial = &events[0];
assert_eq!(denial.target, "urn:iki:tutorial:counter");
assert!(denial.started.is_none() && denial.ended.is_none() && !denial.cache_hit);
assert_eq!(denial.capability.as_deref(), Some(&["urn:cap:kernel:inspect".to_string()][..]));
assert!(denial
    .notes
    .iter()
    .any(|(key, scope)| key == DENIED_NOTE && scope == "urn:cap:tutorial:counter:write"));
```

Why record it at all? Because a refused authority is a security fact, and the one action
that could log it is the one that was refused. A log built on this seam sees every
denial; without it a denial reaches no observer inside the kernel.

In the REPL the same refusal is one line. `trace` renders the tree only when the
resolution succeeded; on an error it prints where it stopped:

```text
ikigai> cap urn:cap:kernel:inspect
narrowed — capability: urn:cap:kernel:inspect
ikigai> trace urn:file:notes.txt
trace  urn:file:notes.txt
  client      ikigai repl  ·  capability: urn:cap:kernel:inspect
  transport   embedded · in-process

urn:file:notes.txt   cap ✗ denied: denied: capability does not grant `urn:cap:fs:read:*` (declared by `urn:file:notes.txt`)
```

The `cap ✗` is the authority dimension made visible; the message names the scope lacked
and the resource that declared it. The denial *event* exists in the kernel either way —
the REPL simply does not draw it.

## What an alias hop looks like

A request whose target arrived through a logical rewrite — [Scope and
alias](../getting-started/scope-and-alias.md) — carries `("alias", "logical -> canonical")`
on **every** event of that resolution: the computed one, the cache hit, the denial, and
the miss. `target` is the canonical name, because that is the name the kernel resolved,
keyed and gated; the alias is disclosed as provenance rather than by editing anything.
The CLI's own table rewrites `urn:fn:` to `urn:iki:fn:`, so an old spelling traces like
this:

```text
ikigai> trace urn:fn:toUpper in="a b"
trace  urn:fn:toUpper
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process

urn:iki:fn:toUpper   toUpper · computed · main · 0ms · alias=urn:fn:toUpper -> urn:iki:fn:toUpper   → 3b  A B
```

Two more channels carry the same fact with no tracer installed, because a rewrite the
operator cannot see is the bad afternoon this was built to prevent: `source
urn:kernel:aliases` lists the rules with per-rule counters, and a denial on a rewritten
target names the hop in its message. A rewrite that lands on nothing is traced too,
under `("alias-unresolved", <the backing name>)` — an ordinary miss records no event,
but this one is the case where the caller asked for a name that *looks* bound.

## What a confined miss looks like

Inside a severed chain a name outside it is `Unresolved`, and that is indistinguishable
from "no such resource" by design — the whole point of `Confine` is that there is no
decision to read back. The trace is the one place the fact "this name *is* bound, just
not in here" can show. Every event of a resolution in a non-empty chain carries
`("scope", <the chain>)`, innermost first, ending in `root` or `severed`; a miss inside
one adds `("scope-unresolved", <the name>)`:

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::{Arc, Mutex};
use futures::executor::block_on;
use ikigai_core::{
    AsyncFnEndpoint, Capability, Confine, EndpointSpace, Error, Exact, FnEndpoint, Iri,
    Kernel, ReprType, Representation, Request, TraceEvent, Tracer, Verb, DENIED_NOTE,
    SCOPE_MISS_NOTE, SCOPE_NOTE,
};

#[derive(Default)]
struct Recorder(Mutex<Vec<TraceEvent>>);
impl Tracer for Recorder {
    fn record(&self, event: TraceEvent) {
        self.0.lock().unwrap().push(event);
    }
}

// The confined endpoint from "Scope and alias": it may see the document, not `title`.
let excerpt = AsyncFnEndpoint::new("excerpt", |inv| {
    Box::pin(async move { inv.source(&Iri::parse(hello_camel::TITLE).unwrap()).await })
});
let corridor = Arc::new(EndpointSpace::new().bind(
    Exact::new("urn:iki:tutorial:document"),
    FnEndpoint::new("document", |_| {
        Ok(Representation::new(ReprType::new("text/plain"), b"the document".to_vec()))
    }),
));
let kernel = Kernel::new(Arc::new(hello_camel::space().bind_arc(
    Exact::new("urn:iki:tutorial:excerpt"),
    Arc::new(Confine::new(
        Iri::parse("urn:iki:tutorial:ctx:document").unwrap(),
        corridor,
        Arc::new(excerpt),
    )),
)));

let recorder = Arc::new(Recorder::default());
let request = Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:excerpt").unwrap());
let err = block_on(kernel.issue_traced(request, &Capability::root(), recorder.clone()))
    .unwrap_err();
assert!(matches!(err, Error::Unresolved(_)));

// The miss is traced: the chain it happened in, and the name that had no door there.
let events = recorder.0.lock().unwrap();
let miss = events
    .iter()
    .find(|e| e.notes.iter().any(|(key, _)| key == SCOPE_MISS_NOTE))
    .expect("the confined miss is recorded");
let note = |key: &str| miss.notes.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str());
assert_eq!(note(SCOPE_NOTE), Some("urn:iki:tutorial:ctx:document severed"));
assert_eq!(note(SCOPE_MISS_NOTE), Some(hello_camel::TITLE));
assert_eq!(note(DENIED_NOTE), None);
```

Read the last three lines as the three questions an operator asks: *where was it
standing* (a chain named for the document, root cut off), *what did it ask for* (the
title), and *was it refused* (no — it was nowhere to be found). A resolution in the empty
chain carries no scope note at all, so events from before the chain existed are
byte-identical to now.

## Across the wire

The events are data, and the wire carries them: a traced call to a served kernel comes
back with the spans that kernel recorded, and the connected `trace` renders them as-is.
The [Python track](../polyglot/python.md) reads the same events from the other side. A
*mount* inside a local resolution is stitched under the mount's own node, so the remote
execution shows where the local one handed off.

## What not to assume

- **The REPL draws a tree only on success.** A denial or a miss is one line there, with
  the reason; the event itself is in the kernel, and a Rust host or a log sees it.
- **The REPL has no scope**, so `("scope", …)` notes are visible only from a host that
  resolves in a chain — `issue_in` records them through the global tracer, `Confine`
  through either.
- **A successful `urn:kernel:*` operation records no event.** Reading the catalog is
  introspection, not throughput, and it returns before the trace path; only its
  *denial* is recorded. An observer therefore sees refusals from that namespace and never
  successes, which is the fact worth keeping.
- **Installing a tracer to see denials means building an event for every resolution.**
  There is no denial-only reporter yet; a host that wants only refusals filters in its own
  `record`. Stated so the cost is not a surprise.

The next chapter is the client that never traces anything and needs none of this to
work: it reads the manifold instead.
