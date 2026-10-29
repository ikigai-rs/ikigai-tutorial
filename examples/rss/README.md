# A river of feeds: RSS with ikigai, no Rust

A worked example for the "Exploring without Rust" part of *The ikigai Book*: a directory of RSS feeds becomes a list
of names, each feed becomes RDF, the feeds become one river, questions about the river are files, and one answer is
reformatted four ways. Every step is a REPL line. The only code is XSLT, SPARQL and three replay files.

It runs **offline** against a synthetic fixture: four invented feeds under `example.com` and `example.org`
(RFC 2606), shaped like the real feeds listed by <https://iheartrss.com/subscriptions.opml> (dates, `guid` forms,
HTML in `description`, `content:encoded`, missing fields, a podcast with no guid). Nothing here comes from a real
member's feed. The live run at the end is the reader's choice, against a few feeds, and is polite.

## What you need

`ikigai` 0.2.5 or later on your PATH (`cargo install ikigai-cli --locked`), or name it with `IKIGAI=`. Revalidation
(the 304s in "Somebody else's rules") needs 0.2.6 or later. `./iki` runs it hermetically over this folder: its own
config home (`.home/`), `ws/` as the file root, and none of your machine's mounts.

```bash
./check.sh        # rebuild the river and the four views offline; fails if anything differs from what is committed
./iki             # a REPL over ws/
```

## The tour (type these into `./iki`)

**1. A directory is a list of names.** The OPML lists feeds; one stylesheet turns it into a newline list, one name per
line. `feeds-local.xsl` maps each feed to the fixture's copy, `urn:file:feeds/{host}.xml`; `feeds.xsl` keeps the URLs.

```text
source urn:xslt:transform src=urn:file:directory.opml stylesheet=urn:file:feeds-local.xsl
```
```text
urn:file:feeds/alice.example.com.xml
urn:file:feeds/bob.example.org.xml
urn:file:feeds/carol.example.com.xml
urn:file:feeds/dave.example.org.xml
```

**2. Each feed becomes RDF, and the feeds become a river.** `..` maps the stage in parentheses over every name.
`items.xsl` writes RSS 2.0 as RDF/XML in RSS 1.0's own vocabulary (RSS 1.0 *was* RDF) plus Dublin Core, one IRI per
item (its `guid` when that is a permalink, else its `link`); `urn:rdf:transrept` turns that into N-Triples.

```text
source urn:xslt:transform src=urn:file:directory.opml stylesheet=urn:file:feeds-local.xsl .. ( urn:xslt:transform stylesheet=urn:file:items.xsl | urn:rdf:transrept as=application/n-triples ) | sink urn:file:river.nt
```
```text
wrote 14886 bytes to river.nt
[9 computed · 1 uncacheable]
```

Run the same line again: `[9 cached · 1 uncacheable]`. Only the write runs again. Ask what hangs from one feed:

```text
dependents urn:file:feeds/dave.example.org.xml
```
```text
  3 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:file:feeds/dave.example.org.xml  text/xml                     1.4 KB  1 thread    root
  urn:rdf:transrept                    application/n-triples        2.8 KB  6 threads   root
  urn:xslt:transform                   application/xml              2.3 KB  5 threads   root
```

Edit that file and rebuild: Dave's lift recomputes, the other three stay cached.

**3. Questions are files.** The queries in `ws/queries/` are SPARQL kept as resources; `compose` fills a parameter.

```text
source urn:file:queries/per-feed.rq | urn:sparql:select graph=urn:file:river.nt as=text/csv
source urn:iki:fn:compose src=urn:file:queries/search.rq q=cach | urn:sparql:select graph=urn:file:river.nt as=text/csv
source urn:iki:fn:compose src=urn:file:queries/since.rq since=2026-10-15 | urn:sparql:select graph=urn:file:river.nt as=text/csv
```

`compose` splices the term as text, so a term with a `"` in it would break the query. `urn:sparql:select` also takes
`bindings=` (a JSON object), which binds a value into the *parsed* query so it is always a term, never syntax; the
query must project the variable it binds.

**4. One answer, four faces.** The same SPARQL answer, as results XML, through four stylesheets in `ws/views/`: an HTML
page, a Markdown digest, an RSS feed of the matches (a search you can subscribe to), and an OPML of the feeds that
matched. `ws/refresh.iki` writes all four into `ws/out/`:

```text
source urn:file:queries/river.rq | urn:sparql:select graph=urn:file:river.nt as=application/sparql-results+xml | urn:xslt:transform stylesheet=urn:file:views/page.xsl | sink urn:file:out/river.html
```

**5. Saved searches and the refresh script.** `ws/save-searches.iki` is a script that writes scripts: it composes two
queries and sinks them into `ws/searches/`. `ws/refresh.iki` rebuilds the river and every view. Both are plain files
of REPL lines:

```bash
./iki --quiet < ws/save-searches.iki
./iki --quiet < ws/refresh.iki
```

`source urn:file:searches` lists that folder, one name per line, so the saved searches are a set a map can walk.

## Somebody else's rules: caching

Most personal feeds say little about caching. In one polite pass over the directory's 115 feeds (2026-10-28,
counts only): 39 send no `Cache-Control`, 28 say `max-age=0`, and only 30 of 112 may be cached for more than ten
seconds. But 95 send an `ETag` and 90 answer a conditional request with **304 Not Modified**. A client that
revalidates downloads almost nothing on a feed that has not changed.

`tools/origin.py` is a stand-in origin that serves the fixture feeds with whatever cache controls a path asks for
(`/cc/max60/…`, `/cc/etag/…`, `/cc/lm/…`, `/cc/nocache/…`, `/cc/nostore/…`, `/cc/none/…`) and logs every request's
validators, so this can be shown without touching anyone's server:

```bash
python3 -I tools/origin.py ws/feeds /tmp/origin.log
```

It prints its port. In one `./iki` session (the cache lives in the process):

```text
source urn:http:get url=http://127.0.0.1:PORT/cc/max60/alice.example.com.xml | urn:text:wc
source urn:http:get url=http://127.0.0.1:PORT/cc/max60/alice.example.com.xml | urn:text:wc
source urn:http:get url=http://127.0.0.1:PORT/cc/etag/bob.example.org.xml | urn:text:wc
source urn:http:get url=http://127.0.0.1:PORT/cc/etag/bob.example.org.xml | urn:text:wc
```

- `max-age=60`: the second read is `[2 cached]`, and the origin saw one request.
- An ETag and nothing else: on ikigai-cli 0.2.5 every read downloads the whole feed again (`[2 uncacheable]`, two
  200s, no `If-None-Match`). From 0.2.6 (ikigai-http 0.5.0, ikigai-core 0.2.7) the answer is kept for
  revalidation: the second read sends `If-None-Match`, the origin answers **304**, and the stored body is served.
  The origin's log shows it:

```text
{"t": 1793265062.737, "path": "/cc/etag/bob.example.org.xml", "if-none-match": null, "if-modified-since": null, "user-agent": "ikigai/0.2.5", "accept-encoding": "gzip", "status": 200}
{"t": 1793265062.738, "path": "/cc/etag/bob.example.org.xml", "if-none-match": "\"985b241130e43cac\"", "if-modified-since": null, "user-agent": "ikigai/0.2.5", "accept-encoding": "gzip", "status": 304}
```

(Recorded on a cli built from main before 0.2.6 was published, so its User-Agent still says 0.2.5.)

## The live run (your choice, and polite)

`ws/refresh-live.iki` reads the real directory (cached a day), takes its first three feeds (`urn:text:head n=3`;
pick others if you like), and builds a live river. It names the tutorial in its User-Agent and asks a long
`max_age=`, so a demo does not hammer anyone. These are people's personal blogs: run it rarely, inside one session.

```bash
./iki --quiet < ws/refresh-live.iki
```

## Files

| path | what |
|---|---|
| `ws/directory.opml` | the fixture directory, shaped like iheartrss.com's `subscriptions.opml` |
| `ws/feeds/*.xml` | four synthetic RSS 2.0 feeds covering the census's shapes |
| `ws/feeds.xsl`, `ws/feeds-local.xsl` | OPML → a list of feed URLs, or of the fixture's files |
| `ws/items.xsl` | RSS 2.0 → RDF/XML (RSS 1.0 + Dublin Core); its comments name the XSLT engine workarounds it carries |
| `ws/queries/*.rq` | river, since, in (category), from (feed), search, search-feeds, per-feed, lines |
| `ws/views/*.xsl` | one SPARQL answer as an HTML page, a Markdown digest, an RSS feed, an OPML |
| `ws/*.iki` | `save-searches`, `refresh`, `refresh-live`: replay files of REPL lines |
| `ws/river.nt`, `ws/out/*` | what `refresh.iki` writes, committed so `check.sh` can tell a change |
| `tools/origin.py` | the stand-in origin for the caching steps |
