# The machine client

[The editor as a client](emacs.md) showed how the catalog generates commands for a
person. This chapter uses the same catalog to generate tools for a program.

Take the two halves in order: a resource that fronts a family of interchangeable
implementations, and then the manifold as a tool list.

## One grammar, several backends

`ikigai-llm` binds a facade at `urn:llm:ask`: one endpoint that chooses a backend and
forwards the request. Each backend also has its own endpoint, so you can select it
directly. A backend may serve different models; the `model=` argument selects a specific
model.

Run these commands in a shell with the CLI from
[Running it](../getting-started/running-it.md#the-cli). The book tests them with
`ikigai-cli` 0.2.9. No earlier chapter sets up a model server: without a custom `llm.json`
in your config home, the CLI defaults to Ollama at `http://localhost:11434/v1`, with
`llama3.2:3b` as its default model. The first command needs a running backend and returns
model-generated text; if Ollama is not running, it reports `unavailable`. The second
lists the backend even when its server is down. A custom `llm.json` selects other
backends, and `IKIGAI_LLM_CONFIG` can point to a configuration file elsewhere.

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

The provider-specific names follow the pattern `urn:llm:{provider}:ask`. The CLI supplies
the default Ollama provider unless you configure others, so use `urn:llm:models` to find
the available backend names rather than guessing.

## An agent's tool list is the manifold under its capability

Now the second half. The Model Context Protocol (MCP) asks a server for three things: a list of
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
endpoint changes to make it available to an agent. The endpoint you bound in
[Binding, and a host of your own](../getting-started/binding.md), with the description
introduced in [Why an endpoint describes itself](../getting-started/self-description.md),
already supplies what the tool projection needs.

Two properties fall out of that, and both are worth more than the integration itself.

**Affordance equals authorization.** The list an agent sees is the manifold under the
capability its session holds — not the full catalog with the forbidden entries filtered out
at call time. A scoped agent cannot enumerate what it may not invoke, so a whole class of
"the model kept trying the thing it is not allowed to do" simply does not arise. Narrowing
authority narrows the tool list, live, and since core 0.1.75 inside a confinement too
([Scope and alias](../getting-started/scope-and-alias.md#not-yet) lists what is still not
chain-aware).

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
`tools/call` and checks the arguments with the same contract checker used by
`urn:kernel:validate` before it invokes the endpoint — so a model that supplies an invalid
argument gets a contract violation back rather than a half-executed action.

## What not to assume

[Where this actually stands](../modules/status.md) set out the limits of loadable
modules. These are the limits to keep in mind when using a model or an MCP client:

- **Generation is not cached by default.** A large language model (LLM) call is not a pure function of its
  inputs, so `urn:llm:ask` does not claim to be one. The caching that Part I taught applies
  to the deterministic parts of a pipeline, which is most of it, and deliberately not to
  this part.
- **MCP sessions require explicit authority.** In `ikigai-cli` 0.2.9, `ikigai mcp`
  refuses to start unless you pass `--grant`, `--scope`, `--principal`, or `--root`.
  An unknown or empty named grant is refused too. Only an explicit `--root` starts an
  unrestricted session and prints `--root given — serving UNRESTRICTED (root)`.
  Like the served kernel in [Who is asking](identity.md), it refuses undecided authority.
  The old default — **runs as root, loudly** (`ikigai-cli` 0.1.28) — no longer applies.
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
