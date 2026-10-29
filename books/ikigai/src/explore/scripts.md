# Process scripts are files

[Composites, one layer at a time](composites.md) ended with copies: `quakes.jsonld`,
`notable.csv`, `notable.json`, each written by a REPL line, and each as old as the last time
you typed that line. Typing them again by hand is the part a computer should do. This chapter
keeps the lines in files and hands the files to `ikigai`: first as a script it replays line by
line, then as a **plan**, a pipeline stored as data that can be checked before it runs, asked
what authority it needs, and run again from the cache. Last, a script whose answer is another
script.

## What you need

The `~/quakes` directory from the last chapter, and a shell in it. Every step here is a shell
command, not a line at the REPL prompt; quit the REPL from the last chapter first, or use
another terminal.

```bash
cd ~/quakes
```

<!-- transcripts: workspace crates/quakes/quakes-2024-01-01.geojson crates/quakes/quakes-2024-01-08.geojson books/ikigai/ns/quakes.context.jsonld -->
<!-- transcripts: normalize workdir -->

The book's checks run every `$` line on this page against `ikigai-cli`
{{#include ../../ikigai-cli.version}}, in order, in a fresh directory holding the two
recorded weeks and the context, so the answers are the ones you get. `$PWD` in an answer
stands for your own directory. Each `ikigai` command below starts a new REPL, so nothing is
cached from one `$` line to the next.

## 1. A file of REPL lines

A script is a text file with one REPL line on each line. This one holds what the composites
are made from: the two fill templates, the question, the row and report templates, and the
stand-in for a note not written yet. If you worked through the last chapter, you have all of
them, and replaying it writes the same bytes again. If you are starting here, it is how you
catch up. Save it as `chapter5.iki`:

<!-- transcript: file chapter5.iki -->

```text
sink urn:file:graph.txt {"@context": $r{urn:json:query?src=urn:file:quakes.context.jsonld&q=.[]}, "@graph": [$r{urn:file:quakes-2024-01-01.geojson}]}
sink urn:file:graph-2024-01-08.txt {"@context": $r{urn:json:query?src=urn:file:quakes.context.jsonld&q=.[]}, "@graph": [$r{urn:file:quakes-2024-01-08.geojson}]}
sink urn:file:queries/notable.rq PREFIX qk: <https://ikigai-rs.dev/ns/quakes#> SELECT ?id ?mag ?place WHERE { ?quake qk:mag ?mag ; qk:place ?place FILTER(?mag >= 6) BIND(STRAFTER(STR(?quake), "eventpage/") AS ?id) } ORDER BY DESC(?mag)
sink urn:file:row.txt <li>M$h{{mag}} $h{{place}}: $h{urn:file:notes/{id}.txt || urn:file:notes/none.txt}</li>
sink urn:file:report.txt <ul>$r{urn:iki:fn:each?src=urn:file:notable.json&template=urn:file:row.txt}</ul>
sink urn:file:notes/none.txt no note yet
```

`ikigai --quiet` with a file on its standard input is the REPL with nobody at the keyboard:
no greeting, no prompt, each line read and answered in turn, and the answers printed exactly
as they would be at the prompt:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . < chapter5.iki
wrote 125 bytes to graph.txt
[uncacheable]
wrote 125 bytes to graph-2024-01-08.txt
[uncacheable]
wrote 202 bytes to queries/notable.rq
[uncacheable]
wrote 87 bytes to row.txt
[uncacheable]
wrote 80 bytes to report.txt
[uncacheable]
wrote 11 bytes to notes/none.txt
[uncacheable]
```

`--files .` makes this directory the file workspace, as `--files ~/quakes` did when you
started the REPL in [A week of earthquakes](index.md#what-you-need).

The refresh is the other half: the lines that read those sources and write the copies. Save
it as `refresh.iki`:

<!-- transcript: file refresh.iki -->

```text
source urn:iki:fn:compose src=urn:file:graph.txt | sink urn:file:quakes.jsonld
source urn:iki:fn:compose src=urn:file:graph-2024-01-08.txt | sink urn:file:quakes-2024-01-08.jsonld
source urn:file:queries/notable.rq | urn:sparql:select graph="urn:file:quakes.jsonld urn:file:quakes-2024-01-08.jsonld" as=text/csv | sink urn:file:notable.csv
source urn:json:query src=urn:file:notable.csv q=. | sink urn:file:notable.json
source urn:iki:fn:compose src=urn:file:report.txt | sink urn:file:report.html
```

Replay it, and ask the shell how it went:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . < refresh.iki; echo "exit $?"
wrote 126539 bytes to quakes.jsonld
[1 computed · 1 uncacheable]
wrote 75659 bytes to quakes-2024-01-08.jsonld
[1 computed · 1 uncacheable]
wrote 216 bytes to notable.csv
[2 computed · 1 uncacheable]
wrote 299 bytes to notable.json
[1 computed · 1 uncacheable]
wrote 243 bytes to report.html
[1 computed · 1 uncacheable]
exit 0
$ cat report.html
<ul><li>M7.5 2024 Noto Peninsula, Japan Earthquake: no note yet</li><li>M6.7 93 km SE of Sarangani, Philippines: no note yet</li><li>M6.4 45 km SSW of Jurm, Afghanistan: no note yet</li><li>M6.2 8 km SW of Anamizu, Japan: no note yet</li></ul>
```

Every copy is rewritten from its sources, in order, and the page fragment at the end is new.
(If you wrote notes in the last chapter, the report has them where this one says
`no note yet`, and the byte count is larger.) After an edit to the context, the question or a
template, this one command brings every copy up to date.

### When a line fails

Here is the same refresh with one name misspelled on its first line, as `typo.iki`:

<!-- transcript: file typo.iki -->

```text
source urn:iki:fn:compose src=urn:file:grahp.txt | sink urn:file:quakes.jsonld
source urn:file:queries/notable.rq | urn:sparql:select graph="urn:file:quakes.jsonld urn:file:quakes-2024-01-08.jsonld" as=text/csv | sink urn:file:notable.csv
```

<!-- transcript: run -->

```console
$ ikigai --quiet --files . < typo.iki; echo "exit $?"
error: not found: `urn:file:grahp.txt` does not exist
wrote 216 bytes to notable.csv
[2 computed · 1 uncacheable]
exit 1
```

The exit status is 1, so a shell script, a `make` rule or a CI job that runs the replay sees
that it failed. But look at what happened in between. `ikigai-cli`
{{#include ../../ikigai-cli.version}} did not stop at the error: the line REPL goes on after a
failed line (its `--help` says so), and only the exit status remembers. The second line ran,
and it read the `quakes.jsonld` the first line failed to write, which is the one from before.
Here that copy happened to be current. After a change to the context it would not be, and
`notable.csv` would be rewritten from a stale graph, with nothing on the screen to say so but
the error two lines up.

So read a replay's output, not only its exit status, and keep a script short enough to read.
A line that depends on the one before it is safe only if the one before it worked.

## 2. A plan is a file too

A script is text, and the only way to know what it does is to run it. A **plan** is the
other kind of stored pipeline: the REPL's `plan` command turns a pipeline into a graph of
steps, an `ik:Process` in Turtle, and runs nothing. Write one for the notable question:

<!-- transcript: run -->

```console
$ mkdir -p plans
$ ikigai --quiet --files . -c 'plan urn:file:queries/notable.rq | urn:sparql:select graph="urn:file:quakes.jsonld urn:file:quakes-2024-01-08.jsonld" as=text/csv' > plans/notable.ttl
$ cat plans/notable.ttl
@prefix ik:   <https://ikigai-rs.dev/ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6> a ik:Process ;
    rdfs:comment "urn:file:queries/notable.rq | urn:sparql:select graph=\"urn:file:quakes.jsonld urn:file:quakes-2024-01-08.jsonld\" as=text/csv" ;
    ik:step <urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:1> ,
            <urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2> ;
    ik:result <urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2> .

<urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:1> a ik:Step ;
    ik:verb "Source" ;
    ik:resolves <urn:file:queries/notable.rq> .

<urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2> a ik:Step ;
    ik:verb "Source" ;
    ik:resolves <urn:sparql:select> ;
    ik:pipeFrom <urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:1> ;
    ik:argument <urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2:arg:as> ,
                <urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2:arg:graph> .

<urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2:arg:as> a ik:Argument ;
    ik:inputName "as" ;
    ik:value "text/csv" .

<urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2:arg:graph> a ik:Argument ;
    ik:inputName "graph" ;
    ik:value "urn:file:quakes.jsonld urn:file:quakes-2024-01-08.jsonld" .
```

<!-- urn-gate: unbound urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6 — a node the plan names itself by, in the Turtle `plan` printed; nothing binds it. -->
<!-- urn-gate: unbound urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:1 — a node the plan names itself by, in the Turtle `plan` printed; nothing binds it. -->
<!-- urn-gate: unbound urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2 — a node the plan names itself by, in the Turtle `plan` printed; nothing binds it. -->
<!-- urn-gate: unbound urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2:arg:as — a node the plan names itself by, in the Turtle `plan` printed; nothing binds it. -->
<!-- urn-gate: unbound urn:plan:b3:9e31f4489224566ca02376654b32db3de25043f7bdc0d1642933cd18022ddee6:step:2:arg:graph — a node the plan names itself by, in the Turtle `plan` printed; nothing binds it. -->

`-c` runs one REPL line and exits, and `plan` prints the graph, so the shell's `>` puts it in
a file. Two steps: step 1 reads the question, step 2 is `urn:sparql:select` with its two
arguments and its input piped from step 1, and the plan's result is step 2's. The plan's own
name is a BLAKE3 hash (`b3:`), and the same line makes the same name every time.

`execute` reads a plan by its name and runs it. Run it twice in one REPL:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . -c 'execute urn:file:plans/notable.ttl' -c 'execute urn:file:plans/notable.ttl'
id,mag,place
us6000m0xl,7.5,"2024 Noto Peninsula, Japan Earthquake"
us6000m2jp,6.7,"93 km SE of Sarangani, Philippines"
us6000m31m,6.4,"45 km SSW of Jurm, Afghanistan"
us6000m0xm,6.2,"8 km SW of Anamizu, Japan"
[3 computed]
id,mag,place
us6000m0xl,7.5,"2024 Noto Peninsula, Japan Earthquake"
us6000m2jp,6.7,"93 km SE of Sarangani, Philippines"
us6000m31m,6.4,"45 km SSW of Jurm, Afghanistan"
us6000m0xm,6.2,"8 km SW of Anamizu, Japan"
[2 cached · 1 computed]
— batch: 2 commands · 2 cached · 4 computed
```

The first run computed three things. The second computed one, and the other two,
`[2 cached]`, came from the cache, because a plan's steps are ordinary reads through the
kernel and are cached like any other. The last line is the `-c` batch's own tally of both.

## 3. The authority a plan needs

A script cannot tell you what it will touch until it runs. A plan can, because each step names
the resource it resolves, and every resource publishes a contract that says what authority it
requires. Three resources answer questions about a plan, each given the plan as `in`:
`urn:plan:validate` checks that it is a plan this kernel can run, `urn:plan:requires` derives
the capability it needs from its steps' contracts, and `urn:plan:eval` runs it. None of them
invokes a step to answer, except `urn:plan:eval`.

Make a plan that reaches the network: the live feed from
[Somebody else's resource](http.md), counted. Writing the plan sends no request.

<!-- transcript: run -->

```console
$ ikigai --quiet --files . -c 'plan urn:http:get url=https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson | urn:text:wc' > plans/live.ttl
```

<!-- urn-gate: unbound urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e — a node the plan names itself by, in the Turtle `plan` printed; nothing binds it. -->
<!-- urn-gate: unbound urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e:step:1 — a node the plan names itself by, in the Turtle `plan` printed; nothing binds it. -->
<!-- urn-gate: unbound urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e:step:2 — a node the plan names itself by, in the Turtle `plan` printed; nothing binds it. -->

Each resource takes the plan as its one required input, so a plan on standard input fills it.
Check it, and ask what it needs:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . -c 'source urn:plan:validate' < plans/live.ttl
<urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e> conforms: it satisfies ikigai_vocab::SHAPES and the executor's checks
[computed]
$ ikigai --quiet --files . -c 'source urn:plan:requires' < plans/live.ttl
<urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e> requires urn:cap:net:*
this capability holds all of it
  urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e:step:1  Source <urn:http:get>  requires urn:cap:net:*
  urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e:step:2  Source <urn:text:wc>  requires nothing
[computed]
```

The plan needs `urn:cap:net:*`, some authority to reach the network, and only for its first
step; counting needs nothing. The session has all authority, so it holds all of it. Narrow the
session first, the way [Somebody else's resource](http.md#5-only-the-hosts-you-allow) did,
and ask again. `cap read-only` leaves the session able to read this directory's files and
nothing else:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . -c 'cap read-only' -c 'source urn:plan:requires' < plans/live.ttl
narrowed — capability: urn:cap:fs:read:$PWD
<urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e> requires urn:cap:net:*
this capability lacks urn:cap:net:* — a run under it is refused at the steps below that lack it
  urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e:step:1  Source <urn:http:get>  requires urn:cap:net:*  LACKS urn:cap:net:*
  urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e:step:2  Source <urn:text:wc>  requires nothing
[computed]
— batch: 2 commands · computed
```

That is the question to ask before a plan runs, and the answer names the step that would be
refused and what it lacks. Nothing was fetched to find out.

The answer has a limit, and the narrowing from
[Somebody else's resource](http.md#5-only-the-hosts-you-allow) shows it. Narrow the session to a different
host, `example.com`, and ask, then run the plan:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . -c 'cap net-example.com' -c 'source urn:plan:requires' < plans/live.ttl
narrowed — capability: urn:cap:net:example.com
<urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e> requires urn:cap:net:*
this capability holds all of it
  urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e:step:1  Source <urn:http:get>  requires urn:cap:net:*
  urn:plan:b3:7b4651c595e64668b9fbf7ab70b4c9248c40fe20ab31c6ab649f8c69e49d761e:step:2  Source <urn:text:wc>  requires nothing
[computed]
— batch: 2 commands · computed
$ ikigai --quiet --files . -c 'cap net-example.com' -c 'source urn:plan:eval' < plans/live.ttl; echo "exit $?"
narrowed — capability: urn:cap:net:example.com
error: denied: capability does not grant `GET` to `earthquake.usgs.gov/earthquakes/feed/v1.0/summary/4.5_week.geojson`
exit 1
```

`urn:plan:requires` said yes, and the run was refused, naming the host. Both are right about
different things. `urn:http:get`'s contract requires `urn:cap:net:*`, which any network
grant satisfies, `urn:cap:net:example.com` among them; which host a URL may reach is decided
when the step runs, by the URL it is given, and that is where the run stopped, before any
request was sent. So `urn:plan:requires` tells you what kind of authority a plan needs and
which steps need it; the host is settled by running it under a capability that names the
host, as that chapter's session named `earthquake.usgs.gov`.

A plan cannot do more than its caller could do one request at a time: every step runs under
the capability of whoever runs the plan, and meets its own target's requirement. A plan you
were sent is a file you can validate and question before it runs, and run under a capability
you narrowed for it.

## 4. A script that writes a script

A script is a file, and a file is a resource, so a composition can answer one. This jq
program turns `notable.json` into two REPL lines per earthquake: one that writes a card
template for it, with its note spliced in, and one that fills the template into a page
fragment. Save it as `queries/cards.jq`:

<!-- transcript: file queries/cards.jq -->

```text
[.[] |
  "sink urn:file:cards/\(.id).txt <h2>M\(.mag) \(.place)</h2><p>$h{urn:file:notes/\(.id).txt || urn:file:notes/none.txt}</p>",
  "source urn:iki:fn:compose src=urn:file:cards/\(.id).txt | sink urn:file:cards/\(.id).html"
] | join("\n")
```

And the script that runs it, `make-cards.iki`, one line: the program piped into the query as
its `q`, as in [Reshaping JSON with jq](jq.md#6-at-a-shell-programs-and-templates-are-files),
with `as=text/plain` so the joined lines come out as text and not as one quoted JSON string:

<!-- transcript: file make-cards.iki -->

```text
source urn:file:queries/cards.jq | urn:json:query src=urn:file:notable.json as=text/plain | sink urn:file:cards.iki
```

Replay it, read what it wrote, and replay that:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . < make-cards.iki
wrote 985 bytes to cards.iki
[2 computed · 1 uncacheable]
$ cat cards.iki
sink urn:file:cards/us6000m0xl.txt <h2>M7.5 2024 Noto Peninsula, Japan Earthquake</h2><p>$h{urn:file:notes/us6000m0xl.txt || urn:file:notes/none.txt}</p>
source urn:iki:fn:compose src=urn:file:cards/us6000m0xl.txt | sink urn:file:cards/us6000m0xl.html
sink urn:file:cards/us6000m2jp.txt <h2>M6.7 93 km SE of Sarangani, Philippines</h2><p>$h{urn:file:notes/us6000m2jp.txt || urn:file:notes/none.txt}</p>
source urn:iki:fn:compose src=urn:file:cards/us6000m2jp.txt | sink urn:file:cards/us6000m2jp.html
sink urn:file:cards/us6000m31m.txt <h2>M6.4 45 km SSW of Jurm, Afghanistan</h2><p>$h{urn:file:notes/us6000m31m.txt || urn:file:notes/none.txt}</p>
source urn:iki:fn:compose src=urn:file:cards/us6000m31m.txt | sink urn:file:cards/us6000m31m.html
sink urn:file:cards/us6000m0xm.txt <h2>M6.2 8 km SW of Anamizu, Japan</h2><p>$h{urn:file:notes/us6000m0xm.txt || urn:file:notes/none.txt}</p>
source urn:iki:fn:compose src=urn:file:cards/us6000m0xm.txt | sink urn:file:cards/us6000m0xm.html
$ ikigai --quiet --files . < cards.iki
wrote 118 bytes to cards/us6000m0xl.txt
[uncacheable]
wrote 69 bytes to cards/us6000m0xl.html
[1 computed · 1 uncacheable]
wrote 115 bytes to cards/us6000m2jp.txt
[uncacheable]
wrote 66 bytes to cards/us6000m2jp.html
[1 computed · 1 uncacheable]
wrote 111 bytes to cards/us6000m31m.txt
[uncacheable]
wrote 62 bytes to cards/us6000m31m.html
[1 computed · 1 uncacheable]
wrote 106 bytes to cards/us6000m0xm.txt
[uncacheable]
wrote 57 bytes to cards/us6000m0xm.html
[1 computed · 1 uncacheable]
$ cat cards/us6000m0xl.html
<h2>M7.5 2024 Noto Peninsula, Japan Earthquake</h2><p>no note yet</p>
```

One card per notable earthquake (with your note in it, if you wrote one), and the script that made them was written by a query over
the data. When next week's copy has a fifth notable earthquake, the same two replays make a
fifth card; nobody edits a script.

Read a generated script before you replay it. It holds whatever the data put in it: a `place`
with a line break in it would be a second REPL line, and that line would run with your
authority. USGS's places have none, and `cat` is how you know. A replay of a generated script
is also a good place for the narrowing in section 3: `cap write` as its first line would keep
it to this directory's files.

## 5. Plans with parameters

The notable plan has its threshold, `6`, written into the question it reads. A plan that took
the threshold as a parameter, bound to a name and read with a value, would be one plan for
every threshold. This page was written before the `ikigai-cli` this book is checked against
could do that: binding a plan to a name comes with engine 0.2.6 (ledger #2509), which the
book's pinned cli, {{#include ../../ikigai-cli.version}}, links, and its REPL's `bind` names a
request with its arguments (named requests, ledger #2447). This part has not taken them up
yet, so here a different threshold is still a different question file and a different plan;
[A river of feeds](rss.md#8-what-the-river-is-waiting-for) waits on the same chapter, and
this book does not work around it.

## 6. What a script does not have yet

The `.iki` format is the REPL's lines and nothing more. It has no comments: a line starting `#` is
an unknown command, an error, and the replay exits 1; it declares no parameters, so a script for a
different week is a copy of the script; it has no shebang line, so it is replayed with
`ikigai --quiet < file` and never run as a program; and a script stored as a resource cannot
yet be run by name from inside the REPL, only through standard input. Those four are the
subject of one ledger item, the `.iki` format's missing pieces (ledger #2485). It is a list
of what is missing, not a promise of when.

Next, the same parts over many publishers at once, each with its own rules for how long a copy
stays good, refreshed by a script like the one above: [A river of feeds](rss.md).
