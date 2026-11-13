# The serving fixture, over QUIC

Not a page of any book: the transcript harness's own check that its serving fixture works against
the real binary, over the transport a book's serving chapters use most. `scripts/test-transcripts.sh`
runs it with Running ikigai's pinned `ikigai-cli` (built with `quic`), `--strict`, like a book.

It proves three things a unit test cannot: a server started on `quic://127.0.0.1:0` binds a port
of its own choosing and prints it, and the next command, written with the port this page recorded,
reaches it (`port` binds the recorded port to the run's); certificates made by `cert generate` and
`cert add-client` are what the server says it trusts, the short fingerprint in its banner bound to
the full one `add-client` printed (`fingerprint` binds both, and refuses a page that disagrees with
itself); and the server is stopped by its process group when the page is done.

<!-- transcripts: normalize workdir, home -->

<!-- transcript: run -->
```console
$ ikigai --config-home "$PWD/cfg" cert generate
wrote server.{crt,key} and client.{crt,key} to $PWD/cfg/quic
to attach a client on another machine, copy client.crt, client.key, and server.crt there.
```

<!-- transcript: run; normalize fingerprint -->
```console
$ ikigai --config-home "$PWD/cfg" cert add-client alice
wrote a new client identity to $PWD/cfg/quic/clients/alice.crt
a running server trusts it at its next connection (it reads clients/*.crt at every handshake); to use it, copy alice.crt, alice.key, and server.crt to the client machine.
fingerprint: c1c8462d57dd0c178e8fec900d57b35992ba2cc625db4c6928a1a92163b4cc2b
  to give it its own authority, add it to the `clients` map in $PWD/cfg/clients.json: "c1c8462d57dd0c178e8fec900d57b35992ba2cc625db4c6928a1a92163b4cc2b": {"grant": "<grant>", "label": "alice"}
```

<!-- transcript: serve; normalize port, fingerprint -->
```console
$ ikigai --config-home "$PWD/cfg" serve quic://127.0.0.1:0
ikigai: mount   none composed — no `mount` lines in the config home
ikigai: watching ~/.ikigai/workspace (a file changed on disk is served fresh)
ikigai: client  base   bc42e0a9dff48e21  $PWD/cfg/quic/client.crt
ikigai: client  alice  c1c8462d57dd0c17  $PWD/cfg/quic/clients/alice.crt
ikigai: serving on quic://127.0.0.1:55682  (per-client workspaces; surface: host + fs; connections re-minted every 600s)  (Ctrl-C to stop)
```

<!-- transcript: run; normalize port -->
```console
$ ikigai --config-home "$PWD/cfg" --connect quic://127.0.0.1:55682 -c 'source urn:iki:fn:toUpper in="over quic"'
OVER QUIC
[computed]
```

The server must start in the enrollment posture for connection-time revocation to apply. A
server started without clients.json keeps its workspace posture; creating the file alone does
not switch that posture. Restart with an enrolled identity, then edit the enrollment file.
The fingerprint binding survives restart, while the port binding is learned again.

<!-- transcript: file cfg/grants.json -->
```json
{"alice": ["urn:cap:fs:read:*"]}
```

<!-- transcript: file cfg/clients.json -->
```json
{"clients": {"c1c8462d57dd0c178e8fec900d57b35992ba2cc625db4c6928a1a92163b4cc2b": {"grant": "alice", "label": "alice"}}}
```

<!-- transcript: restart; normalize port, fingerprint -->
```console
$ ikigai --config-home "$PWD/cfg" serve quic://127.0.0.1:0
ikigai: mount   none composed — no `mount` lines in the config home
ikigai: watching ~/.ikigai/workspace (a file changed on disk is served fresh)
ikigai: client  base   bc42e0a9dff48e21  $PWD/cfg/quic/client.crt
ikigai: client  alice  c1c8462d57dd0c17  $PWD/cfg/quic/clients/alice.crt
ikigai: serving on quic://127.0.0.1:55682  (per-identity grants: 1 enrolled; surface: host + fs; connections re-minted every 600s)  (Ctrl-C to stop)
```

<!-- transcript: run; exit 0 -->
```console
$ ikigai --config-home "$PWD/cfg" --client-cert "$PWD/cfg/quic/clients/alice.crt" --client-key "$PWD/cfg/quic/clients/alice.key" --connect quic://127.0.0.1:55682 -c 'source urn:iki:fn:toUpper in="enrolled"'
ENROLLED
[computed]
```

Each CLI invocation opens a fresh connection. Removing Alice's entry refuses the next one,
without restarting the server; restoring it admits her again.

<!-- transcript: file cfg/clients.json -->
```json
{"clients": {}}
```

<!-- transcript: run; exit 1 -->
```console
$ ikigai --config-home "$PWD/cfg" --client-cert "$PWD/cfg/quic/clients/alice.crt" --client-key "$PWD/cfg/quic/clients/alice.key" --connect quic://127.0.0.1:55682 -c 'source urn:iki:fn:toUpper in="revoked"'
error: unavailable: quic transport: read error: connection lost
[contract unavailable for urn:iki:fn:toUpper: unavailable: quic transport: read error: connection lost — arguments were routed without it (a piped or positional value went to `in`; an undeclared key=value stayed text)]
```

<!-- transcript: file cfg/clients.json -->
```json
{"clients": {"c1c8462d57dd0c178e8fec900d57b35992ba2cc625db4c6928a1a92163b4cc2b": {"grant": "alice", "label": "alice"}}}
```

<!-- transcript: run; exit 0 -->
```console
$ ikigai --config-home "$PWD/cfg" --client-cert "$PWD/cfg/quic/clients/alice.crt" --client-key "$PWD/cfg/quic/clients/alice.key" --connect quic://127.0.0.1:55682 -c 'source urn:iki:fn:toUpper in="restored"'
RESTORED
[computed]
```
