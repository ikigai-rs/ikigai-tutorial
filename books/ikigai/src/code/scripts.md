# Scripts are resources

[A spreadsheet, II](../applied/spreadsheet-2.md) made a formula code as a resource: compiled by
a transreptor, cached like any other answer, and compiled again exactly when its text changed,
because the compiled form hung from the input's golden thread and nothing else. That code still
lived inside the sheet. Somebody wrote the formula language in Rust, and a cell could only say
what that language could.

This section takes the next step: code that **anybody the host trusts can publish into a running
host, at a name**, without a release, and that everybody the host lets in can run. It is the
pattern NetKernel called a scripted endpoint, and the claim of this section is the same as the
claim of the spreadsheet's: a script is not a feature beside the resources. It IS one,
`urn:script:{name}`, with a family of names around it, and every door a host has (the REPL,
HTTP, an agent's tool list, a timer) reaches it the way it reaches anything else. Running a
script is a read or a write, so the cache, the golden threads and the capabilities already
apply to it, and none of them is written again.

Three chapters. This one publishes a query and runs it. [Who a script runs
as](authority.md) asks under whose authority it runs, which is the question a published piece of
code always raises. [A sentence becomes a draft](sentence.md) lets a language model write one, and
shows how little the model is allowed to do.

## The names

The host library is [`ikigai-script`](https://crates.io/crates/ikigai-script). These are the
names it binds that this chapter uses:

| the name | verbs | what it is |
|---|---|---|
| `urn:script:{name}` | Source, Sink, Delete | the script: fetch it WITHOUT running it, publish it, retire it |
| `urn:script:{name}:version:{digest}` | Source | one immutable version, named by its content |
| `urn:script:{name}:result` | Source | run it as a READ: its answer |
| `urn:script:{name}:runs` | Sink | run it for its EFFECTS: answers the run's name |
| `urn:script:{name}:run:{id}` | Source | one recorded run |

Two of these hold state, and the rest are compositions. The script (its head, and every version
it has had) is an atom: a `Sink` writes it and nothing else does. A run is an atom: `…:runs`
writes one and nothing writes it again. A result holds nothing: it is the script's text, run
through the store, under somebody's authority.

A script may be written in three languages: Lisp, a plan, or SPARQL. This section uses SPARQL
only, and not just because the book's data is a graph: a query's authority can be read off its
text, and the next chapter depends on that. `ikigai-script`'s own advice, when assembling
anything, is "a query, then a plan, then Lisp", for the same reason.

## A host of your own, again

[Binding, and a host of your own](../getting-started/binding.md) composed a kernel from spaces
somebody else wrote. This is the same move. `crates/script-host` writes no endpoint at all: it
configures the script host, puts it in front of an in-memory store, and that is the host.

```rust,ignore
{{#include ../../../../crates/script-host/src/lib.rs:space}}
```

The configuration is the three things only a host can decide, and the library refuses to guess
any of them:

- **Where scripts live.** A `Backend`. `MemoryBackend` here, because a book's kernel keeps nothing
  past the process; `DirBackend` keeps them as plain files under a directory the host chooses.
- **The most each script may touch.** A ceiling policy, asked for by script name on every run.
  This chapter's is `Ceiling::unbounded()` for every script, and [the next
  chapter](authority.md) replaces it.
- **Which store a query runs against.** `SparqlDoor::store()` names the store's graph-scoped doors
  under `urn:iki:store:`. Without a door, `language=sparql` is refused.

The kernel around it has one more thing to wire:

```rust,ignore
{{#include ../../../../crates/script-host/src/lib.rs:kernel}}
```

A published query becomes its own entry in the host's catalog, with its parameters as real
arguments, and the kernel caches every description under `urn:kernel:bindings`. The library holds
no kernel, so it cannot cut that thread itself; it tells the host through `on_change`, and the
host, which holds both, makes the cut. A kernel that does not exist yet cannot be handed to the
space it is built from, so the hook reaches it through a cell filled once it does.

The fourth decision is **who is asking**, and it is not configuration. A host's door (the HTTP
door, the socket, the REPL) mints the caller's identity into the capability the request runs
under, and the script host reads it from there:

```rust,ignore
{{#include ../../../../crates/script-host/src/lib.rs:alice}}
```

<!-- urn-gate: illustration urn:iki:tutorial:person:alice — a principal's name, minted into a
     capability by a host's door; a name for a person, not a resource anything binds. -->
<!-- urn-gate: illustration urn:iki:tutorial:items — the name of a named GRAPH in the store, not
     a resource a space binds: the store's doors take it as an argument. -->

`with_principal` puts `urn:iki:tutorial:person:alice` into the capability as
`urn:cap:principal:…`. It is never an argument, so no caller can name itself, and narrowing a
capability never adds or changes one.

The data is four work items, in the vocabulary gonk's work ledger keeps them in, in one named
graph, `urn:iki:tutorial:items`; and two salaries in a second graph, which the next chapter keeps
out of reach:

```rust,ignore
{{#include ../../../../crates/script-host/src/lib.rs:seed}}
```

## Publish a query, and fetch it without running it

The query lists the open items at or above a priority, where 0 is the highest:

```rust,ignore
{{#include ../../../../crates/script-host/src/lib.rs:open_items}}
```

The `# @param` line, in the comment block before the first token, declares a parameter: the query
variable it binds, its type, and a default. It is the only place a parameter can be declared, and
it becomes an argument of the script's own `…:result`, so it shows up in the catalog, in an agent's
tool list and in the Emacs aliases like any endpoint's argument. A value is bound into the PARSED
query as an RDF term, never spliced into the text, so a value that tries to close a string or open
a second pattern arrives as one literal.

Publishing is a `Sink` on the script's name. The program prints each request the way the REPL
would write it, and runs the whole chapter every time; the number picks the section it prints:

```bash
cargo run -p script-host -- 1
```

```text
alice> sink urn:script:open-items language=sparql   (the query, piped)
urn:script:open-items:version:sha256:e1977365…

alice> source urn:script:open-items
# The open items at or above a priority, by number.
# @param priority xsd:integer default 3 -- the lowest priority to list (0 is highest)
…   (11 lines, as published)
```

The answer to a publish is the name of the version it made, and the version is named by its
content: `sha256:` and the hex digest of its language, its declared authority and its text. The
same text published twice is the same version; one more byte is another.

A `Source` of the script's own name answers its text, exactly as it went in. Fetching a script and
running it are different requests on different names, and they need different grants: reading a
script's source is `urn:cap:script:read:{name}`, running it is `urn:cap:script:run:{name}`.
Source code is not the same sensitivity as what it computes, and the two are never one grant.

## Running is a read

`urn:script:open-items:result` is the query's answer. Asked for twice, then with a parameter:

```bash
cargo run -p script-host -- 2
```

```text
alice> source urn:script:open-items:result as=text/tab-separated-values
?number	?title
1	"Write the README"
2	"Fix the flaky test"
4	"Answer the issue"
[computed]

alice> source urn:script:open-items:result as=text/tab-separated-values
?number	?title
1	"Write the README"
2	"Fix the flaky test"
4	"Answer the issue"
[cached]

alice> source urn:script:open-items:result as=text/tab-separated-values priority=1
?number	?title
1	"Write the README"
2	"Fix the flaky test"
[computed]

host> dependents urn:iki:store:graph-update
dependents of urn:iki:store:graph-update  (cut 0 times)
  4 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:iki:store:graph-select    text/tab-separated-values       59 B  4 threads   root
  urn:iki:store:graph-select    text/tab-separated-values       80 B  4 threads   root
  urn:script:open-items:result  text/tab-separated-values       59 B  8 threads   root
  urn:script:open-items:result  text/tab-separated-values       80 B  8 threads   root
```

`[computed]` and `[cached]` are the program asking the kernel, before each read, whether it would
answer from its cache (`Kernel::is_cached`, with the same request and the same capability). The
second read ran nothing. The third is another request, because a parameter is part of what is
asked, so it is another answer and another cache entry.

The last request is the question [Golden threads in
practice](../getting-started/golden-threads.md) asked before a cut: what hangs from this thread?
The host asks it of `urn:iki:store:graph-update`, the thread the store cuts on every write to a
named graph, and four answers hang from it: the store's two answers, and the script's two, which
were computed from them. Nobody wrote "recompute the result when the items change". The result
read the store through the kernel, so the kernel recorded that it did, and the store's answers
carry the store's threads. A script's result is cacheable exactly as far as what it read was.

## Running is a write

An update runs for its effects, so it is not a read at all: its `…:result` is refused, naming
`…:runs`. This one closes an item by number:

```rust,ignore
{{#include ../../../../crates/script-host/src/lib.rs:close_item}}
```

```bash
cargo run -p script-host -- 3
```

```text
alice> sink urn:script:close-item language=sparql   (the update, piped)
urn:script:close-item:version:sha256:f4d4d8e1…

alice> sink urn:script:close-item:runs number=2
urn:script:close-item:run:1

alice> source urn:script:close-item:run:1
urn:script:close-item:run:1
  script:     urn:script:close-item
  version:    sha256:f4d4d8e1…
  principal:  urn:iki:tutorial:person:alice
  started:    2026-10-11T09:00:00.000Z
  ended:      2026-10-11T09:00:00.000Z
  outcome:    ok
  capability: urn:cap:store:read:graph:urn:iki:tutorial:items urn:cap:store:write:graph:urn:iki:tutorial:items
  result:
updated <urn:iki:tutorial:items>: +1 -1 quads

alice> source urn:script:open-items:result as=text/tab-separated-values
?number	?title
1	"Write the README"
4	"Answer the issue"
[computed]
```

A `Sink` on `…:runs` runs the script and answers the name of the run, and the run is an atom of its
own: `run:1`, then `run:2`, never rewritten. Its record says who ran it (the principal the
caller's capability named, never an argument), which version, when it started and ended, how it
ended, and exactly what authority it ran under. A failed run is recorded too, with its error's
type, so "what did this script do last night, and as whom?" is a read.

The run's capability is not Alice's. It is the two grants the update's text derives (write the
graph it writes, and read it, because it has a `WHERE`), and nothing else Alice holds. Where that
comes from is [the next chapter](authority.md).

The last read is the payoff of the one before it. The update wrote the items graph, the store cut
its graph-write thread, and the cached result hung from it: so the next read recomputed, and item 2
is gone. The clock in these records is fixed (`FixedClock`), so the program prints the same
instants on every machine.

## Versions are content

```bash
cargo run -p script-host -- 4
```

```text
alice> sink urn:script:open-items language=sparql   (the same query, ORDER BY DESC(?number))
urn:script:open-items:version:sha256:6a7fcb92…

alice> source urn:script:open-items:result as=text/tab-separated-values
?number	?title
4	"Answer the issue"
1	"Write the README"
[computed]

alice> source urn:script:open-items:version:sha256:e1977365…
…
} ORDER BY ?number
```

Republishing moved the head to a new version, and cut the script's own thread, so the result was
computed again from the new text. The first version is still there, by its digest, and still says
what it said: a version is never edited, only superseded. There is no "roll back" in this version
of the library. Publishing the old text again is the same thing, because it makes the same digest.

## What was built

| the set | the name | what it is |
|---|---|---|
| the host | `script_host::kernel_with` | composition: the script host, configured, in front of the store; a hook that cuts the bindings thread |
| a script | `urn:script:{name}` | an atom, the library's: a head and every version, written only by a `Sink` here |
| a version | `urn:script:{name}:version:{digest}` | an atom, never rewritten, named by its content |
| a result | `urn:script:{name}:result` | composition: the head's text, run through the store's doors, cached as far as what it read |
| a run | `urn:script:{name}:run:{id}` | an atom, written once by a `Sink` on `…:runs` |
| the data | `urn:iki:tutorial:items` | a named graph in the store, written by the host and by `close-item` |

This crate wrote no endpoint. Everything it does is configuration of two libraries and a kernel,
which is the point: publishing code into a running host did not need a feature beside the
resources, because the resources already had everything it needed.
