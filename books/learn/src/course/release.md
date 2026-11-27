# A course release, pinned to a tag

"Release" names two things in ikigai-learn, and this chapter needs both:

- A **course release** is a course at one git tag of its repository:
  `urn:learn:course:name:{c}:release:{r}`. It is what a learner enrolls in. It is planned.
- A **release grant** (in the crate, simply a *release*) is what a learner issues to share part of
  their record in a course with one other party: `urn:learn:release:enrollment:{n}:party:{p}`. It
  is built, in ikigai-learn 0.1.0's `ikigai_learn::release` module.

They meet in one place. A credential, a completion or a set of exercises is always *of* one
course release, and the release grant is how anyone but the learner gets to see it.

## A course release (planned)

A course release is a manifest at a git tag. It holds an ordered list of modules, and each module
an ordered list of unit names. ikigai-repo reads the unit sources at the tag, so everything in the
release resolves the same way for good, and caches for good: a unit at a release,
`urn:learn:unit:name:{u}:release:{r}`, never changes, so nothing ever invalidates it.

ikigai-learn's design note on Lectern describes how a release is published:

1. A git tag on the course repository starts a CI build, which builds every unit at the tag.
2. The outputs are stored as resources: each unit at the release, its forms (`slides`, `page`,
   `transcript`, `handout`), and one `urn:learn:slide:unit:{u}:number:{n}` per slide, numbered
   from 1 the way Lectern's range grammar counts.
3. A slide's HTML is the unit's reveal.js page plus a fragment. reveal.js counts slides from zero,
   so slide `n` is the fragment `#/{n-1}`.

A release must be self-contained. A host URL written into the built HTML would stop resolving the
day the course moves, so a release is built with its assets copied beside it (Lectern does this,
naming each file by its content hash) and with nothing loaded from another host; [Lectern decks
as units](../units/lectern.md) shows the Lectern side. The names under `urn:learn:` resolve only
in a kernel; people and credentials are to reach a release through persistent names, which
ikigai-name resolves and which the design note leaves to be claimed.

Nothing above is built yet: the note is marked "design only, nothing built".

## The release grant (built)

A learner shares part of their own record with another party by issuing a release with one or
more scopes and an expiry. The party is named by a digest: `{p}` is the bare-hex BLAKE3 digest of
the party's principal. There are four scopes, and each is a view:

| Scope | What the party reads | Its view |
|---|---|---|
| `credential` | the credential only | `urn:learn:credential:enrollment:{n}` |
| `completion` | complete or not, and when | `urn:learn:completion:enrollment:{n}` |
| `progress` | mastery per unit and the verdict per exercise | `urn:learn:progress:enrollment:{n}` |
| `submission` | the work itself | `urn:learn:submission:enrollment:{n}:exercise:{e}` |

Responses, attempts, time and drafts are in no scope. A release that names one is refused, and no
view reads them.

### Who reads a view

A view answers the enrollment's learner, and anyone else only while a release naming them shares
that view and has not expired. The decision is made on every call, from the release itself, read
through the kernel: a session caches who you are, never what you may read. A view that is cached
hangs from the release's golden thread and expires with it, so the Delete that revokes a release
denies the very next read. (The ikigai Book's [Golden threads in
practice](../../getting-started/golden-threads.html) teaches them.)

### Narrow, never widen

Only the enrollment's learner issues, narrows or revokes a release. A rewrite may narrow it: fewer
scopes, an earlier expiry. It may never widen it. Widening is a Delete and a new release, so the
learner sees the change they are making.

The learner lists their releases on one enrollment at `urn:learn:release:enrollment:{n}`: Turtle
naming each release as a member of the set, with its party, scopes and expiry. Only the learner
reads that list; a party is Denied it. A host's page of devices and grants can show the list
beside the principal's other grants, and revoke a release with a Delete of its name, through the
door. The list is never cached.

### Three floors under every view

| Capability | What it allows |
|---|---|
| `urn:cap:learn:release:read` | reading a release (the door still answers only its learner and its party) |
| `urn:cap:learn:release:write` | issuing, narrowing and revoking a release (the door still answers only its learner) |
| `urn:cap:learn:{scope}:read` | reading the `{scope}` view (the door still checks the release) |

The grants a host mints are functions of the crate, so a host mints exactly them and a test runs
under exactly them:

- `release::learner_grant(n, learner)`: both release capabilities, all four views, read on the
  enrollment's directory under the state root, write and delete on its `release` directory only,
  and the learner's principal.
- `release::party_grant(release)`: reading the release, and for each scope it shares, that view and
  the one record file the view reads, read on the party's own release, and the party's principal.
  Nothing else on the state root: not the enrollment, not the responses, not the drafts. A host
  derives it from the live release on every call.

So a view stands on three floors: the kernel enforces its declared capability, the door enforces
the release, and ikigai-fs's path rules enforce the grant on the state root.

### Where it lives

Under the state root a host mounts as `urn:file:` (ikigai-fs), one directory per enrollment. With
`{p}` the party's digest and `{e}` an exercise:

```text
enrollment/{n}/enrollment.ttl         the enrollment: its learner
enrollment/{n}/release/{p}.ttl        one release per party
enrollment/{n}/credential.json        the record each view reads:
enrollment/{n}/completion.ttl           one file per scope, so a party's grant
enrollment/{n}/progress.ttl             on the state root is exactly as narrow
enrollment/{n}/submission/{e}.ttl       as its release
```

The files use the vocabulary `urn:learn:term:` (`Release`, `enrollment`, `party`, `scope`,
`expires`, `learner`, `complete`).

The crate says plainly where this departs from the model in [the previous
chapter](model.md): until the mastery and completion compositions are built, the `completion` and
`progress` views read a record file rather than computing over the learner's answers. When the
compositions land, each view computes, and the file it reads becomes the composition's own name.

### Not built yet

Anything that leaves for another system, such as a grade sent back to a learning management
system or an xAPI statement, will leave only under a release, and a subscription that relays
events will carry the release's narrowed grant, checked again at every delivery, so revoking the
release stops delivery. The page of devices and grants is the host's to build, not this crate's.
