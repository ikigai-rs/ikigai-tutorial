# Two teams, one gonk

[A query's authority is its dataset](../../beyond/datasets.html), in The ikigai Book, put two
tenants' graphs in one store and ran one SPARQL text for three callers: each got the answer its
grants allowed, and a join one of them could not have was refused rather than narrowed. That
chapter is the rule, taught with a kernel of its own. This one is the same rule in gonk, where the
tenants are teams, the graphs are their ledgers, and the callers come in through gonk's doors:

| who | door | what they hold |
| --- | --- | --- |
| the owner | the socket | everything: root, as in [Over the socket](../agent/socket.md) |
| `alpha-agent`, team alpha's agent | QUIC | write the `alpha` ledger; run the `report-` scripts |
| `beta-agent`, team beta's agent | QUIC | write the `beta` ledger; run the `report-` scripts |
| `manager` | QUIC | read both ledgers and the repositories; run the `report-` scripts |
| alpha's lead | the socket, sealed to the lead's grant | read `alpha`; publish `report-` scripts |
| anyone on this machine | HTTP, signed out | the `default` ledger, and nothing of either team's |

No part of gonk knows what a team is. Each team's ledger is a named graph in gonk's one store,
every grant is a list of capability tokens, and every answer on this page is a resource the
earlier chapters already used: `next`, the store's query doors, a saved query, the action
manifold. What this page adds is two teams side by side, so that what each one cannot see is as
visible as what it can.

## A repository both teams work in

Both teams change one file, and a code review has something to say about it. The review needs a
model, so the page starts the same stub as [Scripts and sentences in gonk](../scripts/in-gonk.md),
which also answers a review. In a new empty directory, as in [every
chapter](../introduction.md#a-gonk-per-chapter): if the gonk and the `model-stub` from [Scripts and
sentences in gonk](../scripts/in-gonk.md) are still running, stop them first, since this gonk takes
the same port. The stub:

<!-- transcript: serve -->
```console
$ model-stub "$PWD/model.sock" | tee model.log
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
```

<!-- transcript: file shop/src/lib.rs -->
```rust,ignore
/// A name's initials: "Ada Lovelace" is "AL".
pub fn initials(name: &str) -> String {
    name.split(' ').map(|word| &word[..1]).collect()
}
```

Saved as `shop/src/lib.rs`, it becomes a repository, and gonk serves it as a browse root beside
the model's mount, as in [Browse roots](../browse/roots.md):

<!-- transcript: run -->
```console
$ git init -q -b main shop
$ git -C shop add src/lib.rs
$ git -C shop -c user.name=Ada -c user.email=ada@example.com commit -q -m 'initials'
$ mkdir -p .config/ikigai
$ printf 'gonk.browse.root = "shop=%s/shop"\ngonk.mount = "prefer urn:llm:=%s/model.sock"\n' "$PWD" "$PWD" > .config/ikigai/config.toml
```

## The grants

Three identities for the QUIC door, each built from roles, as in [A QUIC
client](../machines/quic-client.md). The scripts are a namespace both teams may run, `report-*`,
which matters later:

<!-- transcript: run -->
```console
$ ikigai-gonk client add alpha-agent --ledger alpha=write --script 'report-*=run' --port 1070 --config-home "$PWD/.config/ikigai"
client `alpha-agent`  …/.config/ikigai/gonk/quic/clients/alpha-agent
…
  enrolled     grant `alpha-agent` (6 scopes) in …/.config/ikigai/gonk/grants.json
…
$ ikigai-gonk client add beta-agent --ledger beta=write --script 'report-*=run' --port 1070 --config-home "$PWD/.config/ikigai"
client `beta-agent`  …/.config/ikigai/gonk/quic/clients/beta-agent
…
  enrolled     grant `beta-agent` (6 scopes) in …/.config/ikigai/gonk/grants.json
…
$ ikigai-gonk client add manager --ledger alpha=read --ledger beta=read --browse read --script 'report-*=run' --port 1070 --config-home "$PWD/.config/ikigai"
client `manager`  …/.config/ikigai/gonk/quic/clients/manager
…
  enrolled     grant `manager` (8 scopes) in …/.config/ikigai/gonk/grants.json
…
$ jq -r 'to_entries[] | .key, (.value[] | "  " + .)' .config/ikigai/gonk/grants.json
alpha-agent
  urn:cap:ledger:read:alpha
  urn:cap:store:read:graph:urn:iki:ledger:graph:alpha
  urn:cap:ledger:write:alpha
  urn:cap:store:write:graph:urn:iki:ledger:graph:alpha
  urn:cap:script:read:report-*
  urn:cap:script:run:report-*
beta-agent
  urn:cap:ledger:read:beta
  urn:cap:store:read:graph:urn:iki:ledger:graph:beta
  urn:cap:ledger:write:beta
  urn:cap:store:write:graph:urn:iki:ledger:graph:beta
  urn:cap:script:read:report-*
  urn:cap:script:run:report-*
manager
  urn:cap:ledger:read:alpha
  urn:cap:store:read:graph:urn:iki:ledger:graph:alpha
  urn:cap:ledger:read:beta
  urn:cap:store:read:graph:urn:iki:ledger:graph:beta
  urn:cap:browse:read:*
  urn:cap:store:read:graph:urn:iki:browse:graph:default
  urn:cap:script:read:report-*
  urn:cap:script:run:report-*
```

Read them as datasets. Each agent's store tokens name one graph, its own ledger's. The manager's
name three: both ledgers and browse's graph, where the review's annotations will land. Nothing
here is a team concept; a team is the set of tokens its ledger's name appears in.

## Start gonk, and fill both ledgers

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read: default; 0 passkey(s), sessions 12 h or 60 min idle; anonymous SPARQL budget 1000 ms
  browse  urn:repo:{shop (watched)}:* — annotations and archive in <urn:iki:browse:graph:default>
…
  quic    udp 0.0.0.0:1070 — 3 trusted certificate(s), 3 enrolled
…
```

The owner files one notice for everyone and two items for each team. A ledger is named in the
resource's IRI (`urn:iki:ledger:alpha:append`), which is what lets a grant bind to it ([Whose
ledger](../ledger/your-own.md#whose-ledger)). One item on each team is `about` the same file:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 The office is closed on Friday' -c 'sink urn:gk:iki:ledger:alpha:append priority=1 about=urn:repo:shop:file:src/lib.rs Initials for names with accents' -c 'sink urn:gk:iki:ledger:alpha:append priority=2 Ship the order form' -c 'sink urn:gk:iki:ledger:beta:append priority=0 about=urn:repo:shop:file:src/lib.rs Initials crash on a double space' -c 'sink urn:gk:iki:ledger:beta:append priority=3 Rotate the supplier keys'
#1 urn:iki:ledger:default:item:…
[uncacheable]
alpha#1 urn:iki:ledger:alpha:item:…
[uncacheable]
alpha#2 urn:iki:ledger:alpha:item:…
[uncacheable]
beta#1 urn:iki:ledger:beta:item:…
[uncacheable]
beta#2 urn:iki:ledger:beta:item:…
[uncacheable]
— batch: 5 commands · 5 uncacheable
```

## Each team's next

Each agent asks for its own ledger's `next`, and asks for the other's:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:iki:ledger:alpha:next'
 1. alpha#1  open    p1  Initials for names with accents
    p1 — priority 1; last updated …
 2. alpha#2  open    p2  Ship the order form
    p2 — priority 2; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 0
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/beta-agent -c 'source urn:iki:ledger:beta:next'
 1. beta#1  open    p0  Initials crash on a double space
    p0 — priority 0; last updated …
 2. beta#2  open    p3  Rotate the supplier keys
    p3 — priority 3; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 0
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:iki:ledger:beta:next'
error: denied: this capability does not hold `urn:cap:ledger:read:beta`. A ledger grant names exactly one ledger — holding a grant over another ledger satisfies the declared family `urn:cap:ledger:read:*` but not this resource, which is the point of naming them
```

The two `next`s are one resource family, `urn:iki:ledger:{ledger}:next`, with one policy, and the
name is the only difference: the ledger checks it against the caller's grant before it reads
anything. A bare `urn:iki:ledger:next` is `default`'s, which neither agent holds, so an agent of
a named ledger always says which ledger it means.

## One text, four answers

`next` is the ledger's own question. A question the ledger did not anticipate is SPARQL, and here
the ikigai Book's chapter applies directly. Here is one, with no graph in it:

<!-- transcript: file open.rq -->
```sparql
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?ledger ?title
WHERE {
  GRAPH ?ledger {
    ?item a ledger:Item ; ledger:status ledger:open ;
          ledger:priority ?priority ; dcterms:title ?title .
  }
}
ORDER BY ?priority ?ledger
```

Over QUIC, gonk's `urn:sparql:select` takes a query with no `graph=` over the union of every
graph the caller may read ([Asking in SPARQL](../agent/socket.md#asking-in-sparql)). The same file,
piped by each identity in turn (a batch's stdin fills a `source` whose one required argument,
`query`, is unfilled):

<!-- transcript: run -->
```console
$ cat open.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:sparql:select as=text/csv'
ledger,title
urn:iki:ledger:graph:alpha,Initials for names with accents
urn:iki:ledger:graph:alpha,Ship the order form
[uncacheable]
$ cat open.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/beta-agent -c 'source urn:sparql:select as=text/csv'
ledger,title
urn:iki:ledger:graph:beta,Initials crash on a double space
urn:iki:ledger:graph:beta,Rotate the supplier keys
[uncacheable]
$ cat open.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/manager -c 'source urn:sparql:select as=text/csv'
ledger,title
urn:iki:ledger:graph:beta,Initials crash on a double space
urn:iki:ledger:graph:alpha,Initials for names with accents
urn:iki:ledger:graph:alpha,Ship the order form
urn:iki:ledger:graph:beta,Rotate the supplier keys
[uncacheable]
$ curl -s -G -H 'Accept: text/csv' --data-urlencode query@open.rq http://127.0.0.1:1070/sparql
ledger,title
urn:iki:ledger:graph:default,The office is closed on Friday
```

Four answers, all correct, and nobody checked the text against a list of permitted graphs. The
fourth is the signed-out HTTP caller's, whose union is the one ledger the HTTP door grants
anonymously, `default`. (`--data-urlencode query@open.rq` is curl's way of sending a file's
content as the `query` parameter.)

## The manager's join

Two teams working on one function is worth knowing before either merges. Which items, in the two
ledgers, are about the same thing?

<!-- transcript: file overlap.rq -->
```sparql
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?about ?alpha ?beta
WHERE {
  GRAPH <urn:iki:ledger:graph:alpha> { ?a ledger:about ?about ; dcterms:title ?alpha }
  GRAPH <urn:iki:ledger:graph:beta> { ?b ledger:about ?about ; dcterms:title ?beta }
}
```

The manager holds both graphs and gets the join. Team alpha's agent names the same dataset, both
ledgers, and is refused, naming the grant it lacks:

<!-- transcript: run -->
```console
$ cat overlap.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/manager -c 'source urn:sparql:select as=text/csv graph="urn:iki:ledger:graph:alpha urn:iki:ledger:graph:beta"'
about,alpha,beta
urn:repo:shop:file:src/lib.rs,Initials for names with accents,Initials crash on a double space
[uncacheable]
$ cat overlap.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:sparql:select as=text/csv graph="urn:iki:ledger:graph:alpha urn:iki:ledger:graph:beta"'
error: denied: reading <urn:iki:ledger:graph:beta> through `urn:iki:store:graph-select` needs the grant `urn:cap:store:read:graph:urn:iki:ledger:graph:beta`, which this capability does not hold. Every graph a scoped read names must be granted: the read is refused rather than answered over the graphs that are, because a narrower dataset than the one asked for returns rows that look right and are not. A grant names exactly one graph; holding `urn:cap:store:read` does not imply it, and is instead the authority for `urn:iki:store:select` over the whole dataset.
```

That is [Refuse, never narrow](../../beyond/datasets.html#refuse-never-narrow), through gonk's
QUIC door. Now the same agent asks without `graph=`, leaving the dataset to its grant:

<!-- transcript: run -->
```console
$ cat overlap.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:sparql:select as=text/csv'
error: invalid argument `query`: this query names <urn:iki:ledger:graph:beta> in a `GRAPH` block, and it is not in this read's dataset, `graph=urn:iki:ledger:graph:alpha`. `urn:iki:store:graph-select` reads exactly the graphs `graph=` names, so that block would match nothing; the query is refused rather than answered empty, because an empty answer to a mistyped or forgotten graph looks exactly like an empty graph. Nothing was evaluated. Correct the IRI, or add the graph to `graph=` (separated by whitespace) under a grant for it
       `query`: A SPARQL SELECT query.
```

**Refused again, by the graph's name.** The dataset is alpha's graph alone, and the `GRAPH
<…beta>` block names a graph outside it. Evaluated, that block would match nothing, and "no
overlap" is the answer alpha's agent would act on: a statement about its grant, not about the
teams. So the store does not evaluate it. Since `ikigai-store` 0.2.22 (the gonk this book pins
links 0.2.23) a query that names a graph outside its dataset is refused, naming the graph, as the
ikigai Book's [escape table](../../beyond/datasets.html#confinement-by-construction) shows; the
gonk this book pinned before linked 0.2.21, which answered the same query with an empty table.
The refusal is not the one before it: that one was a grant `graph=` asked for and alpha does not
hold, this one is a graph the text names and the dataset does not include. Both stop before
anything is read.

## Joined with the code review

The manager's third graph is browse's, where a published review finding becomes an annotation on
the line it is about. The owner runs one review pass over the file with the stub, as in [One-off
review](../review/one-off.md), and publishes the serious finding, which a person would do with a
reproduction in hand:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:shop:review:src/lib.rs'
review by stub · review-v5@stub · 2 findings · reviewed the whole file (142 bytes)
--- annotations (2) ---
L2 [review:stub] "pub fn initials(name: &str) -> String {" -- Returning a String allocates on every call; return a &str instead.
L3 [review:stub] "name.split(' ').map(|word| &word[..1]).collect()" -- Slicing the first byte panics on an empty word, so a name with two spaces in a row, or an empty name, crashes the caller.
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:finding:a385f29c06511574fc4be921 decision=publish reproduced=yes initials("Ada  Lovelace") panics: the empty middle word has no byte 0.'
urn:iki:annotation:a385f29c06511574fc4be921
[uncacheable]
```

The finding's id is a hash of the pass and the quote, so it is the same on every machine that
reviews these bytes. Now the question the manager actually has: what work, in either team, is
about a line someone has published a finding on?

<!-- transcript: file reviewed.rq -->
```sparql
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
PREFIX ik: <https://ikigai-rs.dev/ns#>
PREFIX oa: <http://www.w3.org/ns/oa#>
SELECT ?ledger ?title ?line
WHERE {
  GRAPH ?ledger { ?item ledger:about ?file ; dcterms:title ?title }
  ?note a oa:Annotation ; ik:annotates ?file ; oa:hasSelector ?at .
  ?at oa:exact ?line .
}
ORDER BY ?ledger
```

<!-- transcript: run -->
```console
$ cat reviewed.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/manager -c 'source urn:sparql:select as=text/csv graph="urn:iki:ledger:graph:alpha urn:iki:ledger:graph:beta urn:iki:browse:graph:default"'
ledger,title,line
urn:iki:ledger:graph:alpha,Initials for names with accents,name.split(' ').map(|word| &word[..1]).collect()
urn:iki:ledger:graph:beta,Initials crash on a double space,name.split(' ').map(|word| &word[..1]).collect()
[uncacheable]
$ cat reviewed.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:sparql:select as=text/csv graph="urn:iki:ledger:graph:alpha urn:iki:browse:graph:default"'
error: denied: reading <urn:iki:browse:graph:default> through `urn:iki:store:graph-select` needs the grant `urn:cap:store:read:graph:urn:iki:browse:graph:default`, which this capability does not hold. Every graph a scoped read names must be granted: the read is refused rather than answered over the graphs that are, because a narrower dataset than the one asked for returns rows that look right and are not. A grant names exactly one graph; holding `urn:cap:store:read` does not imply it, and is instead the authority for `urn:iki:store:select` over the whole dataset.
```

Three graphs in one dataset, two teams' work and one review, joined on the file's IRI that the
ledger's `about` and browse's `ik:annotates` both name. That join is why gonk keeps the ledgers
and browse in one store ([The embedding chapter](../embedding/own-kernel.md#the-join) builds it in
a kernel of your own). Alpha's agent may ask its half of it only when it holds the review's graph
too.

## A report one team publishes

A saved query is published once and run by other people later, so its authority cannot come from
whoever runs it; it is derived from the graphs its text names, and it is never more than its
publisher held ([Who a script runs as](../../code/authority.html)). Team alpha's lead publishes
one. A lead is a person, and would sign in with a passkey and publish over HTTP as `ann` did in
[Scripts and sentences in gonk](../scripts/in-gonk.md#the-same-signed-in); this page cannot
perform a passkey ceremony, so it does what that chapter did, at the owner's socket, sealed to
exactly the lead's grant: alpha's ledger read, and publishing in `report-`. What the seal holds is
what the publish records.

<!-- transcript: file report-alpha-open.rq -->
```sparql
# Team alpha's open items, lowest number first.
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title
FROM <urn:iki:ledger:graph:alpha>
WHERE {
  ?item a ledger:Item ; ledger:status ledger:open ;
        ledger:number ?number ; dcterms:title ?title .
}
ORDER BY ?number
```

<!-- transcript: run -->
```console
$ cat report-alpha-open.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'cap seal urn:cap:ledger:read:alpha urn:cap:store:read:graph:urn:iki:ledger:graph:alpha urn:cap:script:read:report-* urn:cap:script:run:report-* urn:cap:script:write:report-* urn:cap:script:delete:report-*' -c 'sink urn:gk:script:report-alpha-open language=sparql public=true'
sealed — capability: urn:cap:ledger:read:alpha, urn:cap:script:delete:report-*, urn:cap:script:read:report-*, urn:cap:script:run:report-*, urn:cap:script:write:report-*, urn:cap:store:read:graph:urn:iki:ledger:graph:alpha · sealed (this is the floor: `cap reset`, `logout` and `login` cannot rise above it)
urn:script:report-alpha-open:version:sha256:…
[uncacheable]
— batch: 2 commands · uncacheable
```

All three identities hold `run` over `report-*`, so the namespace is not what tells them apart:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:script:report-alpha-open:result as=text/csv'
number,title
1,Initials for names with accents
2,Ship the order form
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/manager -c 'source urn:script:report-alpha-open:result as=text/csv'
number,title
1,Initials for names with accents
2,Ship the order form
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/beta-agent -c 'source urn:script:report-alpha-open:result as=text/csv'
error: denied: capability does not grant `urn:cap:store:read:graph:urn:iki:ledger:graph:alpha` (declared by `urn:script:report-alpha-open:result`)
```

**A run grant is not a read grant.** Beta's agent may run every `report-` script, and this one
reads alpha's graph, which its grant does not name; the script's own description declares that
token, so the kernel refuses before the script host runs anything.

The ceiling runs the other way too. Alpha's lead tries to publish a report over both teams'
ledgers:

<!-- transcript: file report-everyone.rq -->
```sparql
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?title
FROM <urn:iki:ledger:graph:alpha>
FROM <urn:iki:ledger:graph:beta>
WHERE { ?item a ledger:Item ; dcterms:title ?title }
```

<!-- transcript: run -->
```console
$ cat report-everyone.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'cap seal urn:cap:ledger:read:alpha urn:cap:store:read:graph:urn:iki:ledger:graph:alpha urn:cap:script:read:report-* urn:cap:script:run:report-* urn:cap:script:write:report-* urn:cap:script:delete:report-*' -c 'sink urn:gk:script:report-everyone language=sparql'
sealed — capability: urn:cap:ledger:read:alpha, urn:cap:script:delete:report-*, urn:cap:script:read:report-*, urn:cap:script:run:report-*, urn:cap:script:write:report-*, urn:cap:store:read:graph:urn:iki:ledger:graph:alpha · sealed (this is the floor: `cap reset`, `logout` and `login` cannot rise above it)
error: denied: a script cannot be given more than its publisher holds, and this capability does not hold urn:cap:store:read:graph:urn:iki:ledger:graph:beta — declare less, or publish under a capability that holds it
```

**No elevation.** A script is not a way to hand someone else's authority to a third party: if the
lead could publish it, the manager running it would be running beta's read under alpha's name.
The refusal is at publish, so no such script exists for anyone to run.

## What a signed-out caller sees

The HTTP door serves anyone on this machine without an account, under the anonymous grant: the
ledgers in `gonk.http.ledger`, `default` unless configured. Neither team's ledger is in it:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/l/alpha
denied: Sign in to read the ledger "alpha". Sign in with passkey is in the header of every gonk page (the front page is /); then come back to this address.
The grant it needs: urn:cap:ledger:read:alpha (to read the ledger "alpha")
$ curl -s -G -H 'Accept: text/csv' --data-urlencode 'query=SELECT ?g (COUNT(*) AS ?n) WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?g' http://127.0.0.1:1070/sparql
g,n
urn:iki:ledger:graph:default,…
$ curl -s -w ' (%{http_code})\n' 'http://localhost:1070/script/report-alpha-open/result?as=text/csv'
denied: capability does not grant `urn:cap:store:read:graph:urn:iki:ledger:graph:alpha` (declared by `urn:script:report-alpha-open:result`) (403)
```

The lead published the report `public=true`, and a signed-out caller still cannot run it.
`public` says **who may run** a script, the anonymous grant's `urn:cap:script:run:public`; what a
run may **read** is the runner's own grant, and the anonymous one does not name alpha's graph. A
public report over the `default` ledger would answer, as `open-by-label` did in [Scripts and
sentences in gonk](../scripts/in-gonk.md#a-saved-query-for-a-person-and-for-a-worker).

## Deleted is gone, unless you hold the graveyard

A delete is a move into the ledger's own graveyard graph, recoverable by hand ([Over the
socket](../agent/socket.md#whose-authority)). The owner deletes alpha's second item:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'delete urn:gk:iki:ledger:alpha:item:2'
deleted alpha#2 urn:iki:ledger:alpha:item:…
  7 quad(s) moved to <urn:iki:ledger:graph:alpha:deleted> and recoverable
  tombstone: urn:iki:ledger:alpha:tombstone:…
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:iki:ledger:alpha:next'
 1. alpha#1  open    p1  Initials for names with accents
    p1 — priority 1; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 1   excluded: 0
[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:iki:ledger:alpha:item:2'
error: not found: alpha#2 was deleted: its tombstone is `urn:iki:ledger:alpha:tombstone:…` and its content, unless purged, is in `urn:iki:ledger:graph:alpha:deleted`
```

Alpha's agent is told the item was deleted and where its content went, which is the ledger's own
answer for a name it once served. The graveyard is a graph like any other, so the question of who sees it is the dataset question
again. The same count of quads per graph, at root and as the manager:

<!-- transcript: file graphs.rq -->
```sparql
SELECT ?g (COUNT(*) AS ?n) WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?g ORDER BY ?g
```

<!-- transcript: run -->
```console
$ cat graphs.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:sparql:select as=text/csv'
g,n
urn:iki:browse:graph:default,…
urn:iki:ledger:graph:alpha,…
urn:iki:ledger:graph:alpha:deleted,7
urn:iki:ledger:graph:beta,…
urn:iki:ledger:graph:default,…
[uncacheable]
$ cat graphs.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/manager -c 'source urn:sparql:select as=text/csv'
g,n
urn:iki:browse:graph:default,…
urn:iki:ledger:graph:alpha,…
urn:iki:ledger:graph:beta,…
[uncacheable]
$ cat graphs.rq | ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/manager -c 'source urn:sparql:select as=text/csv graph=urn:iki:ledger:graph:alpha:deleted'
error: denied: reading <urn:iki:ledger:graph:alpha:deleted> through `urn:iki:store:graph-select` needs the grant `urn:cap:store:read:graph:urn:iki:ledger:graph:alpha:deleted`, which this capability does not hold. Every graph a scoped read names must be granted: the read is refused rather than answered over the graphs that are, because a narrower dataset than the one asked for returns rows that look right and are not. A grant names exactly one graph; holding `urn:cap:store:read` does not imply it, and is instead the authority for `urn:iki:store:select` over the whole dataset.
```

Reading alpha's ledger is not reading what alpha deleted. The token that is, is part of the
ledger's strongest role:

<!-- transcript: run -->
```console
$ ikigai-gonk grants alpha purge | grep deleted
  "urn:cap:store:write:graph:urn:iki:ledger:graph:alpha:deleted",
  "urn:cap:store:read:graph:urn:iki:ledger:graph:alpha:deleted"
```

`delete` writes the graveyard (a delete moves quads into it); `purge` reads it too, because
purging an item means finding its quads there. A role below `purge` sees a deleted item only as
the ledger's own word for it, its tombstone.

## Two tool lists, and a third

An agent's tools are the action manifold under its capability, `urn:kernel:actions`; `ikigai
mcp` projects exactly that list as MCP tools ([Over MCP](../agent/mcp.md)). Each identity asks for
its own, and `comm` compares them:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/alpha-agent -c 'source urn:kernel:actions' 2>/dev/null | LC_ALL=C sort > alpha.tools
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/beta-agent -c 'source urn:kernel:actions' 2>/dev/null | LC_ALL=C sort > beta.tools
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/manager -c 'source urn:kernel:actions' 2>/dev/null | LC_ALL=C sort > manager.tools
$ LC_ALL=C comm -3 alpha.tools beta.tools
urn:script:report-alpha-open:result
urn:script:report-alpha-open:runs
$ LC_ALL=C comm -23 alpha.tools manager.tools
urn:iki:ledger:{ledger}:append
urn:iki:ledger:{ledger}:claim
urn:iki:ledger:{ledger}:close
urn:iki:ledger:{ledger}:comment
urn:iki:ledger:{ledger}:defer
urn:iki:ledger:{ledger}:item:{id}:about
urn:iki:ledger:{ledger}:label
urn:iki:ledger:{ledger}:link
urn:iki:ledger:{ledger}:reopen
urn:iki:store:graph-update
$ LC_ALL=C comm -13 alpha.tools manager.tools | grep -c '^urn:repo:shop:'
19
```

Three things to read in that:

- **Alpha's and beta's lists differ by one report.** The ledger's actions are templates over
  `{ledger}`, declared against the family (`urn:cap:ledger:write:*`), so both writers are offered
  the same ledger tools, and the ledger checks the name each call carries. Alpha's report is in
  alpha's list and not in beta's, because its description declares alpha's graph: the manifold
  offers a published query only to a caller who could run it, which is the ikigai Book's
  [Authority from the text](../../beyond/datasets.html#authority-from-the-text) in gonk.
- **The manager's list has no writer's tools.** Reading two ledgers is not writing either, so
  `append`, `claim`, `close` and the store's update door are absent, not offered and refused.
- **The manager's list has the repository's.** Nineteen of the browse root's resources, which
  neither agent holds a token for.

So an MCP agent built on alpha's certificate is not told what beta's report is called, and an
agent built on the manager's cannot be talked into filing anything: the tool it would need is not
in its list.

## What you have now

| the question | what answered it | shown by |
| --- | --- | --- |
| what is next for my team? | `urn:iki:ledger:{ledger}:next`, under a grant naming the ledger | each agent's `next`, and the other's refused |
| what may a query read? | the union of the graphs the caller holds, or the `graph=` it names | one text, four answers |
| what if one graph is not granted? | refused, naming the grant, or naming the graph if it is only in the text | the manager's join, and alpha's two tries |
| can the teams' work be joined with the review? | yes, for a caller holding all three graphs | the review join |
| who may run a published report? | whoever holds both its run grant and the graphs it reads | `report-alpha-open`, three runners |
| may a report read more than its publisher? | no, refused at publish | `report-everyone` |
| what does a signed-out caller see? | the anonymous ledgers, and a public report only over them | the HTTP door |
| who sees a deleted item? | a caller holding the graveyard's read token (`purge`), or root | the counts per graph |
| what is an agent offered? | the manifold under its grant | the three tool lists |

In resource terms: the **sets** are the ledgers, their items and the reports; the **atoms** that
hold state are one named graph per ledger, one graveyard graph per ledger, browse's graph, and the
scripts' files; everything else is a **composition** asked under the caller's capability, and every
**view** above is one of them. The code written for two teams was none: three `client add` lines,
one sealed publish, and the names of two ledgers.
