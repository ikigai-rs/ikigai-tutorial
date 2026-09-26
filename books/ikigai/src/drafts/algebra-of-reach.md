# The algebra of reach

<!-- Draft: cites Peter Rodgers' paper, which is not yet public. Intended home once it is:
     a sidebar in "Preferential resolution", plus one sentence in "The machine client"
     (the agent's tool list is reach intersected with authority). Prose and tables; no
     code, so nothing can rot. -->

[Preferential resolution](../beyond/preference.md) gave a mount three meanings and
[Scope and alias](../getting-started/scope-and-alias.md) added a fence, a rewrite and a
corridor. Read together they look like a bag of features. They are not; they are six
operations on one set, and knowing which set makes every one of them predictable.

The set is **what a request can reach** from where it stands — the names that some door
in its chain will answer. Peter Rodgers, who co-designed NetKernel with Tony Butterfield,
works this out in *Peter's Hotel: A Set-Theoretic Formalism*: each construct of
resource-oriented computing is a set operation on the reachable set, and the arrangement
of corridors determines the set by composing them in the order resolution walks.

| construct | on the reachable set | the one thing to remember |
|---|---|---|
| import | union | a corridor's doors are *added* to what lies beyond |
| limiter | difference | a family is *removed*, and stays removed in every enclosing corridor |
| mapper | preimage | the outer names that reach an inner family are those the rewrite sends there |
| interception | may subtract | recording and enrichment leave the set alone; **refusal** removes what it refuses |
| trapdoor | restriction to a new chain | what is reachable is what the enclosed corridor holds, and nothing outside it |
| projection across a boundary | restricted union | only doors declared visible are present on the far side — undeclared ones are *absent*, not refused |

The paper's own emphasis lands on the fourth row, and it is the sentence to carry: refusal
is the **one point in the algebra where a decision, rather than structure, bounds what can
be reached.** A limiter, a trapdoor and a projection bound a request by what the chain
*contains* — an excluded name has no door. An interception bounds it by a rule somebody
wrote, and a rule can be misconfigured for one request and right for the next. Where the
goal is a bound that cannot be got wrong per request, use the structural rows.

## The same table, in ikigai

| construct | in ikigai | status |
|---|---|---|
| import | `Mount` — an import restricted to a prefix | built |
| the chain | `Fallback` — ordered, first hit wins; and `Scope`, the per-request chain | built |
| order as meaning | the *override* and *prefer* mounts: composed before or after the local spaces | built (`ikigai-cli`'s mount kinds) |
| mapper | `Alias` and `Rewrite` — the preimage, with the rewrite reported so two names are one resource | built |
| interception | the capability floor on every door, and the transport's clamp | built, and it is a decision |
| trapdoor | `Confine` / `inv.confine` — restriction to `⟨host corridors, S⟩`, no root | built (core 0.1.72) |
| projection | the served kernel: a smaller root per process, plus the clamp on what a peer carries | built, as a choice of root |
| limiter | — | **not built**: resolution has no third outcome, so nothing can carve a family *out* of a chain; subtraction happens only by building a smaller root |

Two consequences of the table are worth stating, because each one settles a question
this book raised elsewhere.

**Reach is computable, and so the tool list is.** Every row is a set operation, so the
reachable set of a static arrangement is determined by the arrangement. That is why
`urn:kernel:actions` can exist at all: an agent's tool list is *reach intersected with
authority* — what the chain can resolve, restricted to what the capability admits — and
both halves are computed from the same declarations. (Today the manifold computes reach
by enumerating bindings; the day the arrangement is itself a resource, it can subtract
limiters and compose mounts by structure.)

**Absence reveals less than refusal.** A projection leaves an undeclared door absent, and
a limiter leaves a name with the same response as no door anywhere. A refusal is
distinguishable by construction: it says something was there. That is the asymmetry
[Who is asking](../beyond/identity.md) closes its second half on — an unbound name and a
refused one still answer differently in ikigai, and the structural rows are the way to
make them answer alike.
