# Who is asking

The previous chapter got away with having no authentication code, because the operating
system had already answered the question. Put the same kernel on a network and nothing
answers it. A UDP datagram arrives from an address; addresses are not identities, and the
`0600` on a socket file protects nothing that is not a file.

So the QUIC transport has to answer three questions the IPC one could delegate: who is on
the other end, what may they do, and what happens to a certificate nobody has decided
about. The third is the one worth the first half of the chapter. The second half is the
distinctions a reader needs once *who* and *what may they do* stop being the same
question — and they stop being the same question the moment a kernel is served.

## Trust without an authority

`ikigai-quic` runs the same session over QUIC — TLS 1.3, one bidirectional stream per
call — with **mutual certificate pinning and no certificate authority**. Each side has a
self-signed identity and the exact peer certificate it will accept: the client pins the
server's, the server requires and pins the client's.

```bash
ikigai cert generate            # a server identity and a client identity
ikigai cert add-client laptop   # an ADDITIONAL client identity, and its fingerprint
```

A CA would buy delegation — the ability to trust certificates you have never seen because
somebody you trust vouched for them. Between a handful of machines one person owns, that
is a cost with no matching benefit: you would run the CA, so trusting it is trusting
yourself with extra steps. Pinning says the same thing with no infrastructure.

There is a second gate in the handshake worth knowing about, because its failure mode is
confusing if you do not: the wire version is the **ALPN protocol id**. A peer speaking a
different wire version does not connect and get a polite error; it fails the TLS handshake.

## The certificate is the credential

Once the handshake succeeds, the certificate stops being a login and becomes the
principal. Two ids come off it, and the crate is emphatic that they are not
interchangeable:

- **`fingerprint`** — lowercase hex SHA-256 of the leaf certificate's DER. This is the
  *identity*: what an operator writes in a config file. Two hard requirements follow from
  that one sentence. It has to be stable forever, which rules out Rust's `DefaultHasher`
  (explicitly not guaranteed stable across releases — keying a config on it would silently
  re-map every enrolled client on some future toolchain upgrade). And it has to be
  *obtainable*, so it is byte-for-byte what `openssl x509 -noout -fingerprint -sha256`
  prints, minus the colons and the case.
- **`segment_id`** — a *namespace*, not an identity. It names the tenant directory this
  client's `urn:file:` names land in, so it can never be recomputed differently without
  orphaning data on disk. It is still the old unstable hash, deliberately and with the
  reason written next to it.

They are passed as a struct rather than as two positional arguments precisely so a call
site has to say which it means.

## The minter, and the `?`

A `Minter` turns that identity into the session — the capability every call on the
connection is bounded by — or refuses the connection outright:

```rust,ignore
{{#include ../../../../crates/two-hosts/src/identity.rs:minter}}
```

The signature is `Arc<dyn Fn(&PeerIdentity) -> Option<Session>>`, and **`None` refuses**.
The server closes the connection with a distinct application code, `UNAUTHORIZED`, whose
message says exactly what happened: the certificate authenticated, and no authority is
configured for it. That is a different event from a failed handshake, and it is reported
as one, because "I do not know you" and "I know you and nobody said what you may do" have
different fixes.

The rule the whole design hangs from, in one sentence from the source:

> A host that cannot decide what a certificate may do must not fall back to a shared
> ceiling or to root.

There is no `unwrap_or` in that listing and there is not supposed to be one. The tempting
alternative — hand an unrecognized client the same ceiling everyone else gets — is how a
forgotten config entry silently becomes an over-grant, and it fails in the direction where
nothing goes wrong until it goes very wrong.

The session is minted **per connection and never cached**, which turns revocation into a
file edit: change the enrollment and the client loses its authority on its next connection,
rather than at the end of some token's lifetime.

## Carrying a capability, and being clamped

A client can also carry a capability with a request — `IssueAs` — and the server does not
take it at face value. It resolves under `session.capability.clamp(&carried)`.

`clamp` keeps only what **both** grant. A peer carrying root gets the session's ceiling; a
peer carrying scopes gets the intersection. There is no operation anywhere in the
capability type that widens one, so a caller can attenuate itself and can never exceed the
principal it authenticated as.

Which raises the obvious question — why carry one at all, if it can only take away? Because
taking away is the point. A client that holds broad authority and is about to hand work to
an agent, a script, or a mount can spend a narrow slice of what it holds and know the
server will enforce the narrowing even if its own process is compromised a second later.

The book's tests state both halves: an unenrolled fingerprint gets `None`, and a carried
capability naming a scope the session does not hold comes back not holding it.

## Three postures, chosen by what the operator configured

A served kernel decides authority in one of three ways, most specific first:

1. **Per-identity grants** — a `clients.json` maps fingerprint to a named grant, and the
   session capability is a function of *which* certificate authenticated. Unenrolled means
   refused; a shared default exists only if the operator wrote one explicitly, because
   absence must never imply one.
2. **A fixed ceiling** — `serve --cap urn:cap:personal:calendar:read:freebusy` gives every
   authenticated client the same authority and nothing else. It also remains the outer
   bound under posture 1.
3. **Neither** — each client gets its own filesystem workspace, transparently rooted at its
   own segment, so a tenant addresses files as if its segment were the root and cannot name
   another's.

Two details in there are worth carrying away as habits rather than as facts about this
crate. A `clients.json` that exists but does not parse **stops the server at startup**: a
broken authority config must not be allowed to degrade into posture 2 or 3. And a grant
whose file scopes name paths no client of that server could ever address is also refused at
startup — it looks like a narrow grant and grants nothing, and a silently inert authority
config is the exact failure the posture exists to prevent.

One limitation, stated plainly: **the served surface is chosen once, at startup**, from the
union of `--cap` and every enrolled grant. Enrolling a grant that needs an endpoint family
the server is not currently offering takes a restart. What is per-connection is the
authority, not the manifold; the clamp is what makes serving one surface to differently
scoped clients safe.

## And a warning about names

The transport can also find its peers by name, announcing over mDNS, so a mount can say
`peer:plasma` instead of an address — useful, because addresses move and names do not.

The rule that comes with it belongs in this chapter rather than the next one:
**discovery supplies an address, never trust.** An announced name is attacker-controlled;
anything on the network can claim to be `plasma`. So a mount by name still requires a
pinned certificate for that name, and an impostor gets a failed handshake rather than a
conversation. What the name buys is ergonomics — it determines the address by announcement
and the identity by convention — and nothing else.

## Identity is not authority

Everything above turned a certificate into a session capability, and it is easy to
leave the chapter believing the two are one thing. They are not, and the rest of this
part depends on keeping them apart.

The **principal** — *who* asked — is a fact a door establishes: the certificate that
authenticated, the passkey that signed, the socket's peer credential. The
**capability** — *what may happen* — is a value that gates. Neither derives from the
other. An anonymous caller over the loopback socket has full authority and no identity
at all; a signed-in caller holding a narrow grant has an identity and very little
authority. What the door does with the principal is *stamp* it: the HTTP transport in
`ikigai-cli` 0.1.27 attaches the authenticated principal to every mutating request as
provenance, read off the connection and refused from the query string, so a write
arriving from a browser carries who made it and nothing the caller typed can forge it.
A ledger records that stamp as the author.

What core does *not* do yet, stated plainly: the request carries no principal. A
capability travels with every invocation; an identity stops at the door that
established it, and an endpoint that wants to know who asked has to be handed the fact
by its host. That is a known gap with a design in progress — an optional principal IRI
beside the capability, readable, never itself authority — and until it lands the
attribution the ledger records is the transport's doing, not the kernel's.

## Authority only shrinks

A capability is a set of scopes, and the only operations on one are intersections.
`attenuate` narrows a held capability to the scopes it names; `clamp` intersects a
carried capability with the ceiling a channel authenticated. There is no join. So along
any chain of sub-requests authority is a staircase that only goes down, and an endpoint
can choose to step down *on purpose*: `inv.issue_attenuated(request, scopes)` and
`inv.source_attenuated(&iri, scopes)` (core 0.1.71) issue a sub-request under this
invocation's capability narrowed to `scopes`. The case that wants it is a module
dereferencing an IRI **its caller named** — resolving a target it did not choose, with
every scope its caller happens to hold. Dropping the scopes it does not need turns "the
caller named the secret and my caller may read secrets" from an exfiltration into a
refusal:

<!-- urn-gate: illustration urn:iki:tutorial:secret — a gated resource bound only inside
     the block below. -->
<!-- urn-gate: illustration urn:iki:tutorial:careless — an endpoint bound only inside the
     block below: it dereferences a caller-named IRI with everything the caller holds. -->
<!-- urn-gate: illustration urn:iki:tutorial:careful — its twin, which narrows first. -->

```rust
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{
    ArgRef, AsyncFnEndpoint, Capability, Description, EndpointSpace, Error, Exact,
    FnEndpoint, Iri, Kernel, ReprType, Representation, Request, Verb,
};

// A resource that requires a scope to read.
let secret = FnEndpoint::new("secret", |_| {
    Ok(Representation::new(ReprType::new("text/plain"), b"s3cr3t".to_vec()))
})
.with_description(
    Description::new("secret").verb(Verb::Source).requires("urn:cap:tutorial:secret:read"),
);
// Two endpoints that fetch whatever IRI the caller passes in `from`.
let careless = AsyncFnEndpoint::new("careless", |inv| {
    Box::pin(async move {
        let from = Iri::parse(inv.inline_str("from")?).map_err(|e| Error::Endpoint(e.to_string()))?;
        inv.source(&from).await
    })
});
let careful = AsyncFnEndpoint::new("careful", |inv| {
    Box::pin(async move {
        let from = Iri::parse(inv.inline_str("from")?).map_err(|e| Error::Endpoint(e.to_string()))?;
        // Fetching a caller-named resource needs no secret-reading authority.
        inv.source_attenuated(&from, ["urn:cap:tutorial:fetch"]).await
    })
});
let kernel = Kernel::new(Arc::new(
    EndpointSpace::new()
        .bind(Exact::new("urn:iki:tutorial:secret"), secret)
        .bind(Exact::new("urn:iki:tutorial:careless"), careless)
        .bind(Exact::new("urn:iki:tutorial:careful"), careful),
));
// The caller may read secrets — and points both endpoints at one.
let caller = Capability::root().attenuate(["urn:cap:tutorial:secret:read", "urn:cap:tutorial:fetch"]);
let fetch = |through: &str| {
    Request::new(Verb::Source, Iri::parse(through).unwrap())
        .with_arg("from", ArgRef::Inline(b"urn:iki:tutorial:secret".to_vec()))
};

let leaked = block_on(kernel.issue(fetch("urn:iki:tutorial:careless"), &caller)).unwrap();
assert_eq!(String::from_utf8_lossy(&leaked.bytes), "s3cr3t");

let refused = block_on(kernel.issue(fetch("urn:iki:tutorial:careful"), &caller)).unwrap_err();
assert!(matches!(refused, Error::Denied(_)));
```

The narrowing is voluntary — `careful` still holds the caller's capability and could
have called `source` — so it defends the endpoint's *downstream*, not the endpoint. That
is the only thing an in-process, self-applied restriction can honestly claim, and it is
worth having because the sub-request is the one place an endpoint hands control of the
*target* to somebody else while keeping control of the *authority*.

Two ways authority could grow, and why both are absent. **Delegation** — an endpoint
acting under authority *it* holds rather than its caller's — is the confused deputy by
another name: a module with store-write authority that passes a caller's string into a
privileged sub-request has handed its authority to the caller. Core has the design (a
grant table only the host can write, keyed on the canonical target, replacement never
union) and deliberately has not built it; the cost today is that a module built on a
gated module makes its callers hold the underlying grant too. **Ambient authority** —
an endpoint reaching the disk or the network without a request — is structurally
impossible for a WebAssembly module, whose only import is the host callback, and is a
matter of review for in-process Rust, where `std::fs` is one line away. Both are stated
here because a reader who has only "capabilities gate authority" will go looking for
them.

The fact underneath all of it: an endpoint has no way to hand a capability to the
kernel. `Capability::root()` is a public constructor, so any code can *build* a strong
capability; what it cannot do is *issue under it*, because the one path back into the
kernel — the invocation — has never offered to take one. An authority-carrying
sub-request takes its authority from the kernel, never from its caller. That single
private seam is what makes the staircase structural rather than polite.

## Declared is enforced — and who enforces

The description is the contract. The kernel checks every scope an action `requires`
**before dispatch and before the cache lookup** — so a refused caller neither enters the
endpoint nor is served a cached answer somebody else computed (core 0.1.49). An
endpoint may add a *finer* ceiling of its own on top: the file workspace's jail is one,
a store's per-graph effect check is another. What it may not do is enforce a scope it
never declared, because the offer, the pre-flight and the enforcement are all read from
the same declaration.

One trap in the declaration itself, and it has shipped: **the two authoring forms are
exclusive per verb.** A flat `.requires(..)` on the description is the contract for
every verb that has no explicit `ActionSpec`. Declare an `ActionSpec` for a verb and its
`requires` *replaces* the flat one for that verb — an explicit action with an empty
`requires` declares that the verb needs no authority, and the flat scope beside it is
dead for that verb. Reading "one action per verb" as "one action per verb, in addition"
is how a published module demanded a broad read token on one of its actions while its
flat sibling was right. The normalized view says which is which:

```rust
# extern crate ikigai_core;
use ikigai_core::{ActionSpec, ArgSpec, Description, Verb};

let mixed = Description::new("note")
    .verb(Verb::Source)
    .requires("urn:cap:tutorial:note:read")
    .action(ActionSpec::new(Verb::Sink).input(ArgSpec::new("content")));

let specs = mixed.action_specs();
let source = specs.iter().find(|a| a.verb == Verb::Source).unwrap();
let sink = specs.iter().find(|a| a.verb == Verb::Sink).unwrap();

// Source had no explicit action: the flat `requires` is its contract.
assert_eq!(source.requires, vec!["urn:cap:tutorial:note:read".to_string()]);
// Sink was declared explicitly and said nothing about authority — so it needs none.
assert!(sink.requires.is_empty());
```

Restate on each explicit action whatever it still needs, and no more.

The reader-facing consequence: the tool list an agent is handed is exactly what it may
invoke — `urn:kernel:actions` computes reach intersected with authority from the same
declarations the kernel enforces — so the manifold cannot lie by omission. One
qualification, from [Scope and alias](../getting-started/scope-and-alias.md): selection
does not yet know the chain, so inside a confinement the manifold can offer an action the
chain cannot resolve. Names, not authority; and not yet.

## What the cache knows about authority

A cached answer is filed under the authority it was computed under, so two callers with
different grants never share an entry, and a result computed inside a confinement is
never served outside it — nor the other way round. That is why `.cacheable()` is safe on
an endpoint whose answer depends on who is asking. Two callers holding the *same* scopes
do share, which is the point: the fingerprint is of the authority, not the identity.

The hole, stated: a result built on a **denial** is cached like any other. A change of
grant has no golden thread, so an endpoint that catches `Denied` and returns a cacheable
fallback serves that fallback after the grant is widened. The fix — no caching at all for
a result built on a refusal — is designed with the rest of the cache-soundness work and
not built.

## Denied, or nowhere

Resolution runs before the capability floor. That order is deliberate — the floor has
to check the *backing* name after any rewrite, and it has to gate cached answers too —
and it has a consequence worth knowing when you serve a kernel to strangers: an unbound
name answers `Unresolved` to everyone, a bound-but-refused one answers `Denied`, so a
caller with no authority can still learn which names exist. Names, not content; and on
every surface served today the callers are the owner or an enrolled peer.

Where that leak matters, the answer is structural, not a smarter refusal: a mount fence
or a [confining corridor](../getting-started/scope-and-alias.md) that leaves the name
with nowhere to go, indistinguishable from a name nobody ever bound. A refusal is a
decision, and a decision can be misconfigured; an absence cannot. Capabilities are
ikigai's decisions; the chain is its structure; use the one whose failure mode you can
live with.
