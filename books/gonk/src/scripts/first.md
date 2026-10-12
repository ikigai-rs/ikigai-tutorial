# Your first saved query

The ikigai Book's [Your first script](../../code/first.html) published a query at a name, and
read the name to run it. gonk does the same for its ledger: a saved query is a page anybody on
this machine can open, and it stays right as the ledger changes. Two steps, the first by hand and
the second from a sentence. Nothing here needs more than [Over the socket](../agent/socket.md).

## Start a model, then gonk

The second step asks a language model. This page uses a stand-in, `model-stub` from this book's
`crates/model-stub`, which answers each question with the same query every time, so the page
prints the same thing on every machine. Start it first:

<!-- transcript: serve -->
```console
$ model-stub "$PWD/model.sock"
model-stub: urn:llm:ask and urn:llm:config on …/model.sock (model stub)
```

Tell gonk where the model is, in its config home, and start a scratch gonk, as in [A ledger of
your own](../ledger/your-own.md):

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.mount = "prefer urn:llm:=%s/model.sock"\n' "$PWD" > .config/ikigai/config.toml
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s); anonymous SPARQL budget 1000 ms
…
```

## 1. Your first saved query

The query counts the open items in the default ledger, `open-count.rq`:

<!-- transcript: file open-count.rq -->
```sparql
# How many items are open.
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
SELECT (COUNT(?item) AS ?open) WHERE {
  GRAPH <urn:iki:ledger:graph:default> { ?item ledger:status ledger:open }
}
```

Publish it over the socket, as the owner, at the name `open-count`. `public=true` lets anybody on
this machine run it, signed in or not:

<!-- transcript: run -->
```console
$ cat open-count.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:script:open-count language=sparql public=true'
urn:script:open-count:version:sha256:…
[uncacheable]
```

Its answer is now a page. Open it in a browser:

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open http://localhost:1070/script/open-count/result
```

The browser shows the answer as SPARQL JSON, where the number is the `value`. `as=text/csv` asks
for a plainer face of the same answer, which is what the checks on this page read:

<!-- transcript: run -->
```console
$ curl -s 'http://localhost:1070/script/open-count/result?as=text/csv'
open
0
```

Nothing is open, because nothing has been filed. File an item, and refresh:

<!-- transcript: run -->
```console
$ curl -s -X POST --data-binary 'Read the first page' http://127.0.0.1:1070/iki/ledger/append
#1 urn:iki:ledger:default:item:…
$ curl -s 'http://localhost:1070/script/open-count/result?as=text/csv'
open
1
```

Nobody told the query that the ledger changed. gonk remembers an answer only until something it
read is written, so the refresh ran the query again. On your own gonk the page is
`http://localhost:1060/script/open-count/result`, and it is right every time you open it.

## 2. Ask gonk in a sentence

Now let the model write the query. `focus=` says which graph the question is about, and `save=`
the name to keep the answer at.

<!-- transcript: run -->
```console
$ printf 'How many open items are there?' | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:nl:sparql save=how-many-open focus=urn:iki:ledger:graph:default'
urn:script:how-many-open:version:sha256:… (a DRAFT of urn:script:how-many-open: review it, then publish it with state=published)
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
SELECT (COUNT(?item) AS ?count) WHERE {
  GRAPH <urn:iki:ledger:graph:default> { ?item ledger:status ledger:open }
}
preview (LIMIT 5):
?count
"1"^^<http://www.w3.org/2001/XMLSchema#integer>
[uncacheable]
```

The answer is a **draft**: the model's query, checked (it parses, and reads only what you may
read), run once as a preview that already says 1, and saved at `how-many-open`, where nobody can
run it yet. Look at it. It is the query you wrote by hand in step 1. Publishing it is a person's
act: the draft's own text, sunk again with `state=published`, and `public=true` as before:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:how-many-open | sink urn:gk:script:how-many-open language=sparql state=published public=true'
urn:script:how-many-open:version:sha256:…
[2 uncacheable]
```

And it is a page, like the one you wrote:

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open http://localhost:1070/script/how-many-open/result
```

<!-- transcript: run -->
```console
$ curl -s 'http://localhost:1070/script/how-many-open/result?as=text/csv'
count
1
```

## Next

[Scripts and sentences in gonk](in-gonk.md) does the same things in depth: who may draft, run and
publish through each of gonk's three doors, why a machine drafts and only a person publishes, and
a draft the check passed that was still wrong.
