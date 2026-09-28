# The TypeScript track

<!-- urn-gate: illustration urn:ts:hello — served by examples/deno/hello.ts, mounted at
     the reader's own `--mount urn:ts:=`; not a name the CLI or this workspace binds. -->
<!-- urn-gate: illustration urn:ts:shout — likewise. -->

The same mirror of Part I, in Deno. Every listing is in `examples/deno/` and was run
against a served `ikigai-cli` 0.1.18 as it was written. `ikigai-deno` is not on JSR yet, so
the listings import it by path — a sibling checkout at `../../../ikigai-deno`; edit the
import if yours is elsewhere.

```bash
ikigai serve /tmp/ikbook-kernel.sock &
deno run -A examples/deno/client.ts /tmp/ikbook-kernel.sock
```

## Resolution

```ts
{{#include ../../../../examples/deno/client.ts:resolve}}
```

`connect` returns a client whose methods are the five verbs; `source` is a `Promise` of
a representation — text, media type, cache verdict. The cache verdict is worth a look on
your first run: if another client already resolved that request against the same kernel,
it says `Hit` before you have done anything. The cache belongs to the kernel, not to the
connection.

## Hello, resource

```ts
{{#include ../../../../examples/deno/hello.ts:endpoint}}
```

```ts
{{#include ../../../../examples/deno/hello.ts:serve}}
```

Explicit `args:` here — the ArgSpec list stated the way the Rust chapter stated it, and
zero-dependency. `ikigai-deno`'s `./zod` entry point derives the same list from a schema
instead (`z.object({ who: z.string().describe(…) })`), and that is the closer analog to
"the signature is the contract"; the sibling repository's `examples/endpoints.ts` shows
it. Either way, a Rust host sees one description.

## Why an endpoint describes itself

```ts
{{#include ../../../../examples/deno/client.ts:describe}}
```

`describe` is the JSON face of `Meta`, parsed; `meta(iri, "text/turtle")` is the graph.
The same two faces the Rust chapters used, reached from a runtime that has never parsed
Turtle.

## Binding

```bash
deno run -A examples/deno/hello.ts /tmp/ts-hello.sock &
ikigai --plain --mount urn:ts:=/tmp/ts-hello.sock -c 'source urn:ts:hello who=Ada'
ikigai --plain --mount urn:ts:=/tmp/ts-hello.sock -c 'source urn:ts:shout text="resource oriented computing" | urn:iki:fn:reverseList'
```

```text
Hello, Ada!
[computed]
RESOURCE ORIENTED COMPUTING!
[2 computed]
```

The second line is the part to notice: a pipe from a TypeScript endpoint into a Rust one,
in the host's grammar, with neither knowing the other's language. The host owns the
topology; the peers own their answers.

## What resolution buys you

```ts
{{#include ../../../../examples/deno/client.ts:cached}}
```

```ts
{{#include ../../../../examples/deno/client.ts:traced}}
```

## Capabilities

```ts
{{#include ../../../../examples/deno/client.ts:scoped}}
```

`instanceof DeniedError` — the taxonomy crossed the wire, which is wire protocol v7's
whole point and what `examples/http_status.ts` in the sibling repository turns into a
403.

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

```bash
deno run -A examples/deno/notes.ts /tmp/ts-notes.sock &
ikigai --plain --mount urn:ts:=/tmp/ts-notes.sock -c 'sink urn:ts:note:milk two liters' \
  -c 'source urn:ts:note:milk' -c 'source urn:ts:note:milk' \
  -c 'sink urn:ts:note:milk three liters' -c 'source urn:ts:note:milk' \
  -c 'source urn:ts:note:index' -c 'delete urn:ts:note:milk' -c 'source urn:ts:note:milk'
```

```text
wrote milk
[uncacheable]
two liters
[computed]
two liters
[cached]
wrote milk
[uncacheable]
three liters
[computed]
milk
[uncacheable]
forgot milk
[uncacheable]
error: not found: no note called milk
```

Read the verdicts. The second read of `milk` was served from the host's cache. The peer
keeps no cache. The write that followed cut the note's golden thread in the host, so the
next read computed again and saw the new text. The peer wrote no invalidation code. The
host cuts a name's thread after every Sink or Delete it forwards to that name. The index is
uncacheable on purpose, because it reads every note and a write cuts only the note it
wrote. Exists, from a client:

```ts
const k = await connect("/tmp/ts-notes.sock"); // the peer itself; Exists is a read
console.log((await k.exists("urn:ts:note:milk")).text); // true, after `sink urn:ts:note:milk two liters`
console.log((await k.exists("urn:ts:note:bread")).text); // false
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
[Binding](../getting-started/binding.md) described. A peer cannot know the prefix it was
mounted at, only that it was aliased, so it answers each door at its declared pattern and at
that pattern with the first segment stripped: `urn:ts:note:{name}` and `urn:note:{name}`.
If the forms were tried door by door, one door's stripped form could shadow the next door's
declared name. Trying every declared form first means a stripped form can only answer a
name that no door declares. A connection whose hello says it comes from an alias mount is
sending stripped names, so for that connection the order reverses, and stripped forms go
first. The Python face follows the same rule, so a family behaves the same whichever language serves it. Because only the first segment can be stripped, a family mounted deeper,
at `urn:ts:note:` for example, needs `--override`, which forwards names unchanged.

A family is how a store in another language holds a game: the last part of the tic-tac-toe
arc, [Tic-tac-toe, VI](../applied/tic-tac-toe-6.md), serves the game's stored cell as one,
from TypeScript, under a Rust host that keeps everything else.

## What an L0 peer cannot do

The same limit as the [Python track](python.md), and it is a property of the wire, not of
the language: a served peer has no back-channel into the kernel invoking it, so a
TypeScript endpoint cannot `source` another resource mid-invocation and cannot inherit a
golden thread. It is a leaf. Composition — pipes, maps, `compose` — happens in the host
above it, as the pipe above showed.
