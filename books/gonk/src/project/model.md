# A model of your own

[A query from a sentence](sentence.md), the next chapter, needs a language model. You already have
published queries and a plan from [A workflow of your own](script.md). Those need no model, and
neither does anything else in this part except review. This chapter sets one up, by whichever
route suits you, and each route ends in the same check: **`urn:llm:ask`
answers**.

## How gonk reaches a model

gonk links no model client. It **mounts** `urn:llm:` from a second process, an `ikigai serve` holding
the model's configuration, so a ledger server carries no outbound HTTP client of its own
([Explain, with a mounted model](../browse/explain.md)).

```text
  gonk ── gonk.mount ──▶ ikigai serve ── llm.json ──▶ a model server
          (urn:llm:)       (a socket or QUIC)            (Ollama, say)
```

So there are two files to write: `llm.json` for `ikigai`, saying where the model is, and one
`gonk.mount` line for gonk, saying where `ikigai` is. Everything here is checked by this book except
what needs a real model, and those commands are marked. What you see from your own model will differ.

| route | use it when | needs |
| --- | --- | --- |
| [Ollama on this machine](#ollama-on-this-machine) | the default: free, private, no account | a few GB of memory |
| [Another OpenAI-compatible server](#another-openai-compatible-server) | you already run one, or want Apple-silicon speed | that server |
| [A peer on another machine](#a-peer-on-another-machine) | a bigger machine has the GPU | two certificates |
| [A hosted API](#a-hosted-api) | you have no suitable hardware | an account, and a decision about your data |

## Ollama on this machine

Install [Ollama](https://ollama.com) and pull a model. An 8B model at 4-bit is about 5 GB, and a model
bigger than your GPU's memory spills into system memory and runs at a fraction of the speed. This book
needs nothing more specific than a model that can write a little SPARQL; a stronger one drafts better.

<!-- transcript: manual — installs software and downloads gigabytes; the check never reaches a real model -->
```console
$ curl -fsSL https://ollama.com/install.sh | sh
$ ollama pull qwen3:8b
$ ollama run qwen3:8b 'Answer with one word: what color is the sky?'
blue
```

On macOS, install the app or use `brew install ollama && brew services start ollama`.

Now tell `ikigai` about it. This page keeps the file in a scratch config home, `peer/ikigai/`; yours is
`~/.config/ikigai/llm.json`, and then the commands below need no `env XDG_CONFIG_HOME=…`. In a
new empty directory, as in [every chapter](../introduction.md#a-gonk-per-chapter):

<!-- transcript: file peer/ikigai/llm.json -->
```json
{
  "default": "coder",
  "providers": {
    "coder": { "base_url": "http://localhost:11434/v1", "model": "qwen3:8b" }
  }
}
```

`coder` is the name gonk asks for when it reviews or explains a file; `default` is what `urn:llm:ask`
uses, which is what drafting asks. One provider can be both. `base_url` is the server's
OpenAI-compatible root.

Serve it on a socket, for this user alone. `--no-config-mounts` keeps this process a leaf, so it cannot
end up mounting gonk while gonk mounts it. If the gonk from [the previous chapter](script.md) is still
running, stop it first, since a later step here takes the same port. Keep each `serve` process and
gonk running in its own terminal. Run the remaining commands in another terminal, from the same
chapter directory.

<!-- transcript: serve -->
```console
$ env XDG_CONFIG_HOME="$PWD/peer" ikigai serve "$PWD/llm.sock" --no-config-mounts 2>&1 | tee peer.log
ikigai: mount   declined (--no-config-mounts) — the config home's `mount` lines are not read
…
ikigai: serving on …/llm.sock  (Ctrl-C to stop)
```

Check that it read the file. `urn:llm:config` needs no running model server and no capability, and
never shows a key:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:m:=$PWD/llm.sock" -c 'source urn:m:llm:config'
{"default":"coder","providers":{"coder":{"base_url":"http://localhost:11434/v1","model":"qwen3:8b","api_key":null}}}
[uncacheable]
```

Then mount it in gonk, with the one line, and start gonk:

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.mount = "prefer urn:llm:=%s/llm.sock"\n' "$PWD" > .config/ikigai/config.toml
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
  nl      urn:nl:grounding, urn:nl:sparql:check — under the caller's own grant; urn:nl:sparql drafts through urn:llm:ask for a caller holding urn:cap:net:localhost (--nl draft), saving DRAFT scripts a person publishes
…
  llm     …/llm.sock — mounted, but no gonk.browse.root, so nothing here derives through it
…
```

gonk dials the peer on first use, so a model that is down never stops a ledger from serving. Ask through
gonk's own door for what the peer says:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:llm:config'
{"default":"coder","providers":{"coder":{"base_url":"http://localhost:11434/v1","model":"qwen3:8b","api_key":null}}}
[uncacheable]
```

That is everything but the model itself. The last check is the one this page cannot run. This output is
from Ollama 0.40.2 and `qwen3:8b`, by hand, on 2026-11-03:

<!-- transcript: manual — it asks a real model, which no check here reaches, and a model's words differ from run to run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:llm:coder:up'
true
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:llm:coder:model'
qwen3:8b
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:llm:ask prompt="Answer with one word: what color is the sky?"'
blue
[uncacheable]
```

**If `urn:llm:ask` answers, you are done**, and can skip to the next chapter.

⚠ Restart gonk whenever you restart the peer. gonk keeps its connection, and a request after the peer
restarted can wait minutes on the old one.

## When it does not answer

These are checked, against a peer whose server is not there. Start a second peer on another socket,
naming a port nothing listens on:

<!-- transcript: file down/ikigai/llm.json -->
```json
{
  "default": "coder",
  "providers": {
    "coder": { "base_url": "http://127.0.0.1:9/v1", "model": "qwen3:8b" }
  }
}
```

<!-- transcript: serve -->
```console
$ env XDG_CONFIG_HOME="$PWD/down" ikigai serve "$PWD/down.sock" --no-config-mounts 2>&1 | tee down.log
ikigai: mount   declined (--no-config-mounts) — the config home's `mount` lines are not read
…
ikigai: serving on …/down.sock  (Ctrl-C to stop)
```

<!-- transcript: run -->
```console
$ ikigai --mount "urn:d:=$PWD/down.sock" -c 'source urn:d:llm:coder:up'
false
[uncacheable]
$ ikigai --mount "urn:d:=$PWD/down.sock" -c 'source urn:d:llm:ask prompt=hello'
error: unavailable: llm transport error: unavailable: http://127.0.0.1:9/v1/chat/completions: Connection Failed: Connect error: Connection refused …
```

And with no peer at all, which is what gonk says when `ikigai serve` is not running:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gone:=$PWD/nothing.sock" -c 'source urn:gone:llm:ask prompt=hello'
ikigai: --mount: connect …/nothing.sock: No such file or directory (os error 2)
```

| what you see | what it means | what to do |
| --- | --- | --- |
| `urn:llm:coder:up` is `false` | the peer is fine; the model server is not answering | start Ollama; check `base_url` |
| `Connection refused` | the same, from an ask | the same |
| `No such file or directory`, or gonk's `not reachable` | nothing is serving the peer's socket | start `ikigai serve` |
| `denied: … does not allow reaching` a host | a capability ceiling does not name that host | spell the host the same way everywhere (below) |
| `llm.json … parse error` on the peer's stderr, and the default registry in `urn:llm:config` | the file has a field this `ikigai` does not know | fix the file: a bad file is not fatal, so read that line |

## Another OpenAI-compatible server

One provider shape covers them all, and they differ only in `base_url`, the model, and whether a key
is needed. These come from `ikigai-llm`'s README and the servers' default ports. This book ran only
Ollama among them, so treat the rest as a starting point and let the checks below decide.

| server | a typical start | `base_url` |
| --- | --- | --- |
| `llama.cpp` | `llama-server -m model.gguf` | `http://localhost:8080/v1` |
| `mlx_lm.server` | `mlx_lm.server --model <model>` (Apple silicon) | `http://localhost:8080/v1` |
| `rapid-mlx` | as its documentation says (Apple silicon) | `http://localhost:8000/v1` |
| vLLM | `vllm serve <model>` | `http://localhost:8000/v1` |
| LM Studio | its local server, from the app | `http://localhost:1234/v1` |

**Omit `model`** and the entry names the server: the model is discovered per request, as the smallest
one it lists that can chat, so swapping the model behind it needs no edit and no restart. Pin `model`
to say which. A `caps` object describes a provider, for the selection `ikigai-llm`'s README covers.

<!-- transcript: file other/ikigai/llm.json -->
```json
{
  "default": "local",
  "providers": {
    "local": { "base_url": "http://localhost:8000/v1" }
  }
}
```

The check is the same as before: serve it, read `urn:llm:config`, ask `urn:llm:ask`.

<!-- transcript: serve -->
```console
$ env XDG_CONFIG_HOME="$PWD/other" ikigai serve "$PWD/other.sock" --no-config-mounts 2>&1 | tee other.log
ikigai: mount   declined (--no-config-mounts) — the config home's `mount` lines are not read
…
ikigai: serving on …/other.sock  (Ctrl-C to stop)
```

<!-- transcript: run -->
```console
$ ikigai --mount "urn:o:=$PWD/other.sock" -c 'source urn:o:llm:config'
{"default":"local","providers":{"local":{"base_url":"http://localhost:8000/v1","model":null,"api_key":null}}}
[uncacheable]
```

## A peer on another machine

The model can be on a bigger machine. The peer listens on QUIC, each side trusts the other's
certificate, and moving it later changes one address. This page runs both ends on this machine. On the
machine with the model, make its identity and give gonk's mount one of its own:

<!-- transcript: file gpu/ikigai/llm.json -->
```json
{
  "default": "coder",
  "providers": {
    "coder": { "base_url": "http://127.0.0.1:11434/v1", "model": "qwen3:8b" }
  }
}
```

<!-- transcript: run -->
```console
$ XDG_CONFIG_HOME="$PWD/gpu" ikigai cert generate
wrote server.{crt,key} and client.{crt,key} to …/gpu/ikigai/quic
to attach a client on another machine, copy client.crt, client.key, and server.crt there.
$ ikigai cert generate --dir mount-cert
wrote server.{crt,key} and client.{crt,key} to mount-cert
to attach a client on another machine, copy client.crt, client.key, and server.crt there.
```

The exchange: the peer trusts gonk's certificate, and gonk pins the peer's. Between two machines, each
`cp` is a copy of a public certificate, and no private key moves:

<!-- transcript: run -->
```console
$ mkdir -p gpu/ikigai/quic/clients
$ cp mount-cert/client.crt gpu/ikigai/quic/clients/gonk.crt
$ cp gpu/ikigai/quic/server.crt mount-cert/server.crt
```

Start the peer. `--cap` is the ceiling on every connection, and a net grant on the model's host is the
only authority this peer lends:

<!-- transcript: serve -->
```console
$ env XDG_CONFIG_HOME="$PWD/gpu" ikigai serve quic://127.0.0.1:4470 --cap urn:cap:net:127.0.0.1 --no-config-mounts 2>&1 | tee gpu.log
ikigai: mount   declined (--no-config-mounts) — the config home's `mount` lines are not read
…
ikigai: serving on quic://127.0.0.1:4470  (fixed ceiling: urn:cap:net:127.0.0.1; surface: host + fs + llm; connections re-minted every 600s)  (Ctrl-C to stop)
```

⚠ **Spell the host the same way three times**: in the `gonk.mount` target, in the peer's `--cap`, and in
`llm.json`'s `base_url`. Mixing `localhost` and `127.0.0.1` can look fine from the owner's socket and
refuse everyone else. This route spells it `127.0.0.1`, and for a machine across the network it is that
machine's name, with the peer binding an address gonk can reach. gonk's
[fresh-machine runbook](https://github.com/ikigai-rs/ikigai-gonk/blob/main/docs/fresh-machine.md) has
the table of what fails.

gonk's mount line names the peer and the certificate directory. On your gonk it replaces the socket
line; this page starts a second gonk beside the first to show it, on its own port and its own homes:

<!-- transcript: run -->
```console
$ mkdir -p remote/config
$ printf 'gonk.mount = "prefer urn:llm:=quic://127.0.0.1:4470 %s/mount-cert"\n' "$PWD" > remote/config/config.toml
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/remote/config" --data-home "$PWD/remote/data" --port 1071 --no-quic --no-backup 2>&1 | tee remote.log
ikigai-gonk 0.1.1 — holding the store at …/remote/data/store
…
  llm     quic://127.0.0.1:4470 — mounted, but no gonk.browse.root, so nothing here derives through it
…
```

The same check, through gonk, over the QUIC mount. This gonk really dialed the peer, so the config it
prints came across that connection. This page has no model server behind the peer, and with yours
running, `urn:gk2:llm:ask` answers as it did above:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk2:=$PWD/remote/data/gonk.sock" -c 'source urn:gk2:llm:config'
{"default":"coder","providers":{"coder":{"base_url":"http://127.0.0.1:11434/v1","model":"qwen3:8b","api_key":null}}}
[uncacheable]
```

## A hosted API

A hosted service that speaks the OpenAI shape is one more provider, with a key:

<!-- transcript: file hosted/ikigai/llm.json -->
```json
{
  "default": "hosted",
  "providers": {
    "hosted": {
      "base_url": "https://api.example.com/v1",
      "model": "a-model-it-serves",
      "api_key": "sk-not-a-real-key"
    }
  }
}
```

<!-- transcript: serve -->
```console
$ env XDG_CONFIG_HOME="$PWD/hosted" ikigai serve "$PWD/hosted.sock" --no-config-mounts 2>&1 | tee hosted.log
ikigai: mount   declined (--no-config-mounts) — the config home's `mount` lines are not read
…
ikigai: serving on …/hosted.sock  (Ctrl-C to stop)
```

<!-- transcript: run -->
```console
$ ikigai --mount "urn:h:=$PWD/hosted.sock" -c 'source urn:h:llm:config'
{"default":"hosted","providers":{"hosted":{"base_url":"https://api.example.com/v1","model":"a-model-it-serves","api_key":"***"}}}
[uncacheable]
```

The key is masked in everything `ikigai` reports. Three things to decide before you use one:

- **The key sits in that file in plain text.** The `ikigai` this book pins links `ikigai-llm` 0.17.1,
  which reads only `api_key`, so `chmod 600` the file and keep it out of every repository. Later
  releases of `ikigai-llm` replace it with `api_key_secret`, naming a secret. The pinned `ikigai` does
  not know that field, says so on stderr, and falls back to its default registry.
- **Your project leaves the machine.** A draft sends the vocabulary, your graphs' shape and worked
  examples. An explanation or a review sends your source files. Choose accordingly.
- **It costs money per call.** gonk holds every caller together to `gonk.mount.calls_per_hour`, 2000 by
  default. Lower it if a bill would hurt.

The final check is the same, `urn:llm:ask`, and it is manual here, since it would reach the network.

<a id="what-you-built"></a>

## What you have now

`urn:llm:ask` answering through gonk, however you chose to run the model, and a table for when it does
not. In [A query from a sentence](sentence.md), you use the model to draft a query and review it
before publishing.
