# The TypeScript track

<!-- urn-gate: illustration urn:ts:hello — served by examples/deno/hello.ts, mounted at
     the reader's own `--mount urn:ts:=`; not a name the CLI or this workspace binds. -->
<!-- urn-gate: illustration urn:ts:shout — likewise. -->

[The Python track](python.md) showed how a client resolves names and how a served peer
answers mounted requests. Here are the same operations in TypeScript, using Deno.
Every listing is in `examples/deno/`; run the shell commands from this repository's root.

The listings import `ikigai-deno` by path because it is not on JSR yet. They expect its
checkout beside this repository. If you do not already have it there, clone it:

```bash
git clone https://github.com/ikigai-rs/ikigai-deno ../ikigai-deno
```

Edit the listings' imports if your checkout is elsewhere. ikigai-deno imports `@std/crypto`
for the BLAKE3 digests in its contract IRIs. Its own `deno.json` maps that name to JSR, but
Deno does not use that configuration when another script imports ikigai-deno by path.
Every `deno run` here therefore passes `--config examples/deno/deno.json`, which supplies
the mapping for these listings. A first run fetches the dependency from JSR.

The blocks that start a peer and mount it ([Binding](#binding) and the family below) are
replayed against the book's pinned `ikigai-cli` on every change to this repository, with
ikigai-deno at its `main`, by `scripts/polyglot-tracks.sh`: the peers are started from these
very lines, and what each `ikigai` command prints, cache verdicts included, is compared with
the page. The client listings were run against a served `ikigai-cli` 0.1.18 as they were
written and are not replayed.

If the kernel from the [Python track](python.md) is still serving `/tmp/ikbook-kernel.sock`,
skip the first two lines. The second one waits for the kernel's socket, because a client
started at once can find none yet.

<!-- transcript: serve -->
```bash
ikigai serve /tmp/ikbook-kernel.sock &
until [ -S /tmp/ikbook-kernel.sock ]; do sleep 0.2; done
deno run -A --config examples/deno/deno.json examples/deno/client.ts /tmp/ikbook-kernel.sock
```

<a id="resolution"></a>

## Resolution: a name, over the wire

```ts
{{#include ../../../../examples/deno/client.ts:resolve}}
```

`connect` returns a client whose methods are the five verbs; `source` is a `Promise` of
a representation — text, media type, cache verdict. The cache verdict is worth a look on
your first run: if another client already resolved that request against the same kernel,
it says `Hit` before you have done anything. The cache belongs to the kernel, not to the
connection.

<a id="hello-resource"></a>

## Hello, resource: an endpoint with explicit args

```ts
{{#include ../../../../examples/deno/hello.ts:endpoint}}
```

```ts
{{#include ../../../../examples/deno/hello.ts:serve}}
```

The explicit `args:` list declares the inputs by hand, as in
[Why an endpoint describes itself](../getting-started/self-description.md), without a schema
library. `ikigai-deno`'s `./zod` entry point derives the same list from a schema
instead (`z.object({ who: z.string().describe(…) })`), and that is the closer analog to
"the signature is the contract"; the sibling repository's `examples/endpoints.ts` shows
it. Either way, a Rust host sees one description.

<a id="why-an-endpoint-describes-itself"></a>

## Why an endpoint describes itself: its description, as JSON or Turtle

```ts
{{#include ../../../../examples/deno/client.ts:describe}}
```

`describe` is the JSON face of `Meta`, parsed; `meta(iri, "text/turtle")` is the graph.
The same two faces the Rust chapters used, reached from a runtime that has never parsed
Turtle.

<a id="binding"></a>

## Binding: a mount is a binding

Start the peer, and wait for its socket before you mount it. The server creates
`/tmp/ikbook-ts` with mode `0700` if it is missing. The socket belongs in a directory
you own: the server refuses to place it directly in `/tmp`, which is owned by root:

<!-- demo: start -->
```bash
rm -f /tmp/ikbook-ts/hello.sock
deno run -A --config examples/deno/deno.json examples/deno/hello.ts /tmp/ikbook-ts/hello.sock &
until [ -S /tmp/ikbook-ts/hello.sock ]; do sleep 0.2; done
```

The wait is not decoration. Deno fetches `@std/crypto` from JSR on a first run before the
peer binds its socket, and an `ikigai --mount` started at once fails with
`No such file or directory`. The `rm -f` clears a socket an earlier run left behind, which
would otherwise end the wait before this peer is listening.

<!-- demo: run -->
```text
$ ikigai --plain --mount urn:ts:=/tmp/ikbook-ts/hello.sock -c 'source urn:ts:hello who=Ada'
Hello, Ada!
[uncacheable]
$ ikigai --plain --mount urn:ts:=/tmp/ikbook-ts/hello.sock -c 'source urn:demo:split "resource,oriented,computing" | urn:ts:shout | urn:iki:fn:reverseList' -c 'source urn:kernel:uncached'
COMPUTING!
ORIENTED!
RESOURCE!
[1 computed · 2 uncacheable]
uncached (why a computed read was not cached; the last 64, most recent first)
  urn:iki:fn:reverseList      ×1  upstream  [root]
  urn:ts:shout                ×1  mounted  [root]

[uncacheable]
— batch: 2 commands · 1 computed · 3 uncacheable
```

The pipe is the part to notice: Rust into TypeScript into Rust, in the host's grammar, with
none of the three knowing the others' language. `urn:demo:split` (Rust) turns the commas
into a list of three lines. `urn:ts:shout` (TypeScript) takes that list and shouts each
line. `urn:iki:fn:reverseList` (Rust) reverses the order of the lines, so `computing` comes
first. Each stage leaves a mark you can see: the list's three items, the capitals and `!` on
every line, and the reversed order. So the answer shows that both crossings happened, and in
that order. The host owns the topology; the peers own their
answers.

Now read the verdicts. Both endpoints declare `cacheable: true`, and the peer marks its
answers that way on the wire. `ikigai-cli` 0.2.9, the version this page is replayed against,
caches neither: every read through `--mount` is `[uncacheable]`, and `urn:kernel:uncached`
gives `mounted` as the reason. Only `split`, a Rust endpoint in the host, was computed and
kept. `reverseList` is uncacheable too, for a different reason, `upstream`: it read an answer
that could not be cached. That is the mount's rule, on purpose: a kernel that mounts a peer stores no copy of
what the peer answers, because a change on the peer's side (a file it watches, a write
that reaches it by another route) never crosses back to the mounting kernel, so a stored
copy could be served stale for as long as the host runs. The `mounted` reason identifies
an answer held here uncached because the peer's cuts do not reach this kernel; `declared`
is reserved for an answer declared uncacheable. So `cacheable: true` on a served peer describes
its answers; through `--mount`, the host does not keep them.

<a id="what-resolution-buys-you"></a>

## What resolution buys you: the kernel's cache and trace, from outside

```ts
{{#include ../../../../examples/deno/client.ts:cached}}
```

```ts
{{#include ../../../../examples/deno/client.ts:traced}}
```

<a id="capabilities"></a>

## Capabilities: a scoped connect

```ts
{{#include ../../../../examples/deno/client.ts:scoped}}
```

The kernel refuses the write, and the client receives a typed `DeniedError` across the
wire. The client can test `instanceof DeniedError` instead of parsing an error message.
`examples/http_status.ts` in the ikigai-deno checkout uses that distinction to return
HTTP 403 for a denied request.

## Families: one door, many names, four verbs

<!-- urn-gate: illustration urn:ts:note:{name} — the family examples/deno/notes.ts serves,
     mounted at the reader's own `--mount urn:ts:=`; not a name the CLI or this workspace
     binds. -->
<!-- urn-gate: illustration urn:ts:note:milk — a note in that family; likewise. -->
<!-- urn-gate: illustration urn:ts:note:bread — a note nobody wrote; likewise. -->
<!-- urn-gate: illustration urn:ts:note:index — the family's index door; likewise. -->
<!-- urn-gate: illustration urn:note:{name} — the alias-stripped form of the family, what a
     `--mount urn:ts:=` host forwards; a placeholder, not a resource. -->
<!-- urn-gate: illustration urn:note:milk — likewise, for one note. -->
<!-- urn-gate: illustration urn:ts:note: — the prefix a deeper mount of that family would
     take; a placeholder, not a resource. -->

Every name so far was one resource. Most interesting resources come in families, such as
a note for every name or a cell for every square, and a Rust endpoint binds a family with a
`UriTemplate`. In TypeScript a family is `family(pattern, options)`, with one chained call per verb. The template's variables are declared as `bindings` and arrive in the handler's arguments by name:

```ts
{{#include ../../../../examples/deno/notes.ts:family}}
```

`{name}` is a Level 1 variable, and it matches the way `ikigai_core::UriTemplate` matches,
line for line. A variable followed by a literal captures up to the leftmost occurrence of
that literal, a trailing variable takes the rest, and every capture is non-empty. The host
and the peer have to agree on which names a pattern covers, because the host reads the
peer's catalog to decide what to forward, so the face mirrors core's rule and does not
approximate it. The catalog lists the template, so `list` shows one family, not every note
anyone has written.

Each verb gets its own contract, which is core's per-verb `ActionSpec` form: a Source that
is cacheable and answers `NotFound` for a missing note, a Sink whose body arrives as
`content`, and a Delete. Exists is not written at all. It defaults to "would Source
succeed", so a `NotFound` answers `false`.

<!-- demo: start -->
```bash
rm -f /tmp/ikbook-ts/notes.sock
deno run -A --config examples/deno/deno.json examples/deno/notes.ts /tmp/ikbook-ts/notes.sock &
until [ -S /tmp/ikbook-ts/notes.sock ]; do sleep 0.2; done
```

<!-- demo: run -->
```text
$ ikigai --plain --mount urn:ts:=/tmp/ikbook-ts/notes.sock -c 'sink urn:ts:note:milk two liters' \
  -c 'source urn:ts:note:milk' -c 'source urn:ts:note:milk' \
  -c 'sink urn:ts:note:milk three liters' -c 'source urn:ts:note:milk' \
  -c 'source urn:ts:note:index' -c 'delete urn:ts:note:milk' -c 'source urn:ts:note:milk'
wrote milk
[uncacheable]
two liters
[uncacheable]
two liters
[uncacheable]
wrote milk
[uncacheable]
three liters
[uncacheable]
milk
[uncacheable]
forgot milk
[uncacheable]
error: not found: no note called milk
— batch: 8 commands · 7 uncacheable
```

Read the verdicts. The note's Source declares itself cacheable, and, as in
[Binding](#binding), `ikigai-cli` 0.2.9 caches nothing a mounted peer answers, so every read
here reached the peer. The peer keeps no cache of its own either, and that is why the read
after the second write saw the new text. The writes and the delete are uncacheable whatever
the host does, because a Sink or Delete answer always is, and the index is uncacheable on
purpose: it reads every note, and a write is about only the note it wrote.

To try Exists from a client, save this as `examples/deno/exists.ts`. The batch above
deleted `milk`, so the script writes it again before checking:

```ts
import { connect } from "../../../ikigai-deno/src/mod.ts";

const k = await connect("/tmp/ikbook-ts/notes.sock"); // the peer itself; Exists is a read
await k.sink("urn:ts:note:milk", "two liters");
console.log((await k.exists("urn:ts:note:milk")).text); // true
console.log((await k.exists("urn:ts:note:bread")).text); // false
k.close();
```

With the notes peer still running, run the script from this repository's root:

```text
deno run -A --config examples/deno/deno.json examples/deno/exists.ts
```

```text
true
false
```

### Which door answers

Two rules, and the second one is the reason the first can be trusted.

**The first declared door that matches wins**, as in core's `EndpointSpace`. The family
would match `urn:ts:note:index` too, with `{name}` capturing `index`. The index works
because it is declared first:

```ts
{{#include ../../../../examples/deno/notes.ts:index}}
```

**Every door has two forms, and every door's declared form is tried before any door's
stripped form.** `--mount urn:ts:=` is an *alias* mount: the host rewrites
`urn:ts:note:milk` to `urn:note:milk` before forwarding it, the way
[Preferential resolution](../beyond/preference.md) described. A peer cannot know the prefix it was
mounted at, only that it was aliased, so it answers each door at its declared pattern and at
that pattern with the first segment stripped: `urn:ts:note:{name}` and `urn:note:{name}`.
If the forms were tried door by door, one door's stripped form could shadow the next door's
declared name. Trying every declared form first means a stripped form can only answer a
name that no door declares. A connection whose hello says it comes from an alias mount is
sending stripped names, so for that connection the order reverses, and stripped forms go
first. The Python face follows the same rule, so a family behaves the same whichever language serves it. Because only the first segment can be stripped, a family mounted deeper,
at `urn:ts:note:` for example, needs `--override`, which forwards names unchanged.

A family is how a store in another language holds a game: the sixth part of the tic-tac-toe
arc, [Tic-tac-toe, VI](../applied/tic-tac-toe-6.md), which you have already read, serves the game's stored cell as one,
from TypeScript, under a Rust host that keeps everything else.

## What an L0 peer cannot do

The same limit as the [Python track](python.md), and it is a property of the wire, not of
the language: a served peer has no back-channel into the kernel invoking it, so a
TypeScript endpoint cannot `source` another resource mid-invocation and cannot inherit a
golden thread. It is a leaf. Composition — pipes, maps, `compose` — happens in the host
above it, as the pipe above showed.
