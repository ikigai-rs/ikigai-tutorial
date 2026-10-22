# A sentence becomes a draft

The chapters before this one published code a person wrote, and [Your first
script](first.md) showed a stand-in model drafting one in passing. This one lets a language model
write it, and the interesting part is not the model. It is how little the model is allowed to do.

[`ikigai-nl`](https://crates.io/crates/ikigai-nl) turns a sentence into a SPARQL query with the
model as a **drafter, never an actor**. Every step is a resource read or write under the CALLER's
capability, never more: what the model is told is what the caller could read for themselves, what
it writes is checked mechanically before anybody sees it, and the most its text can ever become is
a DRAFT script, visible to nobody but the person who asked, until that person reads it and
publishes it. The model runs nothing, saves nothing under its own name, and holds no grant at all.

The steps, each a resource:

| the step | the name | what it does |
|---|---|---|
| ground | `urn:nl:grounding` | what the caller may do and see, as data: actions, vocabulary, each readable graph's shape, worked examples |
| draft | `urn:nl:prompt:sparql`, then the host's `urn:llm:ask` | the ask and the grounding, filled into a prompt that is itself a resource, sent to the model door |
| check | `urn:nl:sparql:check` | parse, derive the authority, check every predicate and class, and a dry run |
| repair | `urn:nl:prompt:sparql-repair` | a refused draft, or one whose preview is empty, goes back with what the check found in it, up to a bound |
| save | `urn:script:{name}` with `state=draft` | a `Sink` under the caller, so the script host checks it once more |

`urn:nl:sparql` runs the five in order. Every one is also a name a person can ask directly, which
is how this chapter shows them.

## The host, and a model that is not one

`crates/sentence-draft` mounts `ikigai-nl` on the host from [Scripts are resources](scripts.md) (the script
host and the store, with the same items), beside the vocabulary the grounding cites and a model
door:

```rust,ignore
{{#include ../../../../crates/sentence-draft/src/lib.rs:host}}
```

The model door, in every gate this book runs, is a **stub**: an endpoint at `urn:llm:ask` that
speaks `ikigai-llm`'s answer format and answers each ask with a canned draft per attempt. It is the
same shape as the stub `ikigai-nl`'s own tests use, and for the same reason: what is under test is
the funnel around the model, which is deterministic, and never the model, which is not. Nothing in
a gate reaches a network. The stub declares the same grant the real door does
(`urn:cap:net:*`, "holds some network grant"), so a caller who could not reach a model cannot reach
this one either. Its drafts are written down where a test can read them:

```rust,ignore
{{#include ../../../../crates/sentence-draft/src/lib.rs:drafts}}
```

The people are the previous chapter's, holding what drafting needs:

```rust,ignore
{{#include ../../../../crates/sentence-draft/src/lib.rs:people}}
```

## What the model may know is what the caller may see

```bash
cargo run -p sentence-draft -- 1
```

```text
alice> source urn:nl:grounding as=application/json   (summarized)
actions     26 of 26
vocabulary  229 of 229
graphs      1 of 1
  urn:iki:tutorial:items  20 triples, 5 predicates
    dcterms:title    literal xsd:string
    rdf:type         iri
    ledger:number    literal xsd:integer
    ledger:priority  literal xsd:integer
    ledger:status    iri: ledger:open ledger:closed
examples    1 of 1
  open-items (sparql, sha256:e1977365…)

carol> source urn:nl:grounding as=application/json   (summarized)
actions     8 of 8
vocabulary  229 of 229
graphs      0 of 0
examples    0 of 0
```

The grounding is a **composition**: it holds no state, and every part of it is a read through the
kernel under the caller's capability. The actions are the caller's own manifold (`urn:kernel:actions`,
the tool list an agent would get), with each action's arguments. The vocabulary is the `ik:` terms.
The graphs are the store's own listing of what the caller may read, each summarized by a few stored
queries (its classes and predicates, with counts, and a few sample triples). Each predicate also
says what KIND of value it takes, an IRI or a literal of some datatype, and, when its values are a
small closed set, what they are: `ledger:status` is the IRI `ledger:open` or `ledger:closed`, never
the string `"open"`. That line exists because of the draft in section 4. The examples are published
scripts the caller may read: here the host names one, `open-items` from [Scripts are resources](scripts.md), and the
grounding shows it to Alice and not to Carol, by asking the script host's own rule.

So Carol's grounding offers eight actions and no data at all, and the payroll graph is not in
Alice's, because she cannot read it. (It would be in Bob's, because he can.) A graph the caller
cannot read contributes nothing, **not even its name**, and a model cannot leak what it was never
shown. (The actions are in the grounding for a drafter that writes calls. One that writes a query
reads only the vocabulary, the graphs and the examples, with `parts=`: a query is not an action
call, and on a host with thousands of actions they are most of the grounding's size.)

The grounding is cached per capability, and hangs from every thread its parts hang from, so a write
to a graph it summarized, a change of bindings, or a republished example recomputes it. Its parts
each carry a `sha256` identity, and the grounding is named by the digest of its parts, so a draft can
record exactly which grounding it was drafted from.

## The check

`urn:nl:sparql:check` is the mechanical half, and it is a resource of its own so that a person
editing a draft can ask the same question the drafter asks:

```bash
cargo run -p sentence-draft -- 2
```

```text
alice> source urn:nl:sparql:check focus=urn:iki:tutorial:items   (a query naming ledger:state, piped)
invalid select over <urn:iki:tutorial:items>
requires: urn:cap:store:read:graph:urn:iki:tutorial:items
error: the predicate <https://ikigai-rs.dev/ns/ledger#state> is used by none of the graphs this query reads and is not in the vocabulary

alice> source urn:nl:sparql:check   (a query over the payroll graph, piped)
invalid select over <urn:iki:tutorial:payroll>
requires: urn:cap:store:read:graph:urn:iki:tutorial:payroll
error: the query reads <urn:iki:tutorial:payroll>, which is not a graph this caller may read: read only the graphs the grounding lists
warning: the predicate <urn:iki:tutorial:salary> is used by none of the graphs this query reads and is not in the vocabulary, as far as the grounding shows (it lists the first predicates of a graph, not all)

alice> source urn:nl:sparql:check   (the same query over a graph that does not exist, piped)
invalid select over <urn:iki:tutorial:nowhere>
requires: urn:cap:store:read:graph:urn:iki:tutorial:nowhere
error: the query reads <urn:iki:tutorial:nowhere>, which is not a graph this caller may read: read only the graphs the grounding lists
warning: the predicate <urn:iki:tutorial:salary> is used by none of the graphs this query reads and is not in the vocabulary, as far as the grounding shows (it lists the first predicates of a graph, not all)
```

<!-- urn-gate: illustration urn:iki:tutorial:nowhere — a graph name nothing holds, to show that a
     graph the caller cannot read is refused in the same words as one that does not exist. -->
<!-- urn-gate: illustration urn:iki:tutorial:salary — a predicate in the payroll graph's
     triples, not a resource. -->

The check parses the draft with the script host's own analysis, so it reads the form, the
parameters and the dataset exactly as the `Sink` that saves it will. An update is refused: a draft
reads. It derives the authority the script host would derive, and compares it with the caller's.
It checks every predicate and every class the query names against the graphs it reads and the
vocabulary. And it runs the query, with its parameters' defaults bound, under a small `LIMIT`,
through the store's graph-scoped door: the dry run, whose rows are the preview.

The second and third answers are the same answer. A graph the caller cannot read is refused **in
the same words** as a graph that does not exist, so neither the caller nor the model learns which
it was. The warning about `salary` is the same in both for the same reason.

Every finding is a value in the answer, never a refusal, so the check caches like any read: the
spreadsheet's rule that a refusal is never cached, applied to a checker.

## A sentence, drafted, checked, repaired and saved

```rust,ignore
{{#include ../../../../crates/sentence-draft/src/bin/draft.rs:draft}}
```

```bash
cargo run -p sentence-draft -- 3
```

```text
alice> sink urn:nl:sparql ask="the titles of the open items" focus=urn:iki:tutorial:items save=open-titles
urn:script:open-titles:version:sha256:… (a DRAFT of urn:script:open-titles: review it, then publish it with state=published)

PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?title WHERE {
  GRAPH <urn:iki:tutorial:items> { ?item ledger:status ledger:open ; dcterms:title ?title }
} ORDER BY ?title

preview (LIMIT 5):
?title
"Answer the issue"
"Fix the flaky test"
"Write the README"

(the model was asked 2 times)
```

`focus=` names the graph the ask is about, as it does for the grounding and the check. (Before
`ikigai-nl` 0.3.0 the drafter alone called it `graph=`, which 0.4.0 still accepts as a deprecated alias.)
The stub's first draft said `ledger:state`, the check refused it, and the drafter asked again: the
same prompt, followed by `urn:nl:prompt:sparql-repair` filled with the previous draft and exactly
what the check found in it, nothing else. The second draft passed, and was saved. The bound on attempts is 3 by default, and drafting also stops
the first time a model answers with a draft it already gave, because a model repeating itself is
not converging. Three is a cost bound, each attempt one model call; nobody has measured where
repair stops paying.

Saving is a `Sink` on `urn:script:open-titles` with `state=draft`, under **Alice's** capability.
That is the last validation, and the one that matters most: the script host analyzes the text
again and refuses a draft whose derived authority Alice does not hold. A caller who may not write
the name gets the valid draft answered, unsaved. A draft that never validates is answered as
failed, with every attempt and what was wrong with it, and no query. The saved text opens with its
provenance as comment lines (the ask, the grounding's identity, the prompts' digests, every
attempt and what the checks found), so it is in the version's digest, and the same graph is the
answer's `as=text/turtle` face.

## A draft the check passed, and should not have

This one is not a hypothetical. It is what a 70-billion-parameter model drafted for a real ask on
gonk's work ledger on 2026-10-11, with the graph renamed to this book's and its spacing tidied, and
the stub answers it word for word:

```bash
cargo run -p sentence-draft -- 4
```

```text
alice> sink urn:nl:sparql ask="the number and title of every open item with priority 2 or higher (0 is highest), ordered by number" focus=urn:iki:tutorial:items save=open-p2
urn:script:open-p2:version:sha256:… (a DRAFT of urn:script:open-p2: review it, then publish it with state=published). its preview is EMPTY: no row matched, and drafting stopped: the model repeated attempt 1, so no later attempt did better; the warnings name the patterns that match nothing on their own. Review it before publishing

# @param limit xsd:integer optional default 2
SELECT ?number ?title
FROM <urn:iki:tutorial:items>
WHERE {
  GRAPH <urn:iki:tutorial:items> {
    ?item a <https://ikigai-rs.dev/ns/ledger#Item> ;
      <https://ikigai-rs.dev/ns/ledger#status> "open" ;
      <https://ikigai-rs.dev/ns/ledger#priority> ?priority ;
      <https://ikigai-rs.dev/ns/ledger#number> ?number ;
      <http://purl.org/dc/terms/title> ?title .
    FILTER (?priority <= ?limit)
  }
}
ORDER BY ?number

warning: the preview is empty: no row matches, and these triple patterns match nothing on their own in the graphs it reads: GRAPH <urn:iki:tutorial:items> { ?item <https://ikigai-rs.dev/ns/ledger#status> "open" }. A string written where the data holds an IRI (or the reverse), or a literal of another datatype, matches nothing: compare each with the kind of value its predicate takes
preview (LIMIT 5):
?number	?title

(the model was asked 2 times)
```

Every check passed: it parses, it reads a graph Alice may read, every predicate and class it names
exists there. What is wrong with it is not a fact about names. It compares the status to the string
`"open"`, and the status in this graph (and in gonk's) is the IRI `ledger:open`; a string is never
equal to an IRI, so nothing matches.

The preview says so. An empty preview is not an error, because an empty answer is sometimes the
right one ("which items are overdue?" on a good day), so the check cannot refuse it. It is a
**warning, with its evidence**: the check runs each triple pattern of the query alone, in its graph,
and names the ones that match nothing on their own. Here that is exactly one pattern, the string
`"open"`. And the drafter treats a valid draft with an empty preview as a reason to repair: it sent
the model the draft and that warning, the way it sends a refused draft and its errors in section 3.
The stub answers this ask with the live demo's draft whatever it is told, so its second answer was
its first again, and drafting stopped there, because a model repeating itself is not converging. A
real model, told that `ledger:status` takes the IRI `ledger:open` (the grounding now says so) and
shown the one pattern that matched nothing, has what it needs to fix it.

When drafting stops with the preview still empty, the draft is kept, and **marked**: the first line
of the answer says the preview is EMPTY and to review it, the JSON face says `emptyPreview`, and in
its provenance the attempt it derives from carries the warning. Nothing is saved silently. The mark
also says what stopped drafting: here "drafting stopped: the model repeated attempt 1", because there
were two attempts, not the bound of 3, and the second gave the first again. That second attempt is
recorded as a **repeat**, not as a new draft: it is not checked again, so it stays invalid with one
error naming the attempt it repeats, its JSON carries `repeatOf: 1`, and in the provenance it is
`prov:alternateOf` attempt 1. Had all three attempts been tried, the mark would say "no attempt
within the bound of 3 did better". (Before `ikigai-nl` 0.4.0 it said that either way.)

Two smaller things are wrong too. It names the priority bound `limit`, a parameter nobody asked for
whose name reads like a row count. And it pairs `FROM <g>` with `GRAPH <g> { … }`, which in
standard SPARQL finds nothing on its own: `FROM` sets only the default graph, so a `GRAPH` block has
no named graphs to match. On this host that pairing happens to be harmless, because the script host
runs a query over one set of graphs that serves as both the default graph and the named ones; the
crate's tests (`crates/sentence-draft/tests/claims.rs`) pin that the string alone empties this
preview, and that the check names that pattern and no other. Run against an ordinary SPARQL
endpoint, it would be wrong twice.

So the mechanical half caught what it could, said why, and asked again. What it cannot do is
decide that an empty answer is wrong; a person reading the marked draft still does that.

## A draft is nobody's script yet

```bash
cargo run -p sentence-draft -- 5
```

```text
bob> source urn:script:open-p2
error: not found: no script is published at urn:script:open-p2

host> source urn:script:open-p2:result
error: conflict: urn:script:open-p2 is a draft; publish it (`state=published`) to run it

alice> source urn:script:open-p2:result
error: conflict: urn:script:open-p2 is a draft; publish it (`state=published`) to run it
```

Bob holds the read grant for `open-p2`, and to him the draft does not exist: `NotFound`, in the same
words as a name nobody ever wrote. `Denied` would tell him "somebody has a draft here", which is
the thing privacy hides. A draft is visible only to its author, the principal the writer's
capability named, until it is published.

Nobody can run a draft, not even the host: its `…:result` is refused for its state, and says what
to do about it. Alice, its author, is told the same thing in the same words: it is a draft, and
publishing it is how to run it. A draft has no contract of its own until it is published (a
script's own catalog entry is made from its published text), so it wears the generic one, which
declares only the run grant; the language's own grant (the Lisp evaluator's, for a Lisp script)
joins it when a script of that language is published. The state is reported before any run grant
is checked, because anyone who can see the draft may know it is one.

(Before `ikigai-script` 0.3.3 the generic contract declared the Lisp evaluator's grant, so Alice,
who holds no Lisp grant, was refused for that by the kernel's floor before the script host could
say "draft": a true refusal, for a reason that had nothing to do with a SPARQL query. The host's
answer also pointed at `urn:script:eval`, which runs only Lisp. Both are fixed; the hint is now
offered only for a language that door takes.)

## A person corrects it, checks it, and publishes it

```bash
cargo run -p sentence-draft -- 6
```

```text
alice> source urn:nl:sparql:check   (the corrected query, piped)
valid select over <urn:iki:tutorial:items>
requires: urn:cap:store:read:graph:urn:iki:tutorial:items
preview (LIMIT 5):
?number	?title
"1"^^<http://www.w3.org/2001/XMLSchema#integer>	"Write the README"
"2"^^<http://www.w3.org/2001/XMLSchema#integer>	"Fix the flaky test"

alice> sink urn:script:open-p2 language=sparql state=published   (the corrected query, piped)
urn:script:open-p2:version:sha256:16c96a61…

alice> source urn:script:open-p2:result as=text/tab-separated-values
?number	?title
1	"Write the README"
2	"Fix the flaky test"

bob> source urn:script:open-p2
…
ORDER BY ?number
```

Alice writes the status as the IRI it is, drops the `FROM`, writes the bound into the text, and asks
the same check the drafter asked. Then she publishes, with her own `Sink`, and it is a script like
any other: Bob can read it, a runner can run it under the previous chapter's rule, and the draft's
history is kept with it. The publish is the act, and a person made it.

## Against a real model

A reader with [Ollama](https://ollama.com) running can put a real model behind the same door. The
program swaps the stub for `ikigai-llm`'s `urn:llm:ask`, which reaches the local server through an
HTTP seam the host fills:

```rust,ignore
{{#include ../../../../crates/sentence-draft/src/bin/draft.rs:model}}
```

<!-- transcript: manual — needs a local Ollama with the model pulled; what a model drafts is not
     deterministic, so no page can show its output. -->
```bash
cargo run -p sentence-draft -- 3 --ollama llama3.1
```

Every step is the same resource, under the same capability. Alice holds
`urn:cap:net:localhost:11434`, the port-aware grant `ikigai-llm` checks before it reaches the
server, and a caller holding no network grant could not draft at all. What changes is
what comes back: the number of attempts, whether the first draft passes, and the query itself, none
of which a book can promise. Section 4 is the one to run more than once: a model shown the warning
and the value kinds may repair the status on its second attempt, or may not. A model that gets the
status right is answering this ask, not every ask, and the mark on an empty preview is still the
signal to look for.

## What was built

| the set | the name | what it is |
|---|---|---|
| the drafting host | `sentence_draft::kernel` | composition: the host from [Scripts are resources](scripts.md), `ikigai-nl`, the vocabulary, a model door |
| the stub model | `urn:llm:ask` (in every gate) | code: canned drafts per attempt, the prompts it was sent, no network |
| a real model | `urn:llm:ask` (with `--ollama`) | composition: `ikigai-llm` over a blocking HTTP client |
| a grounding | `urn:nl:grounding` | composition, `ikigai-nl`'s: the caller's own view, cached per capability |
| a check | `urn:nl:sparql:check` | composition, `ikigai-nl`'s: findings as values, so it caches |
| a draft | `urn:script:{name}` with `state=draft` | an atom, the script host's, written under the caller and private to them |

The one piece of code is the stub, and it is code for the reason the clock in "Time is a resource"
was: it stands in for something outside the kernel that a gate cannot run. Everything else is
configuration. The model's place in all of it is one door, called under the caller's capability,
whose answer is text that is parsed, checked, and saved as something a person has to read.
