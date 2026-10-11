# Who a script runs as

A published script is code one person wrote and another person runs, on a host neither of them
operates. So every run has to answer a question an ordinary endpoint never raises: **whose
authority is this?** The runner's, because they asked? The publisher's, because they wrote it? The
host's, because it is running? Each answer alone is a hole. The runner's alone lets a publisher
plant code that does, with a powerful runner's grants, what the publisher could never do. The
publisher's alone lets anyone who can run a script borrow its author's authority. The host's alone
is root.

`ikigai-script` takes all three, and keeps the least. A run holds the RUNNER's capability, narrowed
to

```text
{ declared } ∩ { what the publisher held at publish } ∩ { the host's ceiling for this script }
  + every exclusion the publisher or the ceiling carries
```

and never more, because the only operation the library uses to make a run's authority is
`Invocation::issue_attenuated`, and attenuation can only narrow. A run cannot hold anything its
runner does not, so the runner's own capability is the fourth term, implicit in all of it.

For a SPARQL script the first term is not written by anybody. It is **derived** from the text:
each graph a query names is `urn:cap:store:read:graph:<iri>`, and the one graph an update writes is
`urn:cap:store:write:graph:<iri>`. So the author cannot understate what a query needs, and cannot
overstate it either.

This chapter runs every case it describes. The host is [the previous
chapter's](scripts.md), with the same items, under a ceiling policy of its own:

<!-- transcript: manual — every section at once; each section's block below is checked. -->
```bash
cargo run -p script-authority
```

The program runs all five sections every time; each block below prints one of them.

## The people

```rust,ignore
{{#include ../../../../crates/script-authority/src/lib.rs:people}}
```

<!-- urn-gate: illustration urn:iki:tutorial:person:bob — a principal's name, minted into a
     capability by a host's door. -->
<!-- urn-gate: illustration urn:iki:tutorial:payroll — the name of a named GRAPH in the store,
     taken by the store's doors as an argument; no space binds it. -->

Alice writes scripts and reads only the items. Bob runs them, and reads more than Alice does: the
payroll graph too. Carol reads nothing. All three are capabilities a host's door would mint, each
naming its person as the principal.

## The host's ceilings

The ceiling is the host's word on each script, and it is written where the host's configuration
lives, one file per script under `<config home>/script-authority/{name}`: one `urn:cap:` scope per
line, `#` comments, a trailing `*` for a family of grants, a lone `*` for no ceiling at all. The
crate keeps those files in memory, where the program can edit one, and hands the script host a
policy that reads them:

```rust,ignore
{{#include ../../../../crates/script-authority/src/lib.rs:ceilings}}
```

A script the host has no file for gets `Ceiling::nothing()`: it may touch nothing, so it cannot
run. That is a choice the host makes in this closure, and the library's own example makes it too.
Failing open would mean any script somebody managed to publish ran with whatever its publisher and
its runner happened to hold.

## A publish is checked against its publisher

```bash
cargo run -p script-authority -- 1
```

```text
carol> sink urn:script:salaries language=sparql   (a query over the payroll graph)
error: denied: a script cannot be given more than its publisher holds, and this capability does not hold urn:cap:store:read:graph:urn:iki:tutorial:payroll — declare less, or publish under a capability that holds it

alice> sink urn:script:open-items language=sparql requires=urn:cap:store:read:graph:urn:iki:tutorial:payroll
error: invalid argument `requires`: a SPARQL script's authority is derived from the graphs its text names, not declared, and this one derives urn:cap:store:read:graph:urn:iki:tutorial:items; the `requires` given says urn:cap:store:read:graph:urn:iki:tutorial:payroll. Omit `requires`

alice> sink urn:script:open-items language=sparql
urn:script:open-items:version:sha256:e1977365…
```

Carol may write a script called `salaries`, and the query names a graph she cannot read. Storing it
anyway and narrowing at run time would store a script whose declared authority says one thing and
whose runs get another, and the action manifold would offer it on the strength of the first. So the
publish is refused, naming what she does not hold. **No elevation** starts here: a script never
holds, even on paper, more than the person who published it.

Alice's first attempt declares a graph her query does not read. For Lisp, `requires=` is how a
script says what it needs. For SPARQL the text already says it, so a `requires` that disagrees is
refused rather than believed. What is stored with the version is the derived set, intersected with
what Alice held when she published: her grant, taken as a snapshot, so revoking something from her
later does not reach back into a version she already published, and granting her something later
does not widen it.

## A run is checked against its runner

```bash
cargo run -p script-authority -- 2
```

```text
alice> source urn:script:open-items:result as=text/tab-separated-values
error: denied: capability does not grant `urn:cap:script:run:open-*` (declared by `urn:script:open-items:result`)

carol> source urn:script:open-items:result as=text/tab-separated-values
error: denied: capability does not grant `urn:cap:store:read:graph:urn:iki:tutorial:items` (declared by `urn:script:open-items:result`)

bob> source urn:script:open-items:result as=text/tab-separated-values
?number	?title
1	"Write the README"
2	"Fix the flaky test"
4	"Answer the issue"
```

Alice published `open-items` and may not run it. Publishing and running are separate grants, and
she holds no run grant at all. The scope the refusal names is worth a second look: not
`urn:cap:script:run:open-items` but `urn:cap:script:run:open-*`. A published query is its own
endpoint, and the kernel checks its declared grant before the endpoint runs, from a description
that cannot depend on who asks. So a name with a hyphen in it declares its top-level namespace as
the gate (`open-items` is in the namespace `open`), and the exact grant is checked inside. Here
that reads oddly, because nobody meant `open` as a namespace; [the last section](#a-namespace-grant-and-an-exclusion-that-wins-over-it)
is where namespaces are meant.

Carol holds the run grant, and the run is refused anyway: the query reads the items, and Carol
cannot. The kernel's floor refused it before the script host ran anything, from the same
description, because a published query's own entry declares the graph grants its text derives. A
run is the runner's capability, narrowed; Carol's has nothing in it to narrow to.

Bob holds the run grant and the items, and gets the answer.

## A run never holds more than its publisher held

This query names no graph. `GRAPH ?g` ranges over whatever graphs the run may read, so it counts
the triples in each, which makes it a probe of exactly that:

```rust,ignore
{{#include ../../../../crates/script-authority/src/lib.rs:graphs}}
```

```bash
cargo run -p script-authority -- 3
```

```text
alice> sink urn:script:graphs language=sparql   (GRAPH ?g: every graph the run may read)
urn:script:graphs:version:sha256:646ac869…

bob> source urn:script:graphs:result as=text/tab-separated-values
?g	?triples
<urn:iki:tutorial:items>	20

bob> sink urn:script:graphs:runs   (the same query, run for the record)
urn:script:graphs:run:1

bob> source urn:script:graphs:run:1
urn:script:graphs:run:1
  script:     urn:script:graphs
  version:    sha256:646ac869…
  principal:  urn:iki:tutorial:person:bob
  started:    2026-10-11T09:00:00.000Z
  ended:      2026-10-11T09:00:00.000Z
  outcome:    ok
  capability: urn:cap:script:write:-team-payroll urn:cap:store:read:graph:urn:iki:tutorial:items
  result:
{"head":{"vars":["g","triples"]},"results":{"bindings":[{"g":{"type":"uri","value":"urn:iki:tutorial:items"},"triples":{"type":"literal","value":"20","datatype":"http://www.w3.org/2001/XMLSchema#integer"}}]}}
```

Bob can read the payroll graph. Alice cannot, and Alice published this. So a run of it, by Bob,
sees one graph. A query reads the caller's readable union **and never more**: the graph grants the
runner holds, that the publisher held, and that the ceiling allows, computed again on every run.
Bob cannot borrow Alice's authority, and Alice's script cannot borrow Bob's.

A query can run through `…:runs` too, when a run should be recorded, and the record shows exactly
what the run held. Two scopes, and one of them is not Bob's: `urn:cap:script:write:-team-payroll` is
an **exclusion** Alice carries (a grant with a `-` taking something back out), and it rides along on
every run of every script she publishes, whoever runs it. Exclusions travel because a publisher who
could not touch something must not be able to publish code that can. This one has nothing to do
with counting triples, and it is carried anyway, because the library cannot know which of a
publisher's exclusions matter to a script, and keeping one that does not costs nothing.

## Nor more than the host's ceiling allows

The same query, published by the host itself. Root holds every graph, so the publisher's half no
longer narrows anything, and the ceiling is all that is left between Bob and the payroll:

```bash
cargo run -p script-authority -- 4
```

```text
host> sink urn:script:every-graph language=sparql   (the same query, published by the host)
urn:script:every-graph:version:sha256:646ac869…

bob> source urn:script:every-graph:result as=text/tab-separated-values
?g	?triples
<urn:iki:tutorial:items>	20
<urn:iki:tutorial:payroll>	2

host> (writes script-authority/every-graph: the items graph alone)

bob> source urn:script:every-graph:result as=text/tab-separated-values
?g	?triples
<urn:iki:tutorial:items>	20
<urn:iki:tutorial:payroll>	2

host> sink urn:kernel:cut thread=urn:script:every-graph:authority
cut urn:script:every-graph:authority

bob> source urn:script:every-graph:result as=text/tab-separated-values
?g	?triples
<urn:iki:tutorial:items>	20

host> sink urn:script:orphan language=sparql   (no ceiling file names it)
urn:script:orphan:version:sha256:646ac869…

bob> source urn:script:orphan:result as=text/tab-separated-values
error: denied: urn:script:orphan reads the caller's readable graphs, and this run may read none: it needs `urn:cap:store:read:graph:` grants the publisher held, the host allows and the runner holds
```

The ceiling file for `every-graph` starts as the family `urn:cap:store:read:graph:*`, so Bob's run
reads both graphs. Then the host narrows the file to the items alone:

```rust,ignore
{{#include ../../../../crates/script-authority/src/bin/authority.rs:narrow}}
```

**And Bob still sees the payroll.** The ceiling is asked on every run, but nothing ran: the answer
was cached, under Bob's capability, from before the edit. The library cannot see a file change,
and the kernel cannot know that a closure's answer moved. Every result hangs from a thread named
for exactly this, `urn:script:{name}:authority`, and cutting it is the host's half of the contract:

<!-- urn-gate: illustration urn:script:{name}:authority — a golden thread's name, which every
     result hangs from; a thread, not a resource, so nothing binds the template. -->

```rust,ignore
{{#include ../../../../crates/script-authority/src/bin/authority.rs:cut}}
```

A host that reads its ceilings from files puts a watcher on that directory and makes the same cut
when a file changes; this one, which edits the file itself, makes it by hand. After the cut, the
next read runs, the policy answers the new ceiling, and the payroll is gone.

`orphan` has no file at all, so the policy answers `Ceiling::nothing()`, and the run is refused
before the store is asked.

## A namespace grant, and an exclusion that wins over it

Granting one script at a time does not scale to a team. A grant can name a **namespace** instead:
`urn:cap:script:{act}:{namespace}-*` covers every script whose name begins `{namespace}-`. Alice
holds `urn:cap:script:write:team-*`, and the exclusion `urn:cap:script:write:-team-payroll`:

```bash
cargo run -p script-authority -- 5
```

```text
alice> sink urn:script:team-open language=sparql
urn:script:team-open:version:sha256:e1977365…

alice> sink urn:script:team-a-open language=sparql
urn:script:team-a-open:version:sha256:e1977365…

alice> sink urn:script:teammate language=sparql
error: denied: publishing a script needs `urn:cap:script:write:teammate`, which this capability does not hold. A script grant names one script (`urn:cap:script:{act}:{name}`) or a namespace of them (`urn:cap:script:{act}:{namespace}-*`), and an exclusion (`…:-{name}`) takes one back out

alice> sink urn:script:team-payroll language=sparql
error: denied: publishing a script needs `urn:cap:script:write:team-payroll`, which this capability does not hold. A script grant names one script (`urn:cap:script:{act}:{name}`) or a namespace of them (`urn:cap:script:{act}:{namespace}-*`), and an exclusion (`…:-{name}`) takes one back out

bob> source urn:script:team-a-open:result as=text/tab-separated-values priority=1
?number	?title
1	"Write the README"
2	"Fix the flaky test"
```

`team-open` and `team-a-open` are in the namespace (namespaces nest: `team-a` is one inside
`team`). `teammate` is not: the wildcard comes only after a `-`, so `team-*` is not a prefix match
on the letters. `team-payroll` is in it, and the exclusion wins, over a namespace grant and over an
exact one alike.

The act stays spelled out in every grant (`write`, `run`, `read`, `delete`), so a namespace grant
to run never publishes, and the wildcard comes only at the end, because the kernel has no infix
wildcard to put anywhere else. There is no "every script" grant below root at all:
`urn:cap:script:write:*` is spelled exactly like the family the script door declares ("holds some
write grant"), and a string that meant "everything" in one reading and "something" in the other is
the confusion a grant must not carry, so it grants nothing.

Bob's `urn:cap:script:run:team-*` runs `team-a-open`, and its ceiling file allows the items, which
Alice held when she published it.

## The rule, once more

| the term | who decides | when | what this chapter showed |
|---|---|---|---|
| declared | the text, for SPARQL | at publish | a `requires` that disagrees is refused |
| the publisher's grant | the publisher's capability | at publish, kept as a snapshot | Carol cannot publish what she cannot read; Bob's run of Alice's query sees Alice's graphs |
| the ceiling | the host, per script | on every run (cut the authority thread when it changes) | the payroll leaves only after the cut; no file, no run |
| the runner's capability | the door that minted it | on every run | Alice cannot run what she wrote; Carol's run has nothing to narrow to |
| exclusions | the publisher and the ceiling | travel with every run | Alice's `-team-payroll` is on Bob's run of `graphs` |

The script host wrote none of this as a second permission system. Every refusal above is either the
kernel's floor, checking what a description declares, or `Capability::attenuate`, which keeps a
grant only when the runner holds it. The library's part is to compute the request it hands to
attenuation, and to refuse a publish that would make that request a lie.
