# A river of feeds

The earthquakes came from one publisher, in one format, on one schedule. A feed reader has
the opposite problem: many small publishers, each with a feed of their own, each deciding
how often it changes and how long a copy stays good. This chapter builds a reader of that
kind out of the same parts as the rest of this part, SPARQL among them, as [Giving the data
names](names.md) asked it of the earthquakes, plus one more: XSLT, which reads XML.

The feeds are a directory of personal blogs, and a directory is a list of names. Each feed
becomes RDF, the feeds together become one graph called the **river**, questions about the
river are files, and one answer is written out four ways: a web page, a digest, a feed of
its own and a directory of the feeds that matched. Every step is a REPL line. The only code
is XSLT, SPARQL and three files of REPL lines.

## What you need

The `ikigai` command, 0.2.5 or later (section 2's revalidation needs 0.2.6), and a copy of
this book's repository, which holds the example in `examples/rss/`:

```bash
git clone https://github.com/ikigai-rs/ikigai-tutorial
cd ikigai-tutorial/examples/rss
./iki --plain
```

`./iki` starts the REPL over the example's own workspace, `ws/`, with a configuration
directory of its own (`.home/`) and none of your machine's mounts, so nothing here touches
the CLI's default workspace. It runs `ikigai` from your PATH; name another binary with
`IKIGAI=`.

The feeds are a fixture: four invented blogs under `example.com` and `example.org`, domains
reserved for examples, shaped like the real feeds in a directory of personal sites (their
dates, their `guid` forms, HTML in descriptions, missing fields, a podcast with no `guid`).
Nothing in `ws/` came from a real person's feed, and nothing in this chapter reads one. The
example's README ends with a live run, which is your choice to make, rarely.

<!-- transcripts: workspace examples/rss/ws -->

The book's checks replay every `ikigai>` block on this page against `ikigai-cli`
{{#include ../../ikigai-cli.version}}, over a copy of `ws/` as git has it, so those answers are
the ones you get. Each block is a fresh REPL unless the text says it goes on from the one
before. The steps against the stand-in origin in section 2 are not replayed, and the section
says why.

Sections 1, 3, 5 and 6 are also Run cells, which run in this page, on a kernel compiled into
it, under the CLI's own engine. Its `urn:file:` names are the same files with the same bytes,
compiled in, so a cell and a REPL line name one resource. The cells on the page share one
kernel and one cache, so run them in order. The page cannot do two things: write a file, and
watch one. So `urn:file:river.nt` in a cell is the copy the example commits, the one section
4's build line writes (`check.sh` holds the two to each other). Sections 2, 4 and 7 stay
REPL transcripts: section 2 needs a web server, section 4 writes the river and watches the
workspace, and section 7 writes and lists files.

## 1. A directory is a list of names

`ws/directory.opml` is the directory. OPML is a small XML format for lists of feeds; each
`<outline>` names one, with its address in `xmlUrl`. A directory of four looks like this,
one line per feed:

```xml
<outline type="rss" version="RSS" text="Alice Writes Things" title="Alice Writes Things" description="Notes on static sites, feeds and the small web." xmlUrl="https://alice.example.com/feed.xml" htmlUrl="https://alice.example.com/"/>
```

`urn:xslt:transform` runs an XSLT stylesheet over a resource: `src` is what to read and
`stylesheet` the stylesheet, and both are names, read through the kernel. `ws/feeds.xsl`
writes each feed's address on a line of its own. `ws/feeds-local.xsl` writes, for each feed,
the name of the fixture's copy of it, `urn:file:feeds/{host}.xml`:

<div class="ikigai-run" data-cmd='source urn:xslt:transform src=urn:file:directory.opml stylesheet=urn:file:feeds.xsl
source urn:xslt:transform src=urn:file:directory.opml stylesheet=urn:file:feeds-local.xsl'>
<pre class="ikigai-run-expected">https://alice.example.com/feed.xml
https://bob.example.org/feed/
https://carol.example.com/index.xml
https://dave.example.org/feed
[computed]
urn:file:feeds/alice.example.com.xml
urn:file:feeds/bob.example.org.xml
urn:file:feeds/carol.example.com.xml
urn:file:feeds/dave.example.org.xml
[computed]</pre>
</div>

A list with one name per line is the shape `..` maps over ([A lab with names in
it](lab.md#4-a-map) mapped `toUpper` over places). Both lists are answers like any other:
cached, and hanging from the directory's thread and the stylesheet's. Add a feed to the
OPML and the list changes; the rest of this chapter uses the second one, so nothing leaves
your machine.

## 2. Somebody else's rules

[Somebody else's resource](http.md#2-how-long-a-copy-stays-good) read a feed whose
publisher said how long a copy stayed good: sixty seconds. Personal feeds mostly say less.
In one polite pass over a real directory of 115 personal feeds, on 2026-10-28, one request
each, the headers said this (counts only; which site said what is not recorded here):

| What the feed's answer said | Feeds |
|---|---|
| No `Cache-Control` at all | 39 |
| `max-age=0` | 28 |
| A copy may be kept more than ten seconds | 30 of 112 |
| An `ETag` | 95 |
| 304 Not Modified, to a conditional request | 90 |

So a copy of most of these feeds may barely be kept, but nearly all of them can say "not
changed" in a few bytes when asked properly. A reader that honors the first and uses the
second downloads almost nothing on a feed that has not changed.

None of that can be shown against a real person's site, and this book never tries. The
example has a stand-in origin instead, `tools/origin.py`: a small web server on your own
machine that serves the fixture feeds with whatever cache headers the path asks for
(`/cc/max60/…`, `/cc/etag/…`, `/cc/none/…`, and a few more listed in the file) and logs
every request it is sent. Start it in a second terminal:

```bash
python3 -I tools/origin.py ws/feeds /tmp/origin.log
```

It prints the port it chose, `port 46413` here. The port is different every run, which is
why the blocks in this section are not replayed: each one prints the port. Yours will show
your port; everything else is what `ikigai-cli` 0.2.6 printed on 2026-10-29.

A feed that says `max-age=60` is kept for sixty seconds, as USGS's was. The second read is
served from the cache, and the origin's log shows one request:

```text
source urn:http:get url=http://127.0.0.1:46413/cc/max60/alice.example.com.xml | urn:text:wc
source urn:http:get url=http://127.0.0.1:46413/cc/max60/alice.example.com.xml | urn:text:wc
cache urn:http:get url=http://127.0.0.1:46413/cc/max60/alice.example.com.xml
```

```text
38
[2 computed]
38
[2 cached]
cached
```

A feed that says nothing is not kept at all: every read is a download. You may decide for
yourself how stale a copy you will accept, with `max_age=` in seconds, and say who you are,
with a `User-Agent` header. A publisher reading their logs then sees a tutorial asking once
an hour, not an anonymous client asking every time:

```text
source urn:http:get url=http://127.0.0.1:46413/cc/none/carol.example.com.xml headers="User-Agent: ikigai-tutorial (+https://ikigai-rs.github.io/ikigai-tutorial/)" | urn:text:wc
cache urn:http:get url=http://127.0.0.1:46413/cc/none/carol.example.com.xml headers="User-Agent: ikigai-tutorial (+https://ikigai-rs.github.io/ikigai-tutorial/)"
source urn:http:get url=http://127.0.0.1:46413/cc/none/carol.example.com.xml max_age=3600 headers="User-Agent: ikigai-tutorial (+https://ikigai-rs.github.io/ikigai-tutorial/)" | urn:text:wc
cache urn:http:get url=http://127.0.0.1:46413/cc/none/carol.example.com.xml max_age=3600 headers="User-Agent: ikigai-tutorial (+https://ikigai-rs.github.io/ikigai-tutorial/)"
```

```text
29
[2 uncacheable]
not cached
29
[2 computed]
cached
```

The same feed, kept for an hour because you asked for an hour. `max_age=` does not
override a publisher who forbids a copy: the same line against `/cc/nostore/` (which sends
`Cache-Control: no-store`) stays `[2 uncacheable]`. And a feed that sends an `ETag` and
nothing else:

```text
source urn:http:get url=http://127.0.0.1:46413/cc/etag/bob.example.org.xml | urn:text:wc
source urn:http:get url=http://127.0.0.1:46413/cc/etag/bob.example.org.xml | urn:text:wc
source urn:kernel:uncached
```

```text
49
[2 computed]
49
[2 computed]
uncached (why a computed read was not cached; the last 64, most recent first)
  urn:text:wc      ×2  expired 1793296165726  [root]

[uncacheable]
```

Nothing in the headers allows a copy to be served as it is, but the `ETag` allows the next
best thing, and `ikigai-cli` 0.2.6 (with ikigai-http 0.5.0) uses it.
The answer is stored already expired, kept for **revalidation**: the second read sends
`If-None-Match` with the stored `ETag`, the origin answers **304 Not Modified** with no body,
and the stored body is served. The origin's log shows one download and one 304:

```text
{"t": 1793296165.725, "path": "/cc/etag/bob.example.org.xml", "if-none-match": null, "if-modified-since": null, "user-agent": "ikigai/0.2.6", "accept-encoding": "gzip", "status": 200}
{"t": 1793296165.726, "path": "/cc/etag/bob.example.org.xml", "if-none-match": "\"985b241130e43cac\"", "if-modified-since": null, "user-agent": "ikigai/0.2.6", "accept-encoding": "gzip", "status": 304}
```

Both reads still say `computed`: the stored body stays expired, so every read asks the
origin again, `cache` on the URL says `not cached` (`source urn:kernel:cache` lists the body
as `expired`), and the `wc` computed from it is not kept either. `urn:kernel:uncached` says
so, with the moment its upstream expired (in milliseconds since 1970). What the 304 saves is
the download, not the computation. On 0.2.5, which kept nothing for revalidation, the same
two reads were `[2 uncacheable]` and the log showed two full 200s with no `If-None-Match`.

Stop the origin when you are done (Ctrl-C in its terminal). The rest of the chapter reads
the fixture's files.

## 3. XSLT reads RSS directly

RSS 1.0 was RDF; RSS 2.0, the format nearly every feed uses now, is plain XML. So turning a
feed into RDF needs no new vocabulary, only a stylesheet. `ws/items.xsl` reads an RSS 2.0
feed and writes RDF/XML in RSS 1.0's own terms (`rss:item`, `rss:title`, `rss:link`,
`rss:description`) plus Dublin Core for the rest (`dc:subject` for a category, `dc:creator`,
`dcterms:isPartOf` for the feed an item belongs to). `urn:rdf:transrept` then rewrites the
RDF/XML as N-Triples, one statement per line.

An item in RDF needs a name, an IRI, and an RSS item has two candidates. Its `guid` is the
publisher's own identifier, but it is only a web address when the feed says
`isPermaLink` (the default) and it starts with `http`. Its `link` is always an address. The
stylesheet takes the `guid` when it is a permalink and the `link` otherwise, and keeps the
`guid` as `dc:identifier` either way. Carol's feed uses `tag:` identifiers, which are not
web addresses, so her items are named by their links:

<div class="ikigai-run" data-cmd='source urn:xslt:transform src=urn:file:feeds/carol.example.com.xml stylesheet=urn:file:items.xsl | urn:rdf:transrept as=application/n-triples | urn:text:grep pattern=rss-cloud'>
<pre class="ikigai-run-expected">&lt;https://carol.example.com/2026/10/24/rss-cloud/&gt; &lt;http://www.w3.org/1999/02/22-rdf-syntax-ns#type&gt; &lt;http://purl.org/rss/1.0/item&gt; .
&lt;https://carol.example.com/2026/10/24/rss-cloud/&gt; &lt;http://purl.org/dc/terms/isPartOf&gt; &lt;https://carol.example.com/index.xml&gt; .
&lt;https://carol.example.com/2026/10/24/rss-cloud/&gt; &lt;http://purl.org/rss/1.0/link&gt; &quot;https://carol.example.com/2026/10/24/rss-cloud/&quot; .
&lt;https://carol.example.com/2026/10/24/rss-cloud/&gt; &lt;http://purl.org/dc/elements/1.1/identifier&gt; &quot;tag:carol.example.com,2026-10-24:/rss-cloud&quot; .
&lt;https://carol.example.com/2026/10/24/rss-cloud/&gt; &lt;http://purl.org/rss/1.0/description&gt; &quot;&lt;p&gt;Tried &lt;a href=\&quot;https://rpc.example.net/\&quot;&gt;rssCloud&lt;/a&gt; today: the feed tells readers when it changes, so nobody has to poll.&lt;/p&gt;&quot; .
&lt;https://carol.example.com/2026/10/24/rss-cloud/&gt; &lt;http://purl.org/dc/terms/issued&gt; &quot;Sat, 24 Oct 2026 16:45:00 GMT&quot; .
&lt;https://carol.example.com/2026/10/24/rss-cloud/&gt; &lt;http://purl.org/dc/elements/1.1/date&gt; &quot;2026-10-24T16:45:00Z&quot;^^&lt;http://www.w3.org/2001/XMLSchema#dateTime&gt; .
[3 computed]</pre>
</div>

That item has no title, which RSS 2.0 allows; the river's queries call it `(untitled)`. Its
description is HTML, kept as text. And its date is there twice. RSS dates are written the
way email dates are, `Sat, 24 Oct 2026 16:45:00 GMT`, which nothing can sort or compare.
The stylesheet keeps that as `dcterms:issued`, and when it can read it, adds a `dc:date` in
the form SPARQL compares, `2026-10-24T16:45:00Z`, typed `xsd:dateTime`. A date it cannot
read gets no `dc:date` rather than a wrong one.

Dave's podcast has no `guid` at all, so its episodes are named by their links too. Its dates
use a one-digit day and an offset, `Wed, 7 Oct 2026 10:00:00 -0700`, and come out keeping
the offset:

<div class="ikigai-run" data-cmd='source urn:xslt:transform src=urn:file:feeds/dave.example.org.xml stylesheet=urn:file:items.xsl | urn:rdf:transrept as=application/n-triples | urn:text:grep pattern=date'>
<pre class="ikigai-run-expected">&lt;https://dave.example.org/episodes/12&gt; &lt;http://purl.org/dc/elements/1.1/date&gt; &quot;2026-10-07T10:00:00-07:00&quot;^^&lt;http://www.w3.org/2001/XMLSchema#dateTime&gt; .
&lt;https://dave.example.org/episodes/11&gt; &lt;http://purl.org/dc/elements/1.1/date&gt; &quot;2026-09-23T10:00:00-07:00&quot;^^&lt;http://www.w3.org/2001/XMLSchema#dateTime&gt; .
&lt;https://dave.example.org/episodes/10&gt; &lt;http://purl.org/dc/elements/1.1/date&gt; &quot;2026-09-09T10:00:00-07:00&quot;^^&lt;http://www.w3.org/2001/XMLSchema#dateTime&gt; .
[3 computed]</pre>
</div>

The stylesheet's comments name the places it works around the XSLT engine's gaps (how it
gets a newline into the text output, why one IRI lives in a variable), so it reads as a
record of what that engine does today, not only as a recipe.

## 4. The river

Now the whole directory at once. Section 1's list of names, mapped (`..`) over the two
stages in parentheses, so each feed is lifted to RDF on its own; the four answers joined,
and the result written to `ws/river.nt`:

```text
ikigai> source urn:xslt:transform src=urn:file:directory.opml stylesheet=urn:file:feeds-local.xsl .. ( urn:xslt:transform stylesheet=urn:file:items.xsl | urn:rdf:transrept as=application/n-triples ) | sink urn:file:river.nt
wrote 14886 bytes to river.nt
[9 computed · 1 uncacheable]
ikigai> source urn:xslt:transform src=urn:file:directory.opml stylesheet=urn:file:feeds-local.xsl .. ( urn:xslt:transform stylesheet=urn:file:items.xsl | urn:rdf:transrept as=application/n-triples ) | sink urn:file:river.nt
wrote 14886 bytes to river.nt
[9 cached · 1 uncacheable]
```

Each name on the list reaches `urn:xslt:transform` as its `src`, the one argument the stage
leaves out. The first build computed nine answers: the list, and two stages for each of four
feeds. The write is the one uncacheable step, because a write has to happen each time it is
asked for. The second build, the same line again, found all nine in the cache, so only the
write ran. (The `--plain` REPL also prints `[fan-out 4 → 1 wide]` under each build; [the
grammar](../front-door/grammar.md#the-map-) explains it.)

Ask what hangs from one feed, then say that another one changed and build again:

<!-- transcript: continues -->

```text
ikigai> dependents urn:file:feeds/dave.example.org.xml
dependents of urn:file:feeds/dave.example.org.xml  (cut 0 times)
  3 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:file:feeds/dave.example.org.xml  text/xml                     1.4 KB  1 thread    root
  urn:rdf:transrept                    application/n-triples        2.8 KB  6 threads   root
  urn:xslt:transform                   application/xml              2.3 KB  5 threads   root
[uncacheable]
ikigai> sink urn:kernel:cut urn:file:feeds/alice.example.com.xml
cut urn:file:feeds/alice.example.com.xml
[uncacheable]
ikigai> source urn:xslt:transform src=urn:file:directory.opml stylesheet=urn:file:feeds-local.xsl .. ( urn:xslt:transform stylesheet=urn:file:items.xsl | urn:rdf:transrept as=application/n-triples ) | sink urn:file:river.nt
wrote 14886 bytes to river.nt
[7 cached · 2 computed · 1 uncacheable]
```

Three things hang from Dave's feed: the file, and the two stages that lifted it. Nothing else
in the river does, so a change to Dave's feed can make only those stale. The cut says
Alice's feed changed, and the rebuild computes exactly two answers, the two stages of
Alice's lift. The other three feeds' lifts, and the list, come from the cache.

Editing the file does the same, with no cut by hand: the CLI watches its workspace and cuts a
file's thread when the file changes. Change a title in `ws/feeds/dave.example.org.xml` with
your editor and build again, and the tally is the same, `[7 cached · 2 computed · 1
uncacheable]`, with the new title in `river.nt` (and 7 more bytes, if you add `, again` to
it). Put the file back afterwards (`git checkout ws/feeds`), because the next sections expect
the fixture as it was.

## 5. Questions are files

The questions are SPARQL, kept in `ws/queries/` as files, so each is a resource with a
name. `urn:sparql:select` runs one over a graph: pipe the query in, name the graph with
`graph=`, and choose the answer's form with `as=`. The river's census, how many items each
feed holds and its newest:

<div class="ikigai-run" data-cmd='source urn:file:queries/per-feed.rq | urn:sparql:select graph=urn:file:river.nt as=text/csv'>
<pre class="ikigai-run-expected">feedTitle,items,newest
Bob's Workbench,3,2026-10-27T21:14:09-05:00
Alice Writes Things,3,2026-10-26T09:30:00Z
carol.example.com,3,2026-10-24T16:45:00Z
Dave Talks,3,2026-10-07T10:00:00-07:00
[2 computed]</pre>
</div>

The other queries have a blank to fill: `since.rq` a day, `in.rq` a category, `from.rq` a
feed and `search.rq` some text. Each blank is a `$r{{name}}` marker, and
`urn:iki:fn:compose`, which filled a template in [Reshaping JSON with
jq](jq.md#4-a-template-that-asks-questions), fills it from an argument of the same name. Their
answers have more columns than fit on a line, so here each goes through
`views/digest.xsl`, which writes a SPARQL answer as a short Markdown list (section 6 is
about views):

<div class="ikigai-run" data-cmd='source urn:iki:fn:compose src=urn:file:queries/since.rq since=2026-10-15 | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl
source urn:iki:fn:compose src=urn:file:queries/in.rq category=caching | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl
source urn:iki:fn:compose src=urn:file:queries/from.rq feed=https://carol.example.com/index.xml | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl
source urn:iki:fn:compose src=urn:file:queries/search.rq q=cach | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl'>
<pre class="ikigai-run-expected"># Digest
&#32;
- 2026-10-27 [My home server caches everything now](https://bob.example.org/2026/10/home-server-cache/) from Bob's Workbench
- 2026-10-26 [Why my feed sends an ETag](https://alice.example.com/posts/ETag/) from Alice Writes Things
- 2026-10-24 [(untitled)](https://carol.example.com/2026/10/24/rss-cloud/) from carol.example.com
- 2026-10-15 [A static site in one afternoon](https://alice.example.com/posts/static-site/) from Alice Writes Things
[3 computed]
# Digest
&#32;
- 2026-10-27 [My home server caches everything now](https://bob.example.org/2026/10/home-server-cache/) from Bob's Workbench
- 2026-10-26 [Why my feed sends an ETag](https://alice.example.com/posts/ETag/) from Alice Writes Things
[3 computed]
# Digest
&#32;
- 2026-10-24 [(untitled)](https://carol.example.com/2026/10/24/rss-cloud/) from carol.example.com
- 2026-10-12 [(untitled)](https://carol.example.com/2026/10/12/coffee/) from carol.example.com
- 2026-10-05 [A longer one about feed readers](https://carol.example.com/2026/10/05/readers/) from carol.example.com
[3 computed]
# Digest
&#32;
- 2026-10-27 [My home server caches everything now](https://bob.example.org/2026/10/home-server-cache/) from Bob's Workbench
- 2026-10-26 [Why my feed sends an ETag](https://alice.example.com/posts/ETag/) from Alice Writes Things
- 2026-09-23 [Episode 11: Caching for people who hate caching](https://dave.example.org/episodes/11) from Dave Talks
[3 computed]</pre>
</div>

A feed is named by its own address (`from.rq` compares it with each item's
`dcterms:isPartOf`). The search is a case-insensitive substring of an item's title, summary,
full text and categories, so `cach` finds "caches", "caching" and "Caching". The summaries are
HTML, so a search also matches markup.

### A term with a quote in it

`compose` splices the term into the query as text. A term with a `"` in it ends the
query's string early, and what follows is read as SPARQL:

<div class="ikigai-run" data-cmd='source urn:iki:fn:compose src=urn:file:queries/search.rq q="href=\"https" | urn:sparql:select graph=urn:file:river.nt as=text/csv'>
<pre class="ikigai-run-expected">error: endpoint error: SPARQL syntax error: error at 18:139: expected NOT</pre>
</div>

Here it is only a syntax error. With a term chosen to be valid SPARQL, the same splice would
change the question, which is why a term somebody else typed must never be spliced.
`urn:sparql:select` has a way that cannot go wrong: `bindings=`, a JSON object of variable
names and values, binds each value into the query *after* it is parsed, so a value is always
a term and never syntax. `queries/search-bound.rq` is the search written for it, with `?q`
where `search.rq` has its marker (a bound variable must be one the query projects, so it
selects `?q` too):

<div class="ikigai-run" data-cmd='source urn:file:queries/search-bound.rq | urn:sparql:select graph=urn:file:river.nt bindings="{\"q\": \"href=\\\"https\"}" as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl'>
<pre class="ikigai-run-expected"># Digest
&#32;
- 2026-10-24 [(untitled)](https://carol.example.com/2026/10/24/rss-cloud/) from carol.example.com
[3 computed]</pre>
</div>

The term is the text `href="https`, quote and all, and it matched the one summary with a
link in it, Carol's. The backslashes are the REPL's quoting, not the query's: inside a
quoted value `\"` is a quotation mark and `\\` a backslash ([the
grammar](../front-door/grammar.md#quoting)), so the value the REPL passes is the JSON
`{"q": "href=\"https"}`.

## 6. One answer, four faces

A SPARQL answer has a standard XML form, `application/sparql-results+xml`, and XSLT reads
XML. So one answer can be written out many ways, a stylesheet for each, and none of them
knows anything about feeds. `ws/views/` holds four:

- `page.xsl`, an HTML page, a list of links with each item's feed and day; everything it
  takes from a feed is written as text, never as markup, so a feed cannot put a script in
  your page;
- `digest.xsl`, a Markdown list, as above;
- `feed.xsl`, an RSS feed of the answer, so a search is something you can subscribe to;
- `opml.xsl`, an OPML directory of the feeds that matched, so the answer to "who writes
  about caching?" can be loaded into a feed reader.

Each face is the same three steps with a different stylesheet at the end. The page, over
`river.rq`, every item in the river, newest first:

<div class="ikigai-run" data-cmd='source urn:file:queries/river.rq | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/page.xsl'>
<pre class="ikigai-run-expected">&lt;html lang='en'&gt;&lt;head&gt;&lt;meta charset='utf-8'/&gt;&lt;title&gt;River&lt;/title&gt;&lt;/head&gt;&lt;body&gt;&lt;h1&gt;River&lt;/h1&gt;&lt;ol&gt;&lt;li&gt;&lt;a href='https://bob.example.org/2026/10/home-server-cache/'&gt;My home server caches everything now&lt;/a&gt; (Bob&amp;apos;s Workbench, 2026-10-27)&lt;/li&gt;&lt;li&gt;&lt;a href='https://alice.example.com/posts/ETag/'&gt;Why my feed sends an ETag&lt;/a&gt; (Alice Writes Things, 2026-10-26)&lt;/li&gt;&lt;li&gt;&lt;a href='https://carol.example.com/2026/10/24/rss-cloud/'&gt;(untitled)&lt;/a&gt; (carol.example.com, 2026-10-24)&lt;/li&gt;&lt;li&gt;&lt;a href='https://alice.example.com/posts/static-site/'&gt;A static site in one afternoon&lt;/a&gt; (Alice Writes Things, 2026-10-15)&lt;/li&gt;&lt;li&gt;&lt;a href='https://carol.example.com/2026/10/12/coffee/'&gt;(untitled)&lt;/a&gt; (carol.example.com, 2026-10-12)&lt;/li&gt;&lt;li&gt;&lt;a href='https://bob.example.org/2026/10/dovetails/'&gt;Dovetails by hand&lt;/a&gt; (Bob&amp;apos;s Workbench, 2026-10-10)&lt;/li&gt;&lt;li&gt;&lt;a href='https://dave.example.org/episodes/12'&gt;Episode 12: The small web&lt;/a&gt; (Dave Talks, 2026-10-07)&lt;/li&gt;&lt;li&gt;&lt;a href='https://carol.example.com/2026/10/05/readers/'&gt;A longer one about feed readers&lt;/a&gt; (carol.example.com, 2026-10-05)&lt;/li&gt;&lt;li&gt;&lt;a href='https://alice.example.com/posts/tomatoes/'&gt;Tomatoes, finally&lt;/a&gt; (Alice Writes Things, 2026-10-02)&lt;/li&gt;&lt;li&gt;&lt;a href='https://bob.example.org/2026/09/still-rss/'&gt;Why I still read RSS&lt;/a&gt; (Bob&amp;apos;s Workbench, 2026-09-28)&lt;/li&gt;&lt;li&gt;&lt;a href='https://dave.example.org/episodes/11'&gt;Episode 11: Caching for people who hate caching&lt;/a&gt; (Dave Talks, 2026-09-23)&lt;/li&gt;&lt;li&gt;&lt;a href='https://dave.example.org/episodes/10'&gt;Episode 10: Live from the garage&lt;/a&gt; (Dave Talks, 2026-09-09)&lt;/li&gt;&lt;/ol&gt;&lt;/body&gt;&lt;/html&gt;
[3 computed]</pre>
</div>

The digest, section 5's first question again, so all three of its answers come from the
cache:

<div class="ikigai-run" data-cmd='source urn:iki:fn:compose src=urn:file:queries/since.rq since=2026-10-15 | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl'>
<pre class="ikigai-run-expected"># Digest
&#32;
- 2026-10-27 [My home server caches everything now](https://bob.example.org/2026/10/home-server-cache/) from Bob's Workbench
- 2026-10-26 [Why my feed sends an ETag](https://alice.example.com/posts/ETag/) from Alice Writes Things
- 2026-10-24 [(untitled)](https://carol.example.com/2026/10/24/rss-cloud/) from carol.example.com
- 2026-10-15 [A static site in one afternoon](https://alice.example.com/posts/static-site/) from Alice Writes Things
[3 cached]</pre>
</div>

A feed of the search for `cach`. Section 5 asked the same question, so only the stylesheet
runs:

<div class="ikigai-run" data-cmd='source urn:iki:fn:compose src=urn:file:queries/search.rq q=cach | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/feed.xsl'>
<pre class="ikigai-run-expected">&lt;rss version='2.0'&gt;&lt;channel&gt;&lt;title&gt;River search&lt;/title&gt;&lt;link&gt;https://example.com/river&lt;/link&gt;&lt;description&gt;Items from the directory&amp;apos;s feeds that matched a saved search.&lt;/description&gt;&lt;item&gt;&lt;title&gt;My home server caches everything now&lt;/title&gt;&lt;link&gt;https://bob.example.org/2026/10/home-server-cache/&lt;/link&gt;&lt;guid&gt;https://bob.example.org/2026/10/home-server-cache/&lt;/guid&gt;&lt;source url='https://bob.example.org'&gt;Bob&amp;apos;s Workbench&lt;/source&gt;&lt;description&gt;A reverse proxy, a cache, and a lot of patience. [&amp;amp;#8230;]&lt;/description&gt;&lt;dc:date xmlns:dc='http://purl.org/dc/elements/1.1/'&gt;2026-10-27T21:14:09-05:00&lt;/dc:date&gt;&lt;/item&gt;&lt;item&gt;&lt;title&gt;Why my feed sends an ETag&lt;/title&gt;&lt;link&gt;https://alice.example.com/posts/ETag/&lt;/link&gt;&lt;guid&gt;https://alice.example.com/posts/ETag/&lt;/guid&gt;&lt;source url='https://alice.example.com/feed.xml'&gt;Alice Writes Things&lt;/source&gt;&lt;description&gt;&amp;lt;p&amp;gt;Readers that send &amp;lt;code&amp;gt;If-None-Match&amp;lt;/code&amp;gt; get a &amp;lt;strong&amp;gt;304&amp;lt;/strong&amp;gt; and no body. Caching is politeness.&amp;lt;/p&amp;gt;&lt;/description&gt;&lt;dc:date xmlns:dc='http://purl.org/dc/elements/1.1/'&gt;2026-10-26T09:30:00Z&lt;/dc:date&gt;&lt;/item&gt;&lt;item&gt;&lt;title&gt;Episode 11: Caching for people who hate caching&lt;/title&gt;&lt;link&gt;https://dave.example.org/episodes/11&lt;/link&gt;&lt;guid&gt;https://dave.example.org/episodes/11&lt;/guid&gt;&lt;source url='https://dave.example.org/'&gt;Dave Talks&lt;/source&gt;&lt;description&gt;Max-age, ETags, and why a 304 is the nicest answer a server can give.&lt;/description&gt;&lt;dc:date xmlns:dc='http://purl.org/dc/elements/1.1/'&gt;2026-09-23T10:00:00-07:00&lt;/dc:date&gt;&lt;/item&gt;&lt;/channel&gt;&lt;/rss&gt;
[2 cached · 1 computed]</pre>
</div>

And a directory of the feeds that search matched:

<div class="ikigai-run" data-cmd='source urn:iki:fn:compose src=urn:file:queries/search-feeds.rq q=cach | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/opml.xsl'>
<pre class="ikigai-run-expected">&lt;opml version='2.0'&gt;&lt;head&gt;&lt;title&gt;Feeds that matched&lt;/title&gt;&lt;/head&gt;&lt;body&gt;&lt;outline htmlUrl='https://alice.example.com/' text='Alice Writes Things' title='Alice Writes Things' type='rss' version='RSS' xmlUrl='https://alice.example.com/feed.xml'/&gt;&lt;outline htmlUrl='https://bob.example.org' text='Bob&amp;apos;s Workbench' title='Bob&amp;apos;s Workbench' type='rss' version='RSS' xmlUrl='https://bob.example.org'/&gt;&lt;outline htmlUrl='https://dave.example.org/' text='Dave Talks' title='Dave Talks' type='rss' version='RSS' xmlUrl='https://dave.example.org/'/&gt;&lt;/body&gt;&lt;/opml&gt;
[3 computed]</pre>
</div>

A cell in this page cannot write a file. At a shell, each line ends in a `sink`, which
writes the face into `ws/out/`:

```text
ikigai> source urn:file:queries/river.rq | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/page.xsl | sink urn:file:out/river.html
wrote 1581 bytes to out/river.html
[3 computed · 1 uncacheable]
ikigai> source urn:iki:fn:compose src=urn:file:queries/since.rq since=2026-10-15 | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/digest.xsl | sink urn:file:out/digest.md
wrote 455 bytes to out/digest.md
[3 computed · 1 uncacheable]
ikigai> source urn:iki:fn:compose src=urn:file:queries/search.rq q=cach | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/feed.xsl | sink urn:file:out/caching.xml
wrote 1542 bytes to out/caching.xml
[3 computed · 1 uncacheable]
ikigai> source urn:iki:fn:compose src=urn:file:queries/search-feeds.rq q=cach | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/opml.xsl | sink urn:file:out/caching.opml
wrote 559 bytes to out/caching.opml
[3 computed · 1 uncacheable]
```

Open `ws/out/river.html` in a browser, or read the others in an editor. The OPML is built
from `search-feeds.rq`, the same search counted per feed, because a directory lists each feed
once. At a shell, each written file is as fresh as the river: these four lines are the end
of a pipeline that starts at the directory, and every answer in it hangs from every file it
read.

## 7. Saved searches and the refresh script

A search worth keeping is a file. Fill `search.rq` once and keep what `compose` wrote: the
result is a complete query, no blank left, under `ws/searches/`:

```text
ikigai> source urn:iki:fn:compose src=urn:file:queries/search.rq q=cach | sink urn:file:searches/caching.rq
wrote 1077 bytes to searches/caching.rq
[1 computed · 1 uncacheable]
ikigai> source urn:iki:fn:compose src=urn:file:queries/search.rq q=rss | sink urn:file:searches/rss.rq
wrote 1076 bytes to searches/rss.rq
[1 computed · 1 uncacheable]
ikigai> source urn:file:searches
caching.rq
rss.rq
[uncacheable]
ikigai> source urn:file:searches | urn:text:wc
2
[1 computed · 1 uncacheable]
```

`ws/save-searches.iki` is those two lines in a file: a script that writes scripts, because each
query it writes is a program `urn:sparql:select` runs. A REPL reads a file of lines as
readily as it reads you, so at a shell, in `examples/rss`:

```bash
./iki --quiet < ws/save-searches.iki
```

Reading a directory, `source urn:file:searches`, answers its listing, one name per line,
and that listing is never cached: it is read fresh every time. So the folder is a set. Save
another search and it is in the set; delete a file and it is gone, with nothing else to
update.

What a map cannot do with that set yet is run it. The listing names the files as the folder
knows them, `caching.rq`, not as the kernel does, `urn:file:searches/caching.rq`, and a stage
mapped over it is handed the short name. In the same REPL as the saves above:

<!-- transcript: continues -->

```text
ikigai> source urn:file:searches .. urn:iki:fn:compose
error: invalid argument `src`: not an IRI: invalid IRI: No scheme found in an absolute IRI
       `src`: the IRI of the template to fill
```

So `ws/refresh.iki`, which rebuilds the river and all four faces, names its searches one by
one. It is section 4's build line and section 6's four lines, with the feed reading the saved
search, `urn:file:searches/caching.rq`, instead of composing it again. After a change to a
feed, the directory or a stylesheet:

```bash
./iki --quiet < ws/refresh.iki
```

Only what the change reached is computed again; the rest comes from the cache of that one
REPL. `examples/rss/check.sh` runs both scripts and fails if `river.nt` or anything in
`ws/out/` differs from the committed copy, which is how the example knows it still works.

## 8. What the river is waiting for

<!-- urn-gate: illustration urn:river:all — a name this chapter imagines for the river's
     build line, which named requests (ledger #2447) make possible; nothing binds it here. -->

Everything above works, and three things in it are clumsier than they should be, all for
the same reason: in this chapter, a request with arguments is not a name.

- The build line is the same long line in section 4 and in `refresh.iki`, and anything that
  wants the river has to repeat it, or read the file it wrote and lose the thread back to the
  feeds. A name for that composition, say `urn:river:all`, would be read like any resource,
  cached like one, and stale when any feed changed.
- A live feed is `urn:http:get` with a `url`, a `max_age` and a `User-Agent`, written out at
  every use. Named once, with the politeness in the name, no line could forget it.
- The saved searches are a set of files, but not yet a set of names a map can walk.

Named requests (ledger #2447) remove all three, and `ikigai-cli` 0.2.6, the release this
book is checked against, has them: the REPL's `bind` names a request with its arguments. This
chapter was written before that release and does not use them yet (nor does [Reshaping JSON
with jq](jq.md#6-at-a-shell-programs-and-templates-are-files) for its headline). The next
chapter introduces them and comes back to this river: [Naming a composition](naming.md)
names the live feed and the saved searches, and says why the build line is not a name yet.
