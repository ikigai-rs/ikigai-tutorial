# The REPL grammar

One grammar drives the `ikigai` REPL, its `-c` one-shot, the cells in this book, and the
browser demo's terminal — the same engine, with a different host behind it. This chapter
is that grammar, one runnable line per construct. Where the in-page kernel binds the
names, the line has a **Run** button; the rest run at a shell.

## A resolution

```text
source <iri> [input]
source <iri> key=value …
```

`source` is the verb. A bare word after the IRI is *positional* and fills whichever
declared argument is left unnamed; `key=value` names one, and `key` has to be an argument
the endpoint declared — the engine routes by the description, which is why a typo in a
name is an error rather than a silently ignored argument.

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

## Quoting

Wrap a word in `"…"` to keep `|`, `..`, `(`, `)`, `;` or a space literal inside an IRI or
an input; `\"` is a literal quote and `\\` a literal backslash. `in="a b"` is one
argument; `in=a b` is one argument and one stray positional.

## The pipe: `|`

```text
source a [input] | b | c
```

The whole output of one stage becomes the next stage's unnamed argument, by value. Each
stage after the first is written as its IRI alone — it is a `source`, so the verb is
implied.

<div class="ikigai-run" data-cmd='source urn:iki:fn:toUpper in="a b" | urn:iki:fn:reverseList'>
<pre class="ikigai-run-expected">A B
[2 computed]</pre>
</div>

The tally counts every resolution the line performed. Cacheability flows down the pipe:
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

Four resolutions — one split, three upper-cases. At a shell the CLI adds a line saying
how wide the fan-out ran (`[fan-out 3 → 1 wide]` on a single thread, `3 → 3 wide` on a
scheduled host); the engine itself only counts.

## The fork: `( a ; b )`

```text
source a [input] | ( b ; c )
```

Fan the same input to each branch and join their outputs.

<div class="ikigai-run" data-cmd='source urn:demo:split "a,b,c" | ( urn:iki:fn:toUpper ; urn:iki:fn:reverseList )'>
<pre class="ikigai-run-expected">A
B
C
c
b
a
[3 computed]</pre>
</div>

Run this after the map cell above and the tally reads `1 cached · 2 computed`: the split
was already in the cache, and the engine counts what it served as well as what it ran.

## Composition: `urn:iki:fn:compose`

Not a grammar construct — an endpoint — but the one the grammar exists to serve.
`compose` takes a *shape*, a text resource with `$a{<iri>}` transclusion markers, resolves
every marker through the kernel, and splices the answers in, recursively:

```bash
ikigai --plain -c 'source urn:iki:fn:compose src=urn:data:page'
```

```text
ikigai compose demo — one pull, recursively assembled

  toUpper : RESOURCE ORIENTED COMPUTING
  wrap    : [hello]
  greet   : Hi, World
  nested  : a shape within a shape: COMPOSED WITHIN A COMPOSED SHAPE
```

The page is one resource that pulled the others in. That is how the browser demo builds
its whole page, and how every part of a composed answer stays under its own golden
thread: the composite is cacheable only while every part is.

## Writing: `sink`

```text
sink <iri> [key=value …] <content>
source a | sink <iri>
```

Leading `key=value` pairs name declared arguments; the rest of the line is the content.
The pipe form stores the upstream value. A successful `sink` cuts the golden thread named
after the target, which is how a write invalidates every cached read that declared that
thread — the file endpoint's reads do; a pure function's do not, having nothing to hang
one on.

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

The `Meta` verb. The type defaults to `text/turtle`; `text/plain` is the human face, and
any other type the host can reach through a transreptor works too.

<div class="ikigai-run" data-cmd='describe urn:iki:fn:reverseList text/plain'>
<pre class="ikigai-run-expected">reverseList — Reverse list
Reverses the order of newline-separated items in the `in` argument.
verbs: Source, Meta
  input in [argument]: newline-separated items
outputs: text/plain;charset=utf-8
[computed]</pre>
</div>

## Probing the cache: `cache`

```text
cache <iri> [args]
```

Would this be served from the cache right now? Answered without resolving, so asking
does not change the answer.

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
error: denied: capability does not grant `urn:cap:fs:write:*` (declared by `urn:file:notes.txt`)
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
when a mount is involved. The kernel's own `urn:kernel:*` resources are intercepted rather
than bound, so they do not appear here; `source` reaches them.

<div class="ikigai-run" data-cmd='list'>
<pre class="ikigai-run-expected">urn:iki:fn:toUpper            → toUpper
urn:iki:fn:reverseList        → reverseList
urn:iki:fn:compose            → compose
urn:iki:fn:conditional        → conditional
urn:demo:wrap                 → wrap
urn:demo:split                → split
urn:demo:greet                → greet
urn:demo:echo/{message}       → echo
urn:iki:tutorial:camel-case   → camel-case
urn:iki:tutorial:title        → title
urn:iki:tutorial:camel-title  → camel-title</pre>
</div>

Eleven names in this page, in the order they were bound — `ikigai-fn`'s eight, then
this book's three chained on top, exactly as [Binding](../getting-started/binding.md)
wrote them. A full CLI lists a few hundred, many served over a socket from another
process — which is [Part III](../beyond/socket.md).

## And one more: `(`

A line that starts with `(` is a Lisp form, evaluated by `urn:lisp:eval` — a full
language whose builtins are the five verbs. Out of scope for this book, and worth
knowing exists.

<!-- urn-gate: illustration urn:demo:echo/{message} — a URI template printed in the
     in-page `list` output above; the family is bound (ikigai-fn's echo), the braces are
     the template's, not a name. -->
