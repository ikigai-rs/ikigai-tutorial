# One-off review, and what a finding is

The [explain chapter](../browse/explain.md) had a model describe a file. A **review** asks it for
something more dangerous: claims that the file is wrong. A claim like that is cheap to make and
expensive to check, and a review system's whole design is about who pays for the checking. In
gonk the answer is one rule, enforced by capability rather than by habit:

> a model's claim is a **finding**, pending until a person decides; only a person's decision
> turns it into an **annotation** that anyone else sees.

This chapter runs one review pass over the explain chapter's repository, with the same stub
model, and walks one finding to each end: declined with a reason, and published with the
evidence that it is real.

## A review pass

`urn:repo:{root}:review:{path}` sends the file to the review provider and parses what comes back.
The stub's answer, in the grammar browse reads — three lines per finding:

```rust,ignore
{{#include ../../../../crates/browse-host/src/lib.rs:review}}
```

One claim is real: `&word[..1]` slices the first byte of each word, and an empty word — two
spaces in a row, or an empty name — has none. The other is not: `initials` builds a string out of
letters taken from several words, which no `&str` borrowed from the input could hold.

```rust
# extern crate browse_host;
# extern crate ikigai_core;
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::Arc;
use browse_host::{ask, kernel, Scratch};
use ikigai_core::Verb;

let repo = Scratch::new();
let calls = Arc::new(AtomicUsize::new(0));
let browse = kernel(repo.path(), calls.clone());
let source = |iri: &str, args: &[(&str, &str)]| ask(&browse, Verb::Source, iri, args).unwrap();

let pass = source("urn:repo:demo:review:src/lib.rs", &[]);
assert!(pass.starts_with(
    "review by stub-1 · review-v5@stub-1 · 2 findings · reviewed the whole file (142 bytes)"));

let pending = source("urn:repo:demo:findings:src/lib.rs", &[("as", "text/plain")]);
assert!(pending.contains("Findings wait for a person."), "{pending}");
assert!(pending.contains(r#"src/lib.rs L3 [pending] major "name.split(' ')"#), "{pending}");
assert!(pending.contains(r#"src/lib.rs L2 [pending] minor "pub fn initials"#), "{pending}");

// Nothing was published.
assert_eq!(source("urn:repo:demo:annotations:src/lib.rs", &[]).trim(), "[]");

// Like an explanation, a pass is archived by content: asking again calls nothing.
assert_eq!(calls.load(Ordering::SeqCst), 1);
source("urn:repo:demo:review:src/lib.rs", &[]);
assert_eq!(calls.load(Ordering::SeqCst), 1);
assert!(source("urn:repo:demo:review-status:src/lib.rs", &[]).starts_with("archived\t"));
```

The first line of the pass is its **statement**: who reviewed, at which version tag
(`review-v5@stub-1`, the review prompt's version and the model), how many findings, and how much
of the file was read. "Reviewed the whole file" is a claim browse checks: a large file is split
into regions that tile it, and a region that did not come back is reported as `coverage
incomplete` rather than silently skipped. A clean file is a pass with zero findings and says
`nothing above threshold`, which is an answer, not a failure.

## What a finding is

Each claim becomes a resource, `urn:iki:finding:{id}`, with the quote it is anchored to (the
exact text, and where it sits), the model's note, and the severity the model **proposed**:
`critical`, `major`, `minor`, `info` or `praise`. Its id is a hash of the pass and the quote, so
the same pass over the same bytes names the same findings, here and on any machine:

| finding | line | proposed | the claim |
| --- | --- | --- | --- |
| `urn:iki:finding:bd771489c5a22c9a4d45f4dd` | 3 | major | the slice panics on an empty word |
| `urn:iki:finding:1d920b0266c30789b5c61b08` | 2 | minor | return a `&str` instead |

A finding is **pending** until someone decides, and pending is the whole of its authority: the
listing says so on its second line, and nothing about a pending finding blocks a commit, a push
or a merge. (A finding gonk's review queue would ask about — `critical` and `major` by default —
can also be sent to a second model, a *judge*, for a verdict that orders the queue. The judge
orders; it does not decide. This chapter's host turns it off, since a judge of a stub would judge
nothing.)

## Deciding

A decision is a Sink to the finding. Declining takes a **reason** from a closed set, because
"why not" is the data that makes the next review better:

| reason | means |
| --- | --- |
| `misread` | the claim is false: the model misunderstood the code or the domain |
| `restates` | the code already says it: a comment that discloses a hazard, restated as a finding |
| `no-issue` | true, but not a defect: style, "could be clearer", an absence asserted as a problem |
| `wont-fix` | real, and deliberately left as it is |
| `duplicate` | already raised |

Publishing takes `reproduced=yes` when the person has made the defect happen. That is not a
formality: in the measurements behind the [next chapter](audit.md), a review model's claims that a
second model had judged credible held, when someone tried to reproduce them, almost never. The
reproduction for this one is a few lines:

```rust,should_panic
// The function under review, exactly as the repository has it.
fn initials(name: &str) -> String {
    name.split(' ').map(|word| &word[..1]).collect()
}

assert_eq!(initials("Ada Lovelace"), "AL");
initials("Ada  Lovelace"); // two spaces: the middle word is empty, and `[..1]` panics
```

So the major finding is published, with the reproduction as its note, and the minor one declined:

```rust
# extern crate browse_host;
# extern crate ikigai_core;
# use std::sync::atomic::AtomicUsize;
# use std::sync::Arc;
use browse_host::{ask, kernel, Scratch};
use ikigai_core::{Error, Verb};

let repo = Scratch::new();
let browse = kernel(repo.path(), Arc::new(AtomicUsize::new(0)));
ask(&browse, Verb::Source, "urn:repo:demo:review:src/lib.rs", &[]).unwrap();
let real = "urn:iki:finding:bd771489c5a22c9a4d45f4dd";
let wrong = "urn:iki:finding:1d920b0266c30789b5c61b08";

let declined = ask(&browse, Verb::Sink, wrong, &[
    ("decision", "decline"),
    ("reason", "misread"),
    ("content", "A borrowed &str cannot hold letters taken from several words."),
]).unwrap();
assert_eq!(declined.trim(), wrong);

let published = ask(&browse, Verb::Sink, real, &[
    ("decision", "publish"),
    ("reproduced", "yes"),
    ("content", r#"initials("Ada  Lovelace") panics: the empty middle word has no byte 0."#),
]).unwrap();
assert_eq!(published.trim(), "urn:iki:annotation:bd771489c5a22c9a4d45f4dd");

let row = ask(&browse, Verb::Source, real, &[]).unwrap();
assert!(row.contains("L3 [published] major -> major [reproduced]"), "{row}");
let row = ask(&browse, Verb::Source, wrong, &[]).unwrap();
assert!(row.contains("L2 [declined] minor -> minor (misread)"), "{row}");

// The published one is now an annotation on the file; the declined one is not.
let notes = ask(&browse, Verb::Source, "urn:repo:demo:annotations:src/lib.rs", &[]).unwrap();
assert!(notes.contains(r#""iri":"urn:iki:annotation:bd771489c5a22c9a4d45f4dd""#), "{notes}");
assert!(notes.contains(r#""derived_from":"urn:iki:finding:bd771489c5a22c9a4d45f4dd""#));
assert!(!notes.contains("1d920b0266c30789b5c61b08"));

// A decision is a record. Changing it means saying which decision is revised.
let again = ask(&browse, Verb::Sink, wrong, &[("decision", "publish")]);
assert!(matches!(again, Err(Error::InvalidArgument { .. })));
assert!(format!("{}", again.unwrap_err())
    .contains("revises=urn:iki:finding:1d920b0266c30789b5c61b08:decision"));

// And the reasons are a closed set.
let made_up = ask(&browse, Verb::Sink, real, &[("decision", "decline"), ("reason", "meh")]);
assert!(format!("{}", made_up.unwrap_err())
    .contains("one of: misread, restates, no-issue, wont-fix, duplicate"));
```

`-> major` is the severity the person settled on (leaving `severity=` out accepts the model's
proposal). The annotation keeps the finding's id and says which finding it was **derived from**, so
the published note and the model's claim behind it stay one hop apart. And a decision is never
overwritten: a second, different one must name the first with `revises=`, and both are kept.

## Who may do which

The rule at the top of the chapter is not a convention. Deriving a review spends the model, so it
needs the network grant; publishing writes what everyone else reads, so it needs
`urn:cap:annotate`; and nothing that may derive is given the other by default:

```rust
# extern crate browse_host;
# extern crate ikigai_core;
# use std::sync::atomic::AtomicUsize;
# use std::sync::Arc;
use browse_host::{ask_as, kernel, Scratch};
use ikigai_core::{Capability, Error, Verb};

let repo = Scratch::new();
let browse = kernel(repo.path(), Arc::new(AtomicUsize::new(0)));
let reader = Capability::scoped(["urn:cap:browse:read:demo".to_string()]);
let deriver = Capability::scoped([
    "urn:cap:browse:read:demo".to_string(),
    "urn:cap:net:localhost".to_string(),
]);

let review = "urn:repo:demo:review:src/lib.rs";
assert!(matches!(ask_as(&browse, &reader, Verb::Source, review, &[]), Err(Error::Denied(_))));
assert!(ask_as(&browse, &deriver, Verb::Source, review, &[]).is_ok());

let publish = ask_as(&browse, &deriver, Verb::Sink, "urn:iki:finding:bd771489c5a22c9a4d45f4dd",
                     &[("decision", "publish")]);
assert!(matches!(publish, Err(Error::Denied(ref why)) if why.contains("urn:cap:annotate")));
```

A reader can read the archive; a deriver can run a pass and read its findings; only a holder of
`urn:cap:annotate` can publish. In gonk those are the `--browse read` and `--browse derive` client
grants, and the person at the browser.

## In gonk

gonk's file page has a **Review** button beside **Explain**, with the same cost line
(`review: already reviewed at this version by … — the findings open instantly from the archive, no
model call.`), and its **Queue** lists the pending findings a person should look at, serious ones
first, with Decline (and its five reasons) and Publish on each. Over the socket the resources are
the ones above, under the `urn:gk:` alias of [Over the socket](../agent/socket.md):
`urn:gk:repo:{root}:review:{path}`, `urn:gk:repo:{root}:findings:{path}`, a Sink to
`urn:gk:iki:finding:{id}`. Running a pass needs a mounted model; this book's checks have none, so
the [explain chapter](../browse/explain.md) shows only that gonk refuses it as `unavailable`.

What gonk can also do — review every file a commit touches, from a `post-commit` hook — is off,
and the next chapter is why.

## What you built

The **atoms** are the file and browse's store, now holding pass records, findings and decisions
beside the explain archive. A **finding** is a resource a composition (the review pass) mints and
only a person moves; a **decision** is a resource too, one per decision, never rewritten; an
**annotation** is a view that exists because a decision says so. The code written for it is still
only the stub model.
