# Introduction

A resource-oriented computing kernel in Rust, taught from the outside in.

## What ikigai is, in one paragraph

Everything is a **resource** named by a URI. You never call a function; you **resolve** a
name against a **kernel**, which routes it to a bound **endpoint** and hands back a
**representation** — bytes plus a media type. There are five verbs, not a vocabulary of
methods: `Source` (read), `Sink` (write), `Exists`, `Delete`, and `Meta` (describe
yourself). **Transreptors** convert one representation into another, so the same resource
can arrive as text, as Turtle, or as HTML. **Golden threads** track what a result was
derived from, so a write invalidates exactly what it should. **Capabilities** gate
authority and attenuate as they pass down a call chain.

That is the whole model. The rest is consequences.

## Why bother

Because naming a thing and *resolving* a thing are different acts, and separating them
buys you a lot at once. If a computation has a name, it can be cached, traced,
substituted, authorized, converted, and moved to another machine without its caller
knowing. A function call gives you none of that; it is a jump with arguments.

The wager of this project is that a system where every step is a resolvable name is
*cheaper to reason about at scale* than one where the steps are opaque calls — and that
this matters most now that programs are being assembled by agents, which need exactly
what resolution gives you: a machine-readable catalog of what can be done, an enforceable
boundary on what may be done, and provenance for what was done.

## What is in here

**Part I — Getting started** builds an endpoint and links it into a kernel you compose
yourself. It is the 95% case, and everything else assumes it.

**Part II — Loadable modules** covers the other shape: a `space()` compiled separately
and routed to at runtime, and the callback that makes a module something quite different
from a remote peer.

Read Part I first, or at least [Resolution](getting-started/resolution.md) and
[Binding](getting-started/binding.md). Part II assumes you know what a `space()` is and
why binding is separate from defining.

## How to read this book

Every Rust block in these pages is compiled and run by `mdbook test`, and the longer code
listings are **included from the crates that compile them** rather than copied. A book
that paraphrases its own examples is a book that will eventually be wrong about them.

```bash
# read it
mdbook serve books/ikigai --open

# check that it still tells the truth
./scripts/test-books.sh
```

The code lives in
[`crates/`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates) — one crate per
part.

## Conventions

Shell commands assume you are at the root of the `ikigai-tutorial` repository. Anything
marked ⚠ is a trap that has actually caught somebody.
