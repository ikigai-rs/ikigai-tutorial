# The Web as an instance

<!-- Draft: cites Peter Rodgers' paper, which is not yet public. Intended home once it is:
     the front door — a sidebar after "Twenty minutes, nothing compiled", or the
     introduction's "Why bother". Prose and one table; no code, so nothing can rot. -->

If [the shape of resolution](../getting-started/shape-of-resolution.md) felt abstract,
here is the one instance of it you have used every day for thirty
years. Peter Rodgers, who co-designed NetKernel, makes the point in
*Peter's Hotel: A Set-Theoretic Formalism*: the Web is not a special
case of resource-oriented computing and not an ancestor of it. It is
the same model, taken to one limit — every name carries its whole
context — and its parts map onto the constructs this book has been
naming.

| on the Web | in the hotel | in ikigai |
|---|---|---|
| a URL | an identifier | an IRI |
| DNS delegation, root to TLD to domain | a short context chain — with no fallback, so nearer an opaque overlay keyed by suffix than an import | a `Mount` fence, or a mount over a wire |
| a server's routing table | a corridor: doors tried in order, first match wins | `EndpointSpace` |
| a reverse proxy or API gateway | an overlay; when it authenticates and records, a gatekeeper | the transport door: per-identity grants, the clamp |
| an HTTP redirect | close to a mapper — the name is rewritten, though the client re-issues it and nothing falls back | `Alias` / `Rewrite` |
| a browser's cap on redirect chains | the hop bound | `AliasTable::max_hops`, eight by default |
| GET, PUT, DELETE; HEAD | source, sink, delete; HEAD is close to *exists* | `Source`, `Sink`, `Delete`, `Exists` |
| the Accept header | choice of representation space, and transreption | `as=` and the transreptor route |
| HTTP caches, `max-age`, ETag | the representation cache; a fixed lifetime as validity, and a revalidation at each observation | the cache — but see below |

Three things the paper notes are visible in everyday practice, and the third is the one
to keep.

**Relative URLs are richer context, shorter names.** A page's base URL is context; a
relative link is a short identifier the context completes. That is the same effect
[Scope and alias](../getting-started/scope-and-alias.md) gets from a mount or a corridor,
and [Names and where you stand](names-and-context.md) is about why it is the *expected*
effect rather than a trick.

**The cap on redirect chains is the Web's answer to a cycle** — the same answer core's
alias table gives, a bound on hops, for the same reason.

**Web caching tracks no dependencies.** It has two models of validity, a fixed lifetime
(`max-age`) and a check at every observation (an ETag round trip that never serves stale
data and never saves the trip), and neither knows *what an answer was derived from*.
Dependency-driven invalidation — a golden thread cut by the write that changed the thing
— is what distinguishes the model's cache from both. It is the one row of the table where
ikigai does not merely instantiate the Web's idea but does something the Web cannot, and
it is the row [What resolution buys you](../getting-started/payoff.md) spends a chapter on.

And the reason the table is worth a page: **the Web is this model at the limit where every
name carries its whole context.** That is why URLs are long. It is also why ikigai's URNs
are long — `urn:iki:fn:toUpper` says everything about where it lives because, most of
the time, there is no corridor around it to say any of that for it. Corridors are what
would let the names get shorter.
