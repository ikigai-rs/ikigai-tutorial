# Binding, and a host of your own

An endpoint is inert. Until something **binds** it to a name, nothing can resolve it —
defining and naming are separate acts, which is the whole point of
[Resolution](resolution.md) showing up in the code.

## Binding

```rust,ignore
{{#include ../../../../crates/hello-camel/src/lib.rs:space}}
```

`space()` is one call to `space_over`, which is where the bindings are:

```rust,ignore
pub fn space_over(state: Arc<TitleState>) -> EndpointSpace {
    ikigai_fn::space()
        .bind(Exact::new("urn:iki:tutorial:camel-case"), camel_case())
        .bind(Exact::new(TITLE), title(state))
        .bind(Exact::new("urn:iki:tutorial:camel-title"), camel_title())
}
```

Two things are happening.

`ikigai_fn::space()` is the built-in function library as a mountable space —
`urn:iki:fn:toUpper`, `urn:iki:fn:compose`, `urn:iki:fn:conditional` and friends. `bind`
is a builder, so a host **starts from somebody else's space and chains its own bindings
on top**. That is the normal shape of a host: mostly other people's endpoints, plus the
few that are yours — here, the three from [Hello, resource](hello-resource.md) and [What
resolution buys you](payoff.md). (The `state` handle is `title`'s memory, passed in so a
test can watch it; `space()` makes a fresh one.)

`Exact::new("urn:iki:tutorial:camel-case")` matches one exact IRI. Bindings can also be URI
templates, which is how `urn:file:{path}` covers a whole tree with one binding.

## Binding authority is a host concern

Notice that the crate defining the camel-case endpoint does not decide it lives at
`urn:iki:tutorial:camel-case`. It offers a constructor; the **host** decides the name. Two
hosts can bind the same endpoint at different names, and a host can refuse to bind it at
all.

## Bind into a namespace you own

Look again at the binding line above, and at the two namespaces in it. The space is
`ikigai-fn`'s; the name is this book's own, under `urn:iki:tutorial:`. **The space you
compose and the name you bind are independent** — chaining onto somebody else's space
gives you no claim on their prefix, and nothing but discipline stops you from binding into
it anyway.

That discipline is what was missing here. This endpoint used to be bound at
`urn:fn:toCamel`: nothing about it belonged to `ikigai-fn`, it merely sat inside
`ikigai-fn`'s prefix. When that library renamed its own namespace, this book's endpoint
was caught in a migration it had no stake in — a name in somebody else's prefix moves on
their schedule, not on yours. Rebinding it under `urn:iki:tutorial:` is the entire fix,
and it is one line.

<!-- urn-gate: unbound urn:fn:toCamel — the name this endpoint used to be bound at.
     Naming it is the point of the paragraph above, and the gate asserts it really is
     gone: if some host starts binding it again, this claim stops being true. -->

## Name resources as nouns

That one line changed in two ways at once, and the section above covered only the first.
The endpoint did not merely leave `ikigai-fn`'s prefix: it also stopped being `toCamel`
and became `camel-case`.

`toCamel` is a verb phrase. It reads as something you *call*, and that is exactly the
reflex [Resolution](resolution.md) is trying to break. A resource is a thing you name and
ask a kernel to resolve; whether the answer is computed now, served from an hour-old
cache, or fetched from another machine is not yours to decide, and a name shaped like a
function call quietly implies otherwise. So the convention is: **resource names are nouns,
and kebab-case** — `camel-case`, not `toCamel`.

Kebab-case is the smaller half of the rule. It matters because these names do not stay in
Rust — they are projected into an agent's tool list, into generated editor commands, into
shell one-liners — and a hyphen is a word separator every one of those already reads as
one.

> ⚠ Now look back at the top of this chapter. `urn:iki:fn:toUpper` breaks the rule, and so
> does its sibling `urn:iki:fn:reverseList` — in the library this book depends on. You have
> not caught an oversight. The convention was settled after those names were already in use
> across a dozen repositories, so fixing them is a second migration with a schedule of its
> own, which is the previous section's point arriving from the other direction: a name you
> use out of somebody else's prefix is correct when *they* say it is.

## Why this book does *not* patch the CLI

You could add `camel-case` to the CLI's own `base_space` in `ikigai-embedded` and get it
on the `ikigai` binary. Don't — not because it fails, but because it teaches the wrong
reflex. **You do not extend ikigai by editing ikigai.** You compose a kernel with the
spaces you want, which is what the CLI itself is doing.

So this book ships its own host instead. The kernel is one line:

```rust,ignore
{{#include ../../../../crates/hello-camel/src/lib.rs:kernel}}
```

`Kernel::new(Arc::new(space()))` would resolve every `Source` in this book just as well.
What it could not do is answer `Meta`, or `urn:kernel:catalog`, which is `Meta` over
every binding: turning a `Description` into bytes is a *projection*, and `ikigai-core`
owns no projection — there is no RDF in the kernel, by design. `ikigai-vocab`'s
`TurtleRenderer` is the projection to Turtle (and to `text/plain` and JSON), and
injecting it is the entire difference between a host that can describe itself and one
that answers `no Meta renderer configured`. The CLI does the same thing with the same
renderer.

And a resolution through it, short enough to read in full:

```rust
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Request, Verb};

let kernel = hello_camel::kernel();

let request = Request::new(Verb::Source, Iri::parse("urn:iki:tutorial:camel-case").unwrap())
    .with_arg("in", ArgRef::Inline(b"resource oriented computing".to_vec()));

let repr = block_on(kernel.issue(request, &Capability::root())).unwrap();
assert_eq!(String::from_utf8_lossy(&repr.bytes), "resourceOrientedComputing");
```

`Capability::root()` is unrestricted authority, which is fine for a local tutorial and is
*not* what a real host hands out — [The file workspace](file-workspace.md) shows the scoped
kind.

## Finding your crate from another project

When you want your endpoints in a host that lives in a different repository, the
dependency is an ordinary Cargo one. During development, a path reference:

```toml
hello-camel = { path = "../ikigai-tutorial/crates/hello-camel" }
```

and once published, a version.

> ⚠ A path reference is a *local* arrangement: never commit one into a shared repository,
> because it only resolves on the machine that has both checkouts laid out that way. This
> workspace deliberately depends on the **published** ikigai crates for the same reason —
> clone it alone and it builds.
