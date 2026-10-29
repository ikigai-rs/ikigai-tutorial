# Reshaping JSON with jq

[A lab with names in it](lab.md) asked the recorded week small questions through
`urn:json:query`. This chapter is about the questions. They are written in jq, the
language of the `jq` command: if you have used `jq` at a shell, you already know it, and
if you have not, every program here is short and explained.

`urn:json:query` runs jq as a resource. It has three arguments:

- `src`, the name of the JSON to read, read through the kernel with your authority;
- `q`, the jq program;
- `as`, the form of the answer (JSON by default).

The Run cells use the recorded week in this page's kernel. The last two sections do the
same at a shell, where the questions and templates become files.

## 1. Paths, filters and reductions

A GeoJSON feed is an object whose `features` array holds one object per earthquake. Each
has an `id`, a `geometry` (where), and `properties` (what): `mag`, `place`, `time`,
`magType`, `title`, and the `url` of USGS's page for it. A path picks a value out, and
`{mag, place, url}` builds a smaller object from the fields it names:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[0].properties | {mag, place, url}"'>
<pre class="ikigai-run-expected">{&quot;mag&quot;:5.1,&quot;place&quot;:&quot;77 km ESE of Fangale’ounga, Tonga&quot;,&quot;url&quot;:&quot;https://earthquake.usgs.gov/earthquakes/eventpage/us6000m2di&quot;}
[computed]</pre>
</div>

`max_by` finds the largest by a field, `select` keeps what passes a test, and `[ … ]`
collects the results into one array:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features | max_by(.properties.mag) | .properties.title" as=text/plain
source urn:json:query src=urn:file:quakes-2024-01-01.geojson q="[.features[] | select(.properties.mag >= 6) | .properties.title]"'>
<pre class="ikigai-run-expected">M 7.5 - 2024 Noto Peninsula, Japan Earthquake
[computed]
[&quot;M 6.2 - 8 km SW of Anamizu, Japan&quot;,&quot;M 7.5 - 2024 Noto Peninsula, Japan Earthquake&quot;]
[computed]</pre>
</div>

Two earthquakes of magnitude 6 or more that week, both in Japan, on the same day: the Noto
Peninsula earthquake and an aftershock near Anamizu, a town on the peninsula.

Sorting and grouping work the same way. The five largest magnitudes, and how many
earthquakes each magnitude type measured (`mb`, body-wave magnitude, is the usual one for
smaller and distant earthquakes; the `mw` types are moment magnitudes):

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features | sort_by(-.properties.mag) | .[0:5] | map(.properties.mag)"
source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features | group_by(.properties.magType) | map({type: .[0].properties.magType, count: length})"'>
<pre class="ikigai-run-expected">[7.5,6.2,5.8,5.7,5.6]
[computed]
[{&quot;type&quot;:&quot;mb&quot;,&quot;count&quot;:143},{&quot;type&quot;:&quot;mwb&quot;,&quot;count&quot;:1},{&quot;type&quot;:&quot;mwr&quot;,&quot;count&quot;:7},{&quot;type&quot;:&quot;mww&quot;,&quot;count&quot;:27}]
[computed]</pre>
</div>

## 2. One answer, or many

A jq program yields a stream of results, and a resource answers once. So
`urn:json:query` says how the stream becomes one answer, and `as` picks the rule:

- leave it out (`application/json`), and one result is the answer, while several become one
  JSON array;
- `as=application/x-ndjson` gives one result per line, which is the list shape `..` maps
  over;
- `as=text/plain` gives one result as plain text, and refuses anything else, because a
  template splicing text needs exactly one value.

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[0:2][] | .id"
source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[0:2][] | .id" as=application/x-ndjson
source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[0:2][] | .id" as=text/plain'>
<pre class="ikigai-run-expected">[&quot;us6000m2di&quot;,&quot;us6000m2d7&quot;]
[computed]
&quot;us6000m2di&quot;
&quot;us6000m2d7&quot;
[computed]
error: invalid argument `as`: as=text/plain answers exactly one scalar result, and the program yielded 2; leave `as` out for JSON
       `as`: the answer's media type (one of: application/json, application/x-ndjson, text/plain)</pre>
</div>

The refusal says what went wrong and what to do instead. It is not cached: the next
attempt asks again, so a fixed program gets a fresh answer.

## 3. Every answer remembers the file

<!-- urn-gate: illustration urn:iki:space:json — the name ikigai-json's space claims, which a
trace prints as the space that answered (`answered-by=`); a space's name, not a resource. -->

Run a question twice and the second answer is cached. Run a different question over the
same file and it is computed, but the file it reads is not read again:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q="[.features[] | .properties.tsunami] | add"
source urn:json:query src=urn:file:quakes-2024-01-01.geojson q="[.features[] | .properties.tsunami] | add"
trace urn:json:query src=urn:file:quakes-2024-01-01.geojson q="[.features[] | select(.properties.tsunami == 1) | .properties.title]"'>
<pre class="ikigai-run-expected">2
[computed]
2
[cached]
trace  urn:json:query
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:json:query   json-query · computed · ThreadId(1) · — · answered-by=urn:iki:space:json   → 95b  [&quot;M 4.9 - 8 km SE of Seldovia Village, A…
└─ urn:file:quakes-2024-01-01.geojson   quakes-window · cached · ThreadId(1) · —</pre>
</div>

`tsunami` is a flag USGS sets to 1 on larger earthquakes in oceanic regions; it does not
say whether a tsunami happened. Two that week: the Noto Peninsula earthquake, and a small
one near Seldovia, Alaska. The trace shows the second question computed, over a file
already in the cache. Each answer hangs from the file's thread, as every answer in
[A lab with names in it](lab.md#7-what-hangs-from-the-sample) did, so a change to the file
recomputes exactly these.

## 4. A template that asks questions

`urn:iki:fn:compose` fills a template, and its `$j{<src> <program>}` marker splices the
answer to a jq question into the text, as plain text. This page holds one template, at
`urn:quakes:template:headline`:

<div class="ikigai-run" data-cmd='source urn:quakes:template:headline
source urn:iki:fn:compose src=urn:quakes:template:headline'>
<pre class="ikigai-run-expected">The week of 2024-01-01: $j{urn:file:quakes-2024-01-01.geojson .features|length} earthquakes of M4.5 and up; the largest, $j{urn:file:quakes-2024-01-01.geojson .features | max_by(.properties.mag) | .properties.title}.
[computed]
The week of 2024-01-01: 178 earthquakes of M4.5 and up; the largest, M 7.5 - 2024 Noto Peninsula, Japan Earthquake.
[computed]</pre>
</div>

Each marker is a source and a program, separated by a space; the program runs to the end
of the marker, so it can hold pipes. `$j` is shorthand: each marker is the request
`urn:json:query` with `as=text/plain`, which is why the program must answer one value.
The template is a resource too, so its answer hangs from the file's thread and the
template's.

## 5. At a shell: the type comes from the name

The REPL runs the same questions. Over the copy of the recorded week you downloaded in
[A week of earthquakes](index.md#what-you-need), type the line after `ikigai>`:

<!-- transcripts: workspace crates/quakes/quakes-2024-01-01.geojson -->

```text
ikigai> source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features|length"
178
[computed]
```

A file has no type of its own. The CLI's file workspace takes one from the name's
extension: `.json` is JSON, and `.geojson` is GeoJSON's registered type,
`application/geo+json`, which is JSON too, so the query reads it as the page's copy is
read. (`ikigai-cli` 0.2.5 and later. On 0.2.4 the workspace did not know `.geojson` and
served the file as plain text, which the query refused, `is text/plain, and nothing this
kernel mounts converts it to application/json`; ledger #2529. There, copy it to a name
ending in `.json` first: `source urn:file:quakes-2024-01-01.geojson | sink
urn:file:quakes-2024-01-01.json`.)

The answers in sections 5 and 6 over the recorded week are replayed against
`ikigai-cli` {{#include ../../ikigai-cli.version}} by the book's checks, over the same file
the download fetches. The ones over the live week cannot be: they are what `ikigai-cli` 0.2.4
printed over the feed as it was on 2026-10-28, and yours will differ. The live week you saved
in [Somebody else's resource](http.md#4-keeping-a-copy) is `week.json`:

```text
source urn:json:query src=urn:file:week.json q=".features | max_by(.properties.mag) | .properties.title" as=text/plain
```

```text
M 7.0 - 191 km ENE of Nagqu, China
[computed]
```

## 6. At a shell: programs and templates are files

A jq program you will use again belongs in a file, and a file is a resource. Write one with
`sink`; the quotation marks make the program one value, so its `|` is part of the program
and not a pipe, and the marks themselves are not stored. Then pipe the file into the query:
a piped value fills the one argument you did not name, here `q`.

<!-- transcript: continues -->

```text
ikigai> sink urn:file:queries/biggest.jq ".features | max_by(.properties.mag) | .properties.title"
wrote 55 bytes to queries/biggest.jq
[uncacheable]
ikigai> source urn:file:queries/biggest.jq | urn:json:query src=urn:file:quakes-2024-01-01.geojson as=text/plain
M 7.5 - 2024 Noto Peninsula, Japan Earthquake
[2 computed]
```

The `sink` made the `queries` directory too. The file is an ordinary one,
`~/quakes/queries/biggest.jq`, so an editor or `jq -f` reads it as well. The same program
over the live week:

```text
source urn:file:queries/biggest.jq | urn:json:query src=urn:file:week.json as=text/plain
```

```text
M 7.0 - 191 km ENE of Nagqu, China
[1 cached · 1 computed]
```

One program, two weeks. The program's own file is a thread as well: edit it and both
answers go stale.

A template is a file the same way. Write the headline from the page as one, over the live
week, and fill it:

```text
sink urn:file:headline.txt This week: $j{urn:file:week.json .features|length} earthquakes of M4.5 and up; the largest, $j{urn:file:week.json .features | max_by(.properties.mag) | .properties.title}.
source urn:iki:fn:compose src=urn:file:headline.txt
```

```text
wrote 171 bytes to headline.txt
[uncacheable]
This week: 189 earthquakes of M4.5 and up; the largest, M 7.0 - 191 km ENE of Nagqu, China.
[computed]
```

The headline is as fresh as the copy it reads: save the feed to `week.json` again and the
next compose recomputes it. What you cannot do yet is point the template straight at
`urn:http:get` with its `url`, because a request with arguments cannot be named as one
resource. That waits on named requests, designed and not yet built
(ledger #2447), and the page this part
builds later is where it matters.

Next, the questions get names of their own under `urn:quakes:`, and the data becomes a
graph that keeps USGS's names for each earthquake: [Giving the data names](names.md). Later
in this part, the headline grows into a page.
