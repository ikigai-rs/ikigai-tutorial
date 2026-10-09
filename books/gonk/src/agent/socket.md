# Over the socket

An agent does its work in a process, and a person reads its work in a browser. The browser
came through gonk's HTTP door. The agent comes through the **socket**: a Unix socket beside the
store, owner-only, which another ikigai process on the same machine mounts as part of its own
namespace. Nothing on the other side of it is an HTTP route; it is the kernel's own wire, so
the agent's tool calls are the ledger's resources, by their own names, with their own
contracts.

This chapter uses `ikigai`, the command-line host from
[`ikigai-cli`](https://github.com/ikigai-rs/ikigai-cli), as that other process. Install it at
the version this book pins, beside gonk ([Which gonk](../introduction.md#which-gonk)):

<!-- transcript: run -->
```console
$ ikigai --version
ikigai 0.1.41
```

## Start a gonk, and find its socket

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
…
  socket  …/.ikigai/gonk.sock — owner only
…
  mount   mount = "prefer urn:iki:ledger:=…/.ikigai/gonk.sock"  (and the same for urn:iki:store:)
```

Two lines of the banner are for this chapter. `socket` says where the door is and who may use
it: the file is created owner-only, so the operating system decides who can connect, and
whoever can connect is the owner. `mount` is the line to give another ikigai process so that it
reaches the ledger through that door. Hold on to both; first, the shortest way in.

## A mount on the command line

`--mount <prefix>=<target>` grafts a remote kernel into this one's namespace under a prefix,
as an **alias**: a name `<prefix>rest` is sent to the remote as `urn:rest`. So with gonk's
socket mounted at `urn:gk:`, the ledger's `urn:iki:ledger:append` is `urn:gk:iki:ledger:append`
here:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 Filed over the socket'
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:items'
   #1  open    p1  Filed over the socket
1 item(s)
[uncacheable]
```

A `--mount` on the command line is the whole topology of that command: `ikigai` composes it
INSTEAD of the `mount` lines in your own config home, so nothing you have configured is involved.

The client's status line says `[uncacheable]` for a read as well as a write, and that is gonk's
choice, not the ledger's. An answer gonk caches hangs from golden threads that live in gonk's
kernel, and a thread does not cross a wire; an answer that left the socket cacheable would arrive
with nothing that could ever invalidate it, and a long-lived client would go on serving it. So
every answer crosses the socket (and the QUIC door) uncacheable, and gonk keeps its own cache
behind it: a mounted read costs one round trip to gonk's cache, never a recompute.

`priority=1` was routed by name because the cli asked the remote for the resource's contract
first, over the same socket, and the contract declares `priority`. That contract is the
agent's tool description, and it is the ledger's own:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'describe urn:gk:iki:ledger:append' | grep -E 'ik:title|ik:inputName'
[computed]
    ik:title "File a ledger item" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:ledger> ik:inputName "ledger" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:content> ik:inputName "content" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:kind> ik:inputName "kind" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:priority> ik:inputName "priority" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:labels> ik:inputName "labels" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:about> ik:inputName "about" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:revision> ik:inputName "revision" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:author> ik:inputName "author" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:key> ik:inputName "key" ;
<urn:ikigai:endpoint:ledger-append:action:sink:input:as> ik:inputName "as" ;
```

(`[computed]` comes first because the cli writes its status line to standard error, which does
not go through `grep`.) Each input is declared with a summary, whether it is required, and its
type; `content` is the one a value or a pipe lands in. `key` is the caller's own name for the
item: an append whose key is already taken files nothing and answers the item that holds it,
which is how [part 5's sync](../spec/ledger.md) and the [bridges](../bridges/roborev.md) file
each thing once. `as=application/json` asks for the answer in the ledger's JSON face. An agent's tool schema is this graph,
read from the ledger itself, so it cannot drift from what the ledger accepts.

## The same names, from the config home

An alias is right for one command. For a host that works in the ledger all day, the banner's
`mount` line goes into its config home instead, and then the ledger's names need no prefix at
all: `prefer` serves `urn:iki:ledger:*` from the socket under the names it already has, and
falls back to a local binding only when the socket is unreachable.

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'mount = "prefer urn:iki:ledger:=%s/.ikigai/gonk.sock"\n' "$PWD" >> .config/ikigai/config.toml
$ env XDG_CONFIG_HOME="$PWD/.config" ikigai -c 'sink urn:iki:ledger:append Filed by its own name' -c 'source urn:iki:ledger:next'
#2 urn:iki:ledger:default:item:…
[uncacheable]
 1.    #1  open    p1  Filed over the socket
    p1 — priority 1; last updated …
 2.    #2  open    p-  Filed by its own name
    unprioritized — no priority set, so it ranks below every item that has one; last updated …
policy: priority-recency (weighs priority, recency, number)
ready: 2   excluded: 0
[uncacheable]
— batch: 2 commands · 2 uncacheable
```

Here the config home IS the lesson, so the page names its own: `XDG_CONFIG_HOME` is where
`ikigai` looks for it, and `ikigai-cli` 0.1.41 has no flag that says it instead. Without that
prefix the command reads yours.

⚠ **Write the socket's path out in full.** The mount line is read as it is written, and
`ikigai-cli` 0.1.41 does not expand a leading `~` in it: `mount = "prefer
urn:iki:ledger:=~/.ikigai/gonk.sock"` fails with *No such file or directory*, naming the
unexpanded path. That is why the line above is written with `printf` and `$PWD`; in your own
config home, write your home directory's path.

A mount claims one prefix, so a host that also wants the store's resources adds the same line
for `urn:iki:store:`, as the banner says. The two are one line each because a mount is a
string prefix, and nothing about `urn:iki:ledger:` covers `urn:iki:store:`.

## Whose authority

The socket's caller is the owner, and the owner holds everything gonk serves. Delete included:

<!-- transcript: run -->
```console
$ env XDG_CONFIG_HOME="$PWD/.config" ikigai -c 'delete urn:iki:ledger:item:2'
deleted #2 urn:iki:ledger:default:item:…
  6 quad(s) moved to <urn:iki:ledger:graph:default:deleted> and recoverable
  tombstone: urn:iki:ledger:default:tombstone:…
[uncacheable]
```

That is the right authority for you, at your own terminal, and the wrong one to hand an agent.
An agent should hold what its work needs and nothing more, and a capability can only be
**narrowed** as it travels: the host narrows its own before it acts, and what crosses the
socket is the narrowed one. `ikigai-gonk grants` prints the tokens for one ledger at one
authority, which is exactly the list to narrow to:

<!-- transcript: run -->
```console
$ ikigai-gonk grants default write
[
  "urn:cap:ledger:read:default",
  "urn:cap:store:read:graph:urn:iki:ledger:graph:default",
  "urn:cap:ledger:write:default",
  "urn:cap:store:write:graph:urn:iki:ledger:graph:default"
]
```

`cap seal` narrows the session to those scopes and makes that the floor, which nothing later in
the session can rise above. Then the agent's work, and two things it was not given (what the
seal does and does not keep it from beyond these is [below](#what-the-seal-does-not-bound)):

<!-- transcript: run -->
```console
$ env XDG_CONFIG_HOME="$PWD/.config" ikigai -c 'cap seal urn:cap:ledger:read:default urn:cap:store:read:graph:urn:iki:ledger:graph:default urn:cap:ledger:write:default urn:cap:store:write:graph:urn:iki:ledger:graph:default' -c 'sink urn:iki:ledger:append author=agent-7 Filed under a narrowed grant' -c 'delete urn:iki:ledger:item:1' -c 'source urn:iki:ledger:book:items'
sealed — capability: urn:cap:ledger:read:default, urn:cap:ledger:write:default, urn:cap:store:read:graph:urn:iki:ledger:graph:default, urn:cap:store:write:graph:urn:iki:ledger:graph:default · sealed (this is the floor: `cap reset`, `logout` and `login` cannot rise above it)
#3 urn:iki:ledger:default:item:…
[uncacheable]
error: denied: capability does not grant `urn:cap:ledger:delete:*` (declared by `urn:iki:ledger:item:1`)
error: denied: this capability does not hold `urn:cap:ledger:read:book`. A ledger grant names exactly one ledger — holding a grant over another ledger satisfies the declared family `urn:cap:ledger:read:*` but not this resource, which is the point of naming them
— batch: 4 commands · uncacheable
```

The filing worked; the delete and the other ledger did not, with the token each one needed
named in the refusal. `author=agent-7` is the ledger's own argument for who filed an item. It
is a claim the caller makes, not an identity the socket proves, which is the next section.

## What the door saw

The socket door writes the same access lines as the HTTP door, and they say what the
narrowing did not change:

<!-- transcript: run -->
```console
$ grep 'door=socket verb=sink' gonk.log
… gonk:Access urn:iki:ledger:append door=socket verb=sink outcome=ok bytes=… dur=… principal=owner q=-
… gonk:Access urn:iki:ledger:append door=socket verb=sink outcome=ok bytes=… dur=… principal=owner q=-
… gonk:Access urn:iki:ledger:append door=socket verb=sink outcome=ok bytes=… dur=… principal=owner q=-
$ grep ' urn:iki:ledger:book:items ' gonk.log
… gonk:Access urn:iki:ledger:book:items door=socket verb=source outcome=denied bytes=- dur=… principal=owner q=-
```

- Every request over the socket is `principal=owner`. The socket knows that whoever connected
  could open an owner-only file, and nothing more: it cannot tell your terminal from an agent
  you started. An agent that should be attributed as itself in the log needs an identity of its
  own, which is a certificate on the QUIC door (a later part) rather than this socket.
- The read of `book` reached gonk and was refused there, so it has a line. The delete has none:
  the declared family `urn:cap:ledger:delete:*` was not held, so it was refused before it was
  dispatched to the ledger at all.

## What the seal does not bound

Look at the fourth token again: `urn:cap:store:write:graph:urn:iki:ledger:graph:default`. The
ledger needs it, because it writes every append and close as a store update under the
*caller's* capability ([Whose ledger](../ledger/your-own.md#whose-ledger)). But it is the
store's token, so on its own it would also let the holder write the ledger's graph directly,
through the store, without the ledger seeing it, and that is how an item could be given an
`author` naming somebody else. So gonk refuses a raw store write to a ledger graph, at every
door, unless the caller is root or holds a separate raw grant for that graph. Mount the store's
names as well, as the banner suggested, and try it from a sealed session; then once more at
root:

<!-- transcript: run -->
```console
$ printf 'mount = "prefer urn:iki:store:=%s/.ikigai/gonk.sock"\n' "$PWD" >> .config/ikigai/config.toml
$ env XDG_CONFIG_HOME="$PWD/.config" ikigai -c 'cap seal urn:cap:ledger:read:default urn:cap:store:read:graph:urn:iki:ledger:graph:default urn:cap:ledger:write:default urn:cap:store:write:graph:urn:iki:ledger:graph:default' -c 'sink urn:iki:store:graph-update graph=urn:iki:ledger:graph:default INSERT DATA { GRAPH <urn:iki:ledger:graph:default> { <urn:example:note> <urn:example:says> "written raw" } }'
sealed — capability: urn:cap:ledger:read:default, urn:cap:ledger:write:default, urn:cap:store:read:graph:urn:iki:ledger:graph:default, urn:cap:store:write:graph:urn:iki:ledger:graph:default · sealed (this is the floor: `cap reset`, `logout` and `login` cannot rise above it)
error: denied: a raw store write to the ledger graph <urn:iki:ledger:graph:default> is refused at this door. A ledger grant carries that graph's store token for the ledger's OWN writes, which it issues itself; it is not authority to write the graph's triples directly — that is how an `author` naming someone else would get in. Write through the ledger's endpoints (`urn:iki:ledger:*`). Raw writes are the owner's, at root over the socket, or an identity's whose grant an operator minted with `--ledger-graph <ledger>` (`urn:cap:gonk:raw-write:graph:urn:iki:ledger:graph:default`)
$ env XDG_CONFIG_HOME="$PWD/.config" ikigai -c 'sink urn:iki:store:graph-update graph=urn:iki:ledger:graph:default INSERT DATA { GRAPH <urn:iki:ledger:graph:default> { <urn:example:note> <urn:example:says> "written raw" } }'
updated <urn:iki:ledger:graph:default>: +1 -0 quads
[uncacheable]
```

The owner at root still writes raw over the socket, which is where migrations and repairs run;
a session that sealed itself to a ledger grant is held to the same rule as a network client.
(That is new at the gonk this book pins: through gonk ed43af3 the socket checked nothing, and
the sealed write above succeeded. The [QUIC chapter](../machines/quic-client.md#writing-the-graph-raw)
shows how an operator gives a raw grant by name, `--ledger-graph`.)

What the seal does **not** bound is the `author` argument on the session's own new items. The
network doors refuse a write whose `author` is shaped like a principal and is not the caller's
own; but the socket names no principal (every request is `owner`), so it cannot run that rule,
and a sealed session may still file an item that claims to be from a passkey:

<!-- transcript: run -->
```console
$ env XDG_CONFIG_HOME="$PWD/.config" ikigai -c 'cap seal urn:cap:ledger:read:default urn:cap:store:read:graph:urn:iki:ledger:graph:default urn:cap:ledger:write:default urn:cap:store:write:graph:urn:iki:ledger:graph:default' -c 'sink urn:iki:ledger:append author=urn:iki:gonk:passkey:ada Not really Ada'
sealed — capability: urn:cap:ledger:read:default, urn:cap:ledger:write:default, urn:cap:store:read:graph:urn:iki:ledger:graph:default, urn:cap:store:write:graph:urn:iki:ledger:graph:default · sealed (this is the floor: `cap reset`, `logout` and `login` cannot rise above it)
#4 urn:iki:ledger:default:item:…
[uncacheable]
— batch: 2 commands · uncacheable
```

It is the same kind of claim as `author=agent-7`, on an item the session filed itself, never a
change to someone else's; but gonk's item page renders a passkey-shaped author as that
passkey's label, so it would read as Ada's. gonk's README states this residual rather than
hiding it. An agent that must not be able to claim a person needs an identity of its own on the
QUIC door, where the author rule runs ([The three doors](../machines/doors.md)).

## Over MCP (coming)

An agent framework that speaks MCP rather than ikigai would reach the same ledger through
`ikigai mcp`, which projects a kernel's capability-scoped manifold as MCP tools: run it with
gonk's socket mounted and a sealed grant like the one above, and the agent's tool list is the
ledger's actions under that grant and nothing else. That is planned and not yet checked
end to end (ledger item 782: tool names for `urn:iki:ledger:*`, idempotent append, and a
per-agent identity so the log can name the agent), so the *Over MCP* chapter is listed as
coming until it is.

## What you have now

| | how it reaches gonk | under what authority | who the log says |
| --- | --- | --- | --- |
| a browser | HTTP, loopback | anonymous, or a passkey's grant | `anon`, or the passkey |
| `curl`, a hook | HTTP, loopback | anonymous | `anon` |
| `ikigai` on this machine | the socket | the owner's, narrowed with `cap seal` | `owner` |

The agent's tool calls were the ledger's resources by their own names: the same `append`,
`next` and `delete` that the page's kernel ran in [the first chapter](../ledger/your-own.md),
with the same contracts, now crossing a process boundary. What changed was only where the
ledger lived and whose authority reached it.
