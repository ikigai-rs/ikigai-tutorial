# Introduction

Book 1 built an endpoint and **linked it in**: your crate exposed a `space()`, the host
depended on it at compile time, and the endpoints ended up in the binary. That is how
nearly all of ikigai works today, including every endpoint in the `ikigai` CLI.

This book is about the other shape — a **module**: an independently compiled `space()`
that a host routes some names to *without linking it*.

## Read this book for one idea

A module is **not** a remote kernel, and the difference is not a matter of degree.

When you resolve against an IPC or QUIC peer, that peer resolves every sub-request on its
own side. It has its own catalog, its own cache, its own spaces. You hand it a name and it
hands you back an answer; nothing crosses back.

A module is the opposite arrangement. Its endpoints must resolve *their* resource
references — an XSLT stylesheet, a SHACL shapes graph, a configuration file — against the
**host's** kernel, because the host owns the catalog and the cache. So a module has to
call **back** into the host in the middle of its own invocation.

Everything else in this book is consequences of that one asymmetry.

## What you will build

A module with one endpoint, a host with one resource, and a greeting that can only be
produced by the module asking the host a question mid-invocation:

```bash
cargo run -p loadable-module
```

```text
host    resolves urn:greet:hello name=urn:host:name
module  asks the host for urn:host:name
out     Hello, Peter!
```

## Before you start

Read [Book 1](../getting-started/introduction.md) first, or at least its chapters on
resolution and binding. This book assumes you know what a `space()` is and why binding is
separate from defining.

The code is in
[`crates/loadable-module`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates/loadable-module),
and as in Book 1 the listings here are pulled from it by anchor rather than copied.

> ⚠ **Read [chapter 6](ch06-status.md) before you plan around any of this.**
> `ikigai-module` describes itself as "Phase 1: in-process proof". The callback machinery
> is real and exercised; the isolation is not there yet.
