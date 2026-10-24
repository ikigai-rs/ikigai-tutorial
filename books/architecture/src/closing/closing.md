# Presence and the audit axis

Two patterns close the loop the rest of the book opened: the one that puts a **person** in the
loop at the moment of a consequential action, and the one that keeps the **record** of what was
done. They are the human-in-the-loop and its provenance — the front and the back of a single
accountable action — and both are drawn here as pictures, because a real passkey ceremony and a
real on-disk log need parts of the system a page cannot stand in for.

## Presence: a prompt that is a proof

Some actions should require a *person*, present, right now — not a session that logged in an hour
ago and not an agent acting on their behalf. Closing a release, approving a payment, merging a
change: these want **presence**, a capability scope that is satisfied only by a fresh, person-only
assertion (a Touch-ID or security-key gesture), minted for *this* request and never held by a
session or named in a stored grant.

```text
   person            the button                     the door
     │          "Close #2 — confirm"                   │
     │ ───────────────────────────────────────────────▶│  mint a challenge bound to the action:
     │                                                  │     nonce  ‖  H( verb ‖ target IRI ‖ digest(content) )
     │ ◀─────────── Touch-ID prompt names the action ───│
     │  gesture                                         │
     │ ───────────────────────────────────────────────▶│  verify the assertion over THAT challenge
     │                                                  │  → mint a one-request presence scope
     │                                                  ▼
     │                                          resolve the close, now
```

Read it twice, as before. The **security** property is that the challenge is bound to the exact
action — the verb, the target, and a digest of the content — so a captured assertion cannot be
replayed against a *different* action: you cannot approve "close #2" and have it count as "close
#7," or as the same close over different content. The **UX** property is that the prompt names
exactly what you are confirming, in words. Those are the same binding. A refusal (you declined, or
the gesture failed) is a **Denied** — the action does not happen — never a success page that
quietly did nothing.

Presence does not cross a network mount by default: a peer cannot assert *your* presence. That is
the right default, and it is the same "authority does not widen" rule the spine was built on.

## The audit axis: provenance as a first-class record

Every consequential action leaves a line on the system's **time axis** — an append-only, ordered,
tamper-evident log. Each entry carries *who* (the principal, stamped by the door, not claimed by
the caller), *what*, and *when*, and the entries form a provenance graph: this answer was attributed
to that principal, derived from those sources, at that time.

This is primarily a **security** pattern — accountability, non-repudiation, the ability to ask
"what happened, and on whose authority" after the fact. But it has a plain **UX** surface. The
"last used" on a devices-and-sessions page is a read of this log. Provenance shown beside an answer
is **citation** — an agent that tells you where its conclusion came from is reading its own trail.
The uniform per-request line a door writes is both an audit record and the raw material for every
"recent activity" view a person ever sees.

The principal column is what ties the axis back to the spine: because identity is minted once at a
door and reaches every endpoint, the log can attribute each line to the one principal responsible,
rather than flattening a whole process into "the server did it." Attribution is per request, the
way authority is.

## What was drawn, and what runs elsewhere

The passkey ceremony above can run for real in a browser that has a platform authenticator, but the
*downstream* enforcement — a door minting a one-request scope and a resolution consuming it — needs
the door, which is why this chapter is a picture. The hash-chained log is on disk, across rotations,
which a page has no file system for. Both are real in a deployment and drawn here, labeled as drawn,
with the same promise the overlay chapter made: a runnable version follows as the tooling to drive
it lands.

## The thesis, once more

You have now seen the worked item from every side. The grant you narrowed is the grant a consent
screen renders. The manifold you watched shrink is the action list a page draws and the tool list
an agent is handed. The door that would serve the item is the one place that authenticates and the
one place that escapes. The cache that made a second read cheap is the boundary that keeps your
answer yours. The governors that would make it resilient are constraints made explicit. And the
presence that gates its riskiest action is a prompt that is a proof, written into a provenance trail
attributed to you.

None of those was a security feature added to a product. Each was a property of resolving a name
under a capability — the one mechanism this book has turned to the light, chapter by chapter, until
both of its faces were in view at once. That is the ikigai architecture: not a UX stack and a
security stack, but one substrate, read two ways.
