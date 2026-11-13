# The HTTP door and its lifecycle

Prerequisite evidence for the planned HTTP and unattended chapters, replayed against the
Operations Book's pin by `scripts/test-transcripts.sh` and `scripts/test-operations-macos.sh`.
The binary prints the port it actually bound on loopback, so the existing `port` normalizer
suffices. A shell writes its PID before replacing itself with the server; this is the PID the
shutdown command must stop. Restart ends the old port binding and learns the replacement's.

<!-- transcripts: normalize workdir, home -->

<!-- transcript: serve; normalize port -->
```console
$ sh -c 'echo $$ > server.pid; exec ikigai --config-home "$PWD/cfg" serve --http 0'
ikigai: mount   none composed — no `mount` lines in the config home
ikigai: watching ~/.ikigai/workspace (a file changed on disk is served fresh)
ikigai: serving HTTP on 127.0.0.1:51234  (public cap; mechanical routing; CORS closed; no proxy trust; rate limits per client: urn:booking:submit 10/3600s, urn:contact:submit 10/3600s, urn:passkey:challenge 30/120s; terminate TLS at your proxy)  (Ctrl-C to stop)
```

<!-- transcript: run; exit 0; normalize port -->
```console
$ curl -fsS -w '\n%{http_code} %{url_effective}\n' 'http://127.0.0.1:51234/iki/fn/toUpper?in=over%20http'
OVER HTTP
200 http://127.0.0.1:51234/iki/fn/toUpper?in=over%20http
```

<!-- transcript: restart; normalize port -->
```console
$ sh -c 'echo $$ > server.pid; exec ikigai --config-home "$PWD/cfg" serve --http 0'
ikigai: mount   none composed — no `mount` lines in the config home
ikigai: watching ~/.ikigai/workspace (a file changed on disk is served fresh)
ikigai: serving HTTP on 127.0.0.1:51234  (public cap; mechanical routing; CORS closed; no proxy trust; rate limits per client: urn:booking:submit 10/3600s, urn:contact:submit 10/3600s, urn:passkey:challenge 30/120s; terminate TLS at your proxy)  (Ctrl-C to stop)
```

<!-- transcript: run; exit 0 -->
```console
$ curl -fsS 'http://127.0.0.1:51234/iki/fn/toUpper?in=after%20restart'
AFTER RESTART
```

Remember this run's address before shutdown; the recorded port is rewritten while the server
is still tracked.

<!-- transcript: run; exit 0 -->
```console
$ printf '%s\n' 'http://127.0.0.1:51234/' > address.txt
```

<!-- transcript: stop; exit 0 -->
```console
$ kill "$(cat server.pid)"
```

Port bindings end at shutdown. The saved address now refuses a connection, and the PID is gone.

<!-- transcript: run; exit 0 -->
```console
$ if kill -0 "$(cat server.pid)" 2>/dev/null; then exit 1; fi
```

<!-- transcript: run; exit 7 -->
```console
$ curl -fsS --connect-timeout 2 "$(cat address.txt)" >/dev/null 2>&1
```
