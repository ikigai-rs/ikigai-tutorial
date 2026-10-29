# Composites, one layer at a time

A **composite** is a resource computed from other resources. The graph in
[Giving the data names](names.md) was one already: a template, filled from the context and the
week. This chapter builds taller ones, a layer at a time, and each layer is a name the next one
reads. Then it asks the kernel what hangs from what, and what a composite costs when one of the
things it reads changes every minute.

The Run cells use this page's kernel and the recorded weeks, so their answers are the ones
printed beside them. The shell steps that write files are replayed against `ikigai-cli`
{{#include ../../ikigai-cli.version}} by the book's checks. The live feed's steps are not,
because the feed moves: they are what the REPL printed on 2026-10-29, and they say so.

## 1. Filter: a question whose answer is a graph

The questions in the last chapter answered rows. A layer that another layer reads has to
answer something that can be read the same way the graph was: a graph. SPARQL's `CONSTRUCT`
does that. This one keeps the earthquakes of magnitude 5.5 and up, with the facts the next
layers need:

<div class="ikigai-run" data-cmd='source urn:quakes:question:strong'>
<pre class="ikigai-run-expected">PREFIX qk: &lt;https://ikigai-rs.dev/ns/quakes#&gt; CONSTRUCT { ?quake a qk:Earthquake ; qk:mag ?mag ; qk:magType ?magType ; qk:place ?place ; qk:time ?time } WHERE { ?quake a qk:Earthquake ; qk:mag ?mag ; qk:magType ?magType ; qk:place ?place ; qk:time ?time FILTER(?mag &gt;= 5.5) }
[computed]</pre>
</div>

`FILTER(?mag >= 5.5)` is the filter; everything else copies statements across. Because `mag`
is typed as a number by the context, `>=` compares numbers, not text.

This page holds the question's answer at a name of its own, `urn:quakes:strong`: the question
read through the kernel and asked of `urn:quakes:graph`, whenever someone reads the name. So the
next layer can ask its questions of it, exactly as the last chapter asked them of the graph. The
count question from that chapter, unchanged, now counts the strong ones:

<div class="ikigai-run" data-cmd='source urn:quakes:question:count | urn:sparql:select graph=urn:quakes:strong as=text/csv
cache urn:quakes:strong'>
<pre class="ikigai-run-expected">quakes
12
[2 computed]
cached</pre>
</div>

Twelve of the week's 178, and the filtered graph is now cached under its name. The count
question did not change: `a qk:Earthquake` matched because the filter's `CONSTRUCT` kept each
earthquake's type.

In this page the questions are fixed, because they are compiled into it. At a shell each one is
a `.rq` file you write and edit, and [section 6](#6-at-a-shell-your-own-notes) writes one.

## 2. Sort: biggest first

The second layer reads the first by name. `ORDER BY DESC(?mag)` puts the biggest first, and
`?place` after it breaks ties, so the order is the same every time:

<div class="ikigai-run" data-cmd='source urn:quakes:question:strongest-first | urn:sparql:select graph=urn:quakes:strong as=text/csv'>
<pre class="ikigai-run-expected">quake,mag,place
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xl,7.5,&quot;2024 Noto Peninsula, Japan Earthquake&quot;
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xm,6.2,&quot;8 km SW of Anamizu, Japan&quot;
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xj,5.8,&quot;42 km NE of Anamizu, Japan&quot;
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m1ld,5.7,&quot;125 km W of Houma, Tonga&quot;
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0ya,5.6,&quot;54 km NE of Anamizu, Japan&quot;
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0xt,5.6,&quot;7 km N of Anamizu, Japan&quot;
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m2cv,5.6,Balleny Islands region
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m26y,5.6,Carlsberg Ridge
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m0y9,5.5,&quot;56 km NE of Anamizu, Japan&quot;
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m1cg,5.5,&quot;68 km WNW of Vinchina, Argentina&quot;
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m2d7,5.5,Balleny Islands region
https://earthquake.usgs.gov/earthquakes/eventpage/us6000m27f,5.5,southern East Pacific Rise
[2 computed]</pre>
</div>

Seven of the twelve are the Noto Peninsula earthquake and its aftershocks, near Anamizu.

## 3. Group and count: by magnitude type

The last chapter's `by-magtype` question, asked of the strong layer instead of the whole week:

<div class="ikigai-run" data-cmd='source urn:quakes:question:by-magtype | urn:sparql:select graph=urn:quakes:strong as=text/csv'>
<pre class="ikigai-run-expected">magType,quakes
mb,4
mwb,1
mww,7
[2 computed]</pre>
</div>

Over the whole week, `mb` (body-wave magnitude) measured 143 of 178. Among the strong ones,
`mww` (moment magnitude, the measure USGS prefers for large earthquakes) measured most. The
question is one resource, read by two layers; nothing was copied.

## 4. A join: the week after

USGS's service gives any window you ask for. This page holds a second one, recorded the same
way as the first: the week after, 2024-01-08 to 2024-01-15, 105 earthquakes, at
`urn:file:quakes-2024-01-08.geojson`, and filled through the same context at
`urn:quakes:graph:2024-01-08`.

`urn:sparql:select` loads every graph it is given, so a question can read the strong layer and
the second week together. This one asks, for each region where the first week had a strong
earthquake, how many earthquakes the second week had there. The region is the end of USGS's
`place`, after its last comma, and the two weeks are told apart by time: `1704672000000` is
2024-01-08T00:00Z in USGS's milliseconds.

<div class="ikigai-run" data-cmd='source urn:quakes:question:week-after | urn:sparql:select graph="urn:quakes:strong urn:quakes:graph:2024-01-08" as=text/csv'>
<pre class="ikigai-run-expected">region,strongest,weekAfter
Japan,7.5,6
Tonga,5.7,9
Balleny Islands region,5.6,0
Carlsberg Ridge,5.6,0
Argentina,5.5,2
southern East Pacific Rise,5.5,0
[2 computed]</pre>
</div>

Six more in Japan, after the aftershocks of the first week; nine in Tonga; none on the ridges
and the Balleny Islands. The `GROUP BY ?region` is the join: the first week's strong earthquakes
and the second week's earthquakes meet on the region they share. The question is
`urn:quakes:question:week-after`, a file like the others, in
[`crates/quakes/queries/`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates/quakes/queries).

## 5. What hangs from what

Every layer read its sources through the kernel, so every answer hangs from the threads of
what it read. The context is under everything:

<div class="ikigai-run" data-cmd='source urn:kernel:dependents thread=urn:file:quakes.context.jsonld'>
<pre class="ikigai-run-expected">dependents of urn:file:quakes.context.jsonld  (cut 0 times)
  10 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:file:quakes.context.jsonld  application/ld+json           918 B  1 thread    root
  urn:json:query                  application/json              746 B  2 threads   root
  urn:quakes:graph                application/ld+json        123.6 KB  5 threads   root
  urn:quakes:graph:2024-01-08     application/ld+json         73.9 KB  5 threads   root
  urn:quakes:strong               text/turtle                  4.5 KB  8 threads   root
  urn:sparql:construct            text/turtle                  4.5 KB  6 threads   root
  urn:sparql:select               text/csv                       12 B  10 threads  root
  urn:sparql:select               text/csv                       36 B  10 threads  root
  urn:sparql:select               text/csv                      158 B  13 threads  root
  urn:sparql:select               text/csv                     1.1 KB  10 threads  root
[uncacheable]</pre>
</div>

Both graphs, the strong layer, the `CONSTRUCT` behind it, and each answer above. The join is the
one with 13 threads: it read both weeks.

A question is a thread too. Ask what hangs from the filter's:

<div class="ikigai-run" data-cmd='source urn:kernel:dependents thread=urn:quakes:question:strong'>
<pre class="ikigai-run-expected">dependents of urn:quakes:question:strong  (cut 0 times)
  6 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:quakes:question:strong  application/sparql-query      275 B  1 thread    root
  urn:quakes:strong           text/turtle                  4.5 KB  8 threads   root
  urn:sparql:select           text/csv                       12 B  10 threads  root
  urn:sparql:select           text/csv                       36 B  10 threads  root
  urn:sparql:select           text/csv                      158 B  13 threads  root
  urn:sparql:select           text/csv                     1.1 KB  10 threads  root
[uncacheable]</pre>
</div>

The filter's answer and everything read from it: the count, the sort, the grouping and the
join. The graphs do not hang from it, and neither does the `CONSTRUCT` (the layer handed it the
question's text, not its name). Cut the question's thread, the way a change to the `.rq` file
would, and ask the grouping again:

<div class="ikigai-run" data-cmd='sink urn:kernel:cut urn:quakes:question:strong
cache urn:quakes:strong
cache urn:quakes:graph
source urn:quakes:question:by-magtype | urn:sparql:select graph=urn:quakes:strong as=text/csv'>
<pre class="ikigai-run-expected">cut urn:quakes:question:strong
[uncacheable]
not cached
cached
magType,quakes
mb,4
mwb,1
mww,7
[1 cached · 1 computed]</pre>
</div>

The strong layer was stale and was computed again; the graph under it was still cached, because
the question is not one of its sources. The grouping's own question was cached, and its answer
recomputed, from the recomputed layer. A cut reaches exactly what hangs from the thread.

## 6. At a shell: your own notes

The REPL has none of this page's `urn:quakes:` names, so at a shell each layer is a file. In
the REPL you started over `~/quakes`, download the week after beside the first:

```bash
cd ~/quakes
curl -sO https://raw.githubusercontent.com/ikigai-rs/ikigai-tutorial/main/crates/quakes/quakes-2024-01-08.geojson
```

<!-- transcripts: workspace crates/quakes/quakes-2024-01-01.geojson crates/quakes/quakes-2024-01-08.geojson books/ikigai/ns/quakes.context.jsonld -->

Fill both weeks as JSON-LD, as [the last chapter](names.md#6-at-a-shell) did for one (if you
still have `graph.txt` and `quakes.jsonld` from it, the first two lines are done already):

```text
ikigai> sink urn:file:graph.txt {"@context": $r{urn:json:query?src=urn:file:quakes.context.jsonld&q=.[]}, "@graph": [$r{urn:file:quakes-2024-01-01.geojson}]}
wrote 125 bytes to graph.txt
[uncacheable]
ikigai> source urn:iki:fn:compose src=urn:file:graph.txt | sink urn:file:quakes.jsonld
wrote 126539 bytes to quakes.jsonld
[1 computed · 1 uncacheable]
ikigai> sink urn:file:graph-2024-01-08.txt {"@context": $r{urn:json:query?src=urn:file:quakes.context.jsonld&q=.[]}, "@graph": [$r{urn:file:quakes-2024-01-08.geojson}]}
wrote 125 bytes to graph-2024-01-08.txt
[uncacheable]
ikigai> source urn:iki:fn:compose src=urn:file:graph-2024-01-08.txt | sink urn:file:quakes-2024-01-08.jsonld
wrote 75659 bytes to quakes-2024-01-08.jsonld
[1 computed · 1 uncacheable]
```

The notes are yours: a file for each earthquake you have something to say about, named after
USGS's id for it, under `notes/`. Write two, and the one that stands in for a note not
written yet:

<!-- transcript: continues -->

```text
ikigai> sink urn:file:notes/us6000m0xl.txt Felt in Kanazawa. The tsunami warning came first.
wrote 49 bytes to notes/us6000m0xl.txt
[uncacheable]
ikigai> sink urn:file:notes/us6000m2jp.txt Mindanao again.
wrote 15 bytes to notes/us6000m2jp.txt
[uncacheable]
ikigai> sink urn:file:notes/none.txt no note yet
wrote 11 bytes to notes/none.txt
[uncacheable]
```

Now the earthquakes the notes are about: the ones of magnitude 6 and up in either week, with
each one's id. A question with both graphs reads both weeks; its answer goes to a file, because
at a shell an answer with arguments has no name of its own yet (named requests, ledger #2447),
and `urn:json:query` turns the CSV into the JSON list the next step reads:

<!-- transcript: continues -->

```text
ikigai> sink urn:file:queries/notable.rq PREFIX qk: <https://ikigai-rs.dev/ns/quakes#> SELECT ?id ?mag ?place WHERE { ?quake qk:mag ?mag ; qk:place ?place FILTER(?mag >= 6) BIND(STRAFTER(STR(?quake), "eventpage/") AS ?id) } ORDER BY DESC(?mag)
wrote 202 bytes to queries/notable.rq
[uncacheable]
ikigai> source urn:file:queries/notable.rq | urn:sparql:select graph="urn:file:quakes.jsonld urn:file:quakes-2024-01-08.jsonld" as=text/csv | sink urn:file:notable.csv
wrote 216 bytes to notable.csv
[2 computed · 1 uncacheable]
ikigai> source urn:json:query src=urn:file:notable.csv q=. | sink urn:file:notable.json
wrote 299 bytes to notable.json
[1 computed · 1 uncacheable]
```

The join with your notes is a template for one row, filled once for each earthquake by
`urn:iki:fn:each`. `{id}` is the row's id, so `urn:file:notes/{id}.txt` is that earthquake's
note, and `||` names what to splice when there is none:

<!-- transcript: continues -->

```text
ikigai> sink urn:file:row.txt <li>M$h{{mag}} $h{{place}}: $h{urn:file:notes/{id}.txt || urn:file:notes/none.txt}</li>
wrote 87 bytes to row.txt
[uncacheable]
ikigai> source urn:iki:fn:each src=urn:file:notable.json template=urn:file:row.txt
<li>M7.5 2024 Noto Peninsula, Japan Earthquake: Felt in Kanazawa. The tsunami warning came first.</li><li>M6.7 93 km SE of Sarangani, Philippines: Mindanao again.</li><li>M6.4 45 km SSW of Jurm, Afghanistan: no note yet</li><li>M6.2 8 km SW of Anamizu, Japan: no note yet</li>
[computed]
```

One answer from three kinds of source: two recorded weeks of USGS's, read as a graph; a
template of yours; and your notes, one file each. (The rows come out on one line, as HTML list
items: a line typed at the prompt cannot hold a line break, so the template cannot end with
one.)

Ask what hangs from a note, and from a note that does not exist yet:

<!-- transcript: continues -->

```text
ikigai> source urn:kernel:dependents thread=urn:file:notes/us6000m0xl.txt
dependents of urn:file:notes/us6000m0xl.txt  (cut 2 times)
  2 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:file:notes/us6000m0xl.txt  text/plain                     49 B  1 thread    root
  urn:iki:fn:each                text/plain                    276 B  8 threads   root

[uncacheable]
ikigai> source urn:kernel:dependents thread=urn:file:notes/us6000m31m.txt
dependents of urn:file:notes/us6000m31m.txt  (cut 0 times)
  1 live cached entry hangs from it — a cut makes it stale, and the next read of each recomputes it
  urn:iki:fn:each  text/plain                    276 B  8 threads   root

[uncacheable]
```

The list hangs from the note it spliced (its eight threads are the four notes it looked for,
the fallback, the template, the JSON and its own name), and also from the note it looked for and did not find: the Jurm earthquake's, `us6000m31m`. Write that note, and the list is stale; the CSV and
the JSON under it are not, because no note is one of their sources:

<!-- transcript: continues -->

```text
ikigai> sink urn:file:notes/us6000m31m.txt East of Herat.
wrote 14 bytes to notes/us6000m31m.txt
[uncacheable]
ikigai> cache urn:iki:fn:each src=urn:file:notable.json template=urn:file:row.txt
not cached
ikigai> cache urn:json:query src=urn:file:notable.csv q=.
cached
ikigai> source urn:iki:fn:each src=urn:file:notable.json template=urn:file:row.txt
<li>M7.5 2024 Noto Peninsula, Japan Earthquake: Felt in Kanazawa. The tsunami warning came first.</li><li>M6.7 93 km SE of Sarangani, Philippines: Mindanao again.</li><li>M6.4 45 km SSW of Jurm, Afghanistan: East of Herat.</li><li>M6.2 8 km SW of Anamizu, Japan: no note yet</li>
[computed]
```

The files in the middle are where this differs from the page. `notable.json` is a copy: it
does not hang from the weeks, and it changes only when you write it again. In the page,
`urn:quakes:strong` is a name, not a copy, and a cut of what it read reaches it.

## 7. What a composite costs

A composite is cached for as long as everything it read is. The recorded weeks and your notes
change only when someone writes them. The live feed from
[Somebody else's resource](http.md) is good for 60 seconds, because USGS says so. Put the feed
into a composite and the composite is good for 60 seconds too.

These steps reach the network, so the book's checks cannot replay them. They are what
`ikigai-cli` 0.2.5 printed on 2026-10-29. Make the list a page fragment of its own, then join
the live feed into it. A template can splice the feed by its URL, through `urn:http:get`; these
steps count the answer's lines with `urn:text:wc` rather than print 130 KB:

```text
sink urn:file:report.txt <ul>$r{urn:iki:fn:each?src=urn:file:notable.json&template=urn:file:row.txt}</ul>
sink urn:file:report.txt <ul>$r{urn:iki:fn:each?src=urn:file:notable.json&template=urn:file:row.txt}</ul> $r{urn:http:get?url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson}
source urn:iki:fn:compose src=urn:file:report.txt | urn:text:wc
cache urn:iki:fn:compose src=urn:file:report.txt
```

```text
wrote 80 bytes to report.txt
[uncacheable]
wrote 176 bytes to report.txt
[uncacheable]
196
[2 computed]
cached
```

A minute later:

```text
cache urn:iki:fn:compose src=urn:file:report.txt
cache urn:iki:fn:each src=urn:file:notable.json template=urn:file:row.txt
source urn:kernel:uncached
```

```text
not cached
cached
uncached (why a computed read was not cached; the last 64, most recent first)
  urn:json:from-csv      ×1  policy  [root]

[uncacheable]
```

The report expired with the feed. The list inside it did not: it is a name of its own, and
it read no feed. Everything that reads the report now expires every minute too, however
expensive it is to compute.

`urn:kernel:uncached` says nothing about it: the one line there is from the CSV conversion at
the shell step before. It logs computations that were never stored, and the report was stored;
it was stored for a minute. Nothing in the kernel's readouts says how
long an entry has left, so the way to see the lifetime is the one above: ask `cache` again
after it.

The fix is to join volatile data where it costs nothing to recompute: into a resource that is
cheap, or already uncacheable, never into the expensive cached one. Put the report back as it
was, and splice it by name into a small template of its own that adds the feed:

```text
sink urn:file:report.txt <ul>$r{urn:iki:fn:each?src=urn:file:notable.json&template=urn:file:row.txt}</ul>
sink urn:file:live.txt $r{urn:iki:fn:compose?src=urn:file:report.txt} $r{urn:http:get?url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson}
source urn:iki:fn:compose src=urn:file:live.txt | urn:text:wc
source urn:iki:fn:compose src=urn:file:report.txt
```

```text
wrote 80 bytes to report.txt
[uncacheable]
wrote 142 bytes to live.txt
[uncacheable]
196
[2 computed]
<ul><li>M7.5 2024 Noto Peninsula, Japan Earthquake: Felt in Kanazawa. The tsunami warning came first.</li><li>M6.7 93 km SE of Sarangani, Philippines: Mindanao again.</li><li>M6.4 45 km SSW of Jurm, Afghanistan: East of Herat.</li><li>M6.2 8 km SW of Anamizu, Japan: no note yet</li></ul>
[cached]
```

The report was cached already: filling `live.txt` read it by name, and that read kept it.
A minute later:

```text
cache urn:iki:fn:compose src=urn:file:report.txt
cache urn:iki:fn:compose src=urn:file:live.txt
source urn:iki:fn:compose src=urn:file:live.txt | sink urn:file:page.html
```

```text
cached
not cached
wrote 137620 bytes to page.html
[1 computed · 1 uncacheable]
```

Only the small template expired. The report it splices was still cached, so the next read
computed one splice and one fetch, not the list. The write at the end is uncacheable anyway,
which is the last place volatile data can go: a write happens every time it is asked for.

What each layer costs is what it reads. A layer that reads only recorded data and your files
lasts until one of them is written; one that reads the feed lasts a minute; and a layer that
reads another by name pays only for its own work when it is recomputed. Later in this part,
a page built from them is served over HTTP.
