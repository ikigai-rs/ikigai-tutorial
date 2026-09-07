# Who is asking

The previous chapter got away with having no authentication code, because the operating
system had already answered the question. Put the same kernel on a network and nothing
answers it. A UDP datagram arrives from an address; addresses are not identities, and the
`0600` on a socket file protects nothing that is not a file.

So the QUIC transport has to answer three questions the IPC one could delegate: who is on
the other end, what may they do, and what happens to a certificate nobody has decided
about. The third is the one worth the chapter.

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
file edit: change the enrolment and the client loses its authority on its next connection,
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
