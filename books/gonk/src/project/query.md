# Ask the project in SPARQL

`next` answers one question. Everything else a project wants to know, like how much of each change is
left or which requirements have work against them, is a query over the graphs gonk holds. A query you
keep is a **script**: published at a name, versioned by its content, and run by reading that name.
None of this chapter needs a model.

## Catch up

In a new empty directory, as in [Start a project](start.md). `gonk.http.ledger` lets a signed-out
browser on this machine read the project's ledger; without it, only `default` is readable that way. If
the gonk from [the previous chapter](two-agents.md) is still running, stop it first, since this one
takes the same port.

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.http.ledger = "library"\n' > .config/ikigai/config.toml
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read: library; …
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

## The queries you already have

`init` published what `.gonk/queries/` and `.gonk/scripts/` hold, each under the ledger's name, so one
gonk can hold many repositories' scripts. A file says `LEDGER` where it names its graph, and `init`
fills that in.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:catalog'
urn:script:library-next-for-artifact  published  sha256:…
urn:script:library-open-items-for-artifact  published  sha256:…
urn:script:library-open-work-by-change  published  sha256:…
urn:script:library-requirements-with-open-work  published  sha256:…
[uncacheable]
$ cat library/.gonk/queries/open-work-by-change.rq
# How much of each change in flight is still open: open and closed items per change.
# `seed` labels every item `change:<id>` for the OpenSpec change its task belongs to.
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
SELECT ?change (SUM(?isOpen) AS ?open) (COUNT(?item) AS ?items) WHERE {
  GRAPH <urn:iki:ledger:graph:LEDGER> {
    ?item a ledger:Item ; ledger:status ?status ; ledger:label ?label .
    FILTER(STRSTARTS(?label, 'change:'))
  }
  BIND(STRAFTER(?label, 'change:') AS ?change)
  BIND(IF(?status = ledger:open, 1, 0) AS ?isOpen)
}
GROUP BY ?change
ORDER BY DESC(?open) ?change
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-open-work-by-change:result as=text/csv'
change,open,items
add-holds,6,6
cap-late-fees,2,2
[uncacheable]
```

Reading `:result` runs it, and `as=text/csv` picks a plain face of the answer. A parameter is declared
in the file's leading comment (`# @param artifact xsd:string`) and passed as an argument:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-open-items-for-artifact:result artifact=cli as=text/csv'
number,title,priority,holder
1,add-holds 1.1: Accept a hold on a title whose copies are all out,,
2,add-holds 1.2: Keep a returned copy for the first holder,,
3,add-holds 1.3: Refuse a renewal while a hold waits,,
4,add-holds 1.4: Add a vector to openspec/vectors.json for every new scenario,,
5,add-holds 2.1: Add `library.py hold MEMBER ISBN`,,
6,add-holds 2.2: Show who a kept copy waits for in `library.py copy`,,
[uncacheable]
```

## Write your own, keep it in the repository

Which tasks has nobody taken yet, among those ready to start? A query is a file in `.gonk/queries/`,
so it is reviewed, versioned and shared like code. The parameter is declared the same way:

<!-- transcript: file library/.gonk/queries/unclaimed-in-change.rq -->
```sparql
# The open, unclaimed items of one change, and whether another open item blocks each.
# @param change xsd:string -- a change's id, such as add-holds
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT ?number ?title ?blocked WHERE {
  GRAPH <urn:iki:ledger:graph:LEDGER> {
    ?item a ledger:Item ;
          ledger:status ledger:open ;
          ledger:number ?number ;
          dcterms:title ?title ;
          ledger:label ?label .
    FILTER(?label = CONCAT('change:', ?change))
    FILTER NOT EXISTS { ?item ledger:claimedBy ?holder }
    BIND(EXISTS { ?other ledger:blocks ?item ; ledger:status ledger:open } AS ?blocked)
  }
}
ORDER BY ?number
```

Running `init` again publishes every file under those directories, and says so:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py init | grep 'scripts   published')
  ok       scripts   published 5: urn:script:library-open-items-for-artifact, urn:script:library-open-work-by-change, urn:script:library-requirements-with-open-work, urn:script:library-unclaimed-in-change, urn:script:library-next-for-artifact
$ (cd library && python3 tools/gonk.py claim 7 | cut -d' ' -f1-4)
library#7 claimed by ada
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-unclaimed-in-change:result change=cap-late-fees as=text/csv'
number,title,blocked
8,cap-late-fees 1.2: Add a vector for the capped return to openspec/vectors.json,true
[uncacheable]
```

Item 7 is claimed, so it is gone, and item 8 is still waiting behind it. The answer stays right as the
ledger changes, because gonk keeps an answer only until something it read is written.

## A page anyone on this machine can open

A published script runs for whoever holds a grant for its name. To let a signed-out browser on this
machine run it, mark it public. This republishes the script's own text with `public=true`:

<!-- transcript: run -->
```console
$ curl -s 'http://localhost:1070/script/library-open-work-by-change/result?as=text/csv'
denied: capability does not grant `urn:cap:script:run:library-*` (declared by `urn:script:library-open-work-by-change:result`; any grant under `urn:cap:script:run:library-` satisfies it)
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-open-work-by-change | sink urn:gk:script:library-open-work-by-change language=sparql state=published public=true'
urn:script:library-open-work-by-change:version:sha256:…
[2 uncacheable]
$ curl -s 'http://localhost:1070/script/library-open-work-by-change/result?as=text/csv'
change,open,items
add-holds,6,6
cap-late-fees,2,2
```

## The spec graph

The ledger is one graph. The tree can be others, and that is what `about` was for: an item names the
requirement it implements by IRI, so the two join. The sample's third query does exactly that, and at
first it answers nothing:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-requirements-with-open-work:result as=text/csv'
capability,requirement,open
[uncacheable]
```

Nothing has lifted the tree. The lift is not in gonk
([In gonk, and not in this page](../spec/ledger.md#in-gonk-and-not-in-this-page)): the full one is
`ikigai-markdown` and the mapping in a host of your own, which is what `crates/openspec-ledger` runs.
For this page, a few lines of Python write just what the query reads, using the same IRIs `seed`
derives:

<!-- transcript: file lift.py -->
```python
import re
import sys
from pathlib import Path

OS = "http://example.org/openspec#"


def slug(name):
    return re.sub(r"[^a-z0-9]+", "-", name.lower()).strip("-")


capability = sys.argv[1]
print(f'<urn:openspec:spec:{capability}> <{OS}name> "{capability}" .')
tree = Path("library/openspec")
specs = [tree / "specs" / capability / "spec.md", *tree.glob(f"changes/*/specs/{capability}/spec.md")]
for spec in specs:
    if spec.exists():
        for name in re.findall(r"^### Requirement: (.+)$", spec.read_text(), re.MULTILINE):
            iri = f"<urn:openspec:spec:{capability}:requirement:{slug(name)}>"
            print(f"{iri} a <{OS}Requirement> .")
            print(f'{iri} <{OS}name> "{name}" .')
            print(f"{iri} <{OS}capability> <urn:openspec:spec:{capability}> .")
```

The owner loads one named graph per capability, over the socket:

<!-- transcript: run -->
```console
$ python3 lift.py overdue | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:store:load graph=urn:openspec:graph:overdue format=text/turtle'
loaded text/turtle: … quads
[uncacheable]
$ python3 lift.py holds | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:store:load graph=urn:openspec:graph:holds format=text/turtle'
loaded text/turtle: … quads
[uncacheable]
$ python3 lift.py renewals | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:store:load graph=urn:openspec:graph:renewals format=text/turtle'
loaded text/turtle: … quads
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:store:graphs'
urn:iki:ledger:graph:library
urn:openspec:graph:holds
urn:openspec:graph:overdue
urn:openspec:graph:renewals
[uncacheable]
```

Ask the published script again, and it still answers nothing:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-requirements-with-open-work:result as=text/csv'
capability,requirement,open
[uncacheable]
```

That is gonk's **script ceiling**. Every run here is held to the ledgers' reads, whoever runs it, even
the owner, so a script cannot reach a graph outside them. It sees no spec graph, so it matches no
row; it is not an error, and an empty answer is not evidence that there is no open work.

So ask the store directly. `graph-select` reads exactly the graphs it names, under the caller's own
grants, and the owner holds them all. The same query text, on one line, for the shell:

<!-- transcript: run -->
```console
$ grep -v '^#' library/.gonk/queries/requirements-with-open-work.rq | sed 's/LEDGER/library/' | tr '\n' ' ' > join.rq
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c "source urn:gk:iki:store:graph-select graph=\"urn:openspec:graph:overdue urn:openspec:graph:holds urn:openspec:graph:renewals urn:iki:ledger:graph:library\" as=text/csv query=\"$(cat join.rq)\""
capability,requirement,open
holds,A member may hold a title with no copy on the shelf,2
holds,A returned copy of a held title waits for the first holder,2
overdue,A late return owes a fee,2
renewals,A loan of a held title is not renewed,1
[uncacheable]
```

Each row is a requirement, from the tree, with a count of open items, from the ledger. That join is
what one store buys. Two things follow if you keep this up:

- **It goes stale.** Nothing reloads the graphs when the tree changes. A host that lifts on every
  commit does ([Changes and tasks as ledger items](../spec/ledger.md)).
- **The join belongs to whoever holds the graphs.** `graph-select` needs a read grant for every graph
  it names, and a published script still cannot read them here.

<a id="what-you-built"></a>

## What you have now

Questions kept as files, published under the project's name, and run by reading a name: over a socket,
over QUIC by anyone granted, and as a page. The [next chapter](script.md) goes from questions to
workflow.
