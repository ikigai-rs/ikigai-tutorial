# Glossary

The nine terms of [Resolution](getting-started/resolution.md), one paragraph each, with
the chapter where you do the thing rather than read about it.

**Resource.** Anything with a name — a URI, in this system a `urn:*`. A resource is not
an object and not a function: it is a name a kernel can resolve, and what you get back
depends on which verb you asked with. `urn:iki:tutorial:camel-case` is one; so is
`urn:kernel:catalog`. You define one in [Hello, resource](getting-started/hello-resource.md)
and give it a name in [Binding, and a host of your own](getting-started/binding.md).

**Verb.** One of five: `Source` (read), `Sink` (write), `Exists`, `Delete`, `Meta`
(describe yourself). The same five for every resource, so there is no method vocabulary
to learn — and `Meta` is the one that makes the system legible to a machine. An endpoint
answering more than one declares a contract per verb, in [Multi-verb
endpoints](building/multi-verb.md).

**Representation.** What a resolution returns: bytes plus a media type. Not a
language-specific value, because the answer may have crossed a process, a machine or a
language. Declaring the type is what lets a transreptor find a route from what you
produced to what somebody asked for. You return one in [Hello,
resource](getting-started/hello-resource.md) and ask for a different one in [What
resolution buys you](getting-started/payoff.md).

**Transreptor.** An endpoint that converts one representation into another, and — the
part that matters — *declares* which conversions it performs, so the kernel can select
it when a request asks for a type the resource does not produce itself. A resource does
not have a format; it has whatever the kernel can reach. You write one in
[Transreption](building/transreption.md).

**Golden thread.** The dependency a cached answer hangs from — declared by the endpoint
(`.depends_on(..)`), conventionally named after the resource whose state it tracks. A
resolution derived from other resolutions inherits their threads; a write to a resource
cuts the thread named after it, and every cached entry whose threads include that one —
directly or transitively — stops being valid at that instant. An entry that declared no
thread, a pure function's, is untouched by any cut. Precise invalidation, not timeouts. You cut one in [What resolution buys
you](getting-started/payoff.md) and again, on a file, in
[Configuration](getting-started/configuration.md).

**Capability.** The authority a resolution runs under, carried with the invocation and
narrowed — never widened — as it passes down a call chain. What an endpoint *declares*
it requires is what the kernel *enforces*, per verb, before the endpoint runs: declared
equals enforced, and either half without the other is a defect. You declare one on your
own endpoint in [Multi-verb endpoints](building/multi-verb.md), and watch one cross a
socket in Part III's [Exercises](beyond/exercises.md).

**Catalog.** `urn:kernel:catalog`: every bound endpoint's description, as one RDF graph,
assembled from the descriptions rather than maintained beside them. Nobody writes it,
so it cannot be out of date. You read your own host's in [What resolution buys
you](getting-started/payoff.md) and query it in [The graph face](building/graph-face.md).

**Manifold.** `urn:kernel:actions`: the capability-scoped list of what the *current
caller* may invoke — the catalog, filtered by authority, per action. It narrows as the
capability narrows, and it is computed, not configured. [The machine
client](beyond/agent.md) is where a client reads it instead of being told.

**Tool list.** What an agent is handed as the things it may do. Here it is not written
down for the agent: it *is* the manifold under the agent's capability, projected — over
MCP, into an editor's generated commands, into a shell's completions. The description an
endpoint carries is the tool definition, which is why [Why an endpoint describes
itself](getting-started/self-description.md) calls it load-bearing.
