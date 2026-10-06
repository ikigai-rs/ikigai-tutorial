# Explain, with a mounted model

The [previous chapter](roots.md) served a repository as resources: listings, bytes, hashes,
state. Every one of them is computed from the files. This chapter adds the first resource a
**model** writes: `urn:repo:{root}:explain:{path}`, a few sentences saying what a file is for,
derived once per version of the file and kept.

Two things make that more than "call an LLM". The answer is **archived**, keyed by the file's
content hash and a version tag, so the second reader of the same bytes gets it from the store
with no model call. And gonk does not link a model at all: it **mounts** one, at `urn:llm:`,
from another process. To browse, a kernel that binds `urn:llm:` to anything that answers is a
kernel with a model, which is what lets this chapter show every rule of the archive with a stub
that costs nothing and says the same thing every time.

## A model is two names

browse asks a model through a **provider**, `urn:llm:{name}:ask`, with the prompt as an
argument, and reads which model that is from `urn:llm:{name}:model`. So a stand-in is a space
binding those two names. This one, from
[`crates/browse-host`](https://github.com/ikigai-rs/ikigai-tutorial/tree/main/crates/browse-host),
answers an explanation or a review (the next chapter's) depending on what it is asked, and
counts every ask:

```rust,ignore
{{#include ../../../../crates/browse-host/src/lib.rs:stub}}
```

It declares `urn:cap:net:*`, as the real `urn:llm:` module does: asking a model is network use,
and the capability is how a host decides who may spend it. The host puts browse beside it:

```rust,ignore
{{#include ../../../../crates/browse-host/src/lib.rs:kernel}}
```

`ExplainConfig` names which provider writes which layer and holds the store the archive lives
in; `Mount` names the roots. `config_home(None)` keeps browse from reading this machine's
configuration, so the run below is the same on every machine.

## Explaining a file

The repository is a directory holding one file, `src/lib.rs`:

```rust,ignore
{{#include ../../../../crates/browse-host/demo/src/lib.rs}}
```

Before anything is explained, **`explain-status`** says what an explain would cost, without
calling the model. Its plain face is three tab-separated fields: the state, the version tag
the answer would carry, and the model.

```rust
# extern crate browse_host;
# extern crate ikigai_core;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use browse_host::{ask, kernel, Scratch, EXPLANATION};
use ikigai_core::Verb;

let repo = Scratch::new();
let calls = Arc::new(AtomicUsize::new(0));
let browse = kernel(repo.path(), calls.clone());
let source = |iri: &str, args: &[(&str, &str)]| ask(&browse, Verb::Source, iri, args).unwrap();

let status = source("urn:repo:demo:explain-status:src/lib.rs", &[]);
assert_eq!(status, "derive\tcode-v1@stub-1\tstub-1");
assert_eq!(calls.load(Ordering::SeqCst), 0);

let explanation = source("urn:repo:demo:explain:src/lib.rs", &[]);
assert_eq!(explanation.trim(), EXPLANATION);
assert_eq!(calls.load(Ordering::SeqCst), 1);

let status = source("urn:repo:demo:explain-status:src/lib.rs", &[]);
assert_eq!(status, "archived\tcode-v1@stub-1\tstub-1");

// Again: from the archive. The model is not asked.
let json = source("urn:repo:demo:explain:src/lib.rs", &[("as", "application/json")]);
assert!(json.contains(r#""derived":false"#), "{json}");
assert!(json.contains(r#""version_tag":"code-v1@stub-1""#), "{json}");
assert_eq!(calls.load(Ordering::SeqCst), 1);
```

The state is `derive` until something has been archived for these bytes at this tag, and
`archived` after. `explain-status` has a third state, `stub`, for a binary file, which is
described in one line and never sent to a model.

The **version tag** is `{prompt}@{model}`. `code-v1` is the prompt browse uses for source code
(a note, a skill file and plain text each have their own); `stub-1` is what
`urn:llm:stub:model` said. Change either and the archived answer no longer applies, which is the
point: an explanation is evidence about *these bytes, asked this way, of this model*, and the
tag says which. The JSON face carries `derived`, true only on the read that called the model.

## The archive is keyed by the bytes

Edit the file, and the next explain is a new derivation; put the old bytes back, and the old
answer is still there:

```rust
# extern crate browse_host;
# extern crate ikigai_core;
# use std::sync::atomic::{AtomicUsize, Ordering};
# use std::sync::Arc;
use browse_host::{ask, kernel, Scratch, INITIALS_RS};
use ikigai_core::Verb;

let repo = Scratch::new();
let calls = Arc::new(AtomicUsize::new(0));
let browse = kernel(repo.path(), calls.clone());
let source = |iri: &str| ask(&browse, Verb::Source, iri, &[]).unwrap();

source("urn:repo:demo:explain:src/lib.rs");
repo.write("src/lib.rs", &format!("{INITIALS_RS}\n/// Shouts.\npub fn shout() {{}}\n"));
assert!(source("urn:repo:demo:explain-status:src/lib.rs").starts_with("derive\t"));
source("urn:repo:demo:explain:src/lib.rs");
assert_eq!(calls.load(Ordering::SeqCst), 2);

// Two versions archived, newest first: tag, content hash, model, when.
let versions = source("urn:repo:demo:explain-versions:src/lib.rs");
assert_eq!(versions.lines().count(), 2, "{versions}");
assert!(versions.lines().all(|v| v.starts_with("code-v1@stub-1\tsha256:")), "{versions}");

// Back to the first bytes: archived, and no third call.
repo.write("src/lib.rs", INITIALS_RS);
assert!(source("urn:repo:demo:explain-status:src/lib.rs").starts_with("archived\t"));
source("urn:repo:demo:explain:src/lib.rs");
assert_eq!(calls.load(Ordering::SeqCst), 2);
```

Nothing was invalidated, and nothing needed to be. An archive keyed by content cannot go stale:
an edit does not change the old entry, it asks for a different one. That is why gonk can keep
explanations forever and why reverting a file, or checking out an old branch, costs nothing.

## The cost line

A person deciding whether to click **Explain** wants to know what it will cost before it costs
anything. The status resource's HTML face is that sentence, and it is what gonk's file page shows
beside the button:

```rust
# extern crate browse_host;
# extern crate ikigai_core;
# use std::sync::atomic::AtomicUsize;
# use std::sync::Arc;
use browse_host::{ask, kernel, Scratch};
use ikigai_core::Verb;

let repo = Scratch::new();
let browse = kernel(repo.path(), Arc::new(AtomicUsize::new(0)));
let cost = || ask(&browse, Verb::Source, "urn:repo:demo:explain-status:src/lib.rs",
                  &[("as", "text/html")]).unwrap();

assert!(cost().contains("explain: not yet explained at this version — stub-1 will write it, \
                         which may take a while. After that it opens instantly."));
ask(&browse, Verb::Source, "urn:repo:demo:explain:src/lib.rs", &[]).unwrap();
assert!(cost().contains("explain: already explained at this version by stub-1 — \
                         opens instantly from the archive, no model call."));
```

Both sentences come from the same three facts the plain face prints; the page does not guess.

## Served: gonk with a model mounted

gonk composes exactly this, with a real store and a real model. The model is named on the
command line (or by a `gonk.mount` line in `config.toml`) as a peer to mount at `urn:llm:` —
a Unix socket, or `quic://host:port` with a certificate directory. Make a repository to browse,
and start a gonk of your own with a mount whose socket nothing is listening on:

<!-- transcript: file demo/src/lib.rs -->
```rust,ignore
/// A name's initials: "Ada Lovelace" is "AL".
pub fn initials(name: &str) -> String {
    name.split(' ').map(|word| &word[..1]).collect()
}
```

<!-- transcript: file .config/ikigai/config.toml -->
```toml
gonk.browse.root = "demo=~/demo"
```

<!-- transcript: serve -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk --port 1070 --no-quic --no-backup --mount "prefer urn:llm:=$PWD/model.sock"
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s)
  browse  urn:repo:{demo (watched)}:* — annotations and archive in <urn:iki:browse:graph:default>
  backup  …
  llm     …/model.sock (prefer; dialled on first use) — explain/review bound; file urn:llm:coder:ask @400, dir urn:llm:ask @600, review urn:llm:coder:ask @800, pr urn:llm:coder:ask @600 tokens. Deriving needs urn:cap:net:localhost, which `--browse derive` mints and nothing else does
…
```

The `llm` line is the mount: where the model is, that it is **dialled on first use** rather than
at startup (so a model that is down does not stop a ledger from serving), and which provider
writes which layer, with each layer's token ceiling. gonk's defaults ask `urn:llm:coder:ask` for
files and reviews and `urn:llm:ask` for directories — names a peer running the `ikigai-llm`
module serves. The last sentence is the capability: deriving spends the model, so it needs a
network grant that only a `--browse derive` client is given.

Ask over the socket, as in the previous chapter:

<!-- transcript: run -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:repo:demo:explain-status:src/lib.rs' -c 'source urn:gk:repo:demo:explain:src/lib.rs'
derive	code-v1@coder	coder
[uncacheable]
error: unavailable: the mounted peer at …/model.sock is not reachable (last dial failed; the next is held off for up to 30s)
— batch: 2 commands · uncacheable
```

The status still answers: it reads the archive and never calls the model. With no peer to ask,
the model is named by the provider's last segment, `coder`, instead of what `urn:llm:coder:model`
would have said. The explain itself is refused as `unavailable`, which is the honest answer: a
model that cannot be reached is not an explanation that is empty, and nothing is archived.

## A real model

<!-- transcript: manual — it needs a running model, which this book's checks do not have (no network, no GPU) -->
```console
$ ikigai-gonk --mount "prefer urn:llm:=quic://models.local:4433 /path/to/model-client-certs"
```

A peer that serves `ikigai-llm` (an `ikigai serve` with the module bound, in front of Ollama or
another backend) is mounted the same way; everything above then behaves as it did with the stub,
except that the first explain of each version takes seconds instead of microseconds and the tag
names a real model. The archive is why that is affordable: the model is paid once per version of
each file, however many people read it.

## What you built

In resource terms: the **atoms** are the files and browse's store (the archive); the **model** is
an input nothing here can derive, bound at two names; `explain` is a **composition** of a file and
a model, written into the archive; `explain-status` and `explain-versions` are **views** of the
archive that never call the model. The one piece of code written to answer a request is the stub
model, and in gonk even that is not code but a mount.
