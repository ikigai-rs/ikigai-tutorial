# The manifold, and the UI that falls out of it

The last chapter resolved a viewer's authority. This one asks the kernel a question that authority
answers directly: **what can I do here?** The answer is the **action manifold** — the set of
actions available to the caller *under their capability* — and the quiet claim of this chapter is
that the manifold a safe agent reads and the action list a page draws are **one list**.

## Watch the list narrow

File an item, then ask the kernel for the whole manifold. Under full authority it is long: every
ledger action, every store query, the kernel's own meta endpoints.

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:append Translate the research note'>
<pre class="ikigai-run-expected">#1 urn:iki:ledger:default:item:…
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='source urn:kernel:actions'>
<pre class="ikigai-run-expected">urn:iki:ledger:{ledger}:append
urn:iki:ledger:{ledger}:comment
urn:iki:ledger:{ledger}:close
urn:iki:ledger:{ledger}:claim
urn:iki:ledger:{ledger}:link
urn:iki:ledger:{ledger}:next
urn:iki:ledger:{ledger}:items
urn:iki:ledger:{ledger}:count
urn:iki:ledger:{ledger}:item:{id}
urn:iki:store:graph-update
urn:iki:store:graph-select
… (and the kernel's meta endpoints: urn:kernel:actions, urn:kernel:catalog, urn:kernel:validate, …)
[computed]</pre>
</div>

Now narrow your authority to a **reader's** — the ledger-read family and the store-read family —
and ask again:

<div class="ikigai-run" data-cmd='cap urn:cap:ledger:read:* urn:cap:store:read:graph:*'>
<pre class="ikigai-run-expected">narrowed — capability: urn:cap:ledger:read:*, urn:cap:store:read:graph:*</pre>
</div>

<div class="ikigai-run" data-cmd='source urn:kernel:actions'>
<pre class="ikigai-run-expected">urn:iki:ledger:{ledger}:next
urn:iki:ledger:{ledger}:items
urn:iki:ledger:{ledger}:count
urn:iki:ledger:{ledger}:item:{id}
urn:iki:store:graph-select
urn:kernel:actions
urn:kernel:validate
… (the read views and the ungated meta endpoints — and nothing else)
[computed]</pre>
</div>

`append`, `comment`, `close`, `claim`, `link`, `graph-update` are **gone**. Not greyed, not
disabled — absent. The manifold narrowed because it is computed *from your capability*, and a
capability that cannot write has nothing to say about a write action.

## Why this is the user interface, not a report about it

This is the crossover worth stating plainly: **an agent's tool list is the manifold under its
capability, and a page's action list is the manifold under the viewer's capability — one sentence.**

A page built this way draws its buttons from the manifold and its forms from each action's
self-description (its argument names, their types, which are required, which take an enumerated
value). The consequence is a UI that cannot lie about authority: an action the viewer lacks is not
rendered at all, so there is no disabled button inviting a person to wonder why, and no form field
offering input the action would refuse. The thing that makes an agent's tool list *safe* — it only
contains what the agent may do — is the thing that makes a human UI *honest*.

Put your authority back:

<div class="ikigai-run" data-cmd='cap reset'>
<pre class="ikigai-run-expected">reset to identity — capability: root (full authority)</pre>
</div>

## The manifold over-offers on purpose; the check is exact

There is a subtlety here that is itself a lesson. The manifold matches on a **family**: an action
appears when you hold *some* grant under its declared prefix (`urn:cap:ledger:read:*` means "some
ledger-read grant"). But a ledger grant names **exactly one ledger**, and a store-read grant names
**exactly one graph**. So holding the family wildcard is enough to be *offered* `next`, and not
enough to *resolve* it:

```text
denied: reading <urn:iki:ledger:graph:default> through `urn:iki:store:graph-select` needs the grant
`urn:cap:store:read:graph:urn:iki:ledger:graph:default`, which this capability does not hold. Every
graph a scoped read names must be granted: the read is refused rather than answered over the graphs
that are, because a narrower dataset than the one asked for returns rows that look right and are not.
```

This is deliberate, and it is the right shape. The manifold is a **menu**, and a menu may list a
dish the kitchen then declines to serve you; what it must never do is *omit* a dish you could have
had. Over-offering on the family and enforcing on the exact grant keeps the menu complete while the
kernel's check stays the only authorization. The declaration a UI reads is the wildcard; the
authority the kernel spends is exact.

## The consent screen is the grant, rendered

When a person delegates — to a teammate, to an agent — the screen that asks them to approve it is
not a separate security dialog. It reads the **same grant** the last chapter described and the same
self-description the manifold's forms come from, and it shows the attenuation **as a diff in human
words**: "this delegate gets *read* on *your project's ledger* — a subset of your *read and write*."
The security object (grantee, scope, expiry) and the UX surface (a sentence a person can approve)
are one record read twice. The devices-and-sessions page is the same grant from the other side: a
list with "last used" and a revoke button per row.

## Denied is not NotFound, and the difference is a UX decision

You saw a refusal above that named the missing scope. That is the **non-sensitive** case: the
kernel separates "you may not" (Denied) from "there is nothing here" (NotFound), and where
existence is not a secret it serves the Denied *plus* an explanation — the withheld-notice, which
tells a person (and an agent) exactly which authority would admit the action.

But an explanation is itself a signal that the thing exists. For a **sensitive** resource — one
whose very presence should not leak to someone who may not see it — the kernel collapses Denied
into NotFound: the "you may not" and the "there is nothing here" become the *same* response, so a
probe cannot tell them apart. The one hard rule underneath both cases is security-motivated and
non-negotiable: a refusal is never dressed up as a successful page. Whether a resource explains its
refusal or hides its own existence is a per-resource sensitivity decision — the same mechanism, the
error experience and the confidentiality control, turned to the light twice.
