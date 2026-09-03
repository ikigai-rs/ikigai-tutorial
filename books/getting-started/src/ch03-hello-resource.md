# Hello, resource

The smallest useful thing: an endpoint that camel-cases text. It is this book's "hello
world", and it is a real endpoint — the listing below is *included from the crate that
compiles it*, so it cannot drift from what actually runs.

## The implementation

```rust,ignore
{{#include ../../../crates/hello-camel/src/lib.rs:impl}}
```

Four things in there are worth slowing down for.

## `inv.inline_str("in")`

Arguments arrive on the `Invocation`, by name, and they may be *inline bytes* or *a
reference to another resource*. `inline_str` says "give me this argument as a UTF-8
string, and fail cleanly if it is not there or is not text."

The reference case is the interesting one, and you get it for free: a caller can pass
`in=urn:something:else` and the kernel resolves that first. Your endpoint does not change.
This is why pipes work — `|` is not a shell feature bolted on, it is one resolution's
output becoming another's argument.

## `Representation::new(text_plain_utf8(), …)`

You return bytes *and their type*. Always. The type is not decoration: it is what lets a
transreptor find a route from what you produced to what somebody asked for.

## `.cacheable()`

You are asserting **this is a pure function of its declared inputs**. Same arguments, same
answer, forever. The kernel may then cache it and hand out the cached representation under
a golden thread.

Get this wrong in the optimistic direction and you have a bug that is very hard to see:
stale answers that look plausible. The rule of thumb is the honest one — *when in doubt,
do not cache*. `toCamel` is genuinely pure, so it says so.

## `char::to_uppercase` returns an iterator

Not a `char`. Some characters upper-case to more than one — German ß becomes SS — so the
API cannot pretend otherwise. `output.extend(first.to_uppercase())` is the Unicode-correct
form; `push` would not compile, which is the type system doing you a favour.

## Try it

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
use hello_camel::to_camel;
use ikigai_core::Endpoint;

// Every endpoint knows its own name.
assert_eq!(to_camel().describe().id, "toCamel");
```
