# Over MCP

An agent framework that speaks the [Model Context Protocol](https://modelcontextprotocol.io/)
rather than ikigai reaches the same ledger through `ikigai mcp`. It projects a kernel's
**capability-scoped manifold** as MCP tools: one tool per action the session may invoke, with
the arguments that action declares. Nothing is written for the agent. The tool list is the
ledger's own `Description`s, filtered by a capability, so it cannot offer what the session
may not do.

The [previous chapter](socket.md) ended on what the socket cannot bound: every request on it is
the owner's, so a sealed session can still file an item whose `author` claims a passkey. An
agent that must not be able to claim a person needs an identity of its own, on the door where
the author rule runs. That is the QUIC door from [A QUIC client](../machines/quic-client.md),
and this chapter puts the two together: the agent gets a client certificate and a grant on the
server, and `ikigai mcp` reaches gonk through that door. Three things to see at the end, each
on a transcript below:

- the agent's tool list is the ledger's writer actions for one ledger, and nothing else;
- a retried filing files once, because the append is keyed;
- the server names the agent, in the item's `author` and on every access line.

Every command here runs `ikigai` at the version this book pins, which speaks MCP over standard
input and output.

## The server: an identity for the agent

As in [A QUIC client](../machines/quic-client.md), the server mints the agent a certificate and
enrolls it under a grant built from a role. The agent may write the `default` ledger:

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ ikigai-gonk client add agent --ledger default=write --port 1070 --config-home "$PWD/.config/ikigai"
client `agent`  …/.config/ikigai/gonk/quic/clients/agent
  fingerprint  …
  principal    urn:iki:gonk:client:…
  enrolled     grant `agent` (4 scopes) in …/.config/ikigai/gonk/grants.json
  restart ikigai-gonk: trusted certificates are read at startup
  this bundle holds the client's PRIVATE key and the server never reads it — move the directory to the client, then from the client:
    ikigai --connect quic://<gonk host>:1070 --cert-dir <the moved directory>
```

Four scopes: the ledger's writer, read and write on `default`, and the store's tokens for that
ledger's graph, which the ledger uses for its own writes. Not delete, not purge, and no other
ledger. gonk enforces this list on every request from that certificate, whatever the client
asks for. Start the server, which reads the certificates it trusts at startup:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  quic    udp 0.0.0.0:1070 — 1 trusted certificate(s), 1 enrolled
…
```

## The agent's side: a bundle and a ceiling

The agent runs on "another machine", the directory `agent/`, with its own config home. It gets
the bundle (on a real second machine, moved there and deleted here):

<!-- transcript: run -->
```console
$ mkdir -p agent/.config/ikigai
$ cp -R .config/ikigai/gonk/quic/clients/agent agent/gonk
```

and a **grant of its own**, in its own config home's `grants.json`. This is the client-side
ceiling: `ikigai mcp --grant agent` serves the manifold under exactly these scopes, so the
tools of everything else the agent's `ikigai` could reach (its files, its secrets, a model) are
filtered out of the list by capability. `show` narrows what is listed further, to the ledger's
tools; it changes what the agent sees, not what it may call. The scopes are the server's four,
which `ikigai-gonk grants default write` prints:

<!-- transcript: file agent/.config/ikigai/grants.json -->
```json
{
  "agent": {
    "scopes": [
      "urn:cap:ledger:read:default",
      "urn:cap:store:read:graph:urn:iki:ledger:graph:default",
      "urn:cap:ledger:write:default",
      "urn:cap:store:write:graph:urn:iki:ledger:graph:default"
    ],
    "show": ["ledger-*"]
  }
}
```

There are two ceilings, then, and they are independent. The server's grant is the one that
holds against a compromised agent, because the agent's machine cannot change it. The client's
is the one that keeps the tool list honest, so an agent is not offered a tool that would only
be refused.

## The tool list

An MCP client starts the server as a child process and talks JSON-RPC to it, one message per
line. Here the messages are a file, so the transcript is repeatable: the handshake, then the
request for the tool list.

<!-- transcript: file agent/list.jsonl -->
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"book","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/list"}
```

The command is the one an agent framework is configured with. `--override` mounts gonk's QUIC
door at `urn:iki:ledger:`, so every ledger name goes there and nowhere else; `--cert-dir` after
it is that mount's identity; `--grant` is the ceiling above. `ikigai` 0.1.41 has no flag for its
config home, so the page points `XDG_CONFIG_HOME` at the agent's directory. The tool names are
picked out of the answer and sorted (byte order, so the list reads the same on every machine), because the full answer carries every argument's
description:

<!-- transcript: run -->
```console
$ env XDG_CONFIG_HOME="$PWD/agent/.config" ikigai mcp --override urn:iki:ledger:=quic://127.0.0.1:1070 --cert-dir agent/gonk --grant agent < agent/list.jsonl | grep -o '"name":"[^"]*__[^"]*"' | LC_ALL=C sort
ikigai mcp: serving the manifold under 4 scope(s)
ikigai mcp: tool visibility — 1 shown, 0 hidden pattern(s)
ikigai mcp: mount   override urn:iki:ledger: -> quic://127.0.0.1:1070  [certs agent/gonk]
"name":"ledger-append__sink"
"name":"ledger-claim__delete"
"name":"ledger-claim__sink"
"name":"ledger-close__sink"
"name":"ledger-comment__sink"
"name":"ledger-defer__delete"
"name":"ledger-defer__sink"
"name":"ledger-doctor__source"
"name":"ledger-item-closed__exists"
"name":"ledger-item-holder-is__exists"
"name":"ledger-item-state-is__exists"
"name":"ledger-item-state__sink"
"name":"ledger-item-state__source"
"name":"ledger-item__exists"
"name":"ledger-item__sink"
"name":"ledger-item__source"
"name":"ledger-items__source"
"name":"ledger-label__delete"
"name":"ledger-label__sink"
"name":"ledger-ledgers__source"
"name":"ledger-lifecycle__exists"
"name":"ledger-lifecycle__source"
"name":"ledger-link__delete"
"name":"ledger-link__sink"
"name":"ledger-next__source"
"name":"ledger-policy__exists"
"name":"ledger-policy__source"
"name":"ledger-reopen__sink"
```

Twenty-eight tools, each an endpoint and a verb: `ledger-append__sink` is a `sink` to
`urn:iki:ledger:append`, `ledger-next__source` a `source` of `urn:iki:ledger:next`. Eight of
them arrived with `ikigai-ledger` 0.5.0 and are not used in this book yet: an item's lifecycle
**state** (`ledger-item-state__*`, a compare-and-set, so of two agents moving one item out of the
same state exactly one wins), the **assertions** an agent can check before it acts
(`ledger-item-state-is__exists`, `-holder-is`, `-closed`), the **lifecycles** themselves as
resources, and the **doctor**, a read-only report of orphaned, abandoned and lease-expired work.
A `__delete` tool is here only where the ledger's writer may delete (a label, a claim, a link, a deferral).
`ledger-item__delete` is not: deleting an item takes the ledger's delete token, and purging
takes another, and this grant holds neither.

Every tool that names a ledger takes it as an argument, `ledger`, with the default `default`
(the ledger whose name the IRI may omit, as in [Whose ledger](../ledger/your-own.md#whose-ledger)).
An agent that leaves it out writes to `default`, which is the ledger it was granted.

## Filing, once

The agent files a task with its own name for it, `key=agent-task-1`, the way a hook files a
finding by the finding's id. Then it files the same thing again, as an agent does when it is
not sure the first call landed. Then it reads the item back by that key, and finally it tries
three things outside its grant: an `author` naming a person's passkey, the item's delete tool,
and another ledger:

<!-- transcript: file agent/calls.jsonl -->
```json
{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"book","version":"1"}}}
{"jsonrpc":"2.0","method":"notifications/initialized"}
{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"ledger-append__sink","arguments":{"content":"Read the notes","key":"agent-task-1"}}}
{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"ledger-append__sink","arguments":{"content":"Read the notes","key":"agent-task-1"}}}
{"jsonrpc":"2.0","id":4,"method":"tools/call","params":{"name":"ledger-item__source","arguments":{"id":"key:agent-task-1"}}}
{"jsonrpc":"2.0","id":5,"method":"tools/call","params":{"name":"ledger-append__sink","arguments":{"content":"Not really Ada","author":"urn:iki:gonk:passkey:ada"}}}
{"jsonrpc":"2.0","id":6,"method":"tools/call","params":{"name":"ledger-item__delete","arguments":{"id":"1"}}}
{"jsonrpc":"2.0","id":7,"method":"tools/call","params":{"name":"ledger-append__sink","arguments":{"ledger":"acme","content":"Not mine"}}}
```

<!-- transcript: run -->
```console
$ env XDG_CONFIG_HOME="$PWD/agent/.config" ikigai mcp --override urn:iki:ledger:=quic://127.0.0.1:1070 --cert-dir agent/gonk --grant agent < agent/calls.jsonl 2>/dev/null
{"jsonrpc":"2.0","id":1,"result":{"protocolVersion":"2024-11-05","capabilities":{"tools":{"listChanged":true}},"serverInfo":{"name":"ikigai","version":"0.1.41"}}}
{"jsonrpc":"2.0","id":2,"result":{"content":[{"type":"text","text":"#1 urn:iki:ledger:default:item:…\n"}],"isError":false}}
{"jsonrpc":"2.0","id":3,"result":{"content":[{"type":"text","text":"#1 urn:iki:ledger:default:item:… existing open\n"}],"isError":false}}
{"jsonrpc":"2.0","id":4,"result":{"content":[{"type":"text","text":"   #1  open    p-  Read the notes\n  iri:      urn:iki:ledger:default:item:…\n  key:      agent-task-1\n  filed:    … by urn:iki:gonk:client:…\n  updated:  …\n"}],"isError":false}}
{"jsonrpc":"2.0","id":5,"result":{"content":[{"type":"text","text":"denied: `author` names a principal (urn:iki:gonk:passkey:ada), and only the door names one: a write may name its own principal or a plain-text author, never another identity. This request is from urn:iki:gonk:client:…."}],"isError":true}}
{"jsonrpc":"2.0","id":6,"result":{"content":[{"type":"text","text":"tool `ledger-item__delete` is not available under this capability"}],"isError":true}}
{"jsonrpc":"2.0","id":7,"result":{"content":[{"type":"text","text":"denied: this capability does not hold `urn:cap:ledger:write:acme`. A ledger grant names exactly one ledger — holding a grant over another ledger satisfies the declared family `urn:cap:ledger:write:*` but not this resource, which is the point of naming them"}],"isError":true}}
```

Read the answers in order:

- **2 and 3: one item.** The second append found the key taken and filed nothing; it answered
  with the item that holds it, marked `existing`. The check and the filing are one store update
  in the ledger, so this holds for concurrent retries as well as sequential ones. An agent that
  keys what it files can retry freely.
- **4: the author is the agent.** The item was filed with no `author` argument, and gonk filled
  it with the name of the identity that asked, `urn:iki:gonk:client:<fingerprint>`, from the
  certificate the connection was made with.
- **5: and only the agent.** On the QUIC door an `author` that names a principal must be the
  caller's own, so the agent cannot file as Ada. This is the residual the socket chapter ended
  on, closed by giving the agent its own door.
- **6: never sent.** The item's delete is not in this session's manifold, so `ikigai mcp`
  refuses the call itself; gonk never sees it.
- **7: sent, and refused by the ledger.** The client's ceiling holds the write *family* (for
  `default`), so the call passes the local check and crosses the wire; the ledger refuses it,
  because a ledger grant names one ledger. A tool that takes `ledger` as an argument can be
  asked about any ledger; only a granted one answers.

## What the server saw

<!-- transcript: run -->
```console
$ grep -E 'quic client| door=quic ' gonk.log
ikigai-gonk: quic client … → grant "agent"
ikigai-gonk: quic client … → grant "agent"
… gonk:Access urn:iki:ledger:default:append door=quic verb=sink outcome=ok bytes=… dur=… principal=urn:iki:gonk:client:… q=-
… gonk:Access urn:iki:ledger:default:append door=quic verb=sink outcome=ok bytes=… dur=… principal=urn:iki:gonk:client:… q=-
… gonk:Access urn:iki:ledger:default:item:key:agent-task-1 door=quic verb=source outcome=ok bytes=… dur=… principal=urn:iki:gonk:client:… q=-
… gonk:Access urn:iki:ledger:default:append door=quic verb=sink outcome=denied bytes=… dur=… principal=urn:iki:gonk:client:… q=-
… gonk:Access urn:iki:ledger:acme:append door=quic verb=sink outcome=denied bytes=… dur=… principal=urn:iki:gonk:client:… q=-
```

One connection line per `ikigai mcp` process, naming the grant. One access line per request
that reached gonk, each naming the agent, refusals included. Two things in the names are worth
reading. Every append is logged as `urn:iki:ledger:default:append`: the tool call carried no
`ledger`, the argument's default filled it, and the request went out under the ledger's full
name. And the delete is not in the log at all, because it never left the agent's machine.

## Wiring it into an agent

An MCP client is configured with a command to start, and this is the command. What the
configuration file looks like is the framework's own; the part that is ikigai's is the line
below, with the server's address and the bundle where the agent's machine keeps it:

<!-- transcript: manual — an agent framework's configuration, and a gonk on another machine -->
```console
$ ikigai mcp --override urn:iki:ledger:=quic://server.local:1060 --cert-dir ~/gonk-agent --grant agent
```

Mount flags go after `mcp`, and they replace the `mount` lines of the agent's own config home
for this process, so nothing else the agent's machine has configured is composed in. A gonk on
the default port opens its QUIC door on UDP 1060.

## What you have now

| | how it reaches gonk | under what authority | who the log says |
| --- | --- | --- | --- |
| a browser | HTTP, loopback | anonymous, or a passkey's grant | `anon`, or the passkey |
| `curl`, a hook | HTTP, loopback | anonymous | `anon` |
| `ikigai` on this machine | the socket | the owner's, narrowed with `cap seal` | `owner` |
| an agent over MCP | QUIC, with its own certificate | its client's grant on the server, and its own ceiling on the client | the agent's `urn:iki:gonk:client:…` |

The agent's tool calls were the ledger's resources by their own names, the same `append`, `item`
and `next` that the page's kernel ran in [the first chapter](../ledger/your-own.md), with the same
contracts. MCP added a way to list them; what made the list safe to hand an agent was the
capability it was computed under, and what made the agent answerable was the door it came in by.
