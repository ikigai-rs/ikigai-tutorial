# A QUIC client, and the grant it runs under

The [previous chapter](doors.md) opened gonk's QUIC door and sent one request through it. This one
sets up a client the way you would for a second machine: an identity minted on the server, a grant
that says exactly what that identity may do, the identity moved to the client, and the server
refusing everything outside the grant — including a certificate it was never told to trust.

Both "machines" here are directories on one machine: the server keeps its two homes in the page's
directory, as in every chapter, and `laptop/` stands in for the client's machine. Everything below
works the same with the second one somewhere else, with `127.0.0.1` replaced by the server's
address.

## A repository, and an identity with a grant

The client will read a repository as well as the ledger, so the server browses one:

<!-- transcript: run -->
```console
$ git init -q -b main notes
$ printf '# Notes\n' > notes/README.md
$ mkdir -p .config/ikigai
$ printf 'gonk.browse.root = "notes=%s/notes"\n' "$PWD" > .config/ikigai/config.toml
$ cat .config/ikigai/config.toml
gonk.browse.root = "notes=…/notes"
```

The root's path is written out in full: a `~` in `config.toml` means the home directory of
whoever runs gonk, which is yours, not this page's.

`client add` mints a certificate and its key, writes them as a **bundle** with the server's own
certificate, and enrolls the client's fingerprint under a grant. The grant is built from roles,
never typed as tokens: `--ledger default=write` is the ledger's writer, `--browse read --root
notes` reads that one repository.

<!-- transcript: run -->
```console
$ ikigai-gonk client add laptop --ledger default=write --browse read --root notes --port 1070 --config-home "$PWD/.config/ikigai"
client `laptop`  …/.config/ikigai/gonk/quic/clients/laptop
  fingerprint  …
  principal    urn:iki:gonk:client:…
  enrolled     grant `laptop` (6 scopes) in …/.config/ikigai/gonk/grants.json
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <the moved directory>
$ ls .config/ikigai/gonk/quic/clients/laptop
client.crt
client.key
server.crt
$ cat .config/ikigai/gonk/grants.json
{
  "laptop": [
    "urn:cap:ledger:read:default",
    "urn:cap:store:read:graph:urn:iki:ledger:graph:default",
    "urn:cap:ledger:write:default",
    "urn:cap:store:write:graph:urn:iki:ledger:graph:default",
    "urn:cap:browse:read:notes",
    "urn:cap:store:read:graph:urn:iki:browse:graph:default"
  ]
}
```

Six tokens: the four-token ledger writer from [the first chapter](../ledger/your-own.md), reading
the `notes` root, and reading browse's graph through the store's narrow door (so the client can
join its items to what browse knows, as in [the embedding chapter](../embedding/own-kernel.md)).
`clients.json`, beside it, maps the fingerprint to the grant's name. Two files, re-read on every
connection: editing either changes what a client may do, or revokes it, without a restart. The
**certificates** gonk trusts are read once, at startup — so start it now:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  quic    udp 0.0.0.0:1070 — 1 trusted certificate(s), 1 enrolled
…
```

## The client

The bundle holds the client's private key, so it belongs on the client and nowhere else. Copy it to
the client's home (on a real second machine, move it there and delete it here — `client add`'s
last lines say so):

<!-- transcript: run -->
```console
$ mkdir -p laptop
$ cp -R .config/ikigai/gonk/quic/clients/laptop laptop/gonk
```

and connect with it, naming the bundle with `--cert-dir`:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir laptop/gonk -c 'sink urn:iki:ledger:append Read the notes' -c 'source urn:repo:notes:tree' -c 'source urn:repo:notes:file:README.md'
#1 urn:iki:ledger:default:item:…

[uncacheable]
.git	dir	-
README.md	file	8
[uncacheable]
# Notes

[uncacheable]
— batch: 3 commands · 3 uncacheable
```

The client pinned the server by the `server.crt` in its bundle, and the server pinned the client
by its fingerprint. There is no certificate authority anywhere: each side knows exactly one key
for the other, and that is the whole of the trust.

## Outside the grant

Everything this client is not granted is refused, by the resource that was asked, naming what it
wanted:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir laptop/gonk -c 'delete urn:iki:ledger:item:1' -c 'sink urn:iki:ledger:acme:append Not mine'
error: denied: capability does not grant `urn:cap:ledger:delete:*` (declared by `urn:iki:ledger:item:1`)
error: denied: this capability does not hold `urn:cap:ledger:write:acme`. A ledger grant names exactly one ledger — holding a grant over another ledger satisfies the declared family `urn:cap:ledger:write:*` but not this resource, which is the point of naming them
```

The first is the family: this grant holds no ledger's delete token at all, so the door refuses it
before dispatch. The second is subtler, and it is why a ledger's name is in its IRI rather than an
argument: the client holds the *write* family (for `default`), so the door lets the request
through, and the ledger itself refuses it, because the token names a different ledger.

A certificate the server was never told to trust gets nothing. Mint one with no grant, after the
server started. The hint it prints names the grant's shape rather than a grant, because a
command copied from it could narrow a client's existing authority (it printed a fixed
`--ledger default=write` until gonk PR 106, ledger #918):

<!-- transcript: run -->
```console
$ ikigai-gonk client add stranger --port 1070 --config-home "$PWD/.config/ikigai"
client `stranger`  …/.config/ikigai/gonk/quic/clients/stranger
  fingerprint  …
  principal    urn:iki:gonk:client:…
  NOT enrolled — a trusted certificate with no grant is refused. Enrol it:
    ikigai-gonk client add stranger --ledger <ledger>=<read|write|delete|purge>
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <the moved directory>
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/stranger -c 'source urn:iki:ledger:items'
error: unavailable: quic transport: …
```

Refused twice over: the running server does not trust the certificate (it was not there at
startup), and if it did, a trusted certificate with no grant is refused anyway. What the client is
told depends on timing, which is why the page prints `…` there: either that the TLS handshake
failed because its certificate `does not match the pinned certificate`, or, if the connection is
torn down first, only `read error: connection lost`. ⚠ The second is all an *honest* client with a
stale bundle may hear, so when a connection is lost straight away, check the bundle against the
server's `clients.json` and restart the server if the certificate was enrolled after it started.

## What the server saw

<!-- transcript: run -->
```console
$ grep -E 'quic client| door=quic ' gonk.log
ikigai-gonk: quic client … → grant "laptop"
… gonk:Access urn:iki:ledger:append door=quic verb=sink outcome=ok bytes=… dur=… principal=urn:iki:gonk:client:… q=-
… gonk:Access urn:repo:notes:tree door=quic verb=source outcome=ok bytes=… dur=… principal=urn:iki:gonk:client:… q=-
… gonk:Access urn:repo:notes:file:README.md door=quic verb=source outcome=ok bytes=… dur=… principal=urn:iki:gonk:client:… q=-
ikigai-gonk: quic client … → grant "laptop"
… gonk:Access urn:iki:ledger:acme:append door=quic verb=sink outcome=denied bytes=… dur=… principal=urn:iki:gonk:client:… q=-
```

One connection line per session naming the grant, and one access line per request the door
dispatched, each naming the client — including the ledger's own refusal, `outcome=denied`. The
delete, refused at the door before dispatch, wrote no line, and the stranger, refused in the
handshake, wrote nothing at all.

## Writing the graph raw

The grant holds the store's write token for the `default` ledger's graph, because the ledger
makes its own writes under the caller's capability ([Whose ledger](../ledger/your-own.md#whose-ledger)).
That token is the ledger's to use. It is not authority to write the graph's triples directly,
through the store, which is how an `author` naming someone else would get past the ledger's own
check; so a network door refuses that raw write, and says what to do instead:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir laptop/gonk -c 'sink urn:iki:store:graph-update graph=urn:iki:ledger:graph:default INSERT DATA { GRAPH <urn:iki:ledger:graph:default> { <urn:example:note> <urn:example:says> "written raw" } }'
error: denied: a raw store write to the ledger graph <urn:iki:ledger:graph:default> is refused at this door. A ledger grant carries that graph's store token for the ledger's OWN writes, which it issues itself; it is not authority to write the graph's triples directly — that is how an `author` naming someone else would get in. Write through the ledger's endpoints (`urn:iki:ledger:*`). Raw writes are the owner's, at root over the socket, or an identity's whose grant an operator minted with `--ledger-graph <ledger>` (`urn:cap:gonk:raw-write:graph:urn:iki:ledger:graph:default`)
```

An identity that really does need to write raw (a repair or migration tool on another machine)
is given it by name, with `--ledger-graph`, which adds one more token. The grant `laptop` already
exists, so changing it takes `--force`, which names what it changes:

<!-- transcript: run -->
```console
$ ikigai-gonk client add laptop --ledger default=write --browse read --root notes --ledger-graph default --force --port 1070 --config-home "$PWD/.config/ikigai"
client `laptop`  …/.config/ikigai/gonk/quic/clients/laptop
  fingerprint  …
  principal    urn:iki:gonk:client:…
  enrolled     grant `laptop` (7 scopes) in …/.config/ikigai/gonk/grants.json
  REPLACED     grant `laptop` (--force) — every identity enrolled under it holds the new list from its next request or connection:
    adds     urn:cap:gonk:raw-write:graph:urn:iki:ledger:graph:default
  restart ikigai-gonk: trusted certificates are read at startup
  the client's identity is unchanged — a client already holding this bundle needs nothing new; it connects as before:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <its bundle>
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir laptop/gonk -c 'sink urn:iki:store:graph-update graph=urn:iki:ledger:graph:default INSERT DATA { GRAPH <urn:iki:ledger:graph:default> { <urn:example:note> <urn:example:says> "written raw" } }'
updated <urn:iki:ledger:graph:default>: +1 -0 quads
[uncacheable]
```

The bundle in `laptop/` was not touched, and the same certificate now holds the new grant, with
no restart: `--force` replaces a client's **grant** and never its key pair. (`ikigai-gonk grants
--ledger-graph default` prints the tokens it adds, for writing a grant by hand.) The raw grant
covers that ledger's graph and not its graveyard of deleted items, which stays the owner's, at
root over the socket.

## A client's lifecycle

`client list` is every client the server knows: each bundle and each enrolled fingerprint, its
grant, and whether a connection from it would be admitted.

<!-- transcript: run -->
```console
$ ikigai-gonk client list --config-home "$PWD/.config/ikigai"
laptop           …  grant laptop       trusted and enrolled
stranger         …  grant -            trusted, NOT enrolled (refused at connection)
```

A key pair that may have leaked is replaced with `--rotate`: a new identity for the same name,
under the same authority. With no scope flags the old certificate's enrollment moves to the new
one, the same grant, and `grants.json` is not touched (with scope flags the grant is rewritten
as for any `client add`, and refused without `--force` if it changes). Until gonk PR 106 (ledger
#918) a rotation without flags left the new certificate enrolled under nothing, so the client
was refused at its next connection:

<!-- transcript: run -->
```console
$ ikigai-gonk client add laptop --rotate --port 1070 --config-home "$PWD/.config/ikigai"
client `laptop`  …/.config/ikigai/gonk/quic/clients/laptop
  fingerprint  …
  principal    urn:iki:gonk:client:…
  enrolled     under grant `laptop`, unchanged — the old certificate's … moved to this one
  ROTATED      the old certificate … was unenrolled — a client still holding the old bundle is refused from its next connection; give it this one
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <the moved directory>
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir laptop/gonk -c 'source urn:iki:ledger:items'
error: unavailable: quic transport: …
```

The old bundle is refused at once: admission is decided per connection, from `clients.json`.
The new one is not trusted until gonk restarts, because the certificates it trusts are read at
startup; so a rotation is a restart and a copy of the new bundle to the client.

A client that should go away entirely is removed, bundle and enrollment:

<!-- transcript: run -->
```console
$ ikigai-gonk client remove stranger --config-home "$PWD/.config/ikigai"
removed  …/.config/ikigai/gonk/quic/clients/stranger
  (its certificate was not enrolled)
  grants.json is unchanged: a grant name may be shared by other certificates and passkeys. Restart ikigai-gonk to stop trusting the certificate at the TLS handshake
$ ikigai-gonk client list --config-home "$PWD/.config/ikigai"
laptop           …  grant laptop       trusted and enrolled
```

`grants.json` is left alone because a grant name may be shared, by other certificates and by
passkeys. `client remove --fingerprint <fp>` removes an enrollment that has no bundle, which is
what `client list` shows as `enrolled, NO bundle`.

## On a second machine

<!-- transcript: manual — a second machine, which a check on one runner does not have -->
```console
$ scp -r server:.config/ikigai/gonk/quic/clients/laptop ~/gonk-laptop
$ ikigai --connect quic://server.local:1060 --cert-dir ~/gonk-laptop -c 'source urn:iki:ledger:next'
```

The same, with the bundle moved rather than copied, the server reachable on UDP 1060 (a gonk on
the default port opens its QUIC door on UDP 1060, every interface), and the client's `ikigai`
built with `--features quic`.
A client that should reach gonk all the time, rather than per command, puts the same two things in
its own config home as a mount, `mount = "prefer urn:iki:ledger:=quic://server.local:1060
/home/you/gonk-laptop"`, and every `urn:iki:ledger:*` it asks then goes to gonk.
