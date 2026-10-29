# Giving the data names

The questions so far knew what `mag` means because you read USGS's documentation. Nothing
in the file says so: to the file, `mag` is three letters, and another feed's `mag` could
mean anything. This chapter gives the recorded week names that say what they mean, and then
asks it questions in SPARQL, the query language for data named that way.

Two kinds of name are involved, and they belong to different people:

- **The earthquakes are USGS's.** Each one already has a page of its own: its `id` after
  `https://earthquake.usgs.gov/earthquakes/eventpage/`. That is a real address, and USGS
  describes the earthquake there. So it is the earthquake's name here too.
- **The words are ours.** `mag`, `place` and `magType` need names that someone publishes
  and anyone can look up. USGS publishes no such vocabulary, so this book does, under
  `https://ikigai-rs.dev/ns/quakes#`, on the project's own domain. A word made up under
  USGS's domain would be one USGS never wrote and nobody could look up.

The Run cells use the recorded week in this page's kernel. The last section does the same
at a shell, where the context is a file you edit.

## 1. The context is data

A JSON-LD **context** is a JSON document that says what another JSON document's keys mean.
Here is the one for USGS's feed. It is a resource like the week, at
`urn:file:quakes.context.jsonld`:

<div class="ikigai-run" data-cmd='source urn:file:quakes.context.jsonld'>
<pre class="ikigai-run-expected">{
  &quot;@context&quot;: {
    &quot;@version&quot;: 1.1,
    &quot;@base&quot;: &quot;https://earthquake.usgs.gov/earthquakes/eventpage/&quot;,
    &quot;qk&quot;: &quot;https://ikigai-rs.dev/ns/quakes#&quot;,
    &quot;geojson&quot;: &quot;https://purl.org/geojson/vocab#&quot;,
    &quot;schema&quot;: &quot;https://schema.org/&quot;,
    &quot;xsd&quot;: &quot;http://www.w3.org/2001/XMLSchema#&quot;,
    &quot;id&quot;: &quot;@id&quot;,
    &quot;type&quot;: &quot;@type&quot;,
    &quot;FeatureCollection&quot;: &quot;geojson:FeatureCollection&quot;,
    &quot;Feature&quot;: &quot;geojson:Feature&quot;,
    &quot;Point&quot;: &quot;geojson:Point&quot;,
    &quot;earthquake&quot;: &quot;qk:Earthquake&quot;,
    &quot;features&quot;: {&quot;@id&quot;: &quot;geojson:features&quot;, &quot;@container&quot;: &quot;@set&quot;},
    &quot;geometry&quot;: &quot;geojson:geometry&quot;,
    &quot;coordinates&quot;: {&quot;@id&quot;: &quot;geojson:coordinates&quot;, &quot;@container&quot;: &quot;@list&quot;},
    &quot;properties&quot;: &quot;@nest&quot;,
    &quot;mag&quot;: {&quot;@id&quot;: &quot;qk:mag&quot;, &quot;@type&quot;: &quot;xsd:double&quot;},
    &quot;magType&quot;: &quot;qk:magType&quot;,
    &quot;place&quot;: &quot;qk:place&quot;,
    &quot;time&quot;: &quot;qk:time&quot;,
    &quot;felt&quot;: &quot;qk:felt&quot;,
    &quot;title&quot;: &quot;schema:name&quot;,
    &quot;url&quot;: {&quot;@id&quot;: &quot;schema:url&quot;, &quot;@type&quot;: &quot;@id&quot;}
  }
}
[computed]</pre>
</div>

Read it from the top:

- `@base` is where a bare `id` is resolved, so the `id` `us6000m0xl` names
  `https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl`, USGS's page for it.
- `qk`, `geojson`, `schema` and `xsd` are prefixes: `qk:mag` is short for
  `https://ikigai-rs.dev/ns/quakes#mag`.
- `type`, `Feature`, `features`, `geometry`, `Point` and `coordinates` are GeoJSON's own
  structure, and they keep the names the
  [GeoJSON-LD vocabulary](https://geojson.org/geojson-ld/) already gives them.
  `coordinates` is an `@list` because its order is its meaning: longitude, latitude, then
  depth in kilometers.
- `properties` is `@nest`. GeoJSON puts an earthquake's facts in an object of their own,
  and `@nest` says they are facts about the earthquake itself, not about that object. One
  of them is USGS's `"type": "earthquake"`, and the context maps the word `earthquake` to
  `qk:Earthquake`, so each one says what it is.
- `mag`, `magType`, `place`, `time` and `felt` are ours, under `qk:`. `mag` is typed as a
  floating-point number (`xsd:double`), so that a magnitude of 5 and one of 5.1 are the
  same kind of value.
- `title` and `url` mean what schema.org's `name` and `url` already mean, so they are
  those.

The vocabulary says what each of our words carries and in what unit. It is the file
[`books/ikigai/ns/quakes.ttl`](https://github.com/ikigai-rs/ikigai-tutorial/blob/main/books/ikigai/ns/quakes.ttl)
in this book's repository, which `https://ikigai-rs.dev/ns/quakes` serves, and it says
there why each word that is ours is not borrowed from somewhere else. A check in the
book's build holds the context and the vocabulary to each other: every word the context
maps to `qk:` is defined there, and every word defined there is one the context maps.

A key the context does not name means nothing to it, and it is left out. Each earthquake's
`properties` has 26 keys; this context names eight of them.

## 2. The week, with its context

USGS's file does not mention the context, and it is not ours to change. So a template puts
the two together, the way [Reshaping JSON with jq](jq.md#4-a-template-that-asks-questions)
filled a headline. This page holds it at `urn:quakes:template:graph`:

<div class="ikigai-run" data-cmd='source urn:quakes:template:graph'>
<pre class="ikigai-run-expected">{&quot;@context&quot;: $r{urn:json:query?src=urn:file:quakes.context.jsonld&amp;q=.[]}, &quot;@graph&quot;: [$r{urn:file:quakes-2024-01-01.geojson}]}
[computed]</pre>
</div>

`$r{…}` splices a resource in as it is. The first marker asks jq for the context out of its
file (`.[]` is the one value inside `{"@context": …}`), and the second is the whole week.
Filled, the template is the week as JSON-LD: GeoJSON with a context.

`urn:sparql:describe` reads a graph and answers everything it says about one name. Its
`graph` argument is the name of a graph to read, so the filled template needs a name of
its own: in this page it is `urn:quakes:graph`. Ask about the Noto Peninsula earthquake,
by USGS's name for it:

<div class="ikigai-run" data-cmd='source urn:sparql:describe graph=urn:quakes:graph query="DESCRIBE <https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl>" as=application/n-triples'>
<pre class="ikigai-run-expected">&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#type&gt; &lt;https://ikigai-rs.dev/ns/quakes#Earthquake&gt; .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#type&gt; &lt;https://purl.org/geojson/vocab#Feature&gt; .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;https://ikigai-rs.dev/ns/quakes#felt&gt; &quot;422&quot;^^&lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;https://ikigai-rs.dev/ns/quakes#mag&gt; &quot;7.5&quot;^^&lt;http://www.w3.org/2001/XMLSchema#double&gt; .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;https://ikigai-rs.dev/ns/quakes#magType&gt; &quot;mww&quot; .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;https://ikigai-rs.dev/ns/quakes#place&gt; &quot;2024 Noto Peninsula, Japan Earthquake&quot; .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;https://ikigai-rs.dev/ns/quakes#time&gt; &quot;1704093009476&quot;^^&lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;https://purl.org/geojson/vocab#geometry&gt; _:b681 .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;https://schema.org/name&gt; &quot;M 7.5 - 2024 Noto Peninsula, Japan Earthquake&quot; .
&lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; &lt;https://schema.org/url&gt; &lt;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl&gt; .
_:b681 &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#type&gt; &lt;https://purl.org/geojson/vocab#Point&gt; .
_:b681 &lt;https://purl.org/geojson/vocab#coordinates&gt; _:b682 .
_:b682 &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#first&gt; &quot;137.271&quot;^^&lt;http://www.w3.org/2001/XMLSchema#double&gt; .
_:b682 &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#rest&gt; _:b683 .
_:b683 &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#first&gt; &quot;37.4874&quot;^^&lt;http://www.w3.org/2001/XMLSchema#double&gt; .
_:b683 &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#rest&gt; _:b684 .
_:b684 &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#first&gt; &quot;10&quot;^^&lt;http://www.w3.org/2001/XMLSchema#integer&gt; .
_:b684 &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#rest&gt; &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#nil&gt; .
[computed]</pre>
</div>

Each line is one statement: a subject, a property, a value. Every earthquake fact is about
`<https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl>`, and every property is a
full IRI, ours or someone else's, that you can look up. The `_:` names are blank nodes,
things the document did not name: the point, and the list of its coordinates, which RDF
writes as a chain of `rdf:first` and `rdf:rest`. 422 people told USGS they felt this one.

The statement's `schema:url` and its subject are the same IRI. USGS already gave every
earthquake its name, in `url`; the context only had to say so.

## 3. What the graph holds

What the context does not name is not in the graph. Ask for every property the graph uses:

<div class="ikigai-run" data-cmd='source urn:sparql:select graph=urn:quakes:graph query="SELECT DISTINCT ?property FROM <urn:quakes:graph> WHERE { ?s ?property ?o } ORDER BY ?property" as=text/csv'>
<pre class="ikigai-run-expected">property
http://www.w3.org/1999/02/22-rdf-syntax-ns#first
http://www.w3.org/1999/02/22-rdf-syntax-ns#rest
http://www.w3.org/1999/02/22-rdf-syntax-ns#type
https://ikigai-rs.dev/ns/quakes#felt
https://ikigai-rs.dev/ns/quakes#mag
https://ikigai-rs.dev/ns/quakes#magType
https://ikigai-rs.dev/ns/quakes#place
https://ikigai-rs.dev/ns/quakes#time
https://purl.org/geojson/vocab#coordinates
https://purl.org/geojson/vocab#features
https://purl.org/geojson/vocab#geometry
https://schema.org/name
https://schema.org/url
[computed]</pre>
</div>

Thirteen, all of them named by the context, or by RDF itself for types and lists. `sig`,
`tsunami`, `status` and the rest of USGS's keys are not there. `FROM <urn:quakes:graph>`
asks about this one graph: `urn:sparql:select` loads each graph it is given under its own
name, beside a vocabulary of its own that the question would otherwise read too.

## 4. The questions are resources

A question you will ask again belongs at a name, like the program in
[Reshaping JSON with jq](jq.md#6-at-a-shell-programs-and-templates-are-files). This page
holds three, at `urn:quakes:question:count`, `…:biggest` and `…:by-magtype`. Read one, then
pipe it into `urn:sparql:select`: a piped value fills the one argument you did not name,
here `query`. `as=text/csv` asks for the rows as CSV:

<div class="ikigai-run" data-cmd='source urn:quakes:question:count
source urn:quakes:question:count | urn:sparql:select graph=urn:quakes:graph as=text/csv'>
<pre class="ikigai-run-expected">PREFIX qk: &lt;https://ikigai-rs.dev/ns/quakes#&gt; SELECT (COUNT(?quake) AS ?quakes) WHERE { ?quake a qk:Earthquake }
[computed]
quakes
178
[1 cached · 1 computed]</pre>
</div>

The 178 earthquakes of the week, counted by what they are rather than by how many lines the
file has. The question was cached already, because the line above it read it; the answer
was computed.

The largest, named by its USGS page:

<div class="ikigai-run" data-cmd='source urn:quakes:question:biggest | urn:sparql:select graph=urn:quakes:graph as=text/csv'>
<pre class="ikigai-run-expected">quake,mag,place
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl,7.5,&quot;2024 Noto Peninsula, Japan Earthquake&quot;
[2 computed]</pre>
</div>

And how many each magnitude type measured, the answer the jq chapter's `group_by` gave:

<div class="ikigai-run" data-cmd='source urn:quakes:question:by-magtype | urn:sparql:select graph=urn:quakes:graph as=text/csv'>
<pre class="ikigai-run-expected">magType,quakes
mb,143
mwb,1
mwr,7
mww,27
[2 computed]</pre>
</div>

`ORDER BY` and `GROUP BY` are SPARQL's sort and group. The questions themselves are in
[`crates/quakes/queries/`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates/quakes/queries),
one file each.

## 5. What hangs from the context

The context is a thread like the week. Everything read through it hangs from it:

<div class="ikigai-run" data-cmd='source urn:kernel:dependents thread=urn:file:quakes.context.jsonld'>
<pre class="ikigai-run-expected">dependents of urn:file:quakes.context.jsonld  (cut 0 times)
  8 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:file:quakes.context.jsonld  application/ld+json           918 B  1 thread    root
  urn:json:query                  application/json              746 B  2 threads   root
  urn:quakes:graph                application/ld+json        123.6 KB  5 threads   root
  urn:sparql:describe             application/n-triples        2.1 KB  6 threads   root
  urn:sparql:select               text/csv                       13 B  7 threads   root
  urn:sparql:select               text/csv                       46 B  7 threads   root
  urn:sparql:select               text/csv                      123 B  7 threads   root
  urn:sparql:select               text/csv                      526 B  6 threads   root
[uncacheable]</pre>
</div>

The context itself; the jq answer inside the template; the graph, filled from the template;
and every question asked of the graph. Cut the context's thread, the way a change to the
file would, and the graph goes stale with everything computed from it:

<div class="ikigai-run" data-cmd='sink urn:kernel:cut urn:file:quakes.context.jsonld
cache urn:quakes:graph
source urn:quakes:question:count | urn:sparql:select graph=urn:quakes:graph as=text/csv'>
<pre class="ikigai-run-expected">cut urn:file:quakes.context.jsonld
[uncacheable]
not cached
quakes
178
[1 cached · 1 computed]</pre>
</div>

The graph was filled again and the question asked of it again, and the count is still 178,
because the context did not really change. The question's own text did not depend on the
context, so it was still cached. At a shell, the context is a file, and it can change.

## 6. At a shell

The REPL has the same resources. In the REPL you started over `~/quakes`, download the
context beside the recorded week:

```bash
cd ~/quakes
curl -sO https://raw.githubusercontent.com/ikigai-rs/ikigai-tutorial/main/books/ikigai/ns/quakes.context.jsonld
```

The steps below are replayed against `ikigai-cli` {{#include ../../ikigai-cli.version}} by
the book's checks, over the same two files. Write the template from the page as a file of
your own, fill it, and keep the filled copy as `quakes.jsonld`. There is no name like
`urn:quakes:graph` at a shell: a request with arguments cannot be named as one resource
yet (named requests, ledger #2447), and `graph` takes a name. So the copy is a file:

<!-- transcripts: workspace crates/quakes/quakes-2024-01-01.geojson books/ikigai/ns/quakes.context.jsonld -->

```text
ikigai> sink urn:file:graph.txt {"@context": $r{urn:json:query?src=urn:file:quakes.context.jsonld&q=.[]}, "@graph": [$r{urn:file:quakes-2024-01-01.geojson}]}
wrote 125 bytes to graph.txt
[uncacheable]
ikigai> source urn:iki:fn:compose src=urn:file:graph.txt | sink urn:file:quakes.jsonld
wrote 126539 bytes to quakes.jsonld
[1 computed · 1 uncacheable]
```

The file workspace gives `quakes.jsonld` its type from its name, `application/ld+json`, so
SPARQL reads it as JSON-LD. Write a question as a file too, and ask it:

<!-- transcript: continues -->

```text
ikigai> sink urn:file:queries/by-magtype.rq PREFIX qk: <https://ikigai-rs.dev/ns/quakes#> SELECT ?magType (COUNT(?quake) AS ?quakes) WHERE { ?quake qk:magType ?magType } GROUP BY ?magType ORDER BY ?magType
wrote 161 bytes to queries/by-magtype.rq
[uncacheable]
ikigai> source urn:file:queries/by-magtype.rq | urn:sparql:select graph=urn:file:quakes.jsonld as=text/csv
magType,quakes
mb,143
mwb,1
mwr,7
mww,27
[2 computed]
```

Now change what the week means. Open `quakes.context.jsonld` in an editor and delete the
`magType` line, or let jq do it, which is what the book's check does. Then fill the
template again, and ask again:

<!-- transcript: continues -->

```text
ikigai> source urn:json:query src=urn:file:quakes.context.jsonld q="del(.[].magType)" | sink urn:file:quakes.context.jsonld
wrote 736 bytes to quakes.context.jsonld
[1 computed · 1 uncacheable]
ikigai> source urn:iki:fn:compose src=urn:file:graph.txt | sink urn:file:quakes.jsonld
wrote 126516 bytes to quakes.jsonld
[1 computed · 1 uncacheable]
ikigai> source urn:file:queries/by-magtype.rq | urn:sparql:select graph=urn:file:quakes.jsonld as=text/csv
magType,quakes
[1 cached · 1 computed]
```

The header, and no rows. USGS's file still says `"magType": "mww"` for the Noto Peninsula
earthquake, but the context no longer gives `magType` a meaning, so the graph has no
magnitude types to group. The question did not change, and it was cached; the answer was
computed again, because writing `quakes.jsonld` cut its thread. (jq wrote the context back
on one line; an editor would have kept your layout.) Download the context again to put the
line back.

The questions now have names, and the earthquakes keep USGS's. Next, answers built from
answers, one layer at a time: [Composites, one layer at a time](composites.md).
