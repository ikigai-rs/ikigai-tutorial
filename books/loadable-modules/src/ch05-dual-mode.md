# Dual-mode crates

Three crates in the ecosystem support being modules — `ikigai-xslt`, `ikigai-jsonld` and
`ikigai-shacl` — and **none of them is module-only**. All three are ordinary linked
libraries by default, and become loadable WebAssembly when you ask:

```bash
cargo build --release --lib --features module --target wasm32-unknown-unknown
```

## How it is wired

The `module` feature turns on optional dependencies rather than changing the crate's
identity:

```toml
[dependencies.ikigai-module]
version = "0.1.6"
optional = true

[features]
module = [
    "dep:ikigai-module",
    "dep:wasm-bindgen",
    # …
]
```

The endpoints are the same endpoints either way. What the feature adds is the wasm-bindgen
surface that lets a host instantiate the artifact and drive a session against it.

## Why dual-mode is the right default

Because *linked or loaded* should be the **host's** decision, not the library's.

The same argument as binding authority in Book 1: a library that can only be a module has
decided something on its consumer's behalf. A CLI that wants XSLT compiled in should be
able to have it; a browser page that cannot link anything should be able to fetch it. One
crate, two deployments, one set of endpoints.

## The one that is module-only, and why

`ikigai-xslt-module` is a separate crate that wraps `ikigai-xslt` as a standalone
`cdylib`, and its description states the reason plainly: *"Built separately and
lazy-loaded by a host, so xrust isn't linked into the host's binary."*

That is the honest edge of the dual-mode story. The feature approach still builds one
artifact from one crate; when what you want is a *separately shipped* artifact with its
own build profile, a thin wrapper crate is clearer than another feature flag.

It is `publish = false` — it is a build product, not a library anyone should depend on.

## What this means for your own crate

Write it as an ordinary space, the way Book 1 did. Add the `module` feature only when
somebody actually needs the loadable shape.

Nothing about the endpoint changes; the module feature is packaging.
