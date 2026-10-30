# Resolve the viewer's authority once

Every request the kernel answers is answered under a **capability** — the authority of whoever is
asking. This chapter is about where that authority comes from, how it narrows, and the one rule
that makes the whole security model tractable: **the viewer's authority is resolved once, at the
edge, and there is no ambient root underneath it.** Nothing deep in the system quietly has more
power than the request that reached it.

If you have read The ikigai Book you have met capabilities and `clamp`. Here we follow the spine
end to end — capability, clamp, principal, grant, corridor — on the worked item, and watch an
attenuation take effect for real.

<a id="a-capability-is-a-resource-and-it-only-narrows"></a>

## Capabilities: a resource that only narrows

A capability is named like everything else (`urn:cap:…`) and carries a set of scopes. The one
operation that matters is **clamp** (attenuation): intersect a capability with a set of scopes to
get a *narrower* one. Clamp can only take authority away. There is no widen. That asymmetry is the
whole safety story: a request handed a clamped capability cannot climb back out of it, however deep
it goes, because every sub-request it makes is answered under the same narrowed authority.

File an item so there is something to act on, then narrow your own authority to a reader's and try
to write:

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:append Translate the research note'>
<pre class="ikigai-run-expected">#1 urn:iki:ledger:default:item:…
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='cap urn:cap:ledger:read:default urn:cap:store:read:graph:*'>
<pre class="ikigai-run-expected">narrowed — capability: urn:cap:ledger:read:default, urn:cap:store:read:graph:*</pre>
</div>

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:comment item=1 A note that will not land'>
<pre class="ikigai-run-expected">error: denied: capability does not grant `urn:cap:store:write:graph:*` (declared by `urn:iki:ledger:comment`; any grant under `urn:cap:store:write:graph:` satisfies it)</pre>
</div>

Two things to notice. The refusal is a **Denied**, not a broken write and not a silent no-op: the
kernel's capability check is the only authorization, so a write the authority does not cover never
reaches the store. And the refusal is **legible** — it names the exact scope that would have
allowed it. A person can read that, and so can an agent; a refusal is an authority fact, not an
opaque failure. Put your authority back before moving on:

<div class="ikigai-run" data-cmd='cap reset'>
<pre class="ikigai-run-expected">reset to identity — capability: root (full authority)</pre>
</div>

(In a real deployment you do not hold root; a door hands you exactly your grant and nothing wider.
The page gives you root so you have something to narrow *from*.)

## The principal: one identity, minted at the edge

Authority has to start somewhere, and in ikigai it starts at a **door** — the component that turns
a proof (a passkey assertion, a client certificate, a token) into a **principal**, a capability
scoped to one identity. One person is one principal whatever credential they used. The principal
reaches every endpoint at every depth of a resolution, and — this matters for the economics chapter
— it **partitions the cache**, so an answer computed under one principal's authority is physically
not reachable under another's.

A session, crucially, caches **authentication only, never authority**: "this is still Dana" is
cached; "Dana may close this item" is resolved fresh each request. Authority that was cached would
be authority that could not be revoked, and a system that cannot revoke cannot be trusted.

## The grant: what a delegation is made of

Between a principal and an action sits a **grant**: a small record with a grantee, a scope, and an
expiry (`notAfter`). Its reader is fail-closed — a grant that cannot be read is a grant that does
not apply. When a person delegates to an agent, the agent's grant is a **subset** of the person's,
checked by the kernel, and the delegation is revocable. This is the resource the next chapter's
consent screen renders: the security object and the UX surface are the same record read twice.

## Corridors: a clamp with a name

A **space** (The ikigai Book's dynamic spaces are the mechanism) can wrap other resources, and a
space that can only clamp is a **corridor**. A personal space is a corridor whose authority is the
viewer's own — "my items" is the ledger seen through a clamp to me. A tenant's graph behind a
corridor they lack the capability for is not *denied* to them; it is **not there** — the name does
not resolve at all, so there is nothing to probe. Corridors stack by intersection, so wrapping a
corridor in another corridor can only narrow further.

That is the spine: authority minted once at a door as a principal, carried as a capability,
narrowed by clamp and by the corridors it passes through, spent on grants the kernel checks and
nothing else trusts. Everything the person sees is built on top of it — which is the next chapter.
