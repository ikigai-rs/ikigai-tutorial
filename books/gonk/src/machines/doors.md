# The three doors, and who is asking

Everything so far reached gonk from the same machine. This part is about reaching it from others,
and it starts with what every request has to answer first: **who is asking, and what may they
do?** A kernel answers that with a capability, and gonk's doors are where a request is given one.
There are three, and each authenticates something different.

| door | reached by | authenticates | the capability it grants |
| --- | --- | --- | --- |
| HTTP | a browser, `curl`, on **loopback only** | that the request is from this machine and not cross-site (the `Host`, and for a write the `Origin`); a passkey session if there is one | the anonymous grant (read and write of the ledgers `gonk.http.ledger` names, `default` unless told otherwise), plus a signed-in passkey's grant |
| socket | another process of **the same user** (`ikigai --mount`) | the socket file's owner-only mode, and the peer's user id | root: the owner can read the store's files anyway |
| QUIC | another **machine** (`ikigai --connect quic://…`) | a client certificate, pinned by fingerprint, over mutual TLS — no certificate authority | the grant its fingerprint is enrolled under |

This page opens all three on one scratch gonk and sends a request through each, then reads gonk's
access log to see what gonk knew about each caller. The [next chapter](quic-client.md) is the QUIC
door in depth.

## Opening the QUIC door

The QUIC door does not open until a certificate is enrolled: a door that every client is refused at
is not worth a listening port. So the first command enrolls one, under the grant name `laptop`, as
a writer of the `default` ledger:

<!-- transcript: run -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk client add laptop --ledger default=write
client `laptop`  …/.config/ikigai/gonk/quic/clients/laptop
  fingerprint  …
  enrolled     grant `laptop` (4 scopes) in …/.config/ikigai/gonk/grants.json
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1060 --cert-dir <the moved directory>
```

Then start gonk with the QUIC door on loopback, and with its standard error — where the access
lines go — copied to a file this page can read back:

<!-- transcript: serve -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk --port 1070 --quic-bind 127.0.0.1:1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s)
…
  socket  …/.ikigai/gonk.sock — owner only
  quic    udp 127.0.0.1:1070 — 1 trusted certificate(s), 1 enrolled
  log     one `gonk:Access` line per request at each door, on stderr (`gonk.log.access = false` turns it off)
…
```

⚠ `--quic-bind` is not optional here. `--port` moves only the HTTP door; with nothing named, the
QUIC door binds UDP port 1060 on every interface, which is a real gonk's, and a scratch gonk that
collides with it exits. Naming a bind also changes its meaning from "if you can" to "must": gonk
refuses to start with a bind named and no certificate enrolled, rather than run with the door
silently shut. TCP 1070 and UDP 1070 are different ports, so the two doors can share the number.

## One request through each door

<!-- transcript: run -->
```console
$ curl -s http://127.0.0.1:1070/iki/ledger/items
no items match
$ curl -s -X POST --data-binary 'From the HTTP door' http://127.0.0.1:1070/iki/ledger/append
#1 urn:iki:ledger:default:item:…
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append From the socket'
#2 urn:iki:ledger:default:item:…

[uncacheable]
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/laptop -c 'sink urn:iki:ledger:append From QUIC' -c 'source urn:iki:ledger:items'
#3 urn:iki:ledger:default:item:…

[uncacheable]
   #3  open    p-  From QUIC
   #2  open    p-  From the socket
   #1  open    p-  From the HTTP door

3 item(s)

[computed]
— batch: 2 commands · 1 computed · 1 uncacheable
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
… gonk:Access urn:iki:ledger:append door=quic verb=sink outcome=ok bytes=… dur=… principal=- q=-
… gonk:Access urn:iki:ledger:items door=quic verb=source outcome=ok bytes=… dur=… principal=- q=-
$ grep 'quic client' gonk.log
ikigai-gonk: quic client … → grant "laptop"
```

`door=` is which door, and `principal=` is who the door could say was asking:

- **HTTP**: `anon` on an anonymous write, a passkey's IRI on a signed-in one, and `-` on any read
  (a read is not attributed, on any door).
- **socket**: `owner`, always — there is exactly one user it lets in.
- **QUIC**: `-`, even on a write. The door *does* know: the connection line below the access lines
  names the certificate's fingerprint (its first sixteen hex digits) and the grant it mapped to.
  What the access line records today is only that the request came in over QUIC; to say *which*
  client wrote, read the connection line, or file under a named author.

`outcome=` is what the resource answered (`ok`, `denied`, `not-found`, …). One kind of refusal
writes **no** line: a request the door's own kernel refuses before dispatching it, because the
caller's capability does not even cover the family the name belongs to. That refusal is still
returned to the caller, with the token it wanted:

<!-- transcript: run -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/laptop -c 'source urn:iki:store:select'
error: denied: capability does not grant `urn:cap:store:read` (declared by `urn:iki:store:select`)
```

The `laptop` grant holds a ledger's tokens, and the store's broad query door wants the broad read
grant, which no QUIC grant may carry (gonk refuses a session whose grant holds it, rather than
serve the whole dataset to a certificate). So the request ends
at the door, and the log stays a record of what was *served*.
