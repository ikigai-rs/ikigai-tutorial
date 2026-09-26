# Names and where you stand

<!-- Draft: cites Peter Rodgers' paper, which is not yet public. Intended home once it is:
     a sidebar in "Scope and alias", after "Where you are standing". Prose only; no code,
     so nothing can rot. -->

Why is `urn:iki:fn:toUpper` so long? The function upper-cases a string. Its name says the
scheme, the ecosystem, the library, and only then the thing — four segments to reach one
word, and the reader who has just come from Python or a shell notices.

Peter Rodgers, who co-designed NetKernel with Tony Butterfield, gives the reason in
*Peter's Hotel: A Set-Theoretic Formalism*, and it is not a naming convention. A resource
is a name *and* a context — [The shape of resolution](../getting-started/shape-of-resolution.md)
wrote it as `ρ(name, context)` — so the information that picks out a resource can live in
either place. A name resolved in an empty context has to carry all of it. A name resolved
in a rich context can carry almost none. The paper states the trade as an inequality on
averages, and it reads in words like this: **the shorter the names, the richer the
context must be, on average.** Not may be — must be. Information that is not in the name
is somewhere, and the somewhere is where you are standing.

Three points on that line, from the short end.

**The null identifier.** The paper's limit case is the empty name: a request that says
nothing and is resolved entirely from context. It is not undefined. In a thin context it
resolves to nothing much; in a rich one — the paper's example is a parent waiting for a
call that has not come — it resolves to something specific and urgent. The silence is the
signal. ikigai has no empty name, because an IRI is absolute by construction; the nearest
thing it has is a *short* name bound by an injected corridor — the time resource pinned
in a corridor named for an instant is one — which is the next point on the line rather
than the end of it.

**Corridors and mounts.** A name inside a corridor is short because the corridor supplies
the rest. `Mount` is the static form: the prefix it fences is context the names inside no
longer need to repeat. `Scope` is the per-request form: a corridor injected ahead of the
root answers a name that, in the root, would have needed a longer spelling or would not
have resolved at all. An alias is the same trade written as a table — a short logical
name for a long backing one, with the difference supplied by the rule rather than by
where you stand. Every one of these is a way of moving information out of the name and
into the arrangement, and every one of them is why the arrangement has to be part of the
cache key.

**The long end.** With no corridor to stand in, a name must carry its whole context, and
that is the condition most of ikigai's names are used under today: the root is one
space, injected corridors are new, and a name like `urn:iki:fn:toUpper` is resolved by
callers who have no other way to say which `toUpper` they mean. The Web is the same
limit taken globally — a URL carries everything because a browser stands nowhere in
particular — which is why URLs are long, and why [The Web as an
instance](web-instance.md) ends on the same sentence.

So the length of the names in this book is not a convention to be tidied later. It is a
measurement of how much context the arrangement currently supplies, and it will shrink
exactly as far as the corridors grow.
