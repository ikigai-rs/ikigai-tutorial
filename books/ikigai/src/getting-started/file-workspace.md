# The file workspace

Files are resources like anything else. `urn:file:{path}` is bound with a URI template, so
one binding covers a whole tree, and reading a file is a `Source` — the same verb as
everything else.

## The jail

The tree is **`$IKIGAI_FILES`, else `~/.ikigai/workspace`**. The CLI creates it if it is
missing, and since `ikigai-cli` 0.1.41 it creates it private (`0700`) whatever your umask;
a directory that already exists is left as you made it. The file endpoint never creates its
own root: a root it cannot open serves nothing.

It is deliberately a dedicated, ikigai-owned sandbox and deliberately *not* your home
directory or your documents. Even the owner's root capability reaches only inside this
tree, and the file endpoint's jail is a hard floor **regardless of capability** — a
second, independent check, because one mechanism guarding your entire filesystem is one
mechanism too few.

"Hard floor" is a claim about the file actually opened, not about the path you typed, and
it holds in that sense since `ikigai-fs` 0.1.8 (`ikigai-cli` 0.1.40). Before that the jail
judged the path you named, and a symbolic link, or a swap between the check and the open,
could put a different file behind it. Now:

- The path is opened **one name at a time**, each from an open handle on the directory
  before it, and the read or write happens on the handle that walk produced, so nothing can
  be swapped in between the check and the use.
- A **symbolic link anywhere in the path** is refused: the last name or a directory on the
  way, pointing inside the tree or out of it.
- A file has **one name**, its path as it is spelled on disk. `./notes.txt`, `a//b` and
  `dir/` are refused, and the error names the spelling to use (`notes.txt`); so is
  `NOTES.txt` for `notes.txt` on a volume that folds case, as macOS does by default. `..`
  and absolute paths never leave the tree.
- On **Windows** there is no confined backend yet, so the file endpoint refuses every
  request rather than serve one unconfined.

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

At the REPL (`ikigai`), or one line at a time with `ikigai --plain -c '…'`:

```text
ikigai> sink urn:file:notes.txt a note
wrote 6 bytes to notes.txt
[uncacheable]
ikigai> source urn:file:notes.txt
a note
[computed]
```

Do not quote the note. The rest of a `sink` line is the content **verbatim**, quote marks
included, so a quoted note is stored with its quotes and read back with them. Quoting is for
the words of a `source` line and for a named argument, so `content="…"` is the way to quote a
body:

```text
ikigai> sink urn:file:notes.txt "a note"
wrote 8 bytes to notes.txt
[uncacheable]
ikigai> source urn:file:notes.txt
"a note"
[computed]
ikigai> sink urn:file:notes.txt content="a note"
wrote 6 bytes to notes.txt
[uncacheable]
ikigai> source urn:file:notes.txt
a note
[computed]
```

There is no `exists` command. The REPL spells four of the five verbs (`source`, `sink`,
`delete`, and `describe` for `Meta`); `Exists` is issued from code, as
`Request::new(Verb::Exists, …)`.

Then look in `~/.ikigai/workspace`: it is an ordinary directory. Nothing about the model
requires the storage to be exotic; it requires the *access* to be named.
