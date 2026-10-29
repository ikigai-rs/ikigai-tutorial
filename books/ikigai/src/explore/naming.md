# Naming a composition

Four chapters in a row worked around the same gap. [Giving the data names](names.md) kept
the filled graph as a file, `quakes.jsonld`, because `graph=` takes a name and a request with
arguments was not one. [Composites, one layer at a time](composites.md) kept `notable.csv`
and `notable.json` for the same reason. [Process scripts are files](scripts.md) replayed a
file of lines to bring those copies up to date. And [A river of feeds](rss.md) wrote its
build line out twice and listed three things that waited on one chapter. This is that
chapter.

The REPL's `bind` gives a request with its arguments a name of its own. The name is read like
any resource, cached like one, and stale when anything it read changes, so a composite at a
shell no longer needs a copy in the middle.

## What you need

The `~/quakes` directory from the earlier chapters, a shell in it, and, for the last
sections, the copy of this book's repository that [A river of feeds](rss.md#what-you-need)
used. As in the last chapter, every step is a shell command: a file of REPL lines replayed
with `ikigai --quiet`, or a few `-c` lines.

```bash
cd ~/quakes
```

<!-- transcripts: workspace crates/quakes/quakes-2024-01-01.geojson crates/quakes/quakes-2024-01-08.geojson books/ikigai/ns/quakes.context.jsonld examples/rss -->
<!-- transcripts: normalize workdir -->

The book's checks run every `$` line on this page against `ikigai-cli`
{{#include ../../ikigai-cli.version}}, in order, in a fresh directory holding the two
recorded weeks, the context and the RSS example, so the answers are the ones you get. `$PWD`
in an answer stands for your own directory. Each `ikigai` command below starts a new REPL, so
nothing is cached from one `$` line to the next. The names are another matter: they are kept
in a file, and that file is what this chapter is about.

<!-- urn-gate: unbound urn:quakes:count — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:quakes:count:csv — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:quakes:strong:count — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:quakes:notable — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:quakes:live — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:river:caching — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:river:search:{q} — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:river:search:rss — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:river:live:{url} — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: unbound urn:river:live:https://alice.example.com/feed.xml — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: illustration urn:quakes:graph — the page kernel of "Giving the data names" binds this name; at a shell it is the one the reader binds on this page, which the replay binds from the page. -->
<!-- urn-gate: illustration urn:quakes:graph:2024-01-08 — the page kernel of "Composites" binds this name; at a shell it is the one the reader binds on this page, which the replay binds from the page. -->

## 1. A name for a request

A binding is one line: a name, `=`, and the request the name stands for, in the words you
would type at the prompt. Each `key=value` in it is **fixed**: the name always sends it. An
argument written `key=<iri>` is a **reference**: the name sends the resource's name, and the
kernel reads it, so the answer hangs from that resource's thread.

The graph is the first thing to name. [Giving the data names](names.md#6-at-a-shell) filled a
template, `graph.txt`, and kept the answer as `quakes.jsonld`, because a file's type comes
from its name and SPARQL needs to know it is reading JSON-LD. `urn:iki:fn:compose` gives its
answer the type of the template it filled. So keep the same template under a `.jsonld` name,
and the filled answer is JSON-LD with no copy in between. Save this as `bind-graphs.iki`:

<!-- transcript: file bind-graphs.iki -->

```text
sink urn:file:templates/week.jsonld {"@context": $r{urn:json:query?src=urn:file:quakes.context.jsonld&q=.[]}, "@graph": [$r{urn:file:quakes-2024-01-01.geojson}]}
sink urn:file:templates/week-2024-01-08.jsonld {"@context": $r{urn:json:query?src=urn:file:quakes.context.jsonld&q=.[]}, "@graph": [$r{urn:file:quakes-2024-01-08.geojson}]}
bind urn:quakes:graph = source urn:iki:fn:compose src=<urn:file:templates/week.jsonld>
bind urn:quakes:graph:2024-01-08 = source urn:iki:fn:compose src=<urn:file:templates/week-2024-01-08.jsonld>
```

`--names names` tells the REPL which file holds its names: here `names`, in this directory.
Without it, the REPL keeps them in its config home, in a file also called `names`, for every
session you start; a file beside the data keeps this part's names with this part.

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names < bind-graphs.iki
wrote 125 bytes to templates/week.jsonld
[uncacheable]
wrote 125 bytes to templates/week-2024-01-08.jsonld
[uncacheable]
bound in <urn:iki:host:names>:
urn:quakes:graph = source urn:iki:fn:compose src=<urn:file:templates/week.jsonld>
[uncacheable]
bound in <urn:iki:host:names>:
urn:quakes:graph:2024-01-08 = source urn:iki:fn:compose src=<urn:file:templates/week-2024-01-08.jsonld>
[uncacheable]
$ cat names
urn:quakes:graph:2024-01-08 = source urn:iki:fn:compose src=<urn:file:templates/week-2024-01-08.jsonld>
urn:quakes:graph = source urn:iki:fn:compose src=<urn:file:templates/week.jsonld>
```

`bind` wrote the file. It is plain text, one binding a line, and you may edit it with an editor
as well: a REPL that is running hears the edit and binds again. (The lines are kept most
specific first, which is the order the names are tried in, not the order you typed them.)

The REPL that bound the names has exited. A new one reads the file and answers them. Ask the
first week how many earthquakes it holds, through the name:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'source urn:json:query src=urn:quakes:graph q=".[\"@graph\"][0].features | length"'
178
[computed]
```

The pages of this part had `urn:quakes:graph` from the start; now your REPL has it too, and it
is your own.

## 2. The questions, by name

The questions are files, as before, and the names read them by reference. Save the count
question from [Giving the data names](names.md#4-the-questions-are-resources), the strong filter
from [Composites](composites.md#1-filter-a-question-whose-answer-is-a-graph), and bindings for
both, as `bind-questions.iki`:

<!-- transcript: file bind-questions.iki -->

```text
sink urn:file:queries/count.rq PREFIX qk: <https://ikigai-rs.dev/ns/quakes#> SELECT (COUNT(?quake) AS ?quakes) WHERE { ?quake a qk:Earthquake }
sink urn:file:queries/strong.rq PREFIX qk: <https://ikigai-rs.dev/ns/quakes#> CONSTRUCT { ?quake a qk:Earthquake ; qk:mag ?mag ; qk:place ?place } WHERE { ?quake a qk:Earthquake ; qk:mag ?mag ; qk:place ?place FILTER(?mag >= 5.5) }
sink urn:file:queries/notable.rq PREFIX qk: <https://ikigai-rs.dev/ns/quakes#> SELECT ?id ?mag ?place WHERE { ?quake qk:mag ?mag ; qk:place ?place FILTER(?mag >= 6) BIND(STRAFTER(STR(?quake), "eventpage/") AS ?id) } ORDER BY DESC(?mag)
bind urn:quakes:count = source urn:sparql:select query=<urn:file:queries/count.rq> graph=urn:quakes:graph
bind urn:quakes:strong = source urn:sparql:construct query=<urn:file:queries/strong.rq> graph=urn:quakes:graph
bind urn:quakes:strong:count = source urn:sparql:select query=<urn:file:queries/count.rq> graph=urn:quakes:strong
bind urn:quakes:notable = source urn:sparql:select query=<urn:file:queries/notable.rq> graph="urn:quakes:graph urn:quakes:graph:2024-01-08"
```

`urn:quakes:strong` is a layer: its answer is a graph, and `urn:quakes:strong:count` asks the
same count question of it. `urn:quakes:notable` reads both weeks; the two names in its `graph`
are one value, so they are quoted. `graph=` is the graph's name, not a reference: `urn:sparql:select`
takes names there and reads each one itself, through the kernel.

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names < bind-questions.iki
wrote 112 bytes to queries/count.rq
[uncacheable]
wrote 199 bytes to queries/strong.rq
[uncacheable]
wrote 202 bytes to queries/notable.rq
[uncacheable]
bound in <urn:iki:host:names>:
urn:quakes:count = source urn:sparql:select graph=urn:quakes:graph query=<urn:file:queries/count.rq>
[uncacheable]
bound in <urn:iki:host:names>:
urn:quakes:strong = source urn:sparql:construct graph=urn:quakes:graph query=<urn:file:queries/strong.rq>
[uncacheable]
bound in <urn:iki:host:names>:
urn:quakes:strong:count = source urn:sparql:select graph=urn:quakes:strong query=<urn:file:queries/count.rq>
[uncacheable]
bound in <urn:iki:host:names>:
urn:quakes:notable = source urn:sparql:select graph="urn:quakes:graph urn:quakes:graph:2024-01-08" query=<urn:file:queries/notable.rq>
[uncacheable]
```

Ask by name:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'source urn:quakes:count as=text/csv' -c 'source urn:quakes:strong:count as=text/csv' -c 'source urn:quakes:notable as=text/csv'
quakes
178
[computed]
quakes
12
[computed]
id,mag,place
us6000m0xl,7.5,"2024 Noto Peninsula, Japan Earthquake"
us6000m2jp,6.7,"93 km SE of Sarangani, Philippines"
us6000m31m,6.4,"45 km SSW of Jurm, Afghanistan"
us6000m0xm,6.2,"8 km SW of Anamizu, Japan"
[computed]
— batch: 3 commands · 3 computed
```

The same three answers the earlier chapters printed, with no `quakes.jsonld`, no
`notable.csv` and nothing to refresh. Each line is one stage, so each tally says
`[computed]` once; the graph and the question file a name reads on its way, it reads through
the kernel, and section 4 shows the kernel keeping track.

## 3. A fixed argument stays fixed

A name stands for its arguments, so the caller cannot change one. Ask `urn:quakes:count` about
the other week:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'source urn:quakes:count graph=urn:quakes:graph:2024-01-08'; echo "exit $?"
error: invalid argument `graph`: fixed by <urn:quakes:count> to urn:quakes:graph
       `graph`: optional: one or more graph source IRIs, comma- or space-separated; each resolved through the kernel, transrepted to RDF when its type is not an RDF syntax and a transreptor converts it, and loaded as a named graph. Omit to query the always-present ikigai vocabulary alone.
exit 1
```

Refused, and the error names the binding that fixed it and the value it fixed. A count of
the other week is a different question, so it is a different name. What a binding leaves out,
the caller still gives: `as=text/csv` above was the caller's, and so is everything else
`urn:sparql:select` takes that the line did not fix. One argument cannot be fixed at all.
`as` chooses which form an answer comes in, and that is the caller's choice every time:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'bind urn:quakes:count:csv = source urn:quakes:count as=text/csv'; echo "exit $?"
error: not a declaration: at <urn:quakes:count:csv>: it fixes `as`, which negotiates the face and is never fixed: declare `as` as an input with a default, the name's default face
exit 1
```

A bound name has no contract of its own: it is described by its target's, with the fixed
arguments marked. `describe` shows it, here only the lines that say what is fixed:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'describe urn:quakes:count' | grep 'ik:fixed'
[computed]
    ik:fixed "<urn:file:queries/count.rq>" .
    ik:fixed "urn:quakes:graph" .
    ik:fixed "<urn:file:queries/count.rq>" .
    ik:fixed "urn:quakes:graph" .
```

Two of each, because `urn:sparql:select` has two actions, `Source` and `Exists`, and the
name fixes the same two arguments of both. The rest of the description is `urn:sparql:select`'s
own: its summary, its other inputs, and the forms `as` may ask for.

## 4. A reference follows its file

The question is a reference, so the name's answer hangs from the question's file. Change the
file and the answer is stale. In one REPL, so the cache carries from line to line, ask the
strong count, raise the filter to magnitude 6, and ask again. Save it as `follow.iki`:

<!-- transcript: file follow.iki -->

```text
source urn:quakes:strong:count as=text/csv
cache urn:quakes:strong:count as=text/csv
sink urn:file:queries/strong.rq PREFIX qk: <https://ikigai-rs.dev/ns/quakes#> CONSTRUCT { ?quake a qk:Earthquake ; qk:mag ?mag ; qk:place ?place } WHERE { ?quake a qk:Earthquake ; qk:mag ?mag ; qk:place ?place FILTER(?mag >= 6) }
cache urn:quakes:strong:count as=text/csv
cache urn:quakes:graph
source urn:quakes:strong:count as=text/csv
```

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names < follow.iki
quakes
12
[computed]
cached
wrote 197 bytes to queries/strong.rq
[uncacheable]
not cached
cached
quakes
2
[computed]
```

Two earthquakes of magnitude 6 and up in the first week, the Noto Peninsula earthquake and
its largest aftershock. The write cut the question file's thread; the strong layer hung from
it and so did the count read from the layer, so both were stale. The graph did not read the
question, so it was still cached. Nobody named the count, the layer or the file in between:
the names say what they read, and the kernel kept track. Editing `queries/strong.rq` in an
editor does the same, because the CLI watches its workspace.

Put the filter back, so the rest of the chapter counts the same twelve:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . -c 'sink urn:file:queries/strong.rq PREFIX qk: <https://ikigai-rs.dev/ns/quakes#> CONSTRUCT { ?quake a qk:Earthquake ; qk:mag ?mag ; qk:place ?place } WHERE { ?quake a qk:Earthquake ; qk:mag ?mag ; qk:place ?place FILTER(?mag >= 5.5) }'
wrote 199 bytes to queries/strong.rq
[uncacheable]
```

## 5. Reading the names, not writing them

The names are a resource too, `urn:iki:host:names`, and its contract says who may change
them:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'describe urn:iki:host:names' | grep -E 'ik:summary "re|ik:requires'
[computed]
    ik:summary "read the names file" ;
    ik:summary "replace the names file and rebind" ;
    ik:requires <urn:cap:host:write:names> ;
```

Reading the table needs nothing; replacing it needs `urn:cap:host:write:names`, and `bind` and
`unbind` replace it. Narrow the session the way [Process scripts are
files](scripts.md#3-the-authority-a-plan-needs) did, with `cap read-only`, and try both:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'cap read-only' -c 'source urn:iki:host:names' -c 'source urn:quakes:strong:count as=text/csv' -c 'unbind urn:quakes:count'; echo "exit $?"
narrowed — capability: urn:cap:fs:read:$PWD
urn:quakes:graph:2024-01-08 = source urn:iki:fn:compose src=<urn:file:templates/week-2024-01-08.jsonld>
urn:quakes:strong:count = source urn:sparql:select graph=urn:quakes:strong query=<urn:file:queries/count.rq>
urn:quakes:notable = source urn:sparql:select graph="urn:quakes:graph urn:quakes:graph:2024-01-08" query=<urn:file:queries/notable.rq>
urn:quakes:strong = source urn:sparql:construct graph=urn:quakes:graph query=<urn:file:queries/strong.rq>
urn:quakes:count = source urn:sparql:select graph=urn:quakes:graph query=<urn:file:queries/count.rq>
urn:quakes:graph = source urn:iki:fn:compose src=<urn:file:templates/week.jsonld>
[computed]
quakes
12
[computed]
error: denied: capability does not grant `urn:cap:host:write:names` (declared by `urn:iki:host:names`)
— batch: 4 commands · 2 computed
exit 1
```

The narrowed session read the table and asked a name in it, and its `unbind` was refused
before anything changed: `cat names` still shows all six. A session you hand a script, or a
person you hand a REPL, can use your names without being able to rename what they mean.

## 6. A name somebody already serves

The names are bound **last**, after everything the CLI serves itself. So a binding can never
take over a name the host answers already: the host's answer wins. That would be a quiet way to
be wrong, so the names layer says when it happens, at
`urn:iki:space:dynamic:names:status`. Bind a name the CLI already serves, `urn:text:wc`, ask
the status, and see whose answer `urn:text:wc` gives:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'bind urn:text:wc = source urn:quakes:count' -c 'source urn:iki:space:dynamic:names:status' -c 'source urn:text:wc "one line"'
bound in <urn:iki:host:names>:
urn:text:wc = source urn:quakes:count
[uncacheable]
space: urn:iki:space:dynamic:names
declaration: urn:iki:host:names
in force: …
installs: …
refused: nothing since the last bind
names file: $PWD/names (text/x-ikigai-names)
shadowed: urn:text:wc is served ahead of the names layer, so its binding never answers
[uncacheable]
1
[computed]
— batch: 3 commands · 1 computed · 2 uncacheable
```

`shadowed:` names the binding that never answers, and `urn:text:wc` still counts lines, as it
did before. (`in force:` is a hash of the table, and `installs:` counts how often the layer has bound it, so both change as you go.) Take the
binding away again:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c 'unbind urn:text:wc'
unbound `urn:text:wc` in <urn:iki:host:names> (was: urn:text:wc = source urn:quakes:count)
[uncacheable]
```

This is why this part names everything of its own under `urn:quakes:`: no host serves it, so
no binding there is shadowed.

## 7. The river's questions

The river from [A river of feeds](rss.md) gets names of its own the same way, in its own
file, `river.names`. Every step here runs in `examples/rss`, with the example's `./iki`, which
passes `--names` through to `ikigai`:

```bash
cd ikigai-tutorial/examples/rss
```

The saved searches are not in git (the example's `.gitignore` leaves `ws/searches/` out), so
write them again first, as [that chapter](rss.md#7-saved-searches-and-the-refresh-script) did:

<!-- transcript: run -->

```console
$ ./iki --quiet < ws/save-searches.iki
wrote 1077 bytes to searches/caching.rq
[1 computed · 1 uncacheable]
wrote 1076 bytes to searches/rss.rq
[1 computed · 1 uncacheable]
```

Three names, as `bind-river.iki`: a saved search, every search, and a live feed read politely.

<!-- transcript: file bind-river.iki -->

```text
bind urn:river:caching = source urn:sparql:select query=<urn:file:searches/caching.rq> graph=urn:file:river.nt
bind urn:river:search:{q} = source urn:iki:fn:compose src=<urn:file:queries/search.rq> q={q}
bind urn:river:live:{url} = source urn:http:get url={url} max_age=3600 headers="User-Agent: ikigai-tutorial (+https://ikigai-rs.github.io/ikigai-tutorial/)"
```

The second and third are **templates**: a `{var}` in the name matches whatever stands there in
the name a caller asks for, and `q={q}` fixes `q` to it. So `urn:river:search:rss` is the
search query filled with `rss`, and every term has a name of its own. A capture is always a
whole value: `q=a{q}` is refused rather than spliced.

<!-- transcript: run -->

```console
$ ./iki --quiet --names river.names < bind-river.iki
bound in <urn:iki:host:names>:
urn:river:caching = source urn:sparql:select graph=urn:file:river.nt query=<urn:file:searches/caching.rq>
[uncacheable]
bound in <urn:iki:host:names>:
urn:river:search:{q} = source urn:iki:fn:compose q={q} src=<urn:file:queries/search.rq>
[uncacheable]
bound in <urn:iki:host:names>:
urn:river:live:{url} = source urn:http:get headers="User-Agent: ikigai-tutorial (+https://ikigai-rs.github.io/ikigai-tutorial/)" max_age=3600 url={url}
[uncacheable]
```

The saved search is now a question you ask by name, written out by the digest view:

<!-- transcript: run -->

```console
$ ./iki --quiet --names river.names -c 'source urn:river:caching as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl'
# Digest
- 2026-10-27 [My home server caches everything now](https://bob.example.org/2026/10/home-server-cache/) from Bob's Workbench
- 2026-10-26 [Why my feed sends an ETag](https://alice.example.com/posts/ETag/) from Alice Writes Things
- 2026-09-23 [Episode 11: Caching for people who hate caching](https://dave.example.org/episodes/11) from Dave Talks
[2 computed]
```

And any search by its term. The name answers the filled query, so it pipes into
`urn:sparql:select` as a query file did:

<!-- transcript: run -->

```console
$ ./iki --quiet --names river.names -c 'source urn:river:search:rss | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl'
# Digest
- 2026-10-26 [Why my feed sends an ETag](https://alice.example.com/posts/ETag/) from Alice Writes Things
- 2026-10-24 [(untitled)](https://carol.example.com/2026/10/24/rss-cloud/) from carol.example.com
- 2026-10-07 [Episode 12: The small web](https://dave.example.org/episodes/12) from Dave Talks
- 2026-09-28 [Why I still read RSS](https://bob.example.org/2026/09/still-rss/) from Bob's Workbench
[3 computed]
```

The live feed is the one this book will not read for you. A template capture can hold a whole
web address, colons, slashes and all, so `urn:river:live:https://alice.example.com/feed.xml`
names that feed with the politeness built in: an hour's copy and a `User-Agent` saying who
asks. Ask it under a session narrowed to another host, and the refusal shows the address the
name captured, with nothing sent:

<!-- transcript: run -->

```console
$ ./iki --quiet --names river.names -c 'cap net-example.org' -c 'source urn:river:live:https://alice.example.com/feed.xml'; echo "exit $?"
narrowed — capability: urn:cap:net:example.org
error: denied: capability does not grant `GET` to `alice.example.com/feed.xml`
exit 1
```

Without the narrowing, that line is a real request to a real address; point it at the stand-in
origin from [the river's section 2](rss.md#2-somebody-elses-rules) instead, and its log shows the
`User-Agent` the name sends.

## 8. What names make plain

Back through what the earlier chapters had to work around:

- **[Reshaping JSON with jq](jq.md#6-at-a-shell-programs-and-templates-are-files)** filled its
  headline from a saved `week.json`, because a template could not splice `urn:http:get` with
  its `url`. A template splices a name, so `bind urn:quakes:live = source urn:http:get
  url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson` is all
  `$j{urn:quakes:live …}` needs. That step reaches the network, so this page does not run it;
  a binding of the same shape over the recorded week, served from this machine over HTTP, filled the
  headline with 178 and the Noto Peninsula earthquake on `ikigai-cli` 0.2.6, on 2026-10-29.
- **[Giving the data names](names.md#6-at-a-shell)** kept the filled graph as `quakes.jsonld`;
  section 1 named it, and the name is never older than its context and its week.
- **[Composites](composites.md#6-at-a-shell-your-own-notes)** kept `notable.csv` and
  `notable.json` as copies that did not hang from the weeks. `urn:quakes:notable` does, and the
  strong layer at a shell is a name, as it was on the page.
- **[Process scripts are files](scripts.md#5-plans-with-parameters)** wanted a plan with a
  parameter. A name can bind a plan, `bind urn:… = source urn:plan:eval in=<plan>`, and copies
  the plan's declared inputs into its own contract; this page does not write such a plan, so
  that part is still to be shown here.
- **[A river of feeds](rss.md#8-what-the-river-is-waiting-for)** listed three. The live feed
  with its politeness is `urn:river:live:{url}`, and the saved searches are names. The build
  line, `urn:river:all`, is still a line: a binding is one request, and the build is a pipeline
  that maps over two stages in parentheses. The way to name a pipeline is to store it as a plan,
  and `ikigai-cli` {{#include ../../ikigai-cli.version}} refuses that one (`.. ( … )`, mapping
  over a fork, has no plan spelling yet). The river keeps `refresh.iki` until it does. And a map
  over `ws/searches/` still hands each stage a file's short name, so the set of saved searches is
  not yet a set of names a map can walk.

What the names make plain is what the copies hid: which answer reads which file. A name says
it in one line of `names`, the kernel holds the answer to it, and a change to anything the line
reads reaches the answer, the next time somebody asks.
