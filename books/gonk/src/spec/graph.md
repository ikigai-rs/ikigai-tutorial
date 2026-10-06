# An openspec/ tree as a graph

The [previous chapter](openspec.md)'s tree is five Markdown files. This chapter makes it a graph
in which every requirement, scenario, change and task has a name — an IRI — and asks it
questions with SPARQL. It writes no parser to do it.

The lifting is [`ikigai-markdown`](https://github.com/ikigai-rs/ikigai-markdown), and it works in
two stages that are worth keeping apart:

1. A **generic lift** that knows Markdown and nothing else: documents, headings with their levels
   and their enclosing section, list items with their checkbox state, paragraphs, links. Every
   node is named under the document's IRI; there are no blank nodes.
2. A **mapping**, which is a *resource*, not code: a small lift profile plus SPARQL `CONSTRUCT`
   queries that turn that structure into a domain — here, OpenSpec's. The lifter has never heard
   of a requirement. Every word of OpenSpec in this chapter is in the mapping.

⚠ `ikigai-markdown` is not on crates.io yet, so this book's workspace takes it from its public
repository at one pinned git revision, the way it pins gonk. That is the only difference a reader
will notice.

## The mapping, by name

A mapping is found by name, `urn:markdown:mapping:{name}`, as a file under the config home:
`<config home>/markdown/mappings/openspec/mapping.ttl`. The one this book uses is
[`crates/openspec-ledger/config/markdown/mappings/openspec/mapping.ttl`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates/openspec-ledger/config/markdown/mappings/openspec/mapping.ttl);
copying that directory into `~/.config/ikigai/markdown/mappings/` installs it for any ikigai host
on your machine. The host in this chapter is given `config/` beside it as its config home instead,
so the book never reads yours:

```rust,ignore
{{#include ../../../../crates/openspec-ledger/src/lib.rs:space}}
```

The store and the ledger are the [first chapter](../ledger/your-own.md)'s; the lift is the third
space. The graph will live in the same store as the ledger, which is what the next chapter needs.

The mapping's **lift profile** has one entry. It turns every RFC 2119 keyword in the tree into a
match the constructs can see, one per occurrence:

```turtle
{{#include ../../../../crates/openspec-ledger/config/markdown/mappings/openspec/mapping.ttl:profile}}
{{#include ../../../../crates/openspec-ledger/config/markdown/mappings/openspec/mapping.ttl:keyword}}
```

A construct could find the *first* keyword in a paragraph with a regular expression, but not
both of "MUST refuse … and MUST NOT change". The profile is where the one-to-many part of reading
text belongs. Then the constructs, six of them. Each runs over **one document's** structural
graph, and decides what that document is from its **path** — OpenSpec's own rule. This is the
first: a capability and its requirements, from a current spec.

```turtle
{{#include ../../../../crates/openspec-ledger/config/markdown/mappings/openspec/mapping.ttl:requirements}}
```

`md:Heading`, `md:level`, `md:text` and `md:path` are the generic lift's words; `os:` is
OpenSpec's, made up for this book (`http://example.org/openspec#`, a namespace reserved for
examples). Every IRI is minted from names in the tree — `urn:openspec:spec:moves:requirement:players-alternate`
— so the same bytes always give the same graph, and a requirement named in a delta spec and the
same requirement in the current spec are one node. The others: each requirement's scenarios and
their steps; the keywords; a change, and the date it was archived if its folder is under
`archive/`; what a delta `adds`, `modifies` or `removes`; and a change's tasks:

```turtle
{{#include ../../../../crates/openspec-ledger/config/markdown/mappings/openspec/mapping.ttl:tasks}}
```

A task's checkbox is the generic lift's `md:checked`; its number, title and cited requirement
are cut from its text. What a task does **not** get here is the requirement itself: the name
`The last move can be undone` is in `tasks.md`, the requirement it names is in the delta spec,
and a construct only ever sees one document. That join is a query, below.

## Lifting one document

`urn:markdown:lift` takes the Markdown (`content`, or a resource IRI as `src`), the path, and the
mapping, and answers N-Quads: the structural graph and the mapped one, together, in a named graph
for the document.

```rust
# extern crate openspec_ledger;
# extern crate ikigai_core;
use ikigai_core::Verb;
use openspec_ledger::{ask, crate_dir, kernel, MAPPING};

let host = kernel();
let tasks = std::fs::read_to_string(
    crate_dir().join("openspec/changes/add-undo/tasks.md")).unwrap();
let quads = ask(&host, Verb::Source, "urn:markdown:lift", &[
    ("content", &tasks),
    ("path", "openspec/changes/add-undo/tasks.md"),
    ("mapping", MAPPING),
]).unwrap();

let graph = "<urn:markdown:doc:openspec/changes/add-undo/tasks.md>";
let task = "<urn:openspec:change:add-undo:task:1.2>";
let os = |p: &str| format!("<http://example.org/openspec#{p}>");
for line in [
    format!(r#"{task} {} "1.2" {graph} ."#, os("number")),
    format!(r#"{task} {} "false"^^<http://www.w3.org/2001/XMLSchema#boolean> {graph} ."#, os("done")),
    format!(r#"{task} {} "The last move can be undone" {graph} ."#, os("cites")),
    format!(r#"{task} {} <urn:openspec:change:add-undo> {graph} ."#, os("change")),
] {
    assert!(quads.lines().any(|l| l == line), "missing: {line}");
}
// The structural half is there too, under the document's own IRI.
assert!(quads.contains("<https://ikigai-rs.dev/ns/md#checked>"));
```

Task 1.2 is unchecked, numbered, part of `add-undo`, and cites a requirement by name. The lift is
cacheable — a function of its arguments and of the mapping it names — so asking again for the same
bytes, path and mapping is answered from the kernel's cache.

## The tree, in the store

A tree is many documents, so a host lifts each and loads it into the store, one named graph per
document. Archiving a change moves files, so lifting a tree also drops the graphs of documents
that are gone:

```rust,ignore
{{#include ../../../../crates/openspec-ledger/src/lib.rs:lift}}
```

and asks the union of those graphs, through the store's narrow read door for a *set* of graphs:

```rust,ignore
{{#include ../../../../crates/openspec-ledger/src/lib.rs:select}}
```

Now the questions. Which requirements are there, and how strong is each?

```rust
# extern crate openspec_ledger;
use openspec_ledger::{crate_dir, kernel, lift_tree, read_tree, select};

let host = kernel();
assert_eq!(lift_tree(&host, &read_tree(&crate_dir())).unwrap(), 5);

let strength = select(&host, r#"
    PREFIX os: <http://example.org/openspec#>
    SELECT DISTINCT ?capability ?requirement ?keyword WHERE {
      ?c a os:Capability ; os:name ?capability ; os:requirement ?r .
      ?r os:name ?requirement ; os:keyword ?keyword .
    } ORDER BY ?capability ?requirement ?keyword"#).unwrap();
assert_eq!(strength, "\
?capability\t?requirement\t?keyword
\"board\"\t\"A new game has an empty board\"\t\"SHALL\"
\"board\"\t\"Three in a line wins\"\t\"SHALL\"
\"moves\"\t\"A taken square is refused\"\t\"MUST\"
\"moves\"\t\"A taken square is refused\"\t\"MUST NOT\"
\"moves\"\t\"Players alternate\"\t\"SHALL\"
");
```

What does the change touch, and which of its tasks implements which requirement? This is the
join no construct could make, across `tasks.md` and the delta spec:

```rust
# extern crate openspec_ledger;
use openspec_ledger::{crate_dir, kernel, lift_tree, read_tree, select};

let host = kernel();
lift_tree(&host, &read_tree(&crate_dir())).unwrap();

let work = select(&host, r#"
    PREFIX os: <http://example.org/openspec#>
    SELECT DISTINCT ?number ?done ?how ?name WHERE {
      ?change os:id "add-undo" ; os:task ?t .
      ?t os:number ?number ; os:done ?done ; os:cites ?name .
      { ?change os:adds ?r BIND("adds" AS ?how) }
        UNION { ?change os:modifies ?r BIND("modifies" AS ?how) }
      ?r os:name ?name .
    } ORDER BY ?number"#).unwrap();
assert_eq!(work, "\
?number\t?done\t?how\t?name
\"1.2\"\tfalse\t\"adds\"\t\"The last move can be undone\"
\"1.3\"\tfalse\t\"adds\"\t\"The last move can be undone\"
\"2.1\"\tfalse\t\"modifies\"\t\"Players alternate\"
");
```

Two things to notice. `DISTINCT` is not decoration: a fact several documents state (that
`add-undo` is a change, say — every file in its folder says so) is in the union once per
document, so counting without `DISTINCT` over-counts. And task 1.1, which cites nothing, is not in
the answer, because the query asked for tasks that cite; nothing about it is lost, it is simply
about its change and no requirement.

## What you built

The **atoms** are the Markdown files and the mapping file; the **graph** is derived from them,
one named graph per document, and rebuilt from them whenever the host asks (`lift_tree` holds
nothing). The lift and the mapping are **compositions** the host did not write; the queries are
**views**. The code this chapter added to the book is a directory walk and two loops of requests.
