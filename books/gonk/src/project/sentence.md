# A query from a sentence

The queries in [Ask the project in SPARQL](query.md) took some knowing: which graph, which predicates,
that a status is an IRI and not a string. A model can write a first draft from a sentence. It writes a
**draft**: checked, previewed, and saved where nobody can run it. Only a person publishes.

This page shows that against the project. [Your first saved query](../scripts/first.md) and
[Scripts and sentences in gonk](../scripts/in-gonk.md) teach who may do what through which door, and
this one assumes them.

## Catch up, with a stand-in model

The check on this page uses `model-stub`, this book's fixed answers, so it prints the same thing on
every machine. **With your own model from the [previous chapter](model.md), the drafts will be
different**, for better or worse, and the steps stay the same.

Start in a new empty directory, as in [Start a project](start.md); this chapter regenerates the
sample and seeds a fresh ledger. If the gonk from [A model of your own](model.md) is still running,
stop it first, since this gonk takes the same port. For the stand-in route, stop that chapter's
peers too. Keep each server running in its own terminal and run the remaining commands in another,
from this chapter's directory.

To use your own model, keep its peer running. Skip the next two blocks, which start and configure
the stand-in. Instead, create `.config/ikigai/config.toml` in this chapter's directory with your
own `gonk.mount` line from [A model of your own](model.md). Use absolute paths for its socket or
certificate directory so the line still works from here. Then start gonk below.

<!-- transcript: serve -->
```console
$ model-stub "$PWD/model.sock" | tee model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
```

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.mount = "prefer urn:llm:=%s/model.sock"\n' "$PWD" > .config/ikigai/config.toml
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ (cd starter && python3 tools/new.py ../library --process solo,spec > /dev/null)
$ git -C library init -q -b main
$ (cd library && python3 tools/gonk.py init --ledger library --socket ../.ikigai/gonk.sock --holder ada | grep -E '  nl |^ready')
  ok       nl        gonk offers `urn:nl:sparql`: drafting a query from a sentence (ikigai-nl)
ready.
$ git -C library add -A
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Generate the library sample'
$ (cd library && python3 tools/gonk.py seed | tail -1)
seed: 8 filed, 0 already filed, 0 checked off (not filed), 6 order link(s) added
```

The `nl` row was `absent` in [Start a project](start.md#a-gonk-on-this-machine). A mounted model is the
whole difference.

## Draft

The ask is piped in. `focus=` names the graph it is about, so the model is shown what mentions that
graph and not the whole vocabulary. `save=` names the draft, under the project's prefix:

<!-- transcript: run -->
```console
$ printf 'the titles of the open items in the library ledger' | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:nl:sparql save=library-titles focus=urn:iki:ledger:graph:library'
urn:script:library-titles:version:sha256:… (a DRAFT of urn:script:library-titles: review it, then publish it with state=published)
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?title WHERE {
  GRAPH <urn:iki:ledger:graph:library> { ?item ledger:status ledger:open ; dcterms:title ?title }
} ORDER BY ?title
preview (LIMIT 5):
?title
"add-holds 1.1: Accept a hold on a title whose copies are all out"
"add-holds 1.2: Keep a returned copy for the first holder"
"add-holds 1.3: Refuse a renewal while a hold waits"
"add-holds 1.4: Add a vector to openspec/vectors.json for every new scenario"
"add-holds 2.1: Add `library.py hold MEMBER ISBN`"
[uncacheable]
```

## Check and repair

That looks like one clean answer, and it was not. Ask for the attempts, and see what happened on the way:

<!-- transcript: run -->
```console
$ printf 'the titles of the open items in the library ledger' | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:nl:sparql save=library-titles-again focus=urn:iki:ledger:graph:library as=application/json' 2>/dev/null | jq -r '.status, (.attempts[] | "attempt \(.n) by \(.model): valid=\(.valid) \(.errors | join("; "))")'
saved
attempt 1 by stub: valid=false the predicate <https://ikigai-rs.dev/ns/ledger#state> is used by none of the graphs this query reads and is not in the vocabulary
attempt 2 by stub: valid=true
```

The model's first draft named a predicate that does not exist. **The check** caught it: every draft is
parsed and held to what the caller may read, every predicate is looked up, and the query runs once as a
preview. The drafter sent that error back and asked again, and the second attempt passed. This loop is
what makes a weak model usable. A wrong name does not need a person, because the graph itself can refuse
it.

## A draft the check cannot save

Some wrong queries are valid. This ask gets one, and here the stand-in behaves like the weakest model
you are likely to meet:

<!-- transcript: run -->
```console
$ printf 'the number and title of every open item about late fees, ordered by number' | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:nl:sparql save=library-fees focus=urn:iki:ledger:graph:library'
urn:script:library-fees:version:sha256:… (a DRAFT of urn:script:library-fees: review it, then publish it with state=published). its preview is EMPTY: no row matched, and drafting stopped: the model repeated attempt 1, so no later attempt did better; the warnings name the patterns that match nothing on their own. Review it before publishing
SELECT ?number ?title
WHERE {
  GRAPH <urn:iki:ledger:graph:library> {
    ?item a <https://ikigai-rs.dev/ns/ledger#Item> ;
      <https://ikigai-rs.dev/ns/ledger#status> "open" ;
      <https://ikigai-rs.dev/ns/ledger#number> ?number ;
      <http://purl.org/dc/terms/title> ?title .
    FILTER (CONTAINS(LCASE(?title), "fee"))
  }
}
ORDER BY ?number
warning: the preview is empty: no row matches, and these triple patterns match nothing on their own in the graphs it reads: GRAPH <urn:iki:ledger:graph:library> { ?item <https://ikigai-rs.dev/ns/ledger#status> "open" }. A string written where the data holds an IRI (or the reverse), or a literal of another datatype, matches nothing: compare each with the kind of value its predicate takes
preview (LIMIT 5):
?number	?title
[uncacheable]
$ grep -c 'late fees' model.log
2
```

It parses, and every predicate exists. But the ledger stores a status as the IRI `ledger:open`, and a
string never equals an IRI, so nothing matches. The check tried each pattern alone and names the one at
fault. The model was told, and wrote the same query again (two asks in the log), so drafting stopped and
saved it **marked**, with the warning on its first line.

Empty is the easy case, because it announces itself. A real model can do something harder to spot. Given
this same ask on this same ledger, `qwen3:8b` on Ollama, by hand on 2026-11-03, wrote a query that passed
the check and returned rows:

```sparql
FILTER(?label = "change:cap-late-fees")
```

It came back with two plausible rows, and its one warning was about a parameter it declared and never
used. But it hard-coded one change's label in place of "about late fees", so the moment a second fee change exists (the `waive-first-fee` of
[A spec change](change.md), say), that query misses it. Only reading it catches that. **A preview shows
that a query runs, not that it means what you asked.**

## Review, repair, publish

A draft cannot be run, by anyone, even you:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-fees:result'
error: conflict: urn:script:library-fees is a draft; publish it (`state=published`) to run it
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:catalog' 2>/dev/null | grep 'draft'
urn:script:library-fees  draft  sha256:…
urn:script:library-titles  draft  sha256:…
urn:script:library-titles-again  draft  sha256:…
```

Read it, and fix what the warning names: the status as the IRI it is. Keep it in the repository with the
others, naming the ledger as `LEDGER`:

<!-- transcript: file library/.gonk/queries/open-about-fees.rq -->
```sparql
# The open items about late fees, by number. Drafted from a sentence, then corrected.
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title WHERE {
  GRAPH <urn:iki:ledger:graph:LEDGER> {
    ?item a ledger:Item ;
          ledger:status ledger:open ;
          ledger:number ?number ;
          dcterms:title ?title .
    FILTER(CONTAINS(LCASE(?title), "fee"))
  }
}
ORDER BY ?number
```

The drafter's own check runs on any text, so try the correction before publishing it. The check reads
a real graph, so the file's `LEDGER` is filled in first:

<!-- transcript: run -->
```console
$ sed 's/LEDGER/library/' library/.gonk/queries/open-about-fees.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:nl:sparql:check focus=urn:iki:ledger:graph:library'
valid select over <urn:iki:ledger:graph:library>
requires: urn:cap:store:read:graph:urn:iki:ledger:graph:library
preview (LIMIT 5):
?number	?title
"7"^^<http://www.w3.org/2001/XMLSchema#integer>	"cap-late-fees 1.1: Stop a late return's fee at 1000 cents"
"8"^^<http://www.w3.org/2001/XMLSchema#integer>	"cap-late-fees 1.2: Add a vector for the capped return to openspec/vectors.json"
[uncacheable]
```

Rows, where the draft had none. Publishing is a person's act, so it goes through `init`, which publishes
the repository's files, and it can only happen at a door that is not a machine's. Over QUIC, a client
certificate may save a draft and may not publish it, whatever its grant says
([A machine drafts, a person publishes](../scripts/in-gonk.md#a-person-publishes)).

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py init | grep -c 'urn:script:library-open-about-fees')
1
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-open-about-fees:result as=text/csv'
number,title
7,cap-late-fees 1.1: Stop a late return's fee at 1000 cents
8,cap-late-fees 1.2: Add a vector for the capped return to openspec/vectors.json
[uncacheable]
```

The draft that started it is still a draft. It can be published as is, by sinking its own text with
`state=published`, as [Your first saved query](../scripts/first.md#2-ask-gonk-in-a-sentence) does for a
draft that was right. This one was wrong, so retire it, and the other two with it:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'delete urn:gk:script:library-fees' -c 'delete urn:gk:script:library-titles' -c 'delete urn:gk:script:library-titles-again'
…
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:catalog' 2>/dev/null | grep -c draft
0
```

## When your own model drafts badly

Work down this list, cheapest first.

1. **Read the warning.** The check names the pattern at fault, and often the correction is one word.
2. **Say more in the sentence.** Name the labels, the columns and the order you want.
3. **Narrow `focus=`.** What the model is not shown, it cannot mistake.
4. **Change the model.** Name a bigger one in `llm.json`, then restart the peer and gonk.
5. **Write it by hand.** Nothing is lost: a draft is a starting point, and the [saved
   queries](query.md) in your repository are the examples the next draft is shown.

<a id="what-you-built"></a>

## What you have now

The whole arc over one project. A spec change became items, `next` ordered them, a claim fenced each,
work was reproduced and fixed, a model's finding was checked and declined, evidence was reported and
turned down and reported again, and a second agent ran beside the first. The project answers questions
in saved queries and plans, and a sentence can start the next one, under a person's review.

From here, the choices are about shape: [Ways of working](../ways/solo.md) compares the processes the
starter packages, and the starter's
[`worker/README.md`](https://github.com/ikigai-rs/ikigai-gonk-starter/blob/main/worker/README.md) sets
up a machine to take items unattended.
