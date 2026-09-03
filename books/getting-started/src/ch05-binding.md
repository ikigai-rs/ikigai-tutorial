# Binding, and a host of your own

An endpoint is inert. Until something **binds** it to a name, nothing can resolve it —
defining and naming are separate acts, which is the whole point of
[chapter 1](ch01-resolution.md) showing up in the code.

## Binding

```rust,ignore
{{#include ../../../crates/hello-camel/src/lib.rs:space}}
```

Two things are happening.

`ikigai_fn::space()` is the built-in function library as a mountable space —
`urn:fn:toUpper`, `urn:fn:compose`, `urn:fn:conditional` and friends. `bind` is a builder,
so a host **starts from somebody else's space and chains its own bindings on top**. That
is the normal shape of a host: mostly other people's endpoints, plus the few that are
yours.

`Exact::new("urn:fn:toCamel")` matches one exact IRI. Bindings can also be URI templates,
which is how `urn:file:{path}` covers a whole tree with one binding.

## Binding authority is a host concern

Notice that the crate defining `toCamel` does not decide it lives at `urn:fn:toCamel`. It
offers a constructor; the **host** decides the name. Two hosts can bind the same endpoint
at different names, and a host can refuse to bind it at all.

## Why this book does *not* patch the CLI

You could add `toCamel` to the CLI's own `base_space` in `ikigai-embedded` and get it on
the `ikigai` binary. Don't — not because it fails, but because it teaches the wrong
reflex. **You do not extend ikigai by editing ikigai.** You compose a kernel with the
spaces you want, which is what the CLI itself is doing.

So this book ships its own host instead, and it is short enough to read in full:

```rust,no_run
# extern crate hello_camel;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request, Verb};

let kernel = Kernel::new(Arc::new(hello_camel::space()));

let request = Request::new(Verb::Source, Iri::parse("urn:fn:toCamel").unwrap())
    .with_arg("in", ArgRef::Inline(b"resource oriented computing".to_vec()));

let repr = block_on(kernel.issue(request, &Capability::root())).unwrap();
assert_eq!(String::from_utf8_lossy(&repr.bytes), "resourceOrientedComputing");
```

`Capability::root()` is unrestricted authority, which is fine for a local tutorial and is
*not* what a real host hands out — [chapter 7](ch07-file-workspace.md) shows the scoped
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
