# The file workspace

Files are resources like anything else. `urn:file:{path}` is bound with a URI template, so
one binding covers a whole tree, and reading a file is a `Source` — the same verb as
everything else.

## The jail

The tree is **`$IKIGAI_FILES`, else `~/.ikigai/workspace`**, created if missing.

It is deliberately a dedicated, ikigai-owned sandbox and deliberately *not* your home
directory or your documents. Even the owner's root capability reaches only inside this
tree, and the file endpoint's jail is a hard floor **regardless of capability** — a
second, independent check, because one mechanism guarding your entire filesystem is one
mechanism too few.

## Two mechanisms, on purpose

It is worth naming why the jail exists *in addition to* capabilities, since capabilities
are supposed to be the authority model:

- **Capabilities** decide what a given caller may do — `urn:cap:fs:read:ws/abc123` grants
  read under one segment, and attenuates as it passes down a call chain.
- **The jail** decides what the *endpoint* can reach at all, and no capability can widen
  it.

A bug in capability minting is then a bug about *which files inside the sandbox* were
reachable, not a bug about whether your SSH keys were.

## Segments

Capabilities are usually scoped to a **segment** of the workspace rather than the whole
thing — `ws/{id}`, where the id is derived from an identity.

There is a running demonstration of that, and it is **not in this repository**: it is
[`ikigai-web-demo`](https://github.com/ikigai-rs/ikigai-web-demo), the kernel-as-WebAssembly
page from [Running it](running-it.md), at
<https://ikigai-rs.github.io/ikigai-web-demo/>. Neither `hello-camel` nor
`loadable-module` has an identity of any kind, so nothing you have built so far can show
you this.

Open that page's **Identity** tab and sign in with a passkey: the credential yields a
stable client id, and the id scopes a private workspace segment. The tab then offers three
steps, and the third is the one to run — you try to write to *somebody else's* segment and
the resolver refuses. The boundary is the capability model, doing the one job it exists
for.

## Reading files through the kernel, not `std::fs`

> ⚠ Inside a module or endpoint, read files by resolving `urn:file:…` rather than calling
> `std::fs` directly.

Two reasons, and the second is the one that catches people:

1. `std::fs` bypasses the jail and the capability check entirely.
2. A kernel read is **golden-threaded** — a filesystem watcher can cut that thread when the
   file changes, so anything derived from it recomputes. A `std::fs` read is invisible to
   the kernel, so a cached result built on it goes stale silently.

It also keeps your code able to run in WebAssembly, where there is no filesystem to call in
the first place.

## Try it

```bash
ikigai --plain -c 'sink urn:file:notes.txt "a note"'
ikigai --plain -c 'source urn:file:notes.txt'
ikigai --plain -c 'exists urn:file:notes.txt'
```

Then look in `~/.ikigai/workspace` — it is an ordinary directory. Nothing about the model
requires the storage to be exotic; it requires the *access* to be named.
