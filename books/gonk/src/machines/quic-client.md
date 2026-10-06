# A QUIC client, and the grant it runs under

The [previous chapter](doors.md) opened gonk's QUIC door and sent one request through it. This one
sets up a client the way you would for a second machine: an identity minted on the server, a grant
that says exactly what that identity may do, the identity moved to the client, and the server
refusing everything outside the grant — including a certificate it was never told to trust.

Both "machines" here are directories on one machine, each a home of its own: the page's directory
is the server's home, and `laptop/` is the client's. Everything below works the same with the
second one somewhere else, with `127.0.0.1` replaced by the server's address.

## A repository, and an identity with a grant

The client will read a repository as well as the ledger, so the server browses one:

<!-- transcript: run -->
```console
$ git init -q -b main notes
$ printf '# Notes\n' > notes/README.md
```

<!-- transcript: file .config/ikigai/config.toml -->
```toml
gonk.browse.root = "notes=~/notes"
```

`client add` mints a certificate and its key, writes them as a **bundle** with the server's own
certificate, and enrolls the client's fingerprint under a grant. The grant is built from roles,
never typed as tokens: `--ledger default=write` is the ledger's writer, `--browse read --root
notes` reads that one repository.

<!-- transcript: run -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk client add laptop --ledger default=write --browse read --root notes
client `laptop`  …/.config/ikigai/gonk/quic/clients/laptop
  fingerprint  …
  enrolled     grant `laptop` (6 scopes) in …/.config/ikigai/gonk/grants.json
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1060 --cert-dir <the moved directory>
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
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk --port 1070 --quic-bind 127.0.0.1:1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  quic    udp 127.0.0.1:1070 — 1 trusted certificate(s), 1 enrolled
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

and connect from that home, naming the bundle with `--cert-dir`:

<!-- transcript: run -->
```console
$ env HOME="$PWD/laptop" XDG_CONFIG_HOME="$PWD/laptop/.config" ikigai --connect quic://127.0.0.1:1070 --cert-dir laptop/gonk -c 'sink urn:iki:ledger:append Read the notes' -c 'source urn:repo:notes:tree' -c 'source urn:repo:notes:file:README.md'
#1 urn:iki:ledger:default:item:…

[uncacheable]
.git	dir	-
README.md	file	8
[computed]
# Notes

[computed]
— batch: 3 commands · 2 computed · 1 uncacheable
```

The client pinned the server by the `server.crt` in its bundle, and the server pinned the client
by its fingerprint. There is no certificate authority anywhere: each side knows exactly one key
for the other, and that is the whole of the trust.

## Outside the grant

Everything this client is not granted is refused, by the resource that was asked, naming what it
wanted:

<!-- transcript: run -->
```console
$ env HOME="$PWD/laptop" XDG_CONFIG_HOME="$PWD/laptop/.config" ikigai --connect quic://127.0.0.1:1070 --cert-dir laptop/gonk -c 'delete urn:iki:ledger:item:1' -c 'sink urn:iki:ledger:acme:append Not mine'
error: denied: capability does not grant `urn:cap:ledger:delete:*` (declared by `urn:iki:ledger:item:1`)
error: denied: this capability does not hold `urn:cap:ledger:write:acme`. A ledger grant names exactly one ledger — holding a grant over another ledger satisfies the declared family `urn:cap:ledger:write:*` but not this resource, which is the point of naming them
```

The first is the family: this grant holds no ledger's delete token at all, so the door refuses it
before dispatch. The second is subtler, and it is why a ledger's name is in its IRI rather than an
argument: the client holds the *write* family (for `default`), so the door lets the request
through, and the ledger itself refuses it, because the token names a different ledger.

A certificate the server was never told to trust gets nothing. Mint one with no grant, after the
server started:

<!-- transcript: run -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk client add stranger
client `stranger`  …/.config/ikigai/gonk/quic/clients/stranger
  fingerprint  …
  NOT enrolled — a trusted certificate with no grant is refused. Enrol it:
    ikigai-gonk client add stranger --ledger default=write
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1060 --cert-dir <the moved directory>
$ env HOME="$PWD/laptop" XDG_CONFIG_HOME="$PWD/laptop/.config" ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/stranger -c 'source urn:iki:ledger:items'
error: unavailable: quic transport: read error: connection lost
```

Refused twice over: the running server does not trust the certificate (it was not there at
startup), and if it did, a trusted certificate with no grant is refused anyway. ⚠ What the client is
told is only that the connection was lost. That is safe — a stranger learns nothing about why — but
it is also all an *honest* client with a stale bundle is told, so when a connection is lost
straight away, check the bundle against the server's `clients.json` first.

## What the server saw

<!-- transcript: run -->
```console
$ grep -E 'quic client| door=quic ' gonk.log
ikigai-gonk: quic client … → grant "laptop"
… gonk:Access urn:iki:ledger:append door=quic verb=sink outcome=ok bytes=… dur=… principal=- q=-
… gonk:Access urn:repo:notes:tree door=quic verb=source outcome=ok bytes=… dur=… principal=- q=-
… gonk:Access urn:repo:notes:file:README.md door=quic verb=source outcome=ok bytes=… dur=… principal=- q=-
ikigai-gonk: quic client … → grant "laptop"
… gonk:Access urn:iki:ledger:acme:append door=quic verb=sink outcome=denied bytes=… dur=… principal=- q=-
```

One connection line per session naming the grant, and one access line per request the door
dispatched — including the ledger's own refusal, `outcome=denied`. The delete, refused at the door
before dispatch, wrote no line, and the stranger, refused in the handshake, wrote nothing at all.

## On a second machine

<!-- transcript: manual — a second machine, which a check on one runner does not have -->
```console
$ scp -r server:.config/ikigai/gonk/quic/clients/laptop ~/gonk-laptop
$ ikigai --connect quic://server.local:1060 --cert-dir ~/gonk-laptop -c 'source urn:iki:ledger:next'
```

The same, with the bundle moved rather than copied, the server reachable on UDP 1060 (gonk's
default `--quic-bind` is every interface), and the client's `ikigai` built with `--features quic`.
A client that should reach gonk all the time, rather than per command, puts the same two things in
its own config home as a mount, `mount = "prefer urn:iki:ledger:=quic://server.local:1060
/home/you/gonk-laptop"`, and every `urn:iki:ledger:*` it asks then goes to gonk.
