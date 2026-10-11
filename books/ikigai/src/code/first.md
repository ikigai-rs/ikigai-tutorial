# Your first script

This section is about code you can **publish into a running host, at a name**, so that other
people can run it without a release. Before the detail, four small steps, one idea each, and one
command each. Nothing here needs more than the chapters before this one.

The scripts are SPARQL queries, over a shelf of books. Each step is a program in this repository
that prints every request it makes, the way the REPL would write it, and what came back. In the
first two steps **you** are the host, so you may do anything; the third step is where that
changes.

## 1. Hello, script

The smallest script there is, `hello.rq`:

```sparql
{{#include ../../../../crates/script-host/src/first/hello.rq}}
```

It answers one row, and reads nothing. (A script always names the graphs it reads, and the host
checks who may read them; this one names an empty graph and uses nothing in it.) And the same
greeting with a parameter, `hello-who.rq`: the comment line says it takes a `who`, a string,
which is `world` unless the caller says otherwise.

```sparql
{{#include ../../../../crates/script-host/src/first/hello-who.rq}}
```

The program publishes the first at the name `urn:script:hello` and reads its result; then
publishes the second at the same name, and reads it twice:

```bash
cargo run -p script-host -- hello
```

```text
you> sink urn:script:hello language=sparql   (hello.rq, piped)
urn:script:hello:version:sha256:…

you> source urn:script:hello:result as=text/csv
greeting
"Hello, world!"

you> sink urn:script:hello language=sparql   (hello-who.rq, piped)
urn:script:hello:version:sha256:…

you> source urn:script:hello:result as=text/csv
greeting
"Hello, world!"

you> source urn:script:hello:result as=text/csv who=Brian
greeting
"Hello, Brian!"
```

Publishing is a `sink` of the text at the script's name. Running it is a `source` of
`…:result`: an ordinary read, which is why `as=` picks the format, as it does for any resource.
The parameter is an argument of that read, `who=Brian`, like any endpoint's argument. Publishing
again at the same name made a new version, and the name now runs the new one.

## 2. A script over data

The host's store holds a shelf of three books:

```sparql
{{#include ../../../../crates/script-host/src/first/shelf.ru}}
```

<!-- urn-gate: illustration urn:iki:tutorial:shelf — the name of a named GRAPH in the store,
     taken by the store's doors as an argument; no space binds it. -->

and `titles.rq` lists the titles on it:

```sparql
{{#include ../../../../crates/script-host/src/first/titles.rq}}
```

The program publishes it, reads it twice, puts a fourth book on the shelf
(`fourth.ru`, the same kind of `INSERT DATA` with one book in it), and reads it again:

```bash
cargo run -p script-host -- shelf
```

```text
you> sink urn:script:titles language=sparql   (titles.rq, piped)
urn:script:titles:version:sha256:…

you> source urn:script:titles:result as=text/csv
title
Middlemarch
Moby-Dick
Walden
(computed)

you> source urn:script:titles:result as=text/csv
title
Middlemarch
Moby-Dick
Walden
(from the cache: nothing ran)

you> sink urn:iki:store:graph-update graph=urn:iki:tutorial:shelf   (fourth.ru, piped)
updated <urn:iki:tutorial:shelf>: +1 -0 quads

you> source urn:script:titles:result as=text/csv
title
Frankenstein
Middlemarch
Moby-Dick
Walden
(computed)
```

The lines in parentheses are the program asking the kernel, before each read, whether it could
answer without running anything. The second read ran nothing: the answer was remembered. The
third read ran again, and has the fourth book in it. Nobody told the script that the shelf had
changed. It read the shelf through the kernel, so the kernel knew what the answer depended on, and
writing the shelf made the remembered answer stale.

## 3. Who may run it

Now two people, and the host is no longer one of them. Alice may publish `titles` and read the
shelf. Bob may read the shelf too, but nobody has said he may run `titles`:

```bash
cargo run -p script-host -- grant
```

```text
alice> sink urn:script:titles language=sparql   (titles.rq, piped)
urn:script:titles:version:sha256:…

bob> source urn:script:titles:result as=text/csv
error: denied: capability does not grant `urn:cap:script:run:titles` (declared by `urn:script:titles:result`)

host> (bob's grant gains one line: urn:cap:script:run:titles)

bob> source urn:script:titles:result as=text/csv
title
Middlemarch
Moby-Dick
Walden
```

Bob is refused, and the refusal names exactly what he lacks: the grant to run this one script.
The host adds that one line to what Bob holds, and he gets the answer. Publishing, running and
reading a script's text are separate grants, so Alice, who published it, could not run it either.

## 4. Ask in a sentence

A language model can write the query for you. Here the model is a stand-in that answers this
sentence with the same query every time, so the page prints the same thing on every machine. ([A
sentence becomes a draft](sentence.md) shows how to put a real one behind the same door.) The
program asks how many books are on the shelf, then publishes what came back, and runs it:

```bash
cargo run -p sentence-draft -- ask
```

```text
you> sink urn:nl:sparql ask="How many books are on the shelf?" graph=urn:iki:tutorial:shelf save=book-count
urn:script:book-count:version:sha256:… (a DRAFT of urn:script:book-count: review it, then publish it with state=published)

PREFIX dcterms: <http://purl.org/dc/terms/>
SELECT (COUNT(?book) AS ?books) WHERE {
  GRAPH <urn:iki:tutorial:shelf> { ?book dcterms:title ?title }
}

preview (LIMIT 5):
?books
"3"^^<http://www.w3.org/2001/XMLSchema#integer>

you> source urn:script:book-count | sink urn:script:book-count language=sparql state=published
urn:script:book-count:version:sha256:…

you> source urn:script:book-count:result as=text/csv
books
3
```

The model's query came back as a **draft**: saved at the name, visible only to the person who
asked, and run by nobody. Before anyone saw it, it was checked (does it parse, and does it read
only what you may read?) and run once as a preview, which already says 3. Publishing it is a
person's act: the draft's own text, sunk again with `state=published`. After that it is a script
like `hello` and `titles`.

## Next

That is the whole idea: code at a name, run as a read, remembered until what it read changes, run
only by someone who may, and written by a model only as a draft a person publishes. The next three
chapters do the same things on a different shape of data, a handful of work items, in more depth:
[Scripts are resources](scripts.md) names every part of a script, [Who a script runs
as](authority.md) says exactly whose authority a run holds, and [A sentence becomes a
draft](sentence.md) shows how little the model is allowed to do.
