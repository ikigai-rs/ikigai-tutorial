# Resolution

## A name, not a call

`urn:fn:toUpper` is a name. Nothing about it says where the code lives, what language it
is in, whether the answer is computed now or was computed an hour ago and cached, or
whether it runs in this process. Those are all decisions the **kernel** makes when it
*resolves* the name.

This is the one idea to take seriously before any of the code makes sense. In a function
call, the caller has already decided almost everything by the time it calls. In a
resolution, the caller has decided only *what it wants*.

## The five verbs

There is no method vocabulary to learn, because there are five verbs and they are the
same five for every resource in the system:

| verb | meaning |
|---|---|
| `Source` | give me a representation of this |
| `Sink` | here is a new state for this |
| `Exists` | is there anything at this name? |
| `Delete` | remove it |
| `Meta` | describe yourself |

`Meta` is the one that surprises people. Every endpoint can be asked what it is, what
arguments it takes, what it returns, and what authority it requires — and it answers in a
machine-readable form. That is what makes the system legible to an agent rather than
merely usable by a programmer, and [chapter 4](ch04-self-description.md) is about it.

## Representations

A resolution returns a **representation**: bytes plus a media type. Not an object, not a
language-specific value — bytes with a declared type, because the answer may have come
from another process, another machine, or another language.

When the type you have is not the type you want, a **transreptor** converts between them.
The consequence worth internalizing: a resource does not have *a* format. It has whatever
formats the kernel can reach from what it has, and asking for `as=text/turtle` is a
routing question, not a serialization call.

## Golden threads

When a resolution is derived from other resolutions, the kernel records the dependency.
Write to something upstream and everything derived from it is invalidated — precisely,
not by guesswork and not by expiry guessing.

This is why caching here is not a bolt-on. A cache that cannot tell you *why* an entry is
still valid has to fall back on timeouts; one that tracks derivation can keep an answer
until the thing it came from actually changes.

> ⚠ It also means **cacheability propagates from your dependencies**. Adding one
> uncacheable source to an otherwise cached resource makes the whole thing uncacheable —
> a correctness no-op that is a large performance change. Nothing warns you; the types are
> identical either way.

## Capabilities

Authority travels with the invocation, not with the process. An endpoint runs under a
**capability**, and when it resolves a sub-request that capability is what the sub-request
runs under — it can be narrowed on the way down, never widened.

The rule the whole system leans on: **declared capabilities are enforced capabilities.**
An action that enforces authority it does not declare makes the catalog lie by promising
more than it can do; one that declares authority it does not enforce is worse. Both are
treated as defects.

## Where this is going

Put those together and you get a system that can describe itself completely: a catalog of
every resolvable name (`urn:kernel:catalog`), and a capability-scoped list of what the
*current* caller may actually invoke (`urn:kernel:actions`). An agent's tool list is not
something you write down for it — it is that second thing, computed.
