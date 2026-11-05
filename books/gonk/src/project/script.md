# A workflow of your own

The [previous chapter](query.md) kept questions. Some questions are steps in how you work: what should
I do next on this artifact, and has anything been closed without evidence? This chapter publishes both
kinds. One is a **plan**, a fixed set of requests, and the other is a query used as a check.

## Catch up

In a new empty directory, as in [Start a project](start.md). If the gonk from [that
chapter](query.md) is still running, stop it first, since this one takes the same port.

The close example uses a dispatcher identity, because an alias scheme belongs to a principal.
Enroll a QUIC client with both ledger write and `gh` alias authority before starting gonk
([The three doors](../machines/doors.md) explains enrollment). A worker keeps neither grant:

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ ikigai-gonk client add dispatcher --ledger library=write --alias library=gh --port 1070 --config-home "$PWD/.config/ikigai" > /dev/null
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ (cd starter && python3 tools/new.py ../library --process solo,spec > /dev/null)
$ git -C library init -q -b main
$ (cd library && python3 tools/gonk.py init --ledger library --socket ../.ikigai/gonk.sock --holder ada | tail -1)
ready.
$ git -C library add -A
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Generate the library sample'
$ (cd library && python3 tools/gonk.py seed | tail -1)
seed: 8 filed, 0 already filed, 0 checked off (not filed), 6 order link(s) added
```

## The plan you already have

An agent working on the service wants `next`, narrowed to the service. The sample ships that as a
plan, `next-for-artifact`: one step, the ledger's own `next`, with the artifact's label passed through.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-next-for-artifact:result labels=artifact:service limit=1' 2>/dev/null | grep -E '^ 1\.|^ready'
 1. library#7  open    p-  cap-late-fees 1.1: Stop a late return's fee at 1000 cents  [artifact:service change:cap-late-fees]
ready: 2   excluded: 6
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-next-for-artifact:result labels=artifact:cli limit=1' 2>/dev/null | grep -E '^ 1\.|^ready'
 1. library#1  open    p-  add-holds 1.1: Accept a hold on a title whose copies are all out  [artifact:cli artifact:service change:add-holds]
ready: 1   excluded: 5
```

The ranking and the reasons are still `next`'s. The plan decided only what to ask, so it cannot
disagree with the ledger. Its authority is derived from the step it names, not declared, so its runner
needs exactly the read `next` needs.

## Publish one of your own

The same thing for a change, instead of an artifact. A plan is a small RDF graph: its inputs, its
steps, and which step's answer is the result. It goes in `.gonk/scripts/`, and it names its ledger as
`LEDGER`, as a query does:

<!-- transcript: file library/.gonk/scripts/next-for-change.plan.ttl -->
```turtle
# What to do next on one change: the ledger's own `next`, narrowed to that change's items.
@prefix ik: <https://ikigai-rs.dev/ns#> .

<urn:plan:next-for-change> a ik:Process ;
    ik:input <urn:plan:next-for-change:input:labels> ;
    ik:step <urn:plan:next-for-change:step:1> ;
    ik:result <urn:plan:next-for-change:step:1> .

<urn:plan:next-for-change:input:labels> ik:inputName "labels" ; ik:required true ;
    ik:summary "The change's label, change:<id>" .

<urn:plan:next-for-change:step:1> a ik:Step ;
    ik:verb "Source" ;
    ik:resolves <urn:iki:ledger:LEDGER:next> ;
    ik:argument <urn:plan:next-for-change:step:1:arg:labels> .

<urn:plan:next-for-change:step:1:arg:labels> a ik:Argument ; ik:inputName "labels" ;
    ik:ref <urn:plan:next-for-change:var:labels> .
```

`init` publishes it beside the rest, and the summary becomes the argument's own help:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py init | grep -c 'urn:script:library-next-for-change')
1
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-next-for-change:result labels=change:cap-late-fees' 2>/dev/null | grep -E '^ 1\.|^ready'
 1. library#7  open    p-  cap-late-fees 1.1: Stop a late return's fee at 1000 cents  [artifact:service change:cap-late-fees]
ready: 1   excluded: 1
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-next-for-change:result' 2>&1 | grep -E '^error|labels`:'
error: missing required argument `labels`
       `labels`: The change's label, change:<id>
```

## A plan may not write

It is tempting to make the plan claim what it found. gonk refuses at publish, whoever publishes it,
and names the reason:

<!-- transcript: run -->
```console
$ sed -e 's/ik:verb "Source"/ik:verb "Sink"/' -e 's/LEDGER/library/' library/.gonk/scripts/next-for-change.plan.ttl | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:script:library-writes language=plan'
error: denied: a plan with a `Sink` or `Delete` step cannot be published here: this server's scripts are READ-ONLY. Every run is held to a ceiling of the ledgers' reads (`urn:cap:ledger:read:*`, `urn:cap:store:read:graph:urn:iki:ledger:graph:*`), because the ledger's own write door and a raw write of its graph need the same store token, and gonk tells them apart only where a request arrives, which a script's steps never do. Write through the ledger's own doors (`urn:iki:ledger:*`)
```

So a workflow here is a read that decides, and a client that acts: `tools/gonk.py`, an agent over MCP,
or you. A write still goes through the ledger's own door, under the caller's own name, which is what
keeps the log honest.

## A close-with-evidence check

`close` refuses a close without evidence, but that rule lives in `tools/gonk.py`. Anyone with a write
grant can close through the ledger's own door, so check what happened rather than trusting the path.
The evidence can be in two places. `evidence=` on a direct close puts it on the close's record;
`gonk.py close` writes structured evidence on the close and also posts a comment that starts
`Evidence:`.
The check accepts either:

<!-- transcript: file library/.gonk/queries/closed-without-evidence.rq -->
```sparql
# Items closed with no evidence, on the close's record or in an `Evidence:` comment.
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title WHERE {
  GRAPH <urn:iki:ledger:graph:LEDGER> {
    ?item a ledger:Item ;
          ledger:status ledger:closed ;
          ledger:number ?number ;
          dcterms:title ?title .
    FILTER NOT EXISTS { ?close ledger:onItem ?item ; ledger:evidence ?evidence }
    FILTER NOT EXISTS {
      ?comment a ledger:Comment ; ledger:onItem ?item ; ledger:body ?body .
      FILTER(STRSTARTS(?body, 'Evidence:'))
    }
  }
}
ORDER BY ?number
```

Publish it, switch the starter to the dispatcher's QUIC identity, then close three items, one
each way. The direct ledger calls still use the owner's socket:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py init | grep -c 'urn:script:library-closed-without-evidence')
1
$ (cd library && python3 tools/gonk.py init --quic quic://127.0.0.1:1070 --cert-dir ../.config/ikigai/gonk/quic/clients/dispatcher | tail -1)
ready.
$ (cd library && python3 tools/gonk.py close 7 --evidence https://github.com/you/library/pull/1 --note 'Merged, CI green')
closed library#7 done (1 evidence)
Alias: gh:you/library#1
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:library:close item=8 evidence=https://github.com/you/library/pull/2 Merged, CI green.'
closed library#8 done (1 evidence)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:library:close item=1 Looks done to me.'
closed library#1 (done)
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-closed-without-evidence:result as=text/csv'
number,title
1,add-holds 1.1: Accept a hold on a title whose copies are all out
[uncacheable]
```

Only the third is listed. This checks that evidence was *named*, not that it holds: a made-up URL
passes, and judging it stays with the dispatcher. To use it as a gate, have a CI job or a person run
it and treat any row as a failure.

A GitHub pull request URL on `gonk.py close` also adds a `gh:` alias, under the dispatcher's
identity. The starter accepts that alias wherever it takes an item number. The ledger can read it
too (percent-encoded inside the resource name):

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:library:item:gh:you%2Flibrary%231' 2>/dev/null | head -3
library#7  closed  p-  cap-late-fees 1.1: Stop a late return's fee at 1000 cents  [artifact:service change:cap-late-fees]  (done)
  iri:      urn:iki:ledger:library:item:…
  key:      urn:openspec:change:cap-late-fees:task:1.1
```

The alias names the ledger item that the pull request resolved. The worker's hand-back only
proposes evidence; the dispatcher adds the alias when accepting it at close. Adding this alias
takes the separate `--alias library=gh` authority as well as an authenticated principal; neither
belongs to the worker's narrow grant. The socket alone has no principal to own the scheme.

A script's versions are named by their content, so the question "which check said that?" has an answer:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:catalog' 2>/dev/null | grep 'closed-without'
urn:script:library-closed-without-evidence  published  sha256:…
```

<a id="what-you-built"></a>

## What you have now

A workflow in files: one plan that narrows `next`, one check that reads what really happened, both
reviewed like code and published by `init`. Every step so far had you write the query. The [next
chapter](model.md) sets up a model, so that a sentence can start one.
