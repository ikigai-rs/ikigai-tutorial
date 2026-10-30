# The REPL grammar

The same command syntax works in the `ikigai` REPL, in a shell with `ikigai -c`, in this
book's runnable cells, and in the [browser demo](../getting-started/running-it.md#the-browser-demo).
Each uses its own host, which determines the available resource names. A command with a
**Run** button works in this page. The CLI examples below run inside `ikigai --plain`
unless they start with `ikigai`, in which case run them in your shell. If your REPL from
[Twenty minutes with the CLI](cli.md) is still open, you can keep using it.

The syntax blocks use `<…>` for a value you supply, `[…]` for an optional part, and `…`
for repetition. Do not type those markers. In the pipe, map, and fork syntax blocks,
`a`, `b`, and `c` stand for resource names.

## A resolution

```text
source <iri> [input]
source <iri> key=value …
```

`source` is the verb, meaning "read." An IRI (Internationalized Resource Identifier)
names the resource to read. Everything after it supplies arguments to that request.
These are command parts, not the subject,
predicate, and object of a Resource Description Framework (RDF) statement.

`key=value` names an argument. An unnamed, or *positional*, value fills a declared
argument that you have not supplied by name. The engine reads the endpoint's description
to find its argument names. For example, `toUpper` declares one argument called `in`:

<div class="ikigai-run" data-cmd='source urn:iki:fn:toUpper in="resource oriented computing"
source urn:iki:fn:toUpper resource oriented computing'>
<pre class="ikigai-run-expected">RESOURCE ORIENTED COMPUTING
[computed]
RESOURCE ORIENTED COMPUTING
[cached]</pre>
</div>

Two spellings of the same request — and the second answers `[cached]`, because the
kernel keys the cache on the request, not the text you typed. Edit the input and run
again: a different request, computed afresh.

For an endpoint with two arguments, name one and supply the other positionally. These
three commands all ask the bundled `greet` endpoint for `Hello, Joe` (type what follows
`ikigai>`, the REPL's prompt, here and below):

```text
ikigai> source urn:demo:greet greeting=Hello name=Joe
Hello, Joe
[computed]
ikigai> source urn:demo:greet greeting=Hello Joe
Hello, Joe
[cached]
ikigai> source urn:demo:greet Hello name=Joe
Hello, Joe
[cached]
```

They are one request, so the second and third are answered from the cache.

Use `describe urn:demo:greet text/plain` to see the declared names, `greeting` and `name`.
The engine checks every name against that description, so a typo is an error, not a wrong
answer. This line is refused:

```text
ikigai> source urn:iki:fn:toUpper inn=abc
error: `urn:iki:fn:toUpper` declares no argument `inn` (its arguments: `in`). To send `inn=abc` as text, name the argument it fills: in="inn=abc"
```

As the error says, text that contains `=` goes to an argument by name: `in="a=b"`. Text whose
part before the `=` is not a name, such as `2+2=4`, is still ordinary positional text. (CLI
0.2.4 and later. Earlier releases sent a misspelled `key=value` on as positional text, so
`inn=abc` came back as `INN=ABC`.)

`demo` is a namespace used for the bundled examples. The host binds the full name
`urn:demo:greet` to an endpoint. Replacing `demo` with another word does not create a new
binding. Later chapters show how to bind your own names.

## Quoting

Wrap a word in `"…"` to keep `|`, `..`, `(`, `)`, `;` or a space literal inside an IRI or
an input; `\"` is a literal quote and `\\` a literal backslash. `in="a b"` is one
argument; `in=a b` is one argument and one stray positional.

Ikigai's quotes do not turn `\n` into a newline. In Bash or Zsh, the shell's `$'…'`
quoting can insert an actual newline before passing the command to ikigai:

```bash
ikigai --plain -c $'source urn:iki:fn:toUpper in="a\nb" | urn:iki:fn:reverseList'
```

Fish has no `$'…'` quoting. There, end the single quotes, write `\n` unquoted, and reopen
them:

```bash
ikigai --plain -c 'source urn:iki:fn:toUpper in="a'\n'b" | urn:iki:fn:reverseList'
```

Either line prints `B` then `A`, on separate lines. The quoting belongs to the shell; do
not type it inside the ikigai REPL. The `split` endpoint in the next example makes a newline list
without needing a literal newline in your command.

## The pipe: `|`

```text
source a [input] | b | c
```

The whole output of one stage becomes the next stage's unnamed argument. Each stage
after the first is written as its IRI alone. The `source` verb is implied.

How the output gets there depends on the stage that receives it (`ikigai-engine` 0.2.2,
`ikigai-cli` 0.2.3 and later). An argument whose contract declares it reads a body
(`ik:readsBody true`, `ArgSpec::reads_body` in Rust) is handed the upstream *as a body*
and reads it as it arrives, so `source urn:file:big.log | sink urn:file:copy.log` copies
the file a chunk at a time rather than holding it. Any other argument gets the output by
value, as before: the kernel collects it whole, under the run's byte budget, and notes
it in that step's trace (`collected`). An output that is already one buffer goes by
value either way. A stored plan (`execute`, `urn:plan:eval`) still pipes by value,
whatever the receiving argument declares. [Streams, I: bytes are a
representation](../streams/bytes.md) is where a body begins.

<div class="ikigai-run" data-cmd='source urn:demo:split "a,b" | urn:iki:fn:toUpper | urn:iki:fn:reverseList'>
<pre class="ikigai-run-expected">B
A
[3 computed]</pre>
</div>

`split` produces two lines, `a` and `b`. `toUpper` changes them to `A` and `B`.
`reverseList` reverses the lines, so `B` comes first. A space does not separate list
items: `reverseList` would leave the single line `A B` unchanged.

The tally reports the cache status of each requested stage or mapped item. It excludes
argument-description fetches and requests made inside an endpoint.

Cacheability flows down the pipe:
a stage over an uncacheable input is no more cacheable than that input, and cutting the
input's thread invalidates the transformed result too.

## The map: `..`

```text
source a [input] .. b
```

Run `b` once per newline-separated item of `a`'s output, and rejoin the results. Newline
lists are the convention every list-shaped endpoint in the ecosystem follows, which is
what makes `..` compose with all of them.

<div class="ikigai-run" data-cmd='source urn:demo:split "a,b,c" .. urn:iki:fn:toUpper'>
<pre class="ikigai-run-expected">A
B
C
[4 computed]</pre>
</div>

Four reported results: one split and three calls to `toUpper`, one per item. The CLI's
`--plain` REPL and `-c` mode also report the fan-out width supported by the scheduler.
`[fan-out 3 → 1 wide]` means three items with capacity for one at a time; `3 → 3 wide`
allows up to three at a time. The width is calculated before dispatch, rather than
measuring how much the work overlapped. The full-screen REPL and this page's cells omit
that extra line.

## The fork: `( a ; b )`

```text
source a [input] | ( b ; c )
```

Send the whole input to each branch, then join the branches' outputs in order. Here,
`toUpper` receives all three lines and returns `A`, `B`, `C`. `reverseList` receives the
original lower-case lines and returns `c`, `b`, `a`.

<div class="ikigai-run" data-cmd='source urn:demo:split "a,b,c" | ( urn:iki:fn:toUpper ; urn:iki:fn:reverseList )'>
<pre class="ikigai-run-expected">A
B
C
c
b
a
[1 cached · 2 computed]</pre>
</div>

The tally reads `1 cached · 2 computed` because the map cell above already resolved the
split, and the engine counts what it served as well as what it ran. Run this before that
cell, on a fresh page, and it reads `3 computed`.

## Composition: `urn:iki:fn:compose`

`compose` is an endpoint that fills in a text template. The book calls that template a
*shape*. A marker such as `$a{urn:data:about}` means "read this resource and insert its
contents here." This inclusion is called *transclusion*. If the included text contains
more `$a{…}` markers, `compose` expands those too.

The CLI comes with a template named `urn:data:page`. Read the template before composing it:

```text
ikigai> source urn:data:page
ikigai compose demo — one pull, recursively assembled

  toUpper : $a{urn:iki:fn:toUpper?in="resource oriented computing"}
  wrap    : $a{urn:demo:wrap?text=hello}
  greet   : $a{urn:demo:greet?greeting=Hi&name=World}
  nested  : $a{urn:data:about}

literal marker (escaped, not expanded): $$a{urn:iki:fn:toUpper?in=x}
[computed]
```

The text after `?` supplies arguments to the resource named inside the marker. `&`
separates arguments. So the greeting uses `greeting=Hi` and `name=World`, both written
directly into this template. They are fixed inputs, not defaults that a caller can
override. `$$a{…}` escapes a marker so it appears literally in the result.

Now ask `compose` to read that template and fill it in. `src` identifies the template:

```text
ikigai> source urn:iki:fn:compose src=urn:data:page
ikigai compose demo — one pull, recursively assembled

  toUpper : RESOURCE ORIENTED COMPUTING
  wrap    : [hello]
  greet   : Hi, World
  nested  : a shape within a shape: COMPOSED WITHIN A COMPOSED SHAPE

literal marker (escaped, not expanded): $a{urn:iki:fn:toUpper?in=x}
[computed]
```

For the nested line, `urn:data:about` contains:

```text
ikigai> source urn:data:about
a shape within a shape: $a{urn:iki:fn:toUpper?in="composed within a composed shape"}
[computed]
```

That second marker explains the uppercase text at the end. The composed result depends
on every resource read along the way. Their golden threads carry through to the result,
so changing one dependency invalidates the cached composition too.

### Make a template and pass it data

In the CLI's REPL, write your own template as a file resource:

```text
ikigai> sink urn:file:greeting.txt Hello, $h{{name}}!
wrote 18 bytes to greeting.txt
[uncacheable]
```

This creates `greeting.txt` in the workspace. `$h{{name}}` inserts a caller-supplied
argument named `name`, escaping it for HTML. The template is separate from the data.

`compose` declares `src` and accepts any other name as a template argument, because the
template decides which names it uses. Pass `name` beside `src`, in the same REPL:

<!-- transcript: continues -->

```text
ikigai> source urn:iki:fn:compose src=urn:file:greeting.txt name=Joe
Hello, Joe!
[computed]
ikigai> source urn:iki:fn:compose src=urn:file:greeting.txt name=Alice
Hello, Alice!
[computed]
```

Both use the same template file;
only the argument changes. This example runs in the CLI, which has the file workspace;
the kernel in this page does not provide one. (CLI 0.2.4 and later. Earlier releases
refused `name=Joe` here; the [Lisp form described below](#and-one-more-) could pass it:
`(invoke 'source "urn:iki:fn:compose" "src" "urn:file:greeting.txt" "name" "Joe")`.)

The legacy Lisp form prints `[uncacheable]` for its outer evaluation; the inner
composition result can still be cached.

## Writing: `sink`

```text
sink <iri> [key=value …] <content>
source a | sink <iri>
```

Leading `key=value` pairs name declared arguments; the rest of the line is the content,
verbatim — spaces, JSON, Turtle and quote marks kept — with one exception (`ikigai-cli`
0.1.46 and later): content that is exactly one quoted token is unquoted by the same rules as
`content="…"`, so `sink <iri> "a note"` stores `a note`, and `sink <iri> "\"a note\""` is how to
store the quote marks. Only `"` quotes; `'a note'` is stored as typed, and so is `"a" and "b"`,
which is not one token. [The file workspace](../getting-started/file-workspace.md#try-it)
replays each case. The pipe form stores the upstream value; with no pipe and no content on the
line, a `sink` under `-c` reads stdin, waits at most two seconds for its first byte, and on
silence sends no `content` at all rather than an empty one (it says so on stderr). A `sink` the
kernel dispatched cuts the golden thread named after the target, whether it succeeded or
(since core 0.1.95) failed, which is how a write invalidates every cached read that declared that
thread, and cached `source` and `exists` answers for that name: since core 0.1.73 every cacheable
`source` or `exists` answer hangs from the thread named after its own target, declared or not, as the [previous
chapter](cli.md#6-cut-a-thread) showed on a pure function.

The `urn:iki:tutorial:*` names in the cell below are this book's own, bound by the page's
kernel and by the tutorial's host; a stock `ikigai` CLI does not bind them, and answers
`error: no endpoint resolved for urn:iki:tutorial:title`. Run the cell here, or in a host of
your own from [Binding](../getting-started/binding.md).

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:title a title from the grammar chapter
source urn:iki:tutorial:camel-title'>
<pre class="ikigai-run-expected">ok
[uncacheable]
aTitleFromTheGrammarChapter
[computed]</pre>
</div>

## Describing: `describe`

```text
describe <iri> [type]
```

`describe` uses the `Meta` verb to ask for a resource's description. The media type
defaults to `text/turtle`, an RDF graph in Turtle format; `text/plain` requests a
human-readable summary. A transreptor converts representations between formats, so
other media types work when the host has a conversion path to them.

<div class="ikigai-run" data-cmd='describe urn:iki:fn:reverseList text/plain'>
<pre class="ikigai-run-expected">reverseList — Reverse list
Reverses the order of newline-separated items in the `in` argument.
verbs: Source, Meta
  input in [argument]: newline-separated items
outputs: text/plain;charset=utf-8
[computed]</pre>
</div>

## Asking: `exists`

```text
exists <iri> [key=value …]
```

The `Exists` verb (`ikigai-cli` 0.1.46 and later): is the resource there, asked without
reading it. The answer is the endpoint's, by convention `true` or `false`. Arguments route as
they do for `source`, and a pipe is refused: the question is about one resource. [The file
workspace](../getting-started/file-workspace.md#try-it) asks it of a file it just wrote and
of one that is not there.

<a id="probing-the-cache-cache"></a>

## Probing: `cache`

```text
cache <iri> [args]
```

Checks whether the requested `source` result is currently cached, without running that
operation. To route arguments, it may first resolve and cache the endpoint's description.

<div class="ikigai-run" data-cmd='cache urn:iki:fn:reverseList in="x"
source urn:iki:fn:reverseList in="x"
cache urn:iki:fn:reverseList in="x"'>
<pre class="ikigai-run-expected">not cached
x
[computed]
cached</pre>
</div>

## Tracing: `trace`

```text
trace <iri> [args]
```

Resolve once, for real, and show the tree: who asked, under what authority, over what
transport, then each invocation with its cache verdict, thread and duration. A composite
shows its sub-resolutions as children; [What resolution buys
you](../getting-started/payoff.md) has one with a child.

## Authority: `cap`

```text
cap                  show the session capability
cap <scope…>         narrow it to these scopes
cap reset            back to the session's identity
```

The session starts as root. Narrow it and every resolution after runs under the narrower
authority — enforced by the kernel, per verb, against what each endpoint declared:

<!-- transcript: manual — the narrowed capability prints this machine's home directory -->

```text
ikigai> cap read-only
narrowed — capability: urn:cap:fs:read:/Users/you/.ikigai/workspace
ikigai> sink urn:file:notes.txt nope
error: denied: capability does not grant `urn:cap:fs:write:*` (declared by `urn:file:notes.txt`; any grant under `urn:cap:fs:write:` satisfies it)
ikigai> cap reset
reset to identity — capability: root (full authority)
```

`read-only` is a profile the CLI's host defines — a name for a set of scopes. A bare
scope IRI works too. [Multi-verb endpoints](../building/multi-verb.md) is where you declare
one on your own endpoint and watch the same refusal.

## Listing: `list`

```text
list
```

Every name bound in the current space — pattern, endpoint id, and where it is served from
when a mount is involved. The kernel's own `urn:kernel:*` resources are intercepted ahead
of the bindings rather than bound; current hosts list them first all the same, and `source`
reaches them.

<div class="ikigai-run" data-cmd='list'>
<pre class="ikigai-run-expected">urn:kernel:actions                        → kernel-actions
urn:kernel:aliases                        → kernel-aliases
urn:kernel:cache                          → kernel-cache
urn:kernel:cached                         → kernel-cached
urn:kernel:catalog                        → kernel-catalog
urn:kernel:constraint                     → kernel-constraint
urn:kernel:cut                            → kernel-cut
urn:kernel:dependents                     → kernel-dependents
urn:kernel:explain                        → kernel-explain
urn:kernel:scheduler                      → kernel-scheduler
urn:kernel:threads                        → kernel-threads
urn:kernel:topology                       → kernel-topology
urn:kernel:uncached                       → kernel-uncached
urn:kernel:validate                       → kernel-validate
urn:iki:fn:toUpper                        → toUpper
urn:iki:fn:reverseList                    → reverseList
urn:iki:fn:compose                        → compose
urn:iki:fn:conditional                    → conditional
urn:iki:fn:each                           → each
urn:demo:wrap                             → wrap
urn:demo:split                            → split
urn:demo:greet                            → greet
urn:demo:echo/{message}                   → echo
urn:iki:tutorial:camel-case               → camel-case
urn:iki:tutorial:title                    → title
urn:iki:tutorial:camel-title              → camel-title
urn:iki:tutorial:ttt:cell:{x}:{y}         → ttt-cell
urn:iki:tutorial:ttt:cells:{list}         → ttt-cells
urn:iki:tutorial:ttt:board                → ttt-board
urn:iki:tutorial:ttt:checkset:{x}:{y}     → ttt-checkset
urn:iki:tutorial:ttt:winner               → ttt-winner
urn:iki:tutorial:ttt:turn                 → ttt-turn
urn:iki:tutorial:ttt:move:{x}:{y}         → ttt-move
urn:iki:tutorial:ttt:reset                → ttt-reset
urn:iki:tutorial:ttt:template:{name}      → ttt-template
urn:iki:tutorial:ttt:view:board           → composeOver
urn:iki:tutorial:ttt:view:square:{x}:{y}  → composeOver
urn:iki:tutorial:ttt:view:status          → composeOver
urn:iki:tutorial:ttt:view:reply           → composeOver
urn:iki:tutorial:ttt:view:game:{game}     → composeOver
urn:iki:tutorial:ttt:view:play:{x}:{y}    → ttt-view-play
urn:iki:tutorial:ttt:view:reset           → ttt-view-reset
urn:iki:tutorial:ttt:stored:{x}:{y}       → ttt-stored
urn:iki:tutorial:time:now                 → time-now
urn:iki:tutorial:time:today               → time-today
urn:iki:tutorial:time:instant             → time-instant
urn:iki:tutorial:time:event:{name}        → time-event
urn:iki:tutorial:time:until:{name}        → time-until
urn:iki:tutorial:time:template:{name}     → time-template
urn:iki:tutorial:time:view:clock          → composeOver
urn:iki:tutorial:sheet:input:{ref}        → sheet-input
urn:iki:tutorial:sheet:feed:{name}        → sheet-feed
urn:iki:tutorial:sheet:tick:{name}        → sheet-tick
urn:iki:tutorial:sheet:compile            → sheet-compile
urn:iki:tutorial:sheet:formula:{ref}      → sheet-formula
urn:iki:tutorial:sheet:refs:{ref}         → sheet-refs
urn:iki:tutorial:sheet:precedents:{ref}   → sheet-precedents
urn:iki:tutorial:sheet:range:{from}:{to}  → sheet-range
urn:iki:tutorial:sheet:cell:{ref}         → sheet-cell
urn:iki:tutorial:sheet:template:{name}    → sheet-template
urn:iki:tutorial:sheet:view:cell:{ref}    → composeOver
urn:iki:tutorial:sheet:view:grid          → composeOver
urn:iki:tutorial:sheet:view:cached:{ref}  → composeOver
urn:iki:tutorial:sheet:view:cost:{ref}    → composeOver
urn:iki:tutorial:sheet:view:cost          → composeOver
urn:iki:tutorial:sheet:view:edit          → sheet-view-edit
urn:iki:tutorial:push:listener:{name}     → push-listener
urn:iki:tutorial:push:carries             → push-carries
urn:iki:tutorial:ttt:arrangement:{name}   → ttt-arrangement
urn:iki:tutorial:ttt:build                → ttt-build
urn:sexpr:arrangement-to-rdf              → sexpr-arrangement-to-rdf
urn:sexpr:arrangement-from-rdf            → sexpr-arrangement-from-rdf
urn:iki:tutorial:check:player:x           → player-x
urn:iki:tutorial:check:player:o           → player-o
urn:iki:tutorial:check:shapes             → player-shapes
urn:iki:tutorial:check:to-turtle          → to-turtle
urn:toolbox:page                          → toolbox-page
urn:toolbox:none                          → toolbox-none
urn:toolbox:text:{message}                → echo
urn:toolbox:path:{rest}                   → toolbox-path
urn:toolbox:input:{iri}                   → toolbox-input
urn:toolbox:check:{iri}                   → toolbox-check
urn:toolbox:{name}                        → toolbox-file
urn:shacl:validate                        → shacl-validate
urn:iki:toolbox:dist:{path}               → book-toolbox-dist
urn:iki:tutorial:shop:stock               → shop-stock
urn:iki:tutorial:shop:prefixes            → live-declaration
urn:iki:tutorial:shop:template:summary    → shop-template-summary
urn:iki:tutorial:shop:template:row        → shop-template-row
urn:iki:tutorial:shop:template:records    → shop-template-records
urn:iki:tutorial:shop:summary             → shop-summary
urn:iki:tutorial:shop:records             → shop-records
urn:json:query                            → json-query
urn:json:from-csv                         → json-from-csv
urn:json:to-csv                           → json-to-csv
urn:compress:gzip                         → compress-gzip
urn:decompress:gzip                       → decompress-gzip
urn:compress:zlib                         → compress-zlib
urn:decompress:zlib                       → decompress-zlib
urn:file:quakes-2024-01-01.geojson        → quakes-window
urn:quakes:template:headline              → quakes-headline
urn:file:quakes.context.jsonld            → quakes-context
urn:quakes:template:graph                 → quakes-graph-template
urn:quakes:graph                          → quakes-graph
urn:file:quakes-2024-01-08.geojson        → quakes-window-2024-01-08
urn:quakes:template:graph:2024-01-08      → quakes-graph-template-2024-01-08
urn:quakes:graph:2024-01-08               → quakes-graph-2024-01-08
urn:quakes:strong                         → quakes-strong
urn:quakes:question:count                 → quakes-question-count
urn:quakes:question:biggest               → quakes-question-biggest
urn:quakes:question:by-magtype            → quakes-question-by-magtype
urn:quakes:question:strong                → quakes-question-strong
urn:quakes:question:strongest-first       → quakes-question-strongest-first
urn:quakes:question:week-after            → quakes-question-week-after
urn:rdf:union                             → rdf-union
urn:rdf:diff                              → rdf-diff
urn:rdf:transrept                         → rdf-transrept
urn:sparql:select                         → sparql-select
urn:sparql:ask                            → sparql-ask
urn:sparql:describe                       → sparql-describe
urn:sparql:construct                      → sparql-construct
urn:file:directory.opml                   → rss-directory.opml
urn:file:feeds.xsl                        → rss-feeds.xsl
urn:file:feeds-local.xsl                  → rss-feeds-local.xsl
urn:file:items.xsl                        → rss-items.xsl
urn:file:feeds/alice.example.com.xml      → rss-feeds-alice.example.com.xml
urn:file:feeds/bob.example.org.xml        → rss-feeds-bob.example.org.xml
urn:file:feeds/carol.example.com.xml      → rss-feeds-carol.example.com.xml
urn:file:feeds/dave.example.org.xml       → rss-feeds-dave.example.org.xml
urn:file:queries/from.rq                  → rss-queries-from.rq
urn:file:queries/in.rq                    → rss-queries-in.rq
urn:file:queries/lines.rq                 → rss-queries-lines.rq
urn:file:queries/per-feed.rq              → rss-queries-per-feed.rq
urn:file:queries/river.rq                 → rss-queries-river.rq
urn:file:queries/search.rq                → rss-queries-search.rq
urn:file:queries/search-bound.rq          → rss-queries-search-bound.rq
urn:file:queries/search-feeds.rq          → rss-queries-search-feeds.rq
urn:file:queries/since.rq                 → rss-queries-since.rq
urn:file:views/digest.xsl                 → rss-views-digest.xsl
urn:file:views/feed.xsl                   → rss-views-feed.xsl
urn:file:views/opml.xsl                   → rss-views-opml.xsl
urn:file:views/page.xsl                   → rss-views-page.xsl
urn:file:river.nt                         → rss-river.nt
urn:xslt:transform                        → xslt-transform
urn:text:wc                               → wc
urn:text:head                             → head
urn:text:tail                             → tail
urn:text:grep                             → grep
urn:text:sort                             → sort
urn:text:uniq                             → uniq
urn:text:nl                               → nl
urn:text:rev                              → rev</pre>
</div>

That is everything the kernel in this page binds. The kernel's own `urn:kernel:*` rows come
first; then Part I's twelve names, in the order they were bound: `ikigai-fn`'s nine, then
this book's three chained on top, exactly as [Binding](../getting-started/binding.md) wrote
them. After those come the Applied parts' tic-tac-toe, time and spreadsheet names, the
names "Checking a resource" uses, and the shop and `ikigai-json` names that "Transreption" and
"Short names in a space" use, the gzip and zlib names of
[Streams, I](../streams/bytes.md), then the recorded weeks, the headline template and the
quakes context, graphs, questions and layer of [Exploring without Rust](../explore/index.md)
([the kernel in the browser](../browser.md#one-recorded-file) says why the page binds those
files), the RDF and SPARQL names [Giving the data names](../explore/names.md) asks
with, and last the example's files, XSLT and the text tools that [A river of
feeds](../explore/rss.md) runs on. A host built from Part I alone lists only Part I's share of
it. A full CLI lists a few hundred, many served over a socket from another
process — which is [Part III](../beyond/socket.md).

## And one more: `(`

A line that starts with `(` is a Lisp form, evaluated by `urn:lisp:eval`, whose builtins
are the five verbs. It is a Scheme (Steel underneath), but not the whole of one: a program
sees an **allowlist** (`ikigai-lisp` 0.2.0), and the verbs are its only way out. On the
list are `lambda`, `define`, `let` and `if`; numbers, lists and strings; `read` on a string
port the program is handed; `with-handler`; and the verbs with their general form
`invoke`. Every other global is rebound to a refusal, so `display`, the environment and
the host's own stdin are out of reach, and the forms that would leave the evaluation,
`require`, `defmacro`, are refused before compiling. A refusal is an error the program can
catch, and a typed `Denied` if it does not.

<!-- urn-gate: illustration urn:demo:echo/{message} — a URI template printed in the
     in-page `list` output above; the family is bound (ikigai-fn's echo), the braces are
     the template's, not a name. -->
