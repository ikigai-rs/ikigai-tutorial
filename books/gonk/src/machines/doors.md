# The three doors, and who is asking

Everything so far reached gonk from the same machine. This part is about reaching it from others,
and it starts with what every request has to answer first: **who is asking, and what may they
do?** A kernel answers that with a capability, and gonk's doors are where a request is given one.
There are three, and each authenticates something different.

| door | reached by | authenticates | the capability it grants |
| --- | --- | --- | --- |
| HTTP | a browser, `curl`, on **loopback only** | that the request is from this machine and not cross-site (the `Host`; for a write the `Origin`; for a read from another page, that it navigated here rather than loading this as an image, script, frame or `fetch`, and then only the read half of the grant); a passkey session if there is one | the anonymous grant (read and write of the ledgers `gonk.http.ledger` names, `default` unless told otherwise), plus a signed-in passkey's grant |
| socket | another process of **the same user** (`ikigai --mount`) | the socket file's owner-only mode, and the peer's user id | root (the owner can read the store's files anyway), or what an owner process narrowed itself to with `cap seal` |
| QUIC | another **machine** (`ikigai --connect quic://…`) | a client certificate, pinned by fingerprint, over mutual TLS — no certificate authority | the grant its fingerprint is enrolled under, and a name for the client, `urn:iki:host:gonk:client:<fingerprint>` |

This page opens all three on one scratch gonk and sends a request through each, then reads gonk's
access log to see what gonk knew about each caller. The [next chapter](quic-client.md) is the QUIC
door in depth.

## Opening the QUIC door

The QUIC door does not open until a certificate is enrolled: a door that every client is refused at
is not worth a listening port. So the first command enrolls one, under the grant name `laptop`, as
a writer of the `default` ledger. Like the server below, it is told which config home to write
into, and `--port` says which port the server will listen on, so the `--connect` line it prints
is right:

<!-- transcript: run -->
```console
$ ikigai-gonk client add laptop --ledger default=write --port 1070 --config-home "$PWD/.config/ikigai"
client `laptop`  …/.config/ikigai/gonk/quic/clients/laptop
  fingerprint  …
  principal    urn:iki:host:gonk:client:…
  enrolled     grant `laptop` (4 scopes) in …/.config/ikigai/gonk/grants.json
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <the moved directory>
```

`principal` is the name gonk will give this client on every request it makes: the certificate's
fingerprint, as an IRI. It is a name, not authority; what the client may do is the grant. Then
start gonk, with its standard error — where the access lines go — copied to a file this page can
read back:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s); anonymous SPARQL budget 1000 ms
…
  socket  …/.ikigai/gonk.sock — owner only
  quic    udp 0.0.0.0:1070 — 1 trusted certificate(s), 1 enrolled
  log     one `gonk:Access` line per request at each door, on stderr; the Queue badge's poll only when it fails, takes a second, or is the 60th (`gonk.log.access = false` turns it off)
…
```

`--port` moves both network doors: HTTP on TCP 1070, loopback only, and QUIC on UDP 1070, every
interface, because a QUIC door is for other machines. TCP 1070 and UDP 1070 are different ports,
so the two doors share the number. (Through gonk's audit round 4 the QUIC door stayed on UDP 1060,
its default then, whatever `--port` said, so a scratch gonk with an enrolled client collided with
a real one.)
`--quic-bind IP:PORT` still names a bind outright, and naming one changes its meaning from "if
you can" to "must": gonk refuses to start with a bind named and no certificate enrolled, rather
than run with the door silently shut.

## One request through each door

<!-- transcript: run -->
```console
$ curl -s http://127.0.0.1:1070/iki/ledger/items
no items match
$ curl -s -X POST --data-binary 'From the HTTP door' http://127.0.0.1:1070/iki/ledger/append
#1 urn:iki:ledger:default:item:…
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append From the socket'
#2 urn:iki:ledger:default:item:…

[uncacheable]
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/laptop -c 'sink urn:iki:ledger:append From QUIC' -c 'source urn:iki:ledger:items'
#3 urn:iki:ledger:default:item:…

[uncacheable]
   #3  open    p-  From QUIC
   #2  open    p-  From the socket
   #1  open    p-  From the HTTP door

3 item(s)

[uncacheable]
— batch: 2 commands · 2 uncacheable
```

Three doors, one ledger. The socket and QUIC clients are the same program, `ikigai`, in its two
remote forms: `--mount` composes gonk's resources into a local kernel under a prefix of your
choosing (here `urn:gk:`), and `--connect` sends every command to the server as it is typed, so
the names are gonk's own.

## What gonk knew

Every request a door dispatches is one line in the access log, on standard error, in a fixed
`key=value` shape made for `grep`:

<!-- transcript: run -->
```console
$ grep ' gonk:Access ' gonk.log
… gonk:Access urn:iki:ledger:items door=http verb=source outcome=ok bytes=… dur=… principal=- q=-
… gonk:Access urn:iki:ledger:append door=http verb=sink outcome=ok bytes=… dur=… principal=anon q=-
… gonk:Access urn:iki:ledger:append door=socket verb=sink outcome=ok bytes=… dur=… principal=owner q=-
… gonk:Access urn:iki:ledger:append door=quic verb=sink outcome=ok bytes=… dur=… principal=urn:iki:host:gonk:client:… q=-
… gonk:Access urn:iki:ledger:items door=quic verb=source outcome=ok bytes=… dur=… principal=urn:iki:host:gonk:client:… q=-
$ grep 'quic client' gonk.log
ikigai-gonk: quic client … → grant "laptop"
```

`door=` is which door, and `principal=` is who the door could say was asking:

- **HTTP**: `anon` on an anonymous write and `-` on an anonymous read; a signed-in passkey's IRI
  on **every** request, reads included. That last part is new in the gonk this book pins (gonk
  PR 122): before it the HTTP library told the door who a write was from and never a read, so a
  signed-in read was logged `-` too.
- **socket**: `owner`, always — there is exactly one user it lets in.
- **QUIC**: the client's name, `urn:iki:host:gonk:client:<fingerprint>`, on **every** request, reads
  included. The certificate authenticated the connection, so the door knows which client is
  asking whatever it asks. The connection line below the access lines says the same thing in
  short (the fingerprint's first sixteen hex digits) with the grant it mapped to.

The QUIC name is not only in the log. The door mints it into the request's **capability**, as
core's principal scope (`urn:cap:principal:<iri>`, `ikigai-core` 0.1.93), and that is where gonk
reads it. A capability carries at most one principal, and when the door narrows a capability the
client carried to the grant it holds, the result keeps the door's name and drops any the client
sent, so it cannot be forged from the client side, and a client that narrows its own capability
(an agent's grant carried with the request) is named all the same. (`ikigai-quic` also still
stamps the name on each request as an argument called `principal`, after removing any the client
sent; until gonk PR 122 that argument was where gonk read it.) A write that names no `author`
is filed with this name as its author, and a write that names a *different* principal as its
author is refused:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/laptop -c 'source urn:iki:ledger:item:3' -c 'sink urn:iki:ledger:append author=urn:iki:gonk:passkey:ada Not mine to sign'
   #3  open    p-  From QUIC
  iri:      urn:iki:ledger:default:item:…
  filed:    … by urn:iki:host:gonk:client:…
  updated:  …
[uncacheable]
error: denied: `author` names a principal (urn:iki:gonk:passkey:ada), and only the door names one: a write may name its own principal or a plain-text author, never another identity. This request is from urn:iki:host:gonk:client:….
— batch: 2 commands · uncacheable
```

A plain-text author (`author=agent-7`, as in [Over the socket](../agent/socket.md)) is still
kept: it renders as text, and claims nothing a door could check.

**The client's name has a second, older spelling, and that is where it matters.** The name
follows every host's convention, `urn:iki:host:{name}:client:<fingerprint>`, with this host's
name, `gonk`. Before gonk PR 126 (ledger #1183, 2026-10-10) gonk minted
`urn:iki:gonk:client:<fingerprint>`, and that spelling is a **permanent alias**: never minted
again, and never rewritten where it is stored, so an item filed, or an access line written, by a
QUIC client before the change still says it. Read an older ledger or log, and match both. The door
treats the two as one actor: a client may name *itself* as `author` in either spelling, the old
spelling of *another* machine's fingerprint is refused like the new one, and the item page shows
both spellings of one fingerprint as one actor, in the current form. Enrollment is unchanged:
`clients.json` is keyed by fingerprint, so an enrolled certificate needs nothing redone.

`outcome=` is what the resource answered (`ok`, `denied`, `not-found`, …). One kind of refusal
writes **no** line: a request the door's own kernel refuses before dispatching it, because the
caller's capability does not even cover the family the name belongs to. That refusal is still
returned to the caller, with the token it wanted:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/laptop -c 'source urn:iki:store:select'
error: denied: capability does not grant `urn:cap:store:read` (declared by `urn:iki:store:select`)
```

The `laptop` grant holds a ledger's tokens, and the store's broad query door wants the broad read
grant, which no QUIC grant may carry (gonk refuses a session whose grant holds it, rather than
serve the whole dataset to a certificate). So the request ends
at the door, and the log stays a record of what was *served*.

The grant's narrower store tokens are there for the ledger, which writes under the caller's
capability; they do not let the client write the ledger's graph through the store instead. That
raw write is refused at this door and at HTTP unless the grant was minted with `--ledger-graph`,
which [the next chapter](quic-client.md#writing-the-graph-raw) shows.
