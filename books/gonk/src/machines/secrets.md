# What gonk keeps secret

<!-- transcripts: normalize workdir -->

A gonk that talks to other machines holds a few things nobody else may read: the key it signs
onboarding entitlements with, the pairing code of each invite it has handed out, the private keys
of its QUIC door, and its passkey sessions. This chapter says where each one is kept, who can
reach it, and which of them will move when ikigai's secrets store becomes configurable. It also
says what gonk never holds.

What the secrets store itself is (`urn:secret:{name}`, its per-name scopes, the presence gate,
the macOS Keychain and the file store, and why the Keychain can prompt again) is general to every
ikigai host, so it is taught once, in [Secrets](../../operations/running/secrets.html) in
*Running ikigai*. This page is the gonk half, and uses those words without explaining them again.

Every fact below is from gonk's source at the revision this book pins
([`books/gonk/gonk.rev`](https://github.com/ikigai-rs/ikigai-tutorial/blob/main/books/gonk/gonk.rev)),
and each section names the file it comes from.

| What | Where it lives | Who can reach it | Source |
|------|----------------|------------------|--------|
| the onboarding controller's signing key | the secrets store, `ikigai-gonk.onboard.controller` (its public half `….pub`, and a copy in `<config home>/gonk/onboard/controller.pub.pem`) | gonk itself, for one signing at a time, on an approval only the owner's socket may make | `src/onboard.rs`, `src/onboard_init.rs` |
| each open invite's pairing code | the secrets store, `ikigai-gonk.onboard.invite.{id}`, until the invite is used | the owner's socket (mint, approve); a check reads it | `src/onboard.rs` |
| the QUIC door's private keys | files under `<config home>/gonk/quic/`, `0600` in `0700` directories | this user | `src/quic.rs` |
| passkey sessions | `<config home>/gonk/sessions.json`, each token only as its SHA-256 | this user | `src/identity.rs` |

## The onboarding controller's key, in the secrets store

When gonk onboards a node ([`src/onboard.rs`](https://github.com/ikigai-rs/ikigai-gonk/blob/ca4070f20c4732f2c35e04ad24869531eb0c2ba9/src/onboard.rs)),
it is the **controller**: it signs each node's entitlement with an Ed25519 key, and the layer host
verifies the signature with the public half. The private half lives in ikigai-secret's store,
under the name `ikigai-gonk.onboard.controller` (the `key=` word of the `gonk.onboard` line names
another), and never in gonk's own files.

`ikigai-gonk onboard init` ([`src/onboard_init.rs`](https://github.com/ikigai-rs/ikigai-gonk/blob/ca4070f20c4732f2c35e04ad24869531eb0c2ba9/src/onboard_init.rs))
mints it with `urn:secret:generate type=ed25519`, which writes both halves into the store and
answers only the public one. Here it runs on a file store in this page's directory, named with
`IKIGAI_SECRETS`, so nothing touches your own store. On a Mac the store is the Keychain, and
`generate` asks for Touch ID first: that needs a person at the machine, so it is not shown here.
The page sets up no layer host and no certificate set for one, so `init` stops after its first two
steps, says why, and exits 1; the key is minted all the same:

<!-- transcript: run -->
```console
$ IKIGAI_SECRETS="$PWD/secrets" ikigai-gonk onboard init --yes --door socket --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai"; echo "exit $?"
minted    the controller key `ikigai-gonk.onboard.controller` into the keystore; its public half is $PWD/.config/ikigai/gonk/onboard/controller.pub.pem
…
exit 1
$ ls secrets
ikigai-gonk.onboard.controller
ikigai-gonk.onboard.controller.pub
$ stat -c '%a %n' secrets secrets/ikigai-gonk.onboard.controller
700 secrets
600 secrets/ikigai-gonk.onboard.controller
```

(`stat -c` is GNU's; on a Mac, `stat -f '%Lp %N'`.) A rerun mints nothing: generate never
replaces a key, and `init` keeps the one whose public half is already on disk.

gonk reads the private half **for one call**: each signing reads it from the store, hands it to
`urn:sign:sign` in a kernel of its own that no door binds, and keeps nothing. And it reads it
only on an approval. Every onboarding write (invite, approve, revoke, apply) requires
`urn:cap:gonk:onboard:admin`, which no grant may hold, so only the owner's socket, whose
capability is root, can make one: never the passkey door, never a QUIC client. The reads of the
`/onboard` page need `urn:cap:gonk:onboard:read`, which a passkey's grant may carry, and none of
them answers a secret.

gonk binds no `urn:secret:{name}` on any of its doors. It reads the store directly, as a host,
so no grant can carry a scope that reaches these items, and the admin capability above is the
whole of who may cause a read.

## Each invite's pairing code, kept until it is used

An invite comes with a ten-character **pairing code**, shown once, which the node proves it holds
by posting `HMAC-SHA256(key = code, message = its certificate's fingerprint)`. A hash of the code
cannot check an HMAC keyed by the code, so the code itself has to be kept. gonk keeps it in the
secrets store, beside the controller key, as `ikigai-gonk.onboard.invite.{id}`; the ledger's store
holds only the invite's record and the code's SHA-256 (`src/onboard.rs`).

The code is read when a request is checked, and **deleted** when its invite is approved. After
that a check says the code "is no longer kept (it was used or withdrawn)", and the same proof can
never be checked again. Minting an invite and approving one are both admin writes, so both happen
only from the owner's socket. An invite lasts 24 hours; an expired one is refused at the check
whether or not its code is still in the store.

## The QUIC door's keys, in files

The QUIC door ([`src/quic.rs`](https://github.com/ikigai-rs/ikigai-gonk/blob/ca4070f20c4732f2c35e04ad24869531eb0c2ba9/src/quic.rs))
is mutual TLS with pinned certificates, and its keys are **files**, not secrets-store items:
`gonk/quic/server.crt` and `server.key`, this server's identity, made on first use; and for each
client minted here, a bundle in `gonk/quic/clients/<name>/` with `client.crt`, `client.key` and the
server's `server.crt` to pin. Minting a client is enough to see them:

<!-- transcript: run -->
```console
$ ikigai-gonk client add laptop --ledger default=read --port 1070 --config-home "$PWD/.config/ikigai" > /dev/null
$ stat -c '%a %n' .config/ikigai/gonk/quic .config/ikigai/gonk/quic/server.key .config/ikigai/gonk/quic/clients/laptop .config/ikigai/gonk/quic/clients/laptop/client.key
700 .config/ikigai/gonk/quic
600 .config/ikigai/gonk/quic/server.key
700 .config/ikigai/gonk/quic/clients/laptop
600 .config/ikigai/gonk/quic/clients/laptop/client.key
```

Every file there is written to a temporary file created `0600` and renamed into place, so a key is
never readable at the process's umask even for a moment, and each directory is made `0700`. A
certificate without its key, or a key without its certificate, is refused at start rather than
regenerated, because every client has pinned the certificate.

A client bundle holds that client's **private** key, and it belongs on the client:
[A QUIC client](quic-client.md) moves it there. A client that makes its own key and hands gonk
only its certificate (`client add --cert`) leaves no `client.key` on the server at all, and a node
that onboards itself does the same: it posts its certificate's fingerprint and proof, never a key.

## Passkey sessions

A passkey sign-in ([`src/identity.rs`](https://github.com/ikigai-rs/ikigai-gonk/blob/ca4070f20c4732f2c35e04ad24869531eb0c2ba9/src/identity.rs))
mints a 256-bit session token, which rides in the `gonk_session` cookie, set `HttpOnly` by the
server so no script on a `localhost` page can read it. gonk keeps the token only as its SHA-256:
in memory, and in `gonk/sessions.json` (written whole, readable by this user only) so that
sessions survive a restart. A stolen `sessions.json` names sessions but cannot present one. A
passkey's own entry in `gonk/clients.json` is its **public** key; the private key never leaves the
authenticator. Invite codes for passkeys are kept the same way, as hashes in `gonk/invites.json`.
Signing in needs a browser and an authenticator, so [The ledger in a browser](../ledger/browser.md)
shows it by hand, not as a transcript.

## What gonk never holds

- **A worker's GitHub token.** A worker pushes branches and opens pull requests with its own
  credentials, on its own machine ([Remote workers](../ways/workers.md)); gonk has no setting for
  one. When gonk runs `git` itself, a read of a local checkout removes `GH_TOKEN`, `GITHUB_TOKEN`
  and their enterprise forms from git's environment
  ([`src/git_env.rs`](https://github.com/ikigai-rs/ikigai-gonk/blob/ca4070f20c4732f2c35e04ad24869531eb0c2ba9/src/git_env.rs)),
  and `checkout`'s clones and fetches use the operator's own git credentials with prompting off,
  storing none ([`src/checkout.rs`](https://github.com/ikigai-rs/ikigai-gonk/blob/ca4070f20c4732f2c35e04ad24869531eb0c2ba9/src/checkout.rs)).
  An offer leaving for a shared layer is refused when its text looks like a token or a private key
  ([`src/egress.rs`](https://github.com/ikigai-rs/ikigai-gonk/blob/ca4070f20c4732f2c35e04ad24869531eb0c2ba9/src/egress.rs)),
  so a token pasted into an item does not travel with it.
- **A collaborator's keys.** What gonk holds of another party is public: a certificate it trusts
  and the fingerprint that maps to a grant (`src/quic.rs`), a passkey's public key
  (`src/identity.rs`), a node's posted certificate (`src/onboard.rs`). The private halves stay
  with their owners.

## What moves under the coming `[secret]` routes

ikigai-secret's decided design for configurable storage, which [Running ikigai describes and
says is not built yet](../../operations/running/secrets.html#what-is-coming), lets a host route
names to stores by pattern. Once gonk reads a `[secret]` section, a route for `ikigai-gonk.*`
decides where gonk's store items go:

- **They move with the route:** the controller key (`ikigai-gonk.onboard.controller` and its
  `.pub`) and every open invite's code (`ikigai-gonk.onboard.invite.{id}`). They are already
  secrets-store items, and gonk picks its store with ikigai-secret's own `default_backend()`, so
  a route changes where they are kept and nothing about who can cause a read. On a server, an
  `age-file` store keeps them encrypted at rest; on a Mac, one off the Keychain stops a service
  waiting on a prompt.
- **They do not move:** the QUIC door's certificates and keys, the passkey sessions and invite
  hashes, `clients.json` and `grants.json`. They are files in gonk's config home, not
  secrets-store items, and the design does not route files. A change to that is a change to gonk,
  not a route.

Until then, gonk on a Mac keeps the first group in the Keychain, and anywhere else in the file
store at `$IKIGAI_SECRETS` or `~/.ikigai/secrets`.
