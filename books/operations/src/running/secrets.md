# Secrets

<!-- transcripts: normalize workdir -->

An API token, a signing key, a password for a mail relay: a kernel that does real work needs a few
values nobody else may read. The `ikigai` binary keeps them in a **secrets store** and serves them
as resources, so a script names a secret by its IRI (`key=urn:secret:shop.sign`) and never holds
the value in its own text. This chapter is about what that store is, who may read it, and where it
keeps things on your machine today.

It is general: it is true of any host that mounts `ikigai-secret`, the `ikigai` binary first. What
gonk keeps in the same store, and why, is [What gonk keeps secret](../../gonk/machines/secrets.html)
in The gonk Book, the other half of this chapter.

## The resource

[`ikigai-secret`](https://github.com/ikigai-rs/ikigai-secret) binds one template and two reserved
names:

| IRI | Verbs | Needs |
|-----|-------|-------|
| `urn:secret:{name}` | Source (read), Exists | `urn:cap:secret:read:<name>` |
| `urn:secret:{name}` | Sink (store, replacing) | `urn:cap:secret:write:<name>` |
| `urn:secret:{name}` | Delete | `urn:cap:secret:delete:<name>`, then presence |
| `urn:secret:generate` | Source, `into=<name> type=ed25519\|age` | `urn:cap:secret:generate` and the write scopes for `<name>` and `<name>.pub`, then presence |
| `urn:secret:unlock` | Source | `urn:cap:secret:unlock`, then presence |

Four rules hold for every one of them:

- **A scope per name, and nothing by default.** Holding `urn:cap:secret:read:shop.api-token`
  reads that one secret and no other. A capability that names no secret scope reads none.
- **Refused before the store is asked.** The capability check runs first, so a caller without the
  scope gets the same refusal for a name that exists and a name that does not: it cannot learn
  which names are there.
- **Never cached.** A secret read is uncacheable, so every read reaches the store, and nothing
  keeps a copy after the request that asked for it.
- **Presence for what cannot be undone.** Unlock, generate and delete ask the **presence gate**
  after every capability and argument check. On a Mac that is a Touch ID prompt; elsewhere the
  gate does nothing today.

Run the commands below in one working directory, starting empty. Every command names the store
directory with `IKIGAI_SECRETS="$PWD/secrets"`, so nothing here touches your own store (the
next section says where that is). On a Mac the binary uses the Keychain instead and ignores
`IKIGAI_SECRETS`, so run this page on Linux, or read it there.

## Store, read, ask

A Sink stores whatever is piped into it, replacing any value already under the name:

<!-- transcript: run -->
```console
$ printf 'tok-7Hq2' | IKIGAI_SECRETS="$PWD/secrets" ikigai -c 'sink urn:secret:shop.api-token'
stored
[uncacheable]
$ IKIGAI_SECRETS="$PWD/secrets" ikigai -c 'source urn:secret:shop.api-token' -c 'exists urn:secret:shop.api-token' -c 'exists urn:secret:shop.smtp'
tok-7Hq2
[uncacheable]
true
[uncacheable]
false
[uncacheable]
— batch: 3 commands · 3 uncacheable
```

Every answer says `[uncacheable]`: the second read of a secret goes to the store again, as the
first did.

A `-c` batch runs under the owner's whole authority, which holds every secret scope. Narrow it
with `cap` (from [The REPL and its modes](../binary/repl.md)) to a read of some OTHER name, and
both reads are refused, the name that is there and the name that is not, in the same words:

<!-- transcript: run; exit 1 -->
```console
$ IKIGAI_SECRETS="$PWD/secrets" ikigai -c 'cap urn:cap:secret:read:shop.other' -c 'source urn:secret:shop.api-token' -c 'source urn:secret:shop.smtp' -c 'exists urn:secret:shop.api-token'
narrowed — capability: urn:cap:secret:read:shop.other
error: denied: reading `shop.api-token` requires `urn:cap:secret:read:shop.api-token`
error: denied: reading `shop.smtp` requires `urn:cap:secret:read:shop.smtp`
error: denied: asking whether `shop.api-token` exists requires `urn:cap:secret:read:shop.api-token`
```

`Exists` is a read too: knowing a name exists is something only its reader may learn. Narrowed to
the read of `shop.api-token` itself, the read succeeds and every other verb is still refused, each
naming the family of scope it needed:

<!-- transcript: run; exit 1 -->
```console
$ IKIGAI_SECRETS="$PWD/secrets" ikigai -c 'cap urn:cap:secret:read:shop.api-token' -c 'source urn:secret:shop.api-token' -c 'sink urn:secret:shop.api-token new' -c 'delete urn:secret:shop.api-token' -c 'source urn:secret:generate into=shop.enc type=age' -c 'source urn:secret:unlock'
narrowed — capability: urn:cap:secret:read:shop.api-token
tok-7Hq2
[uncacheable]
error: denied: capability does not grant `urn:cap:secret:write:*` (declared by `urn:secret:shop.api-token`; any grant under `urn:cap:secret:write:` satisfies it)
error: denied: capability does not grant `urn:cap:secret:delete:*` (declared by `urn:secret:shop.api-token`; any grant under `urn:cap:secret:delete:` satisfies it)
error: denied: capability does not grant `urn:cap:secret:generate` (declared by `urn:secret:generate`)
error: denied: capability does not grant `urn:cap:secret:unlock` (declared by `urn:secret:unlock`)
— batch: 6 commands · uncacheable
```

Delete has a scope of its own, `urn:cap:secret:delete:<name>`, rather than riding on the write
scope, so a write grant issued before Delete existed does not gain it.

## Keys, minted where they are kept

`urn:secret:generate` makes a keypair inside the store: `type=ed25519` a signing key (read by
`ikigai-sign`), `type=age` an encryption key (read by `ikigai-encrypt`). The private half goes to
`<into>`, the public half to `<into>.pub`, and the answer is the public half only:

<!-- transcript: run -->
```console
$ IKIGAI_SECRETS="$PWD/secrets" ikigai -c 'source urn:secret:generate into=shop.sign type=ed25519'
-----BEGIN PUBLIC KEY-----
…
-----END PUBLIC KEY-----
[uncacheable]
$ IKIGAI_SECRETS="$PWD/secrets" ikigai -c 'source urn:secret:generate into=shop.enc type=age'
age1…
[uncacheable]
```

Generate never replaces a key. Rotating one is minting a new name, so a second generate into the
same name is refused before anything is asked:

<!-- transcript: run; exit 1 -->
```console
$ IKIGAI_SECRETS="$PWD/secrets" ikigai -c 'source urn:secret:generate into=shop.sign type=ed25519'
error: invalid argument `into`: `shop.sign` already holds a secret, and generate never replaces one; mint into a new name
       `into`: the NEW key name to mint into (private at `<into>`, public at `<into>.pub`); neither may hold a secret already
```

Name keys `<who>.<role>`, with `sign` and `enc` for the roles, so that `shop.sign.pub` reads as
"the shop's signing key, public half". A name may contain `-` but never begin with one.

## Unlock and delete, and the presence gate

On Linux the presence gate is a no-op, so these answer at once:

<!-- transcript: run -->
```console
$ IKIGAI_SECRETS="$PWD/secrets" ikigai -c 'source urn:secret:unlock' -c 'delete urn:secret:shop.api-token' -c 'exists urn:secret:shop.api-token'
unlocked
[uncacheable]
deleted
[uncacheable]
false
[uncacheable]
— batch: 3 commands · 3 uncacheable
```

On a Mac each of `unlock`, `generate` and `delete` shows a Touch ID sheet first, and nothing is
deleted or minted if it is refused. The check that runs this book has no Mac, no finger and no
screen, so there is no transcript of that here: run it at the machine. The gate is asked only after
every capability and argument check has passed, so a caller who would be refused anyway is never
shown a prompt. The gate accepts biometrics only, with no password fallback, which is a barrier
for anyone who cannot use Touch ID; `ikigai-secret`'s
[ACCESSIBILITY.md](https://github.com/ikigai-rs/ikigai-secret/blob/main/ACCESSIBILITY.md) records
it. A host that must delete unattended mounts the space with a gate of its own.

## Where secrets live today

There are two stores, chosen by the platform. Nothing in `config.toml` chooses between them yet.

**On a Mac, the login Keychain.** Each secret is a generic-password item under the service
`ikigai-secret`, account `<name>`. `IKIGAI_SECRETS` is not read.

**Everywhere else, the file store.** A directory: `$IKIGAI_SECRETS` when it is set, else `secrets`
in the data home (`~/.ikigai/secrets`). It is not hardware-backed: file modes are what stand
between a secret and anyone who reads the disk or a backup of it. One file per secret, the file's
whole content the value:

<!-- transcript: run -->
```console
$ ls secrets
shop.enc
shop.enc.pub
shop.sign
shop.sign.pub
$ stat -c '%a %n' secrets secrets/shop.sign
700 secrets
600 secrets/shop.sign
```

(`stat -c` is GNU's; on a Mac, `stat -f '%Lp %N'`.) The directory is made `0700` and every file
`0600` whatever your umask is. An entry others can read is tightened and the fact logged once on
stderr, because whatever was in it may already have been read; one others can write, or one owned
by another user, is refused outright, because its content cannot be trusted. Writes go to a
temporary file that is renamed over the old one, so a failed write leaves the previous value.
`IKIGAI_SECRETS` must be absolute; a relative one is refused rather than read against whatever the
working directory is:

<!-- transcript: run; exit 1 -->
```console
$ IKIGAI_SECRETS=secrets ikigai -c 'exists urn:secret:shop.sign'
error: endpoint error: IKIGAI_SECRETS is `secrets`, a path relative to the working directory; the secrets store must be an absolute directory
```

With `HOME` unset and no `IKIGAI_SECRETS` there is no store, and every request says so.

**Only regular files, since `ikigai-secret` 0.5.1.** That release, from the third unled audit
(ledger #4023), refuses any entry in the file store that is not a regular file (a symlink, a FIFO,
a directory, a device) before it judges the entry's mode or reads it. A symlinked store directory
still works. The binary this book pins links an older `ikigai-secret`, which you can see for
yourself:

<!-- transcript: run -->
```console
$ ikigai --version --verbose | grep -E '^ikigai-(secret|core) '
ikigai-core 0.1.108
ikigai-secret 0.2.0
```

So under this release a symlink in the store is followed, and judged by the file it names. Keep
the store's entries regular files; a release that links 0.5.1 or later enforces it.

### Why the Keychain asks again

A Keychain item keeps an **access list**: the applications allowed to read it without asking,
named by their code-signing identity. `cargo install` builds a new binary and signs it ad hoc, so
to the Keychain every rebuild is a new applicant, and the next read shows "allow access" again,
even after you chose **Always Allow** for the last build. Two things follow:

- Sign the binary with a **stable** identity (a self-signed code-signing certificate of your own
  is enough), and "Always Allow" survives a rebuild, because the requirement it recorded still
  matches.
- **A macOS update can still drop it.** The access lists are re-evaluated, the grants can go, and
  a service under `launchd` cannot read its secret until someone answers a prompt at the screen.

An item the `security` command created is owned by `security`, and it prompts a signed reader on
every access. An item `ikigai` wrote itself through `sink` is owned by `ikigai`, and does not.
Prompts, and answering them, need a person at the Mac; none of this section can be shown by a
transcript.

## What is coming

Where a secret is stored is decided in code today, which is why a Linux server keeps everything in
a plaintext file store and why a service on a Mac can stop at a Keychain prompt. Two designs in
`ikigai-secret` are decided and **being built**. `ikigai-secret` 0.5.2 has the first part, the
section's parser, its routing over `keychain`, `secure-enclave` and `file` stores, and the inspection
resources, but no host reads the section yet: until the `ikigai` binary and gonk do, the
configuration below has no effect, and everything on this page works as described:

- [`docs/storage-backends.md`](https://github.com/ikigai-rs/ikigai-secret/blob/main/docs/storage-backends.md)
  adds a `[secret]` section to the config home's `config.toml`: named **stores** (`keychain`,
  `secure-enclave`, `file`, `age-file`, `fnox`) and ordered **routes** that send names to them by
  pattern, with one store per name and no fallthrough. `age-file`, a file store whose items are
  encrypted at rest, becomes the default for new installs; an existing install keeps the store it
  has until its operator copies secrets over. A `fnox` store reaches the stores
  [fnox](https://fnox.jdx.dev) speaks to. The face, its per-name scopes, the refusal before the
  store, never caching and the presence gate stay as they are above.
- [`docs/unattended-unlock.md`](https://github.com/ikigai-rs/ikigai-secret/blob/main/docs/unattended-unlock.md)
  says where the `age-file` store's own identity lives so a service can read its secrets with
  nobody at the screen (the Secure Enclave or a YubiKey on a Mac, a TPM or a systemd credential on
  Linux), always with a recovery recipient off the machine, and how an install moves off the
  Keychain, step by step.

This chapter will say how to configure them when a host release reads them.
