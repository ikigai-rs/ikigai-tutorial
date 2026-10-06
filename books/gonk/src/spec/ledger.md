# Changes and tasks as ledger items, and next over them

The [previous chapter](graph.md) put the spec and the ledger in one store. This one connects
them: every open task of a change in flight becomes a ledger item **about the requirement it
implements**, `next` ranks them beside everything else in the ledger, checking a box closes its
item, and archiving the change closes the rest. The tree stays the source of truth for *what*;
the ledger is where *who, when and what next* live; and neither copies the other's job.

## Which tasks, and which items

The whole decision is two SPARQL queries over the union of the spec graphs and the ledger's own
graph — the join gonk's single store exists to make cheap:

```rust,ignore
{{#include ../../../../crates/openspec-ledger/src/lib.rs:queries}}
```

`UNFILED` is an open task of an unarchived change that nothing in the ledger is `ledger:about`
yet, with the requirement its parenthesis names *if the change adds or modifies one by that
name* — so a typo in a task files an item about the task alone rather than about the wrong
requirement. `FINISHED` is an open item about a task that is checked off, or whose change has
been archived. Acting on the rows is one ledger request each:

```rust,ignore
{{#include ../../../../crates/openspec-ledger/src/lib.rs:sync}}
```

That function is the one piece of real code in this part, and it is code for a reason: there is
no resource that turns a result set into ledger writes. Everything it *decides* is in the two
queries; everything it *does* is the ledger's `append` and `close`. And it is idempotent by
construction — a task with an item is not `UNFILED`, an item that is closed is not `FINISHED` — so
a host can run it on every commit, or on a file watcher, and a second run with nothing changed
does nothing.

⚠ **Why `about` and not a key.** The idempotence above is a query, `FILTER NOT EXISTS { ?item
ledger:about ?task }`. `ikigai-ledger` 0.4.0 adds a keyed append — file *at most one* item per
key, enforced by the ledger itself, so two hosts syncing at once cannot both file task 1.2 — and
the task's IRI would be the key. 0.4.0 was not on crates.io when this chapter was written, so the
book uses 0.3.0, and two concurrent syncs could file a task twice. One host syncing, as here, is
safe.

## The walk

A copy of the committed tree, in a scratch directory, so a box can be checked and the change
archived for real:

```rust
# extern crate openspec_ledger;
# extern crate ikigai_core;
use ikigai_core::Verb;
use openspec_ledger::{ask, kernel, lift_tree, sync_ledger, Workspace};

let tree = Workspace::new();
let host = kernel();
lift_tree(&host, &tree.tree()).unwrap();

// Three open tasks; 1.1 is already checked, so it is never filed.
let filed = sync_ledger(&host).unwrap();
assert_eq!(filed.len(), 3, "{filed:?}");
assert!(filed[0].starts_with("#1 urn:iki:ledger:default:item:") && filed[0].ends_with("add-undo 1.2"));
assert!(filed[2].ends_with("add-undo 2.1"));
assert!(sync_ledger(&host).unwrap().is_empty(), "a second sync files nothing");

// Each is about its task and the requirement it implements.
let item = ask(&host, Verb::Source, "urn:iki:ledger:item:1", &[]).unwrap();
assert!(item.contains("about:    urn:openspec:change:add-undo:task:1.2"), "{item}");
assert!(item.contains("about:    urn:openspec:spec:moves:requirement:the-last-move-can-be-undone"));

// So the ledger can be asked by requirement.
let undo = ask(&host, Verb::Source, "urn:iki:ledger:items", &[
    ("about", "urn:openspec:spec:moves:requirement:the-last-move-can-be-undone"),
]).unwrap();
assert!(undo.contains("2 item(s)"), "{undo}");

// And `next` ranks them with everything else in the ledger.
let next = ask(&host, Verb::Source, "urn:iki:ledger:next", &[]).unwrap();
assert!(next.contains("ready: 3"), "{next}");

// Check off 1.2 in tasks.md, lift, sync: its item closes, with the reason.
tree.check("add-undo", "1.2");
lift_tree(&host, &tree.tree()).unwrap();
assert_eq!(sync_ledger(&host).unwrap(), ["closed #1 (done) — checked off in tasks.md"]);

// Archive the change: its folder moves, the lift sees the new paths, the rest close.
tree.archive("add-undo", "2026-10-07");
lift_tree(&host, &tree.tree()).unwrap();
assert_eq!(sync_ledger(&host).unwrap(), [
    "closed #2 (done) — archived with add-undo on 2026-10-07",
    "closed #3 (done) — archived with add-undo on 2026-10-07",
]);
let next = ask(&host, Verb::Source, "urn:iki:ledger:next", &[]).unwrap();
assert!(next.starts_with("nothing is ready"), "{next}");
```

The labels name the change (`[add-undo openspec]`), the title carries the task's number, and the
close reason is a comment on the item, so a person reading the ledger later sees *why* it closed
without opening the tree. Nothing in the tree was written by any of this: the ledger points at the
spec by IRI, and the spec does not know the ledger exists.

## What `about` buys

Because an item names the requirement, the questions that cross the two are one query each. Which
requirement has the most open work? Which open item is about a requirement that a change has since
**removed**? Is there a requirement with scenarios and no item, no test, no change in flight? Each
is a `SELECT` over the same union `select` already asks, and none needs a new resource — which is
the point of keeping the spec, the work and the code's annotations in one store as gonk does.

## In gonk, and not in this page

gonk holds two of this chapter's three spaces — the store and the ledger — and not the lift:
`ikigai-markdown` is not in its manifest, and what is not compiled into gonk cannot be reached
through it. So the split is the natural one. A host of your own links the lift and runs
`lift_tree` and `sync_ledger` as above, with its store and ledger names pointed at gonk's socket
(as in [Over the socket](../agent/socket.md), where `urn:gk:iki:store:load`,
`urn:gk:iki:store:graph-select` and `urn:gk:iki:ledger:append` are all served to the owner).
The graphs then live in gonk's durable store beside the ledger, and gonk's own pages show the
items. (This book's checks run the in-process host above, not that arrangement.) gonk does not
run the sync itself; doing that on a watcher over a browse root's
`openspec/` directory would be a separate piece of work.

There are no Run cells in this part, on purpose. The page's kernel is the ledger over an in-memory
store, compiled to wasm, and the lift could join it — but `urn:markdown:mapping:{name}`, the name
this chapter installs the mapping under, is the one resource `ikigai-markdown` does **not** build
for wasm (there is no config home in a browser), and the tree itself would have to be baked into
the page. A page that ran a different mapping lookup over a frozen copy of the tree would be
showing something this chapter does not teach, so the Rust above is the runnable form.

## What you built

The **atoms** are the tree's files and the ledger's graph. The spec graphs are derived from the
files and the items are derived from the spec graphs, each by a composition the host did not write;
`UNFILED` and `FINISHED` are **views** that span both; `sync_ledger` is the glue that acts on
them. Close an item by hand and it stays closed; delete the tree and the items still say what they
were about, by name.
