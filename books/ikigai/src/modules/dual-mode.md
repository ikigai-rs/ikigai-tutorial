# Dual-mode crates

Three crates in the ecosystem support being modules — `ikigai-xslt` 0.1.3,
`ikigai-jsonld` 0.1.2 and `ikigai-shacl` 0.1.2, the versions published as this was
re-checked (2026-09-26) — and **none of them is module-only**. All three are ordinary linked libraries by default (each has a
`module` Cargo feature), and become loadable WebAssembly when you ask:

```bash
cargo build --release --lib --features module --target wasm32-unknown-unknown
```

That needs the target's standard library (`rustup target add wasm32-unknown-unknown`) in
the `rustc` cargo actually runs. If the build fails with E0463, "can't find crate for
`core`", after you have added the target, another Rust is ahead of rustup's on PATH —
Homebrew's, for example, which ships no wasm32 standard library. `command -v rustc` says
which one you have; put `~/.cargo/bin` first on PATH.

## How it is wired

The `module` feature turns on optional dependencies rather than changing the crate's
identity:

```toml
[dependencies.ikigai-module]
version = "0.1.9"   # the floor that builds against the core these crates use
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

The same argument as binding authority in Part I: a library that can only be a module has
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

Write it as an ordinary space, the way Part I did. Add the `module` feature only when
somebody actually needs the loadable shape.

Nothing about the endpoint changes; the module feature is packaging.
