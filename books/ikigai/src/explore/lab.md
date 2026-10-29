# A lab with names in it

A laboratory notebook works because every sample has a label: you can point at it, ask
about it, and know later which answer came from which sample. The REPL is that kind of
lab. Every sample, and every question you ask about one, has a name, and the kernel keeps
track of which answers came from what.

This chapter's sample is the recorded week of earthquakes. Every cell below runs in this
page's kernel, so there is nothing to install: press **Run**. The commands are the same
ones the `ikigai` REPL takes, and the last section runs a few of them at a shell.

## 1. The sample has a name

`urn:file:quakes-2024-01-01.geojson` is the recorded week. Before reading it, ask what it
is. `describe` asks a resource to describe itself, and `text/plain` asks for the
description in words:

<div class="ikigai-run" data-cmd='describe urn:file:quakes-2024-01-01.geojson text/plain'>
<pre class="ikigai-run-expected">quakes-window — A recorded week of earthquakes
USGS earthquakes of M4.5 and up from 2024-01-01 to 2024-01-08, 178 events, as GeoJSON, recorded once and never changed: the window &quot;Exploring without Rust&quot; runs on.
verbs: Source, Meta
[computed]</pre>
</div>

It answers `Source` (it can be read) and `Meta` (it can describe itself), and nothing else:
in this page the recorded week cannot be changed. The last line is the verdict: the kernel
computed this description rather than finding it in its cache.

Reading the whole thing would print 125,765 bytes of GeoJSON, which is a lot of text to scroll
past. So the next section asks a question instead.

## 2. A question is a resource too

`urn:json:query` is a resource that answers a question about another resource. It takes
the name of the thing to read (`src`) and a question written in jq (`q`). This chapter uses
only small questions; [Reshaping JSON with jq](jq.md) is about the language. How many
earthquakes does the week hold, and what is the first one in the file?

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features|length"
source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[0].properties.title" as=text/plain'>
<pre class="ikigai-run-expected">178
[computed]
M 5.1 - 77 km ESE of Fangale’ounga, Tonga
[computed]</pre>
</div>

USGS lists the newest first, so the first is the last earthquake of the week, in Tonga.
`as=text/plain` asks for the answer as plain text rather than as JSON, so the title comes
back without quotation marks.

Nothing about the question is special to this file. `src` is any name the kernel can read,
and the query reads it through the kernel, with the same permissions you have.

## 3. A pipe

`|` hands one answer to the next resource as its input. Here the place of the first
earthquake goes to `urn:iki:fn:toUpper`, which capitalizes text:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[0].properties.place" as=text/plain | urn:iki:fn:toUpper'>
<pre class="ikigai-run-expected">77 KM ESE OF FANGALE’OUNGA, TONGA
[2 computed]</pre>
</div>

Two resolutions: the question, then `toUpper` over its answer. `toUpper` knows nothing
about earthquakes or JSON. It takes text, and the pipe gives it text.

## 4. A map

Many resources in ikigai treat their input as a list, one item per line. `..` runs the
next resource once for each line and joins the answers. This question asks for the places
of the first three earthquakes, one per line (`join("\n")` is jq for "one per line"; inside
the quoted program, `\"` is a quotation mark), and maps `toUpper` over them:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[0:3] | map(.properties.place) | join(\"\n\")" as=text/plain .. urn:iki:fn:toUpper'>
<pre class="ikigai-run-expected">77 KM ESE OF FANGALE’OUNGA, TONGA
BALLENY ISLANDS REGION
BALLENY ISLANDS REGION
[2 cached · 2 computed]</pre>
</div>

Look at the tally: four resolutions, the question and three calls to `toUpper`, and two of
them were cached. Two earthquakes in a row were in the Balleny Islands region, so the map
asked `toUpper` the same question twice, and the second time the kernel already had the
answer. And the Tonga place is the one the pipe above capitalized, so that answer was
already there too. That is the cache at work with no instruction from you: an answer is
kept under the name and the arguments that produced it, and the same request finds it.

## 5. A fork

`( a ; b )` sends the same input to each branch and joins the answers in order:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[0:3] | map(.properties.place) | join(\"\n\")" as=text/plain | ( urn:iki:fn:toUpper ; urn:iki:fn:reverseList )'>
<pre class="ikigai-run-expected">77 KM ESE OF FANGALE’OUNGA, TONGA
BALLENY ISLANDS REGION
BALLENY ISLANDS REGION
Balleny Islands region
Balleny Islands region
77 km ESE of Fangale’ounga, Tonga
[1 cached · 2 computed]</pre>
</div>

`reverseList` reverses the order of the lines. The question itself was cached by the cell
above, which is why this tally counts it as cached.

## 6. What a question read

<!-- urn-gate: illustration urn:iki:space:json — the name ikigai-json's space claims, which a
trace prints as the space that answered (`answered-by=`); a space's name, not a resource. -->

`trace` resolves a request and shows what it did: who asked, with what authority, and
every resource it read on the way, each with its own verdict. Ask a question this page has
not asked yet, the earliest earthquake of the week (the last in the file):

<div class="ikigai-run" data-cmd='trace urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features[-1].properties.title"'>
<pre class="ikigai-run-expected">trace  urn:json:query
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:json:query   json-query · computed · ThreadId(1) · — · answered-by=urn:iki:space:json   → 54b  &quot;M 4.7 - 272 km N of Saipan, Northern Ma…
└─ urn:file:quakes-2024-01-01.geojson   quakes-window · cached · ThreadId(1) · —</pre>
</div>

The query was computed, and the line under it is the recorded week, which the query read
and found in the cache. The `—` is where a duration would go; this page's kernel has no
clock ([the kernel in the browser](../browser.md) says why). No one told the kernel that
this question depends on that file. It saw the query read it, and it remembers. Trace a
question that is already cached and the tree stops at the top: nothing was read.

## 7. What hangs from the sample

That memory has a name too. Every cached answer hangs from one or more **threads**: named
dependencies the kernel can cut. Reading the recorded week hangs the answer from the thread
named after the file, and anything computed from that answer hangs from it as well.
`urn:kernel:dependents` lists what hangs from a thread:

<div class="ikigai-run" data-cmd='source urn:kernel:dependents thread=urn:file:quakes-2024-01-01.geojson'>
<pre class="ikigai-run-expected">dependents of urn:file:quakes-2024-01-01.geojson  (cut 0 times)
  10 live cached entries hang from it — a cut makes them stale, and the next read of each recomputes it
  urn:file:quakes-2024-01-01.geojson  application/geo+json       122.8 KB  1 thread    root
  urn:iki:fn:reverseList              text/plain                     81 B  3 threads   root
  urn:iki:fn:toUpper                  text/plain                     22 B  3 threads   root
  urn:iki:fn:toUpper                  text/plain                     35 B  3 threads   root
  urn:iki:fn:toUpper                  text/plain                     81 B  3 threads   root
  urn:json:query                      application/json                3 B  2 threads   root
  urn:json:query                      application/json               54 B  2 threads   root
  urn:json:query                      text/plain                     35 B  2 threads   root
  urn:json:query                      text/plain                     43 B  2 threads   root
  urn:json:query                      text/plain                     81 B  2 threads   root
[uncacheable]</pre>
</div>

Every question this chapter asked is there, and so is every `toUpper` answer computed from
one: they all depend on the file, whether they read it directly or through another
answer.

Now cut the thread. A cut says "this changed": every answer that hangs from it becomes
stale, and the next request for it is computed again. `cache` asks whether an answer is in
the cache without computing anything:

<div class="ikigai-run" data-cmd='cache urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features|length"
sink urn:kernel:cut urn:file:quakes-2024-01-01.geojson
cache urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features|length"
source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features|length"'>
<pre class="ikigai-run-expected">cached
cut urn:file:quakes-2024-01-01.geojson
[uncacheable]
not cached
178
[computed]</pre>
</div>

The answer is the same, 178, because the recorded week did not really change. At a shell
it can: write to the file and the write cuts the thread for you, or edit it with another
program, and the CLI notices and cuts it. [Golden threads in
practice](../getting-started/golden-threads.md) is the chapter on threads.

## 8. The same lab at a shell

Everything above works in the `ikigai` REPL too, against the copy you downloaded in
[A week of earthquakes](index.md#what-you-need). The REPL also has resources this page
does not, such as the text tools under `urn:text:`, which work like their Unix namesakes.
USGS's file puts each earthquake on a line of its own, so `grep` and `wc` can already
count them. In the REPL you started with `--files ~/quakes`, type the lines after each
`ikigai>` (the prompt is the REPL's, not yours to type). Each answer is printed with its
tally:

<!-- transcripts: workspace crates/quakes/quakes-2024-01-01.geojson -->

```text
ikigai> source urn:file:quakes-2024-01-01.geojson | urn:text:wc
178
[1 computed · 1 uncacheable]
ikigai> source urn:file:quakes-2024-01-01.geojson | urn:text:grep pattern=Japan | urn:text:wc
62
[1 cached · 2 uncacheable]
ikigai> source urn:file:quakes-2024-01-01.geojson | urn:text:grep pattern="Noto Peninsula" | urn:text:wc
1
[1 cached · 2 uncacheable]
```

The book's checks replay this block against `ikigai-cli` {{#include ../../ikigai-cli.version}},
over the same file the download fetches, so these answers are the ones you get.

178 lines, one per earthquake. 62 of the lines mention Japan, most of them aftershocks of
the New Year's Day earthquake, and one line names the Noto Peninsula itself: the main
shock. Counting lines is a crude way to ask about JSON (a place name in another field
would count too), and [Reshaping JSON with jq](jq.md) asks the same questions properly.

The `1 computed` in the first tally is the file, read once and kept: the later lines found
it in the cache, which is their `1 cached`. None of the text tools' answers were kept,
which is every `uncacheable`. The kernel says why, if you ask:

```text
source urn:kernel:uncached
```

It printed this, on `ikigai-cli` 0.2.5:

```text
uncached (why a computed read was not cached; the last 64, most recent first)
  urn:text:wc        ×1  one-shot content b3:236f4cfad7f59de75883fed22dcbc1c0c8a7982e0fc989f1d526f0271e335199; upstream  [root]
  urn:text:grep      ×1  one-shot content b3:4b04300f5127c249fc59806380cb1a6ab4e2a4d6c9f2f5997dcc33e09b0be210  [root]
  urn:text:wc        ×1  one-shot content b3:aedbd75b64018f949e9dabc1a83829e1bc9559c7c1877715b2a1eb17e893a1e9; upstream  [root]
  urn:text:grep      ×1  one-shot content b3:4939de7f64f2a335e38e45e1a998737dc6e20fa39b99171c1c8321e1397d7696  [root]
  urn:text:wc        ×1  one-shot content b3:f06db695a3cfc70b1fd99f5e02b5b3528bc5d8589feb3d7403b9fd8a5662232d  [root]

[uncacheable]
```

The pipe hands each text tool what came before it as a stream, read as it arrives, under
a content id made up for that one hand-over (the `b3:` names, which are new every time, so
this block is not replayed with the one above). An answer built on input that can be read
only once is never stored (`one-shot content`), and a `wc` after a `grep` was handed a
stream that was itself one-shot (`upstream`). The same tools over a short value typed on
the line are cached as usual. This is a gap in the text tools, recorded as ledger #2530,
not something to work around: for now, the questions worth caching go through
`urn:json:query`, as the page's do. (On `ikigai-cli` 0.2.4 the counts were the same, but a
text tool reading the file itself was tallied as computed, and the reason given for it was
the cache's `policy`.)

Next, the same questions about data that is not yours and not still:
[Somebody else's resource](http.md).
