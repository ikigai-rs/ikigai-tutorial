# Checking a resource

[Transreption](transreption.md) ended on a rule: the format is a property of the route, not
of the resource. Follow it one step further. If a route reaches Turtle, the resource has a
**Turtle face**, whatever it was written in, and anything with a Turtle face can be
*checked*: does it parse, and does what it says fit the shapes it is supposed to fit. This
chapter calls that a CHECK. The RDF Toolbox
([`ikigai-toolbox`](https://github.com/ikigai-rs/ikigai-toolbox)) is the page that does it in a
browser, and this chapter builds what the page reads and how it reads it.

The code is `crates/check-resource`. Unlike the other chapters of this part, it does not chain
onto Part I's space: its host holds two players of tic-tac-toe, a transreptor, the shapes a player
must fit, and two published spaces beside them. Nothing in it starts a server, so there is nothing
to stop afterwards.

```rust,ignore
{{#include ../../../../crates/check-resource/src/lib.rs:space}}
```

## A resource with no Turtle in it

Each player answers N-Triples, one triple per line, and nothing else. Reading one requires a
scope, declared the way [Multi-verb endpoints](multi-verb.md) declared the counter's write: the
kernel enforces the `requires`, so the endpoint's code holds no check of its own.

```rust,ignore
{{#include ../../../../crates/check-resource/src/lib.rs:players}}
```

The Turtle face comes from a transreptor, declared lossless because every triple survives the
trip. It is the kind of endpoint [Transreption](transreption.md) wrote, in the direction that
reaches the pivot:

```rust,ignore
{{#include ../../../../crates/check-resource/src/lib.rs:to_turtle}}
```

## The check reads as you

The toolbox reads a resource through one name, `urn:toolbox:input:{iri}`: the resource sourced
with `as=text/turtle`, and when it answers in another RDF syntax, carried to Turtle by whatever
transreptor plan the kernel finds. The name declares no capability of its own. It issues that
read under the **caller's** capability, so a check never shows a reader more than they could
read directly, and the toolbox is not a way around a scope.

```rust,ignore
{{#include ../../../../crates/check-resource/src/lib.rs:face}}
```

<!-- urn-gate: illustration urn:cap:tutorial:counter:write — a scope the stranger below holds,
     named only to show that holding some scope is not holding the one a player requires. -->

```rust
# extern crate check_resource;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{Capability, Error};
use check_resource::{kernel, turtle_face, PLAYER_X, READ};

let kernel = kernel();

// A reader holding the scope gets the player's Turtle face: N-Triples in, Turtle out,
// and nothing in the player knows Turtle exists.
let reader = Capability::scoped([READ]);
let face = block_on(turtle_face(&kernel, PLAYER_X, &reader)).unwrap();
assert_eq!(
    face,
    "@prefix ttt: <http://example.org/ttt#> .\n\
     <urn:iki:tutorial:check:player:x> a ttt:Player ;\n\
     \tttt:name \"Ada\" ;\n\
     \tttt:mark \"X\" .\n"
);

// A caller without it is refused, and the refusal names the player, not the toolbox.
let stranger = Capability::scoped(["urn:cap:tutorial:counter:write"]);
let refused = block_on(turtle_face(&kernel, PLAYER_X, &stranger)).unwrap_err();
assert!(matches!(refused, Error::Denied(_)));
assert_eq!(
    refused.to_string(),
    "denied: capability does not grant `urn:cap:tutorial:players:read` \
     (declared by `urn:iki:tutorial:check:player:x`)"
);
```

## Shapes, in a paragraph

[SHACL](https://www.w3.org/TR/shacl/) describes what a graph should look like, in a graph of its
own. A **node shape** picks the nodes it applies to (here, `sh:targetClass`: every
`ttt:Player`), and its **property shapes** constrain what hangs off each one: how many values a
predicate may have, of which datatype, from which list. A shape is **closed** when it also says
`sh:closed true`: then a node may carry only the predicates its property shapes name, and those
listed in `sh:ignoredProperties`, and any other triple is a violation. An open shape lets a
stray property through unremarked; a closed one turns it into a finding. That is why
`ikigai-layers` requires `sh:closed true` on every node shape of a layer: an entry carries exactly
what its layer declared, and a property nobody declared is refused rather than kept. The player
shapes are closed the same way:

```rust,ignore
{{#include ../../../../crates/check-resource/src/lib.rs:shapes}}
```

## Checking data against shapes

The toolbox page takes a second input for this (`shapes=`), read exactly like the first, and
checks one against the other. The page does the checking in your browser tab, with
[turtle-lsp](https://github.com/abhishekkundu25/turtle-lsp) compiled to WebAssembly, and no Rust
listing can run a browser tab. So here the shapes check runs on the host instead, through
`urn:shacl:validate` from [`ikigai-shacl`](https://github.com/ikigai-rs/ikigai-shacl). The inputs
are the same and are read the same way, as Turtle faces under one caller's capability, but the
engine is a different one:

```rust,ignore
{{#include ../../../../crates/check-resource/src/lib.rs:check}}
```

```rust
# extern crate check_resource;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::Capability;
use check_resource::{check, kernel, PLAYER_O, PLAYER_X, READ, SHAPES};

let kernel = kernel();
let reader = Capability::scoped([READ]);

// X has a name, a mark and a type, and nothing else: it conforms.
let x = block_on(check(&kernel, PLAYER_X, SHAPES, &reader)).unwrap();
assert!(x.conforms);

// O carries `ttt:wins`, which the closed shape does not name: one violation.
let o = block_on(check(&kernel, PLAYER_O, SHAPES, &reader)).unwrap();
assert!(!o.conforms);
let found = &o.outcome().violations[..];
assert_eq!(found.len(), 1);
assert_eq!(found[0].focus_node, PLAYER_O);
assert_eq!(found[0].path.as_deref(), Some("http://example.org/ttt#wins"));
assert_eq!(found[0].component, "http://www.w3.org/ns/shacl#ClosedConstraintComponent");
```

The report is data, like every other answer in this book: `as=application/json` gives the
structure above, and the default, `text/turtle`, is the same findings as a SHACL validation
report graph you can query.

`ikigai-shacl` is linked here with `default-features = false`, which builds its subset
validator: closed shapes and the common constraints, with no network stack. A term outside the
subset is refused by name rather than skipped. The default build links
[rudof](https://github.com/rudof-project/rudof) and covers all of SHACL Core.

## The page itself

On a host that serves the toolbox, the check is one link. `urn:toolbox:check:{iri}` is the page
checking that resource, `shapes=` adds the shapes, and a host routes `/toolbox/check/{iri}` to it,
so the check of player O against the player shapes is:

```text
/toolbox/check/urn%3Aiki%3Atutorial%3Acheck%3Aplayer%3Ao?shapes=urn%3Aiki%3Atutorial%3Acheck%3Ashapes#shacl
```

The page reads both inputs through `urn:toolbox:input:{iri}`, as your own browser session, so
what you saw above is what it sees. What the page needs that this crate does not have is the
**build**: the page, its scripts and the 8.3 MB engine, which `ikigai-toolbox`'s `build.sh`
writes to a directory a host binds at `urn:iki:toolbox:dist:{path}`. This crate binds no build,
so asking it for the page fails on the first file the page is composed from:

<!-- urn-gate: unbound urn:iki:toolbox:dist:{path} — where a host binds a toolbox build; this
     chapter's host binds none. -->
<!-- urn-gate: unbound urn:iki:toolbox:dist:page.template.html — a toolbox build's page template;
     this chapter's host binds no build, and the listing below shows that it does not. -->

```rust
# extern crate check_resource;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{Capability, Error, Iri, Request, Verb};

let page = Iri::parse("urn:toolbox:check:urn:iki:tutorial:check:player:o").unwrap();
let err = block_on(check_resource::kernel().issue(Request::new(Verb::Source, page), &Capability::root()))
    .unwrap_err();
assert!(matches!(err, Error::Unresolved(ref name) if name.as_str() == "urn:iki:toolbox:dist:page.template.html"));
```

gonk, the host *The gonk Book* is about, serves the page at `/toolbox` when its configuration
names a build (`gonk.toolbox.dist`).

## A CHECK is not `urn:kernel:validate`

The kernel has a name that sounds like this chapter and is not: `urn:kernel:validate`. It checks
a **request** against the contract of the action it would invoke, before anything runs: required
inputs present, values from the declared lists, no unknown arguments, the capability sufficient.
A CHECK reads a **resource** that already exists and asks whether what it says fits a set of
shapes. One asks "may this call run as written?", the other "is this data what it should be?".
Both answer in SHACL's report vocabulary, which is where the confusion starts: the focus of a
`urn:kernel:validate` result is an action's contract, and the focus of a CHECK result is a node
in your data. [The machine client](../beyond/agent.md), later in the book, puts
`urn:kernel:validate` to work as the last step before an agent's call runs. Here it is
pre-flighting a call to `urn:shacl:validate` that forgot its `data` and asked for a face the
endpoint does not have:

```rust
# extern crate check_resource;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

let preflight = Request::new(Verb::Source, Iri::parse("urn:kernel:validate").unwrap())
    .with_arg("endpoint", ArgRef::Inline(b"urn:shacl:validate".to_vec()))
    .with_arg("verb", ArgRef::Inline(b"source".to_vec()))
    .with_arg("args", ArgRef::Inline(b"shapes=urn:iki:tutorial:check:shapes&as=text/csv".to_vec()));
let report = block_on(check_resource::kernel().issue(preflight, &Capability::root())).unwrap();
let report = String::from_utf8(report.bytes).unwrap();

assert!(report.contains("sh:conforms false"));
assert!(report.contains("required input `data` is missing"));
assert!(report.contains(
    "`text/csv` is not an accepted value of `as` (one of: text/turtle, application/json)"
));
```

Nothing was validated against shapes there, and no player was read: the request never ran.

## What not to assume

- **This book's pages do not run the toolbox.** The listings above run under `mdbook test`, on
  the host side. The page and its engine run in a browser, served by a host that has a build.
- **Two engines, one question.** The page's SHACL engine is turtle-lsp's; the listings' is
  `ikigai-shacl`'s subset. Both check SHACL Core's closed shapes, but they are different code, and
  their messages differ even where their findings agree.
- **A check sees what you may read.** A reader who cannot read a resource cannot check it, and
  the refusal is the resource's, as above. That is the design, not a gap.
