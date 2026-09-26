<!-- DRAFT for placement (ikigai-www or a Bounded chapter — Brian's call). Not linked from
     the book and not deployed. Mirrors the book chapter "The shape of resolution"
     (books/ikigai/src/getting-started/shape-of-resolution.md); same five ideas, same coda,
     with room for the pictures. Each picture is described in one sentence a designer could
     draw from. -->

# The shape of resolution

There is a way of building software in which nothing is ever called. A program asks for a
*name*, and something between the program and the world decides what that name means right
now — where the answer lives, whether it was computed a moment ago and kept, who is allowed
to have it, and in what form. NetKernel has worked this way for twenty years; the Web works
this way whether or not anyone means it to; ikigai is my attempt to make it the ordinary
way to write a program. Peter Rodgers, who co-designed NetKernel, has recently written the
model down as set theory, and the surprise of reading it was how little of it is about
computers. Five things fall out that are true of any system that resolves names, and they
can be carried as pictures rather than proofs. Here they are — the doors and corridors are
Peter's; the keys, the vault, and the clock are the parts I had to add to make the code
honest.

## 1. A resource is a function of its name *and* its context

Picture a hotel where every door has a keypad. A door does not hold one code; it
recognizes a *family* of codes — one exact code, or a pattern with a blank in it. A
corridor is a row of such doors, tried in order, and the first door that recognizes your
code is the one that opens. Some corridors end in another corridor. Some have a whole wing
grafted on under a prefix. Some have a door that, rather than opening, rewrites your code
and sends you down the corridor behind it.

*Picture: a single code arriving from the left at a row of keypad doors labeled "corridor
A"; a dotted arrow continues past the last door into a second row labeled "corridor B" and
past that into empty space, with a caption under corridor A reading "the one you are
standing in decides."*

Now the thing that a function call has no word for. What a code opens is not a property of
the code. It is a property of the code *and the corridor you are standing in*. Write it once
and then stop: a resource is ρ(name, context). The same name on two machines is two
resources. The same name in two corridors is two resources. They may answer alike, and
nothing about the name says so.

Every term in a resolving system is a piece of this picture. A grammar is a door. A space
is a corridor with first-match-wins. A fallback is a corridor that ends in another. A mount
is a wing under a prefix. A rewrite — an alias, when it is a table — is the door that
changes your code and tries again. And the corridor you are standing in is the *context*,
which in ikigai is a thing a request carries: a named chain of corridors to try ahead of
the host's own, chosen per request. Resolve `title` in the host and you get the host's
title; resolve it standing in a corridor that binds the same name to a different door and
you get the corridor's. One name, two contexts, two resources — and, as the next idea
insists, two cache entries.

The chain has one more property worth stating early. A piece of code running inside the
system can *narrow* the chain for its own sub-requests — put a corridor where the root was
and cut the root off, so that a name the corridor does not bind has nowhere to go. It cannot
widen the chain, and it cannot get ahead of a corridor the host put there. That asymmetry
is idea 3 wearing a different coat, and it is the reason the chain can be exposed at all.

## 2. A cache is sound only if its key holds everything the answer depended on

A cache is a vault of answers, each behind a key. The rule is short and the consequences
are not: the key must have one slot for every fact the answer turned on. Name. Arguments.
Who asked. Which corridors were consulted.

*Picture: a physical key whose bit has four distinct notches, each labeled — "name",
"arguments", "who asked", "which corridors" — and beside it a second key with only three
notches, opening the same lock, captioned "served to someone it was not computed for."*

Leave a slot out and the vault hands an answer to someone it was not computed for — another
caller's arguments, another caller's authority, another corridor's door. This is the
failure to fear, because it is not a *miss*. It is a hit. Hits are fast and quiet, and
nothing logs them.

ikigai's key has three slots that cover the four facts: a content-addressed identity of the
request (verb, name, and every argument, including the format asked for); a fingerprint of
the capability it ran under; and a fingerprint of the chain it was resolved in. The chain
slot holds the whole chain rather than the corridors actually consulted, which is the sound
side of the trade-off: two chains that differ only in a corridor neither request touched do
not share an entry, and that costs some cache hits rather than any correctness.

The rule that actually bites people is a corollary. Because a slot has to hold every
dependency, **an answer is only as cacheable as the least cacheable thing it depended on.**
The cautionary tale is three sentences long. On a live server, one more source was joined
into a cached resource — a correctness no-op, the kind of change that does not warrant a
second look. Every read went from about twenty microseconds to about one second. Sixty-eight
tests stayed green, because the types are identical either way. Nothing catches this except
measuring the read before and after, which is now a rule in that codebase and ought to be a
rule in yours.

There is a converse mistake: a slot too many. Two names for one resource make two entries
to fill and two invalidations to fire, and a write through one name invalidates only what
that name declared. So in ikigai, whoever rewrites a name *reports* it, and the kernel keys
the cache, cuts the thread, and checks authority on the name reported — the one the
resource was actually reached by. Set the two moves side by side and you have the whole
idea in one line: a corridor splits one name into two resources; a reported rewrite joins
two names into one; and in both cases what changed is the key.

## 3. Authority only ever shrinks

A capability is a set of scopes — a ring of keys, to stay in the hotel. When a piece of
code resolves a sub-request on someone's behalf, the sub-request runs under that someone's
ring or a subset of it. Passing authority on can *intersect* it; it can never widen it. So
along any chain of calls, authority is a staircase that only goes down.

*Picture: a descending staircase seen from the side, each step labeled with a shrinking set
— {read, write, net}, {read, write}, {read} — and one step drawn in dashed outline going
back up, crossed out, labeled "the confused deputy."*

The one line of math is t ∩ s ⊆ t, and it is the reason the staircase has no up-step: an
intersection cannot contain anything either side lacks. The place an up-step *would* go has a
name. A module that acts under authority its caller does not hold — that reads a file
because the module's author could, not because the caller could — is the confused deputy,
the oldest hole in the access-control literature, and the way to not have it is to not
build the step. In ikigai there is no operation on the capability type that widens one.
Attenuation is intersection. When a capability arrives over a wire, it is clamped to what
the channel itself authenticated, which is the same intersection seen from the other side.
A client can spend a narrow slice of what it holds and know the narrowing will be enforced
even if its own process is compromised a second later.

Idea 2 already told you where this shows up in the vault: the capability is a slot in the
key. An answer computed under broad authority is never handed to a caller holding narrow
authority, not because anyone checks, but because the keys do not match.

## 4. Transreption keeps information; projection loses it

A resource does not have *a* format. The same image as PNG and as WebP is one resource
twice. The thumbnail beside them is something else.

*Picture: two framed copies of the same photograph side by side, labeled "png" and "webp",
with a double-headed arrow between them; to their right, a much smaller frame holding a
blurred version, labeled "thumbnail — a different resource."*

The one line is from information theory: H(f(X)) ≤ H(X), with equality exactly when f is
injective. Read it as a test. A conversion that loses nothing — one representation to
another, with the way back intact — can be undone on what it represents, so a kernel is
free to perform it on your behalf without asking. That is a *transreption*, and in a
resolving system "give me this as Turtle" is a routing question, not a serialization call:
the kernel looks for a converter that gets from what it has to what you asked for, one hop
or two through a hub format, and runs it. A summary, an embedding, a classification, a
thumbnail is not injective. It cannot be undone, so it is not a format of the resource. It
is a *projection*: a different resource, with its own name, that your code resolves on
purpose and that the kernel will never substitute for the original.

The test to apply is the inequality, not the file size. Gzip is a transreption; a
resolution ("Q4 revenue: 4.2 M") is a projection even when it is longer than the number it
came from.

## 5. The cost of serving an immutable answer is bounded by how fast reality changes

Two clocks tick at once. Reality changes at some rate. Readers ask at some other rate. In a
system where derivation is *recorded* — where each answer knows which threads it hangs
from, and a write cuts the thread — a reader never invalidates anything. Only a change
does. So a cached answer is recomputed at most once per change, and at most once per read
(you cannot recompute what nobody asked for).

*Picture: two horizontal timelines, "changes" with sparse ticks and "reads" with dense
ticks, and a third line beneath them marking a dot at the first read after each change;
the dots are as sparse as the changes.*

If both clocks are memoryless, the recompute rate is λ_o λ_c / (λ_o + λ_c) — and that is
the only formula in this essay, so it deserves its sentence of English. A recompute needs
a change and then the next read after it. The expected gap between recomputes is therefore
the expected wait for a change plus the expected wait for a read, and the rate is the
reciprocal of that sum. It is smaller than either rate on its own. When readers are fast
relative to change it approaches the change rate; when change is fast relative to readers
it approaches the read rate. Either way, the cost of serving an immutable answer is bounded
by how fast reality changes, not by how often it is asked for. Ten thousand readers of a
value that changes once an hour cost one recompute an hour. That is not a caching trick; it
is what caching *is*, once the key is complete and the invalidation is recorded rather than
guessed.

## What ikigai builds for each

Ideas are cheap and the code is where they get tested, so here is the honest ledger.

- **Name and context** — the resolution chain is a first-class thing a request carries: a
  named list of corridors tried ahead of the host's root, and a confinement operation an
  endpoint can use to narrow it from inside. Built, as of ikigai-core 0.1.72.
- **The key** — three slots: a content-addressed request identity, a capability
  fingerprint, and a chain fingerprint. Built. The chain slot is the whole chain, not the
  corridors consulted, which is sound and over-partitioned; the refinement is open.
- **Authority shrinks** — attenuation is intersection, clamping is intersection at the
  wire, and every capability an endpoint declares is one it enforces. Built, and the rule
  the whole system leans on.
- **Lossless versus lossy** — a transreptor declares the pair of formats it converts
  between, and the kernel selects on the declaration. Whether the conversion is actually
  injective is *declared, not enforced*: nothing today checks it. That is a gap, and it is
  stated as one.
- **The clock** — golden threads: every derived answer carries the threads it hangs from,
  a write cuts the thread named after its target, and a read never cuts anything. Built,
  and the reason a composite invalidates without ever naming what it depended on.

One sentence ties them together. A *pinned* context — a corridor that answers "now" with
the same "then" every time it is asked — turns an uncacheable question into an immutable
one, and each of the five is what makes that safe: the corridor is in the key, the
authority is in the key, the conversion to whatever form you asked for lost nothing, and
only a change to the pinned answer could ever cut it. Resolve *as of* a moment and you have
a resource that will answer the same way forever, at a cost bounded by how often that
moment changes — which is never. Context tourism, we have started calling it: the same
name, visited from somewhere else.
