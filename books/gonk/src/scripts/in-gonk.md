# Scripts and sentences in gonk

[The ikigai Book](../../code/scripts.html) showed code as a resource: a query published at a
name, fetched without running, run as a read, versioned by its content, and run under the least of
what its publisher held, what its runner holds and what the host allows. [A sentence becomes a
draft](../../code/sentence.html), two chapters later, let a model write the query, as a **draft**
nobody but the person who asked could see, until a person published it. [Your first saved query](first.md) did
each once in gonk, as its owner. This chapter does not teach either idea again. It shows the same
two things in gonk, which mounts [`ikigai-script`](https://crates.io/crates/ikigai-script) and
[`ikigai-nl`](https://crates.io/crates/ikigai-nl) behind its three doors, and so has to answer a
question a library cannot: **who is asking**, through which door, and what each of them may do.

The cast, all on this page's one scratch gonk:

| who | door | what they hold |
| --- | --- | --- |
| the owner | the socket | everything: root, as in [Over the socket](../agent/socket.md) |
| `bot`, a drafting agent | QUIC | read the ledger, draft from a sentence, save drafts in the `team-` namespace |
| `reviewer`, a second machine | QUIC | read the ledger, read and run the `team-` scripts |
| `worker`, a remote worker | QUIC | read, claim and comment on the ledger; run public scripts |
| anyone on this machine | HTTP, signed out | the configured ledger, and every PUBLIC script, within a budget |
| `ann`, a person | HTTP, signed in with a passkey | write the ledger, draft, publish in the `team-` namespace |

`ann` is the one this page cannot be: signing in is a passkey ceremony in a browser, which no check
can perform. Her commands are shown where they belong and marked as not run; everything she does
is also done here by a door the check can drive, and the page says which.

## A model that is not one

gonk drafts through the model its `gonk.mount` line mounts at `urn:llm:`, as
[Explain](../browse/explain.md) does. A transcript must never reach a real model, so this page
starts a **stub** on a socket and mounts that instead: `model-stub`, from this book's
`crates/model-stub`, answers `urn:llm:ask` with a fixed SPARQL draft for each ask, the way the
ikigai Book's stub does in process, and prints a line for every ask it is sent. The mount is the
real one, so everything between the drafter and the model is gonk's own code; only the model's
words are canned. In a new empty directory, as in [every
chapter](../introduction.md#a-gonk-per-chapter): if the gonk and the `model-stub` from [Your first
saved query](first.md) are still running, stop them first, since this gonk takes the same port.

<!-- transcript: serve -->
```console
$ model-stub "$PWD/model.sock" | tee model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
```

The config names the mount, and lowers the anonymous run budget from its default of 30 a minute to
3, so the page can show the limit without sending thirty requests:

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.mount = "prefer urn:llm:=%s/model.sock"\n' "$PWD" > .config/ikigai/config.toml
$ printf 'gonk.script.anonymous_runs_per_minute = 3\n' >> .config/ikigai/config.toml
$ cat .config/ikigai/config.toml
gonk.mount = "prefer urn:llm:=…/model.sock"
gonk.script.anonymous_runs_per_minute = 3
```

## Grants: `--script` and `--nl`

Two flags on `client add` and `passkey invite` grant scripts and drafting, beside the `--ledger`
roles the earlier chapters used:

- **`--script <name>=<read|run|publish>`**, repeatable, each authority holding the one before:
  `read` fetches a script's source without running it, `run` runs it, and `publish` publishes,
  replaces and retires it. The name is ONE script (`open-by-label`) or a **namespace**,
  `<namespace>-*`: `team-*` covers `team-report` and `team-a-report`, never `teammate` or `team`.
  There is no "every script" grant below the owner. `public` is the one name that is not a
  script: `--script public=run` runs and reads every script its publisher marked public, which is
  what a signed-out HTTP caller holds.
- **`--nl draft`**: may draft a query from a sentence, which spends the mounted model, so it is the
  mount's network grant, `urn:cap:net:<the mount's host>` (`localhost` for a socket). It is
  refused on a server with no `gonk.mount`, which is why the config came first. A draft reads only
  what the caller's other grants read, and is SAVED only under a name its `--script … =publish`
  covers.

⚠ Quote a namespace: `'team-*=publish'`. Unquoted, zsh refuses the line (`no matches found`)
before gonk ever sees it, and another shell may expand the `*` against the files in the directory.

<!-- transcript: run -->
```console
$ ikigai-gonk client add bot --ledger default=read --nl draft --script 'team-*=publish' --port 1070 --config-home "$PWD/.config/ikigai"
client `bot`  …/.config/ikigai/gonk/quic/clients/bot
  fingerprint  …
  principal    urn:iki:host:gonk:client:…
  enrolled     grant `bot` (7 scopes) in …/.config/ikigai/gonk/grants.json
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <the moved directory>
$ ikigai-gonk client add reviewer --ledger default=read --script 'team-*=run' --port 1070 --config-home "$PWD/.config/ikigai"
client `reviewer`  …/.config/ikigai/gonk/quic/clients/reviewer
…
  enrolled     grant `reviewer` (4 scopes) in …/.config/ikigai/gonk/grants.json
…
$ ikigai-gonk client add worker --worker default --script public=run --port 1070 --config-home "$PWD/.config/ikigai"
client `worker`  …/.config/ikigai/gonk/quic/clients/worker
…
  enrolled     grant `worker` (… scopes) in …/.config/ikigai/gonk/grants.json
…
$ ikigai-gonk passkey invite ann --ledger default=write --nl draft --script 'team-*=publish' --port 1070 --config-home "$PWD/.config/ikigai"
passkey invite for `ann` — grant `ann` (… scopes) in …/.config/ikigai/gonk/grants.json
  valid for 30 minutes, once
  open  http://localhost:1070/#invite=…
  in a browser on this machine (localhost, not 127.0.0.1); the passkey is enrolled in …/.config/ikigai/gonk/clients.json
```

What the flags wrote, grant by grant:

<!-- transcript: run -->
```console
$ jq -r 'del(.ann) | to_entries[] | .key, (.value[] | "  " + .)' .config/ikigai/gonk/grants.json
bot
  urn:cap:ledger:read:default
  urn:cap:store:read:graph:urn:iki:ledger:graph:default
  urn:cap:net:localhost
  urn:cap:script:read:team-*
  urn:cap:script:run:team-*
  urn:cap:script:write:team-*
  urn:cap:script:delete:team-*
reviewer
  urn:cap:ledger:read:default
  urn:cap:store:read:graph:urn:iki:ledger:graph:default
  urn:cap:script:read:team-*
  urn:cap:script:run:team-*
worker
  urn:cap:ledger:read:default
  urn:cap:store:read:graph:urn:iki:ledger:graph:default
  urn:cap:ledger:claim:default
  urn:cap:ledger:comment:default
  urn:cap:store:write:graph:urn:iki:ledger:graph:default
  urn:cap:script:read:public
  urn:cap:script:run:public
```

`bot`'s seven: the ledger's two read tokens, the mount's host, and all four script tokens over
the namespace (`write` and `delete` are what `publish` means). `reviewer` holds `read` and `run`
over the same names and nothing that drafts. `worker`'s `--worker default` is a ledger role
(`--ledger default=worker` says the same): read, claim and comment, never close, for a
remote worker whose dispatcher closes. It carries the ledger graph's store WRITE token, because
the ledger writes a claim or a comment as a store update under the caller's own capability; the
socket and QUIC doors refuse a narrowed caller's raw write to a ledger graph all the same
([Over the socket](../agent/socket.md), [A QUIC client](../machines/quic-client.md)). `ann`'s grant is `bot`'s with the ledger's writer in
place of its reader.

## Start gonk

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  scripts urn:script:* in …/.ikigai/scripts — sparql and plan, read-only (the ledgers' reads); anonymous runs PUBLIC scripts and read-only plans, 3 a minute in all (draft checks too), each held to the anonymous budget; a QUIC client drafts, a person publishes; backed up with the store
  nl      urn:nl:grounding, urn:nl:sparql:check — under the caller's own grant; urn:nl:sparql drafts through urn:llm:ask for a caller holding urn:cap:net:localhost (--nl draft), saving DRAFT scripts a person publishes
…
  llm     …/model.sock — mounted, but no gonk.browse.root, so nothing here derives through it
…
  quic    udp 0.0.0.0:1070 — 3 trusted certificate(s), 3 enrolled
…
```

Two lines are this chapter's. `scripts` says where the scripts live (plain files beside the store,
not in it: a version records what its publisher held, and that is authority, so no store grant may
reach it), that every script here is **read-only** (a query, or a plan whose steps only read,
held to the ledgers' reads), and the two rules this page shows: anonymous callers run only public
scripts, within a shared budget, and a QUIC client drafts while a person publishes. `nl` says who
may draft: a caller holding the mount's host.

A ledger to ask about, filed by the owner over the socket:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 Passkey list and revoke' -c 'sink urn:gk:iki:ledger:append priority=2 Dependency floors never re-resolved' -c 'sink urn:gk:iki:ledger:append priority=3 A note for later' -c 'sink urn:gk:iki:ledger:label item=2 triage' -c 'sink urn:gk:iki:ledger:label item=3 triage'
#1 urn:iki:ledger:default:item:…
[uncacheable]
#2 urn:iki:ledger:default:item:…
[uncacheable]
#3 urn:iki:ledger:default:item:…
[uncacheable]
labeled #2 triage
[uncacheable]
labeled #3 triage
[uncacheable]
— batch: 5 commands · 5 uncacheable
```

## A saved query, for a person and for a worker

The owner publishes a query that lists the open items carrying a label. Its one parameter is
declared in its leading comment; `FROM` names the graph it reads, which is what the publish derives
its authority from (the store's read token for that graph, which the owner holds):

<!-- transcript: file open-by-label.rq -->
```sparql
# Open items carrying a label, lowest number first.
# @param label xsd:string -- the label an item must carry
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title
FROM <urn:iki:ledger:graph:default>
WHERE {
  ?item a ledger:Item ; ledger:status ledger:open ; ledger:label ?label ;
        ledger:number ?number ; dcterms:title ?title .
}
ORDER BY ?number
```

`public=true` marks it public: anyone holding `public=run` may run it, the signed-out HTTP caller
included.

<!-- transcript: run -->
```console
$ cat open-by-label.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:script:open-by-label language=sparql public=true'
urn:script:open-by-label:version:sha256:…
[uncacheable]
```

A person at a browser on this machine runs it with no account at all, by its HTTP name, the
parameter as a query argument and the answer as CSV:

<!-- transcript: run -->
```console
$ curl -s 'http://localhost:1070/script/open-by-label/result?label=triage&as=text/csv'
number,title
2,Dependency floors never re-resolved
3,A note for later
```

A remote worker runs the same script over QUIC, under its own certificate, and takes the first
item. What it may do next is its ledger role's, not the script's: a worker claims and comments, and
its dispatcher closes.

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/worker -c 'source urn:script:open-by-label:result label=triage as=text/csv'
number,title
2,Dependency floors never re-resolved
3,A note for later
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/worker -c 'sink urn:iki:ledger:claim item=2 worker'
#2 claimed by worker
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/worker -c 'sink urn:iki:ledger:close item=2'
error: denied: this capability holds neither `urn:cap:ledger:close:default` nor `urn:cap:ledger:write:default`, and this resource takes either. A ledger grant names exactly one ledger, and a grant for another verb (a worker's claim and comment, say) does not reach this one
```

One query, one name, two callers at two doors, each held to its own grant. Neither could publish
it, and neither needed to read the store directly.

## The anonymous budget

A signed-out HTTP caller cannot be told apart from any other, so all of them share ONE window of
runs a minute (30 by default; 3 on this page). Every run of a script counts, and so does an ad hoc
plan and a draft's dry run; an answer from gonk's cache does not, and neither does reading a
script's source. The person's run above was the first. The worker's claim since then wrote the ledger, which
cut every cached answer that read it, so the same run now computes again, and counts; asked again
straight away, it is answered from the cache, and does not:

<!-- transcript: run -->
```console
$ curl -s 'http://localhost:1070/script/open-by-label/result?label=triage&as=text/csv'
number,title
2,Dependency floors never re-resolved
3,A note for later
$ curl -s 'http://localhost:1070/script/open-by-label/result?label=triage&as=text/csv'
number,title
2,Dependency floors never re-resolved
3,A note for later
$ curl -s 'http://localhost:1070/script/open-by-label/result?label=urgent'
{"head":{"vars":["number","title"]},"results":{"bindings":[]}}
$ curl -s -w ' (%{http_code})\n' 'http://localhost:1070/script/open-by-label/result?label=later'
endpoint error: rate-limited: `urn:iki:gonk:anonymous:runs` is capped at 3/60s — retry after …s (500)
```

Four requests and three runs: the first triage run, the recomputed one and `urgent` (no item
carries that label, so the answer is empty SPARQL JSON, the face when no `as=` is asked for) fill
the window, and `later` is refused, naming the window and when a slot frees.
(Each run is also held to the anonymous SPARQL budget, 1 s by default, as the anonymous caller's
own `/sparql` is.) ⚠ The refusal arrives as `500`; a `429` with `Retry-After` would say the same
thing to a client that does not read the body.

## Drafting: who may, and what it costs

A signed-out caller may not draft at all, and is refused before any model is asked:

<!-- transcript: run -->
```console
$ curl -s -X POST --data-binary 'the titles of the open items' 'http://localhost:1070/nl/sparql?save=team-titles&focus=urn:iki:ledger:graph:default'
denied: capability does not grant `urn:cap:net:localhost` (declared by `urn:nl:sparql`)
$ grep -c ' ask ' model.log
0
```

The token it names is the one `--nl draft` mints. gonk DECLARES it on `urn:nl:sparql`, so the
kernel refuses before the grounding is computed, and the manifold offers the drafter only to the
callers who may draft.

### What the model is shown, and `focus=`

The drafter tells the model what the caller may see, and refuses a grounding over 1 MiB rather than
trimming it. Of the grounding it reads only what a query needs, asked for with `parts=`: the
vocabulary, the shape of each graph the caller may read, and the worked examples. It never reads
the actions, the caller's manifold, because a query calls none of them, and they are what grows
with a gonk: each browse root adds its own family. So the size of a gonk does not refuse a draft.
Measured on a scratch gonk at this book's pin browsing 48 roots, as the live gonk did on
2026-10-11: the whole grounding, actions included, is 4,632,254 bytes and refused unfocused; the
parts the drafter reads are 109,915 bytes, and an unfocused draft there was drafted, repaired and
saved. (The live gonk, whose drafter then read the whole grounding, refused every unfocused draft
that day; [ledger #1243](http://localhost:1062/l/default/item/1243) is the fix.)

This page's gonk shows the same shape at its own size. The actions are counted and left out, and
`focus=`, a graph IRI or a topic word, narrows what is left to what mentions it:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:nl:grounding parts=vocabulary,graphs,examples as=application/json' 2>/dev/null | jq -r '.actions.note, (.vocabulary, .graphs, .examples | "\(.source): \(.shown) of \(.of)")'
left out: parts= did not ask for it (103 offered)
urn:ikigai:vocab: 199 of 199
urn:iki:store:graphs: 1 of 1
urn:script:catalog: 1 of 1
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:nl:grounding parts=vocabulary,graphs,examples focus=urn:iki:ledger:graph:default as=application/json' 2>/dev/null | jq -r '.vocabulary, .graphs, .examples | "\(.source): \(.shown) of \(.of)"'
urn:ikigai:vocab: 0 of 199
urn:iki:store:graphs: 1 of 1
urn:script:catalog: 1 of 1
```

Unfocused, the model would be shown all 199 terms of the vocabulary; focused on the ledger graph,
the graph's own shape and the example that reads it, and no term, since none mentions the graph.

Every draft on this page passes `focus=` all the same: what the model is not shown, it cannot
mistake. (`graph=`, the drafter's older name for it, is accepted for one more release.)

## A machine drafts

`bot` drafts the live demo's ask, word for word, over QUIC. The ask is piped, so it arrives as
`content`; `save=` names the draft inside the namespace `bot` may publish in:

<!-- transcript: run -->
```console
$ printf 'the number and title of every open item with priority 2 or higher (0 is highest), ordered by number' | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/bot -c 'sink urn:nl:sparql save=team-open-p2 focus=urn:iki:ledger:graph:default'
urn:script:team-open-p2:version:sha256:… (a DRAFT of urn:script:team-open-p2: review it, then publish it with state=published). its preview is EMPTY: no row matched, and drafting stopped: the model repeated attempt 1, so no later attempt did better; the warnings name the patterns that match nothing on their own. Review it before publishing
# @param limit xsd:integer optional default 2
SELECT ?number ?title
FROM <urn:iki:ledger:graph:default>
WHERE {
  GRAPH <urn:iki:ledger:graph:default> {
    ?item a <https://ikigai-rs.dev/ns/ledger#Item> ;
      <https://ikigai-rs.dev/ns/ledger#status> "open" ;
      <https://ikigai-rs.dev/ns/ledger#priority> ?priority ;
      <https://ikigai-rs.dev/ns/ledger#number> ?number ;
      <http://purl.org/dc/terms/title> ?title .
    FILTER (?priority <= ?limit)
  }
}
ORDER BY ?number
warning: the preview is empty: no row matches, and these triple patterns match nothing on their own in the graphs it reads: GRAPH <urn:iki:ledger:graph:default> { ?item <https://ikigai-rs.dev/ns/ledger#status> "open" }. A string written where the data holds an IRI (or the reverse), or a literal of another datatype, matches nothing: compare each with the kind of value its predicate takes
preview (LIMIT 5):
?number	?title
[uncacheable]
$ cat model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
model-stub: ask 1: the number and title of every open item with priority 2 or higher (0 is highest), ordered by number
model-stub: ask 2 (a repair): the number and title of every open item with priority 2 or higher (0 is highest), ordered by number
```

That is what Llama 3.3 70B wrote on the live gonk, and the stub writes it here. The check passes
it: it parses, and every predicate it names is one the ledger graph uses. But its preview is empty,
and the check says why, by trying each triple pattern on its own and naming the one that matches
nothing: `ledger:status "open"`. The ledger stores the status as the IRI `ledger:open`, and a
string never equals an IRI. So the drafter sends the model that warning and asks again (`ask 2 (a
repair)` in the stub's log). The stub answers with the same draft, as a model that cannot see its
mistake would, and a repair that repeats an earlier attempt stops the drafting: asking a third time
would buy nothing. The draft is saved all the same, MARKED on the first line of the answer, so the
person who reviews it starts from the warning. (`FROM` beside a `GRAPH` block naming the same graph
looks like a second error, and in standard SPARQL it would be one; here it is harmless, because a
script's graphs are passed as both its default and its named graphs, as the ikigai Book's [claims
test](../../code/sentence.html#a-draft-the-check-passed-and-should-not-have) pins.)

### The attempts, and where the draft came from

`as=` picks the face of the answer. `as=application/json` is every attempt with what the check
said of it — here for an ask whose first draft the check refuses, so the drafter repairs it:

<!-- transcript: run -->
```console
$ printf 'the titles of the open items' | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/bot -c 'sink urn:nl:sparql save=team-titles focus=urn:iki:ledger:graph:default as=application/json' 2>/dev/null | jq -r '.status, .script, (.attempts[] | "attempt \(.n) by \(.model): valid=\(.valid) \(.errors | join("; "))"), "preview: \(.check.preview.rows | length) row(s)"'
saved
urn:script:team-titles
attempt 1 by stub: valid=false the predicate <https://ikigai-rs.dev/ns/ledger#state> is used by none of the graphs this query reads and is not in the vocabulary
attempt 2 by stub: valid=true
preview: 3 row(s)
```

`as=text/turtle` is the drafting's provenance in PROV-O: the activity, the grounding and prompts
it used (each by digest), every attempt as an entity with its model, its text and its verdict, and
the script it produced:

<!-- transcript: run -->
```console
$ printf 'the titles of the open items' | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/bot -c 'sink urn:nl:sparql save=team-titles-prov focus=urn:iki:ledger:graph:default as=text/turtle' 2>/dev/null | grep -E 'draftValid|wasRevisionOf|script:team-titles-prov>' | sed -E 's/[0-9a-f]{64}/…/g'
<urn:nl:sparql:drafting:…:attempt:1> <https://ikigai-rs.dev/ns#draftValid> "false"^^<http://www.w3.org/2001/XMLSchema#boolean> .
<urn:nl:sparql:drafting:…:attempt:2> <https://ikigai-rs.dev/ns#draftValid> "true"^^<http://www.w3.org/2001/XMLSchema#boolean> .
<urn:nl:sparql:drafting:…:attempt:2> <http://www.w3.org/ns/prov#wasRevisionOf> <urn:nl:sparql:drafting:…:attempt:1> .
<urn:script:team-titles-prov> <http://www.w3.org/ns/prov#wasGeneratedBy> <urn:nl:sparql:drafting:…> .
<urn:script:team-titles-prov> <http://www.w3.org/ns/prov#wasDerivedFrom> <urn:nl:sparql:drafting:…:attempt:2> .
```

The same provenance is written into the draft itself, as comment lines above the query, so it
travels with the text into the version a person publishes.

## A draft is its author's

To anyone else, even a caller holding the same namespace, a draft does not exist. `reviewer` may
read and run every `team-` script, and is told there is no such script; the owner's socket names
nobody, so the owner is root there and sees every draft:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/reviewer -c 'source urn:script:team-open-p2'
error: not found: no script is published at urn:script:team-open-p2
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:catalog'
urn:script:open-by-label  published public  sha256:…
urn:script:team-open-p2  draft  sha256:…
urn:script:team-titles  draft  sha256:…
urn:script:team-titles-prov  draft  sha256:…
[uncacheable]
```

A draft cannot be run either, by anyone. Its author asks first, then the owner:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/bot -c 'source urn:script:team-titles:result'
error: conflict: urn:script:team-titles is a draft; publish it (`state=published`) to run it
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:team-titles:result'
error: conflict: urn:script:team-titles is a draft; publish it (`state=published`) to run it
```

Both are told the same true thing: it is a draft, and publishing it is how to run it. Only a caller
who may see the draft hears that much; to `reviewer` it does not exist.

## A person publishes

`bot` holds the namespace's publish grant, and still cannot publish, because its door is QUIC:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/bot -c 'source urn:script:team-titles | sink urn:script:team-titles language=sparql state=published'
error: denied: a client certificate may save `urn:script:team-titles` as a DRAFT (`state=draft`) and may not publish it: a machine drafts, a person publishes (sign in with a passkey, or use the owner's socket, which sees every draft)
[uncacheable]
```

**A machine drafts, a person publishes.** One grant table serves certificates and passkeys alike,
so the rule cannot live in the grant; it lives at the door, and the QUIC door is the one whose
callers are machines. A machine's draft is also its own, so of the people, only the owner (at the
socket, which sees every draft) can publish it; `ann` could publish a draft SHE made.

The owner reviews `team-open-p2`, writes the status as the IRI it is and the bound into the text,
and checks the result before publishing it. The check is the drafter's own, dry run included, with
`focus=` naming the graph:

<!-- transcript: file team-open-p2.rq -->
```sparql
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title
WHERE {
  GRAPH <urn:iki:ledger:graph:default> {
    ?item a ledger:Item ;
          ledger:status ledger:open ;
          ledger:priority ?priority ;
          ledger:number ?number ;
          dcterms:title ?title .
    FILTER (?priority <= 2)
  }
}
ORDER BY ?number
```

<!-- transcript: run -->
```console
$ cat team-open-p2.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:nl:sparql:check focus=urn:iki:ledger:graph:default'
valid select over <urn:iki:ledger:graph:default>
requires: urn:cap:store:read:graph:urn:iki:ledger:graph:default
preview (LIMIT 5):
?number	?title
"1"^^<http://www.w3.org/2001/XMLSchema#integer>	"Passkey list and revoke"
"2"^^<http://www.w3.org/2001/XMLSchema#integer>	"Dependency floors never re-resolved"
[uncacheable]
```

Rows, where the draft had none. Publishing it under the draft's name replaces the draft, and
`public=true` lets a signed-out caller run it:

<!-- transcript: run -->
```console
$ cat team-open-p2.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:script:team-open-p2 language=sparql state=published public=true'
urn:script:team-open-p2:version:sha256:…
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/reviewer -c 'source urn:script:team-open-p2:result as=text/csv'
number,title
1,Passkey list and revoke
2,Dependency floors never re-resolved
[uncacheable]
```

`reviewer`, who was told a minute ago that the script did not exist, runs it. And a published
script is never replaced by a draft: drafting under its name now is refused before any model is
asked.

<!-- transcript: run -->
```console
$ printf 'all the open items' | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/bot -c 'sink urn:nl:sparql save=team-open-p2 focus=urn:iki:ledger:graph:default'
error: invalid argument `save`: urn:script:team-open-p2 is a published script, and a draft saved there would replace it; choose another name
$ grep -c ': all the open items$' model.log
0
```

## The same, signed in

`ann` does all of this over HTTP, with the session cookie her passkey's sign-in set (see [The
ledger in a browser](../ledger/browser.md) for the ceremony). These commands are the HTTP spelling
of what `bot` and the owner did above; the body of a `POST` is the ask or the query, and every
argument is a query argument:

<!-- transcript: manual — needs a session cookie, which only a passkey ceremony in a browser can mint -->
```console
$ curl -X POST -b 'gonk_session=…' --data-binary 'the titles of the open items' 'http://localhost:1070/nl/sparql?save=team-ann-titles&focus=urn:iki:ledger:graph:default'
urn:script:team-ann-titles:version:sha256:… (a DRAFT of urn:script:team-ann-titles: review it, then publish it with state=published)
…
$ curl -X POST -b 'gonk_session=…' --data-binary 'the titles of the open items' 'http://localhost:1070/nl/sparql?save=team-ann-titles2&focus=urn:iki:ledger:graph:default&as=application/json'
$ curl -X POST -b 'gonk_session=…' --data-binary 'the titles of the open items' 'http://localhost:1070/nl/sparql?save=team-ann-titles3&focus=urn:iki:ledger:graph:default&as=text/turtle'
$ curl -b 'gonk_session=…' 'http://localhost:1070/script/team-ann-titles'
$ curl -X POST -b 'gonk_session=…' --data-binary @team-ann-titles.rq 'http://localhost:1070/script/team-ann-titles?language=sparql&state=published'
$ curl -b 'gonk_session=…' 'http://localhost:1070/script/team-ann-titles/result?as=text/csv'
```

Draft, the attempts, the provenance, review, publish, run. She may publish because her draft is
her own and her door is not a machine's; she could not publish `bot`'s.

## What each door may do

Everything above, in one table:

| what | HTTP, signed out | HTTP, signed in | QUIC (a machine) | the socket (the owner) |
| --- | --- | --- | --- | --- |
| run and read a PUBLIC script | **yes**, within the shared budget | yes | with `--script public=run` | yes |
| run and read a named script | no | per `--script` | per `--script` | yes |
| publish, replace, retire | no | per `--script … =publish` | **no**: drafts only | yes |
| see a draft | no | its own | its own | every draft |
| draft from a sentence | **no**, before any model call | with `--nl draft` | with `--nl draft` | yes |
| what a run may read | the least of: the runner's grant, what the publisher held at publish, and gonk's ceiling (the ledgers' reads) | | | |

## What you built

Nothing new, and that is the point. The **atoms** are the scripts (plain files, each version
content-addressed, each recording what its publisher held) and the ledger's graph. Every
answer on this page is a composition the libraries already define — a script's result, a draft
from the grounding and the model door — and gonk's own contribution is the line between them and
each caller: which door, which principal, which grant, and the two rules no grant can carry (a
machine drafts, a person publishes; a draft is its author's).

[Two teams, one gonk](../teams/two-teams.md#a-report-one-team-publishes) publishes a report under
one team's grant: the other team may run every report in the namespace and still cannot run that
one, and a report over both teams' ledgers is refused at publish.
