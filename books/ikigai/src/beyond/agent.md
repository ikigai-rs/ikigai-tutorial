# The machine client

The previous chapter generated a command surface for a person. This one generates the same
thing for a program, out of the same catalog, and the fact that it is the same is the
point the whole book has been walking toward.

Take the two halves in order: a resource that fronts a family of interchangeable
implementations, and then the manifold as a tool list.

## One grammar, several backends

`ikigai-llm` binds a facade at `urn:llm:ask` and a directly addressable endpoint per
configured backend. Asking is the front grammar; a specific model is still a name you can
resolve if you want that model and no other.

```bash
ikigai -c 'source urn:llm:ask prompt="name three uses for a paperclip"'
ikigai -c 'source urn:llm:models'
```

The facade picks a backend — an explicit `provider=` argument, else a `needs=` expression
resolved against what each backend can actually do, else the configured default — and then
does something worth noticing:

> Rewrite the target; carry every argument through unchanged. Going via `inv.issue`
> records the backend result as a dependency, so its expiry and golden threads propagate
> to this facade result.

It **re-issues through the kernel**. It does not call a backend object it holds; it names
the backend as a resource and asks for it, exactly as any other caller would. Which means
the facade inherits the backend's cacheability and invalidation for free — and, less
obviously, that everything in
[Preferential resolution](preference.md) applies *inside* it. Override
`urn:llm:` and the facade itself lives on a peer. Override one backend name and a local
facade routes to a remote model. The composition happens between the two halves because
they were never wired together in the first place.

That is the general shape, not a trick this one crate plays: **a facade that re-issues
through the kernel is composable; a facade that holds its implementations is not.**

<!-- urn-gate: illustration urn:llm:{provider}:ask — a template, and which providers exist depends on the reader's own llm.json. -->

The provider-specific names follow the pattern `urn:llm:{provider}:ask` — which providers
exist is a matter of local configuration, so `urn:llm:models` is how you ask rather than
guess.

## An agent's tool list is the manifold under its capability

Now the second half. The Model Context Protocol asks a server for three things: a list of
tools, a typed input schema for each, and a way to call one.

ikigai has all three already, under other names, and had them before MCP existed — because
they are what a resolution needs, not what a protocol asked for:

| MCP wants | ikigai already has |
|---|---|
| `tools/list` | `urn:kernel:actions` — the manifold, already narrowed to this capability |
| an input schema | the `ArgSpec` contracts on each action's `Description` |
| `tools/call` | kernel invocation |

So the projection is a *translation*, not a feature: an action becomes a tool descriptor,
and the tool name maps back to the endpoint and verb it came from. Nothing about an
endpoint changes to make it available to an agent. The endpoint you wrote in Part I, with
the description you wrote for the engine's benefit, is an agent tool already.

Two properties fall out of that, and both are worth more than the integration itself.

**Affordance equals authorization.** The list an agent sees is the manifold under the
capability its session holds — not the full catalog with the forbidden entries filtered out
at call time. A scoped agent cannot enumerate what it may not invoke, so a whole class of
"the model kept trying the thing it is not allowed to do" simply does not arise. Narrowing
authority narrows the tool list, live.

**Federation is free at the tool layer.** The manifold is projected from the composed
kernel, mounts and all. A `prefer` mount pointing `urn:llm:` at a heavier machine puts that
machine's models behind exactly the tool names the local kernel would have projected, with
no client-side configuration anywhere. The agent never learns there is a second machine.

## The funnel, and where a model is actually needed

The projection is one way to hand an agent a surface; the kernel also offers a narrowing
path for choosing *within* it, which is worth knowing exists because of what it says about
where inference belongs:

1. `urn:kernel:actions` — deterministic narrowing. Which actions this capability may
   invoke, and which of those match the shape of what you have.
2. `urn:agent:select` — a model, used only for what is left over after step 1.
3. `urn:kernel:validate` — a pre-flight check of the arguments against the declared
   contract, before anything is invoked.

The ordering is the argument. Inference is the residual step, not the first one: the
expensive, non-deterministic, unauditable component is asked the smallest question that
remains after the deterministic machinery has done what it can, and its answer is validated
before it has any effect.

Step 3 is not optional advice, either. The MCP server re-checks the capability on every
`tools/call` and pre-flights the arguments through `urn:kernel:validate` before it invokes
anything — so a model that hallucinates an argument gets a contract violation back rather
than a half-executed action.

## What not to assume

Part II ended with a list like this and this part should too, for the same reason: a
tutorial that leaves you to discover a limit by hitting it has wasted your afternoon.

- **Generation is not cached by default.** An LLM call is not a pure function of its
  inputs, so `urn:llm:ask` does not claim to be one. The caching that Part I taught applies
  to the deterministic parts of a pipeline, which is most of it, and deliberately not to
  this part.
- **An MCP session with no grant runs as root, loudly** (`ikigai-cli` 0.1.18). `ikigai mcp`
  with no `--grant` or `--scope` prints `running UNRESTRICTED (root)` and then does. A named grant that
  resolves to no scopes lands in the same branch. That is the opposite of the served
  kernel's rule from [Who is asking](identity.md), where an unrecognized certificate is
  refused — the difference being that an MCP server is spawned by the human whose authority
  it runs under, and a QUIC server is not. Convenient, and exactly the sort of asymmetry to
  know about rather than discover.
- **The tool list is a projection, not a promise.** Endpoints appear and disappear as
  capability, configuration and mounts change. That is the design working, and a client
  that caches a tool list across sessions is fighting it.

## Where that leaves the book

Part I bound a resource. Part II routed a prefix of the name space to code the host had
never seen. This part put a kernel behind a socket, gave it a way to know who was asking,
and gave a caller three honest things to mean by "resolve this over there".

What connects them is that none of it required a resource to know where it would be used.
The camel-case endpoint from the first chapter can be linked into a host, reached through a
module boundary, served over a socket, mounted from another machine, called from an editor
by a generated function, or offered to an agent as a tool — and it is the same short function
throughout, because it was never asked to care.
