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

This page runs the same two reads. Its kernel holds the chapter's host beside the spaces the
book's other chapters run in, and `cap` narrows the page's session the way it narrows a shell's,
until `cap reset` gives the authority back. Run it, then run it again: the reader's face comes
from the cache the second time, because the cache keys on the capability as well as the name.

<div class="ikigai-run" data-cmd='cap urn:cap:tutorial:players:read
source urn:toolbox:input:urn:iki:tutorial:check:player:x
cap reset
cap urn:cap:tutorial:counter:write
source urn:toolbox:input:urn:iki:tutorial:check:player:x
cap reset'>
<pre class="ikigai-run-expected">narrowed — capability: urn:cap:tutorial:players:read
@prefix ttt: &lt;http://example.org/ttt#&gt; .
&lt;urn:iki:tutorial:check:player:x&gt; a ttt:Player ;
	ttt:name "Ada" ;
	ttt:mark "X" .
[computed]
reset to identity — capability: root (full authority)
narrowed — capability: urn:cap:tutorial:counter:write
error: denied: capability does not grant `urn:cap:tutorial:players:read` (declared by `urn:iki:tutorial:check:player:x`)
reset to identity — capability: root (full authority)</pre>
</div>

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

The same check runs in this page, written the way a shell writes it: the Turtle face piped
into `urn:shacl:validate`, where it fills `data`, the one required input left unnamed. X
conforms. O's report is the graph the paragraph below describes, and its one result names the
property: `sh:resultPath ttt:wins`, from `sh:ClosedConstraintComponent`. The toolbox's own
engine checks the same two players further down, in [The page itself](#the-page-itself).

<!-- urn-gate: illustration urn:ikigai:shacl:report:b3:1258eb4246e33211b9d99fd84328c8723a8aae4c2627e1c1973fca3041184544:report — the report graph's own
     node, skolemized from the report's digest; it names a node in the answer, not a resource. -->
<!-- urn-gate: illustration urn:ikigai:shacl:report:b3:1258eb4246e33211b9d99fd84328c8723a8aae4c2627e1c1973fca3041184544:result.0 — the report's one result,
     a node in the same answer. -->

<div class="ikigai-run" data-cmd='source urn:toolbox:input:urn:iki:tutorial:check:player:x | urn:shacl:validate shapes=urn:iki:tutorial:check:shapes as=application/json
source urn:toolbox:input:urn:iki:tutorial:check:player:o | urn:shacl:validate shapes=urn:iki:tutorial:check:shapes'>
<pre class="ikigai-run-expected">{
  "conforms": true,
  "violations": []
}
[2 computed]
@prefix sh: &lt;http://www.w3.org/ns/shacl#&gt; .
&lt;urn:ikigai:shacl:report:b3:1258eb4246e33211b9d99fd84328c8723a8aae4c2627e1c1973fca3041184544:report&gt; a sh:ValidationReport ;
	sh:conforms false ;
	sh:result &lt;urn:ikigai:shacl:report:b3:1258eb4246e33211b9d99fd84328c8723a8aae4c2627e1c1973fca3041184544:result.0&gt; .
&lt;urn:ikigai:shacl:report:b3:1258eb4246e33211b9d99fd84328c8723a8aae4c2627e1c1973fca3041184544:result.0&gt; a sh:ValidationResult ;
	sh:focusNode &lt;urn:iki:tutorial:check:player:o&gt; ;
	sh:resultMessage "Closed constraint not satisfied" ;
	sh:resultPath &lt;http://example.org/ttt#wins&gt; ;
	sh:resultSeverity sh:Violation ;
	sh:sourceConstraintComponent sh:ClosedConstraintComponent ;
	sh:sourceShape &lt;http://example.org/ttt#PlayerShape&gt; ;
	sh:value "3" .
[2 computed]</pre>
</div>

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

This book does bind one. It publishes a build of `ikigai-toolbox` 0.1.0 beside its pages, in
`toolbox/`, and the page's kernel binds `urn:iki:toolbox:dist:{path}` to it: a door that asks the
browser for `toolbox/{path}` when, and only when, a name needs one of its files.

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:dist}}
```

So the page is a host too. Pressing a "Check … here" button below asks this page's kernel for
`urn:toolbox:check:{iri}` with `shapes=`, which composes the toolbox page from the build's own
template. The page then does what a server's routing would do: it points the composed page's
`/toolbox/…` files at the published build, and answers its two inputs, `/toolbox/input/{iri}`,
with `urn:toolbox:input:{iri}` from the same kernel. The toolbox runs in a panel that opens in
place, and its engine checks the two Turtle faces in your tab. Nothing of the toolbox is loaded
until you press one: the first press fetches its scripts and the 8.3 MB engine, and Close (or
Escape) puts focus back on the button.

<div class="toolbox-check" data-iri='urn:iki:tutorial:check:player:o' data-shapes='urn:iki:tutorial:check:shapes' data-name='player O against the player shapes' data-faces='building/check/player-o.ttl building/check/shapes.ttl'>
<p class="toolbox-check-fallback"><a href="../toolbox/toolbox.html?from=/ikigai-tutorial/building/check/player-o.ttl&amp;from=/ikigai-tutorial/building/check/shapes.ttl#shacl" target="_blank" rel="noopener">Check player O against the player shapes in the RDF Toolbox</a>, in a new tab, on copies of the two Turtle faces published with this page.</p>
</div>

<div class="toolbox-check" data-iri='urn:iki:tutorial:check:player:x' data-shapes='urn:iki:tutorial:check:shapes' data-name='player X against the player shapes' data-faces='building/check/player-x.ttl building/check/shapes.ttl'>
<p class="toolbox-check-fallback"><a href="../toolbox/toolbox.html?from=/ikigai-tutorial/building/check/player-x.ttl&amp;from=/ikigai-tutorial/building/check/shapes.ttl#shacl" target="_blank" rel="noopener">Check player X against the player shapes in the RDF Toolbox</a>, in a new tab, on copies of the two Turtle faces published with this page.</p>
</div>

The SHACL tab of O's check shows the same finding as the cell above, worded by turtle-lsp. In
the page, you hold the root of your own kernel, so the panel reads both players. Without the
kernel (it did not load, or scripts are off), each button is a link instead, and the toolbox
opens in a new tab on copies of the Turtle faces published beside this chapter. A test holds
those copies equal to what the kernel answers a reader who holds the scope, but a copy is not a
read: nothing checks your capability on a file anyone can download.

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

- **Three places run a check here, and they are not the same.** The listings run under
  `mdbook test`, on the host side. The cells run in your tab, in the page's kernel, with
  `ikigai-shacl`'s subset. The panel runs the toolbox page and its engine in your tab, served by
  the page as its host.
- **Two engines, one question.** The page's SHACL engine is turtle-lsp's; the listings' is
  `ikigai-shacl`'s subset. Both check SHACL Core's closed shapes, but they are different code, and
  their messages differ even where their findings agree.
- **A check sees what you may read.** A reader who cannot read a resource cannot check it, and
  the refusal is the resource's, as above. That is the design, not a gap.

A caller holding some other scope checks nothing, and the refusal comes from the player, before
any shapes are read:

<div class="ikigai-run" data-cmd='cap urn:cap:tutorial:counter:write
source urn:toolbox:input:urn:iki:tutorial:check:player:o | urn:shacl:validate shapes=urn:iki:tutorial:check:shapes
cap reset'>
<pre class="ikigai-run-expected">narrowed — capability: urn:cap:tutorial:counter:write
error: denied: capability does not grant `urn:cap:tutorial:players:read` (declared by `urn:iki:tutorial:check:player:o`)
reset to identity — capability: root (full authority)</pre>
</div>
