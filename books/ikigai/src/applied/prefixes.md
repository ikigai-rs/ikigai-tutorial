# Short names in a space

[A space that changes while it runs](live-space.md) put a declaration resource in front of a
dynamic space and changed the space by writing it. This chapter uses the same mechanism for a
smaller job: names. A composition that reads a dozen resources spells out a dozen full names,
`urn:json:query` and `urn:iki:tutorial:shop:stock` again and again. Short names would make it
easier to write and to read. The question is where short names may exist without changing what
any other name means.

The answer here is the one [Scope and alias](../getting-started/scope-and-alias.md) gave for
renaming: a name is resolved in a space, and an alias rule is a declaration in one. A **prefix
table** is an alias whose rules rewrite `json:` to `urn:json:` and `shop:` to
`urn:iki:tutorial:shop:`. It lives in a dynamic space, and the space is carried as a corridor
around the requests that compose. Inside, `shop:stock` is a name. Outside, there is no such name.

The data is the shop's stock from [Transreption](../building/transreption.md#a-format-that-is-not-rdf):
a CSV table, read as JSON by `urn:json:query`. The code is `crates/shop`. Nothing here needs a
server or a REPL. The live host on the page of A space that changes while it runs, if it is still open, is a different page's
kernel and changes nothing here.

## The prefix table is data

The table is an arrangement, in the s-expression form [A space that changes while it
runs](live-space.md) wrote:

```text
{{#include ../../../../crates/shop/prefixes.arrangement}}
```

Each rule is `:resolve-at root`, which that chapter's
[view](live-space.md#a-view-a-name-for-a-name-the-host-already-serves) introduced: the
rewritten name is asked again from the root of the caller's chain, under the caller's own
capability. A short name is a view of a full name, so it grants nothing. A caller who may not
read `urn:iki:tutorial:shop:stock` may not read `shop:stock` either.

The table lives at `urn:iki:tutorial:shop:prefixes`, a declaration resource of the same kind
as that chapter's (the crate reuses `live_space::declaration`): `Sink` writes an
arrangement, and `Source` answers the `ik:` Turtle core builds a space from. The space is
ikigai-fn's `prefix_space`, which is a `DynamicSpace` with a check of its own:

```rust,ignore
{{#include ../../../../crates/shop/src/lib.rs:space}}
```

The check refuses a table that could do anything but rename. It accepts one alias, of rules
that resolve from the root, over an empty space with no doors. A prefix under a scheme the
host's own names use is refused too: `urn`, `http`, `https`, `file`, `data` and the rest. The
corridor is consulted ahead of the root, so a prefix `urn:` could stand in for every name the
host has. Under the reserved schemes it never can.

## Carried as a corridor

The host starts an empty prefix space called `shop` and the loop that rebinds it, exactly as
that chapter's host did:

```rust,ignore
{{#include ../../../../crates/shop/src/lib.rs:start}}
```

What is new is how a request gets into it. The prefix space is not a layer of the root. It is
a corridor, injected around one request:

```rust,ignore
{{#include ../../../../crates/shop/src/lib.rs:read_in}}
```

A corridor rides the request it was injected for, and every sub-request that request gives
rise to, at any depth, inherits it. That matters here. A marker's `shop:stock` is not asked by
the caller: it is asked by `urn:json:query`, an endpoint that lives in the root, which asks for
its `src` while it runs. A `Level` scopes only the sub-requests of endpoints found inside it,
and `urn:json:query` is not inside anything, so a `Level` would not reach that read. The
corridor does.

In the page, the cells below that say "in the prefix space shop" run on the page's kernel with
that space carried as a corridor. The others run at the page's root. Run them in order: each
one changes what the next one reads.

## Empty, then a table

<!-- urn-gate: illustration urn:iki:space:dynamic:shop — the prefix space's name, and its
     thread's. A space and a thread, not a resource: only its `:status` door answers. -->

Until a table is written, the space holds nothing, and the summary, whose template is written
with short names, cannot be composed:

<div class="ikigai-run" data-prefixed='shop' data-cmd='source urn:iki:space:dynamic:shop:status
source urn:iki:tutorial:shop:summary'>
<pre class="ikigai-run-expected">space: urn:iki:space:dynamic:shop
declaration: urn:iki:tutorial:shop:prefixes
in force: nothing (the space is empty)
installs: 0
refused: the declaration could not be read: not found: urn:iki:tutorial:shop:prefixes
[uncacheable]
error: no endpoint resolved for shop:stock
       did you mean `urn:iki:tutorial:shop:stock`? (bound here)</pre>
</div>

Write the table, on one line, and ask again. The summary composes, and `shop:summary`, the
summary's own short name, answers the same: a name in a cell's line is resolved in the cell's
chain like any other. The last line passes a short name as a value, `src=shop:stock`, and the
query reads it in the corridor it inherited:

<div class="ikigai-run" data-prefixed='shop' data-cmd='sink urn:iki:tutorial:shop:prefixes (alias (prefix "json:" "urn:json:" :resolve-at root) (prefix "shop:" "urn:iki:tutorial:shop:" :resolve-at root) (endpoints))
source urn:iki:space:dynamic:shop:status
source urn:iki:tutorial:shop:summary
source shop:summary
source urn:json:query src=shop:stock q=".[0].item" as=text/plain'>
<pre class="ikigai-run-expected">ok
[uncacheable]
space: urn:iki:space:dynamic:shop
declaration: urn:iki:tutorial:shop:prefixes
in force: b3:24a60324ce76a9284a2c82696491fea287d9f88b1f304992fbe4dc480d4cba74
installs: 1
refused: nothing since the last bind
[uncacheable]
4 items, 49 on the shelves; out of stock: sugar.
[computed]
4 items, 49 on the shelves; out of stock: sugar.
[computed]
flour
[computed]</pre>
</div>

`shop:summary` is computed, not served from the line before: a name is a cache key, so the
short name has an answer of its own, which depends on the full name's.

## The summary: CSV, JSON, jq and `$j`

Here is the summary's template. Each `$j` is a jq selection of the stock, spliced as text
([Tic-tac-toe, V](tic-tac-toe-5.md#the-template-language) showed that `$j` is shorthand for a
`$h` of `urn:json:query`):

```text
{{#include ../../../../crates/shop/templates/summary.txt}}
```

Follow one marker down. `$j{shop:stock length}` asks `urn:json:query` (its own name, fixed) for
`src=shop:stock` and `q=length`. The query endpoint reads `shop:stock`, which the corridor
rewrites to `urn:iki:tutorial:shop:stock` and asks again from the root. That answers `text/csv`,
so the query plans the CSV transreptor, `urn:json:from-csv`, and runs `length` over the array of
records it makes. Four resources, two of them reached by short names, and none of them written
for this summary.

The same, in Rust:

```rust
# extern crate shop;
# extern crate ikigai_core;
use ikigai_core::Error;
use shop::{source, Host, PREFIXES, PREFIX_TABLE, SUMMARY};

let mut host = Host::start("shop");
host.write(PREFIXES, PREFIX_TABLE);

let summary = || source(SUMMARY, &[]);
assert_eq!(
    host.read_in(summary()).unwrap(),
    "4 items, 49 on the shelves; out of stock: sugar."
);
assert!(host.is_cached_in(&summary()));

// Outside the space, the template's short names do not exist.
assert!(matches!(host.read(summary()), Err(Error::Unresolved(_))));
assert!(matches!(host.read(source("shop:stock", &[])), Err(Error::Unresolved(_))));
```

## Outside, no such name

At the page's root, with no corridor, the same two names:

<div class="ikigai-run" data-cmd='source shop:stock
source urn:iki:tutorial:shop:summary'>
<pre class="ikigai-run-expected">error: no endpoint resolved for shop:stock
       did you mean `urn:iki:tutorial:shop:stock`? (bound here)
error: no endpoint resolved for shop:stock
       did you mean `urn:iki:tutorial:shop:stock`? (bound here)</pre>
</div>

`shop:stock` is not a name the page's kernel has. The summary is, but its template asks for
`shop:stock`, so outside the space it fails the same way. The prefix table changed what names
exist for the requests that carry it, and for nothing else on the page.

## A collection: `each`

`urn:iki:fn:each` fills one template once per element of a JSON array. The array is
`shop:records`, a template that is one `$r` of `json:query` over the stock with the program
`.`, and the row is `$h{{item}} ($h{{count}})`:

<div class="ikigai-run" data-prefixed='shop' data-cmd='source urn:iki:fn:each src=shop:records template=shop:template:row separator=", "
source tmpl:row'>
<pre class="ikigai-run-expected">flour (12), sugar (0), rice (30), lentils (7)
[computed]
error: no endpoint resolved for tmpl:row
       did you mean `urn:iki:tutorial:shop:template:row`? (bound here)</pre>
</div>

The first line fills the row once per record. The second asks for `tmpl:row`, and the table
has no `tmpl:` prefix.

## A new prefix, while it runs

Writing a new prefix is a `Sink` on the declaration. The host's loop hears the write as a cut
of the declaration's thread, rebinds the space, and the names inside it change. Nothing
restarts. Add `tmpl:` for the shop's templates, and ask again. Then ask about the summary,
which the first table composed:

<div class="ikigai-run" data-prefixed='shop' data-cmd='sink urn:iki:tutorial:shop:prefixes (alias (prefix "json:" "urn:json:" :resolve-at root) (prefix "shop:" "urn:iki:tutorial:shop:" :resolve-at root) (prefix "tmpl:" "urn:iki:tutorial:shop:template:" :resolve-at root) (endpoints))
source tmpl:row
cache urn:iki:tutorial:shop:summary
source urn:iki:tutorial:shop:summary'>
<pre class="ikigai-run-expected">ok
[uncacheable]
$h{{item}} ($h{{count}})
[computed]
not cached
4 items, 49 on the shelves; out of stock: sugar.
[computed]</pre>
</div>

The summary was cached, and the new table cut it, though the summary uses no `tmpl:` name. That
is that chapter's **coarse cut**: every answer served through the space hangs from the
space's one thread, and every successful bind cuts it. It is never wrong, only sometimes
wasteful: what was composed with the old table recomputes once.

## The data changes too

A write to the stock cuts the stock's thread, and the summary hangs from it through every
`$j`. This line keeps only what is on a shelf, the same round trip
[Transreption](../building/transreption.md#cached-and-cut-by-a-write) ran, with the stock named
short where it is a value:

<div class="ikigai-run" data-prefixed='shop' data-cmd='source urn:json:query src=shop:stock q="map(select(.count | tonumber > 0))" | urn:json:to-csv | sink urn:iki:tutorial:shop:stock
source shop:summary'>
<pre class="ikigai-run-expected">ok
[2 computed · 1 uncacheable]
3 items, 49 on the shelves; out of stock: nothing.
[computed]</pre>
</div>

## A table refused

A table that declares a prefix under a scheme the host's names use is refused before it binds.
Here it claims `file:`:

<div class="ikigai-run" data-prefixed='shop' data-cmd='sink urn:iki:tutorial:shop:prefixes (alias (prefix "file:" "urn:iki:tutorial:shop:" :resolve-at root) (endpoints))
source urn:iki:space:dynamic:shop:status
cache shop:summary
source shop:summary'>
<pre class="ikigai-run-expected">ok
[uncacheable]
space: urn:iki:space:dynamic:shop
declaration: urn:iki:tutorial:shop:prefixes
in force: b3:680caaa80e550dcaf0c1c73445e5c5afd4a8e8a6cdd326c9b034921867d3e8ac
installs: 2
refused: the host's check refused it: prefix `file:` is under the reserved scheme `file:` — the host's own names live there, and a prefix may never stand in for one
refused digest: b3:5c613c265b34d7e9f9166633782a60a66117656c2ebbe07601ba97f5b99636fd
[uncacheable]
cached
3 items, 49 on the shelves; out of stock: nothing.
[cached]</pre>
</div>

As in [A space that changes while it runs](live-space.md#a-change-refused), a refused change cuts nothing, and the table that bound last stays
in force. The status says what was refused and why.

## What it does not do

- **The cut is coarse**, as in that chapter (ledger #2034): any new table recomputes
  everything composed through the space.
- **The reserved schemes are the whole guarantee.** A host that serves names under a scheme of
  its own (`page:`, say) and also declares a prefix `page:` would have that prefix stand in for
  its names inside the corridor. Core has no placement for "answer only if the root misses", so
  ikigai-fn refuses the schemes every ikigai host uses, and every ikigai host's names are `urn:`.
- **A short name is not a line's target when it takes arguments.** Inside a composition every
  short name works, arguments and all: a marker's `json:query?src=shop:stock&q=.` builds its
  request directly. A REPL line is different. The engine recognizes `key=value` by the target's
  contract, which it reads from the target's `Meta`, and the `Meta` of a short name describes
  the forwarding endpoint the rule binds, which declares no inputs. So `source json:query
  src=shop:stock q=.` sends `src=…` as positional text and is refused for a missing `q`. The
  cells above name the endpoint in full and pass short names as values.
- **A sandbox bound in the page is an abort.** jq runs metered and bounded. Natively a bound
  reached is a typed refusal. In this page, WebAssembly cannot unwind, so it traps the page's
  kernel until a reload. The cells here are far inside the bounds; keep an edited program small.

## What was built

| what | where | what it is |
|---|---|---|
| the stock | `urn:iki:tutorial:shop:stock` | an atom: CSV, written by a `Sink`, cached and cut |
| the CSV as JSON | `urn:json:query`, `urn:json:from-csv` | ikigai-json: jq over the stock, through the CSV transreptor |
| the prefix table | `urn:iki:tutorial:shop:prefixes` | data: an alias of root-resolving prefix rules, written by a `Sink` |
| the prefix space | `urn:iki:space:dynamic:shop` | ikigai-fn's `prefix_space`: the table, bound, and checked |
| the corridor | `Kernel::issue_in` | the space carried by a request and every sub-request it makes |
| the summary | `urn:iki:tutorial:shop:summary` | a template of three `$j` markers, filled |
| the list | `urn:iki:fn:each` | a row template filled once per record |

The only code is the two atoms. The CSV reader, the query language, the template language, the
iteration and the short names are published resources, composed. A prefix is not a feature of
the template language: it is a name, declared in a space, and the space decides where it exists.
