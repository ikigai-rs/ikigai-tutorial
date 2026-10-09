# Browse roots: a repository as resources

A ledger item about a file is only as useful as the reader's way to the file. gonk can serve
the repositories you work in beside the ledger, as resources under `urn:repo:{root}:*`: a
directory's listing, a file's bytes, the working tree's state, a content hash. They come from
[`ikigai-browse`](https://github.com/ikigai-rs/ikigai-browse), and gonk adds the one thing a
library cannot: it **watches** each repository, so the reads can be cached and still never be
stale.

This page makes a repository of its own, serves it, reads it, and changes it while gonk runs.

## A repository, and a root

Any git working tree will do. This one is made here, with a commit identity set on the
repository alone so that nothing outside it is touched:

<!-- transcript: run -->
```console
$ git init -q -b main notes
$ printf '# Notes\n\nA repository this book creates.\n' > notes/README.md
$ git -C notes config user.name 'A Reader'
$ git -C notes config user.email reader@example.com
$ git -C notes add README.md
$ git -C notes commit -qm 'Start the notes'
```

A **browse root** gives a directory a name, and the name is the `{root}` in every resource
gonk serves for it. Roots live in the config home's `config.toml`, one line each, as
`name=path`. A path may start with `~`, which is the home directory of whoever runs gonk; this
page's root is not in yours, so it writes the path out in full:

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.browse.root = "notes=%s/notes"\n' "$PWD" > .config/ikigai/config.toml
$ cat .config/ikigai/config.toml
gonk.browse.root = "notes=…/notes"
```

`--browse-root notes=PATH` on the command line does the same for one run. gonk reads its roots
when it starts, and refuses to start if one names a directory that is not there: a missing
root is the shape of a typo, not something to serve as empty.

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s)
  browse  urn:repo:{notes (watched)}:* — annotations and archive in <urn:iki:browse:graph:default>
…
```

The `browse` line names each root and whether its watcher started, and says where the browse
family keeps what it writes (annotations, archived explanations): one graph of the same store
the ledger lives in, so a SPARQL query can join a ledger item to the file it is about.

## Reading it

The repository is reached the way the ledger was in [Over the socket](../agent/socket.md):
another ikigai process mounts gonk's socket. With the alias at `urn:gk:`, the root's listing
is `urn:gk:repo:notes:tree`:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:notes:tree'
.git	dir	-
README.md	file	41
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:notes:file:README.md'
# Notes

A repository this book creates.
[uncacheable]
```

A listing line is `name`, `kind` and `size`, tab-separated. The file resource answers the
file's own bytes, under the media type its extension maps to. Two more resources answer the
questions a reader of a repository keeps asking, *is this what I think it is* and *has
anything changed*:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:notes:hash:README.md'
sha256:6140d9c09fbe92774e6cf987758c63e9b4cba8638ee23b6c6c66999c6274de72
[uncacheable]
$ shasum -a 256 notes/README.md
6140d9c09fbe92774e6cf987758c63e9b4cba8638ee23b6c6c66999c6274de72  notes/README.md
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:notes:state'
… clean
[uncacheable]
```

`hash` is the sha256 of the file's bytes, so it is the same number `shasum` gives; on a
directory it is a Merkle hash over the entries, so one edit changes exactly the hashes on the
path from that file to the root. `state` is the commit `HEAD` names and whether the working
tree is `clean` or `dirty:{n}`. Each of these is a name you can put in a ledger item's `about`,
which is how browse's file page and a ledger item find each other.

Every listing has a graph face too, with stable IRIs and no blank nodes, so it can be queried
or joined:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:notes:tree as=text/turtle'
@prefix ik: <https://ikigai-rs.dev/ns#> .

<urn:repo:notes:tree> a ik:Directory ;
    ik:repo "notes" ;
    ik:entry <urn:repo:notes:tree:.git>, <urn:repo:notes:file:README.md> .

<urn:repo:notes:tree:.git> a ik:Directory ;
    ik:fileName ".git" ;
    ik:path ".git" .

<urn:repo:notes:file:README.md> a ik:File ;
    ik:fileName "README.md" ;
    ik:path "README.md" ;
    ik:byteSize 41 .

[uncacheable]
```

## Changing it while gonk runs

Commit a change, without touching gonk:

<!-- transcript: run -->
```console
$ printf 'Read through gonk, without a restart.\n' >> notes/README.md
$ git -C notes commit -qam 'Say how it is read'
$ sleep 1
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:notes:file:README.md' -c 'source urn:gk:repo:notes:hash:README.md'
# Notes

A repository this book creates.
Read through gonk, without a restart.
[uncacheable]
sha256:…
[uncacheable]
— batch: 2 commands · 2 uncacheable
```

The new content, at once, from a server that was never restarted. Here is what made that
both fast and right:

- `ikigai-browse` declares its reads **live and uncacheable**. That is the only honest thing a
  library can say: caching is a promise that something will notice when the file changes,
  and a library cannot know whether its host is watching.
- gonk is watching. It watches every root (FSEvents on macOS, inotify on Linux) and, when a
  file changes, **cuts the golden thread** the root's reads hang from. So for a watched root,
  and only for one, it declares the plain reads cacheable: a read of an unchanged file is a
  cache hit, and the first read after a change recomputes. That is what `(watched)` on the
  banner promised. A root whose watcher cannot start is named as unwatched there, and its reads
  are served live, never cached and unwatched.
- The cache is **gonk's**, behind the socket. The `[uncacheable]` this client prints is about
  its own cache: golden threads live in one kernel and do not cross a wire, so an answer that
  left gonk cacheable would arrive with nothing that could ever cut it, and a long-lived client
  (an agent's `ikigai mcp`, say) would go on serving the old file. So gonk sends every answer
  across the socket and the QUIC door uncacheable, and keeps the cache entry, and the thread it
  hangs from, on its own side: a repeated read is one round trip to gonk's cache, never a
  recompute.
- The `sleep 1` is honest, not decoration. The notification arrives with the platform's own
  latency, well under a second, so a read issued in the same instant as the write can still
  see the previous version once.

The watch ignores what git ignores: a change inside `.git/` (apart from the files `git status`
reads) or inside a `target/` directory cuts nothing, so a build does not invalidate a root a
thousand times a minute. The Turtle face above said `[uncacheable]` for a related reason: only
the plain read with no arguments is declared cacheable, because the other faces read state the
watch does not track.

## In a browser, and from your own config

The browser's way in is `/browse`, the roots the caller's grant may read. An anonymous caller
may read none, because the anonymous grant is ledger tokens only, so what it gets is the
sign-in notice:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/browse | grep -o 'Sign in to browse the repositories.'
Sign in to browse the repositories.
```

A passkey invited with `--browse read`, as in [the browser chapter](../ledger/browser.md),
reads every root; `--browse read --root notes` reads only this one. The pages are browse's own
HTML faces (crumbs, listings, a highlighted file view with `#L{n}` anchors and the annotation
margin) inside gonk's header.

For an ikigai host that should see the roots under their own names, as the ledger was in the
last chapter, the mount line is the same shape with a different prefix:

```toml
mount = "prefer urn:repo:=/Users/you/.ikigai/gonk.sock"
```

⚠ `urn:repo:` is one prefix over two families: the browse roots (`urn:repo:{root}:…`) and
`ikigai-repo`'s git and GitHub facades (`urn:repo:status`, `urn:repo:log`, `urn:repo:pr:list`),
which then run in gonk's process. A prefix mount cannot say "the roots remote and the facades
local".

## What is next for the browser

A root is also the place an explanation is about: browse can derive one per file or
directory from a model gonk mounts, archived once per content version, so the hash above is
the key that says when one is stale. That needs a model, and is the next chapter,
*Explain, with a mounted model* (coming).

In resource terms: the **atoms** are the files on disk, which git and you change and gonk
does not; `tree`, `file`, `hash` and `state` are **views** of them; and the watcher is what
turns a change to an atom into a cut thread, so every view derived from it recomputes on its
next read and no sooner.
