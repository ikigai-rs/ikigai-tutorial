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

## What an L0 peer cannot do

The same limit as the [Python track](python.md), and it is a property of the wire, not of
the language: a served peer has no back-channel into the kernel invoking it, so a
TypeScript endpoint cannot `source` another resource mid-invocation and cannot inherit a
golden thread. It is a leaf. Composition — pipes, maps, `compose` — happens in the host
above it, as the pipe above showed.
