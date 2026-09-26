# Authority, three ways

<!-- Draft: cites Peter Rodgers' paper, which is not yet public. Intended home once it is:
     the opening of the second half of "Who is asking", before "Identity is not
     authority". Prose and one table; no code, so nothing can rot. -->

[Who is asking](../beyond/identity.md) answers three questions in order — who is on the
other end, what may they do, what happens to a certificate nobody has decided about —
and then spends its second half on the distinctions between *who* and *what may happen*.
Peter Rodgers, who co-designed NetKernel with Tony Butterfield, makes the same cut in
*Peter's Hotel: A Set-Theoretic Formalism* and gives each question a construct of its
own. Laying the two side by side is the shortest way to see what ikigai has, what it
decided against, and where it is honestly weaker than the model.

## Three questions, three constructs

| the question | the paper's construct | what it does | in ikigai |
|---|---|---|---|
| **who** is asking | the gatekeeper — an overlay every path to a protected corridor is meant to pass through | verifies the request's signed context, records it, may refuse it | the door: a pinned certificate, a passkey, a socket's peer credential; per-identity grants; the transport stamping the principal on a write |
| **what** may be asked for | the mapper | subsets or renames the interface | the served surface (a smaller root per process), `urn:kernel:actions`, `Alias` |
| what the answer may **touch** | the trapdoor | confines the computation to a corridor with no parent | `Confine` — plus the capability floor at every hop |

Each construct is independent of what it encloses, so all three can be placed around an
existing corridor without changing it — the property that lets a host add authority to a
space it did not write. And the order is not free. The gatekeeper goes outermost, because
it may resolve further resources in the requester's own context to decide; the trapdoor
goes innermost, because it severs exactly that context; the mapper sits between them,
where it can name public things on the way in and internal things on the way out.

ikigai's arrangement is the same order with one substitution, and the substitution is
the interesting part. Where the paper has three structural constructs, ikigai has two
structural ones and a value that travels: the capability. The gatekeeper's *refusal* is
the capability floor, evaluated at every door rather than at one; the mapper is a choice
of root and a table; the trapdoor is `Confine`. The staircase — authority only shrinks
along a chain of sub-requests — is what makes the travelling value safe to carry inward,
and the paper has no counterpart for it because its authority is decided at the
gatekeeper and then not carried at all.

## Decision, or structure

The paper's sharpest sentence about authority is about *what kind of thing* a bound is.
Limiters, trapdoors and projections bound a request by what the chain contains: an
excluded name has no door. A refusal is different — it is a decision, taken by an
endpoint, and a decision can be misconfigured for one request, bypassed by another
path, or made to contradict a second decision elsewhere. Where the goal is a bound that
cannot be got wrong per request, the structural constructs are the ones to use.

Put ikigai on that line and the honest statement is this. **Capabilities are ikigai's
decisions.** Every door checks a declaration against a value, the check is uniform and
the declaration is the contract the manifold reads — which is as good as a decision
gets, and still a decision: a scope can be granted too widely, a description can omit
what the code enforces, and the ecosystem has paid for both. **The chain is ikigai's
structure.** A mount fence and a confining corridor leave a name with nowhere to go, and
there is nothing in them to misconfigure per request; that is why the corridor exists
beside the capability rather than instead of it.

And the place where the two still show through: an unbound name answers `Unresolved` to
everyone, a bound-but-refused one answers `Denied`. Resolution runs before the floor,
deliberately — the floor must check the backing name after any rewrite, and it must gate
cached answers too — so a caller holding nothing can learn which names exist by asking.
The paper's limiter would leave the requester with the same response as no door
anywhere; ikigai has no limiter yet, and a served surface that omits a corridor reveals
nothing while a capability that refuses reveals that something was there. Names, not
content, and on every surface served today the callers are the owner or an enrolled
peer. The structural answer is available when it matters: put the name behind a fence or
inside a corridor, and the refusal never has to be made.

Three questions, then, and a fourth the paper leaves to the embodiment: not *who*,
*what*, or *touch*, but **how far down** — and that one is the capability's, all the way
to the bottom of the staircase.
