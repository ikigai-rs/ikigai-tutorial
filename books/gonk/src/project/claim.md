# Claiming: lease, expiry, takeover

A claim says who is working on an item right now. It is a **lease**, not a promise, so a crashed agent
cannot hold work forever, and an expired lease is **not free**, so nobody picks up half-finished work
by accident. This chapter runs a claim through its whole life with two holders, then shows what gonk
says about who holds it.

[A spec change, and the items it becomes](change.md) showed how `seed` files tasks and `next` finds
ready items. Here you will claim one of those items, so another worker knows it is taken.

## Catch up

In a new empty directory, as in [Start a project](start.md), with one difference: gonk's QUIC door is
open and a machine called `build-box` is enrolled in it before gonk starts, for the end of this
chapter. If the gonk from [the previous chapter](change.md) is still running, stop it first, since
this one takes the same port.

<!-- transcript: run -->
```console
$ ikigai-gonk client add build-box --worker library --port 1070 --config-home "$PWD/.config/ikigai" | grep -E 'enrolled'
  enrolled     grant `build-box` (… scopes) in …/.config/ikigai/gonk/grants.json
```

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ (cd starter && python3 tools/new.py ../library --process solo,spec > /dev/null)
$ git -C library init -q -b main
$ (cd library && python3 tools/gonk.py init --ledger library --socket ../.ikigai/gonk.sock --holder ada | tail -1)
ready.
$ git -C library add -A
$ git -C library -c user.name=Ada -c user.email=ada@example.com commit -q -m 'Generate the library sample'
$ (cd library && python3 tools/gonk.py seed | tail -1)
seed: 8 filed, 0 already filed, 0 checked off (not filed), 6 order link(s) added
```

A second holder works in a working tree of its own, because a holder name belongs to a tree:

<!-- transcript: run -->
```console
$ git -C library worktree add -q ../bob -b bob
$ (cd bob && python3 tools/gonk.py init --socket ../.ikigai/gonk.sock --holder bob | tail -1)
ready.
```

## Take one

`claim` takes the item for 30 minutes unless `--lease` says otherwise. `status` asks gonk rather than
trusting the tree's own note:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py claim 7)
library#7 claimed by ada until …
$ (cd library && python3 tools/gonk.py status)
process: solo, spec
holds a live claim: item library#7 is held by `ada` until …
```

The claim is a fence, for everyone else and for this tree:

<!-- transcript: run -->
```console
$ (cd bob && python3 tools/gonk.py claim 7)
error: conflict: library#7 is already claimed by ada. Release it first (`delete urn:iki:ledger:library:claim item=7`), or take it up with them — a claim is a fence, and stealing one silently is how two workers end up in the same tree
$ (cd bob && python3 tools/gonk.py next | grep 'not library#7')
  not library#7: claimed by ada
$ (cd library && python3 tools/gonk.py claim 1)
gonk.py: this working tree already holds item 7 on `library`: one item per working tree. Release or close it first
```

`claim 7 --renew` extends your own lease. During long work, renew it before it expires, including at
each commit or push. To stop without finishing, `release` hands the item back and says why:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py claim 7 --renew --lease 2h)
library#7 renewed by ada until …
$ (cd library && python3 tools/gonk.py release 7 --note 'Starting over on a short lease, to watch it run out')
library#7 released
```

## Let it expire

An agent that crashes stops renewing. A one-second lease shows what follows:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py claim 7 --lease PT1S)
library#7 claimed by ada until …
$ sleep 2
$ (cd library && python3 tools/gonk.py status)
process: solo, spec
no live claim: the lease on item library#7 ran out at …. It is not free (nobody else may take it without a takeover), but renew before writing: `python3 tools/gonk.py claim 7`
$ (cd bob && python3 tools/gonk.py next | grep 'not library#7')
  not library#7: lease expired: claimed by ada until …
$ (cd bob && python3 tools/gonk.py claim 7)
error: conflict: library#7's lease for ada expired at …, and an expired lease is never silently free: take it with `sink urn:iki:ledger:library:claim item=7 takeover=true from=ada`
```

`next` stops offering the item and says why, and a plain claim is refused. The item may hold half a
change on a branch somewhere, and only someone who has looked can say it is abandoned. The ledger's
doctor lists what needs that look:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:library:doctor' 2>/dev/null | grep -A1 '^  lease-expired'
  lease-expired  library#7's lease for ada expired at …
                 remedy: take it over (`sink urn:iki:ledger:library:claim item=7 takeover=true from=ada`), or release it (`delete urn:iki:ledger:library:claim item=7`)
```

## Take it over

Ada could still renew: an expired lease stays with its holder until someone else acts. Once Ada has
clearly gone, Bob takes it, naming the holder it comes from, and says so on the item. A takeover succeeds only if
that holder still has it and the lease has really run out, so a live lease cannot be taken this way.

<!-- transcript: run -->
```console
$ (cd bob && python3 tools/gonk.py claim 7 --takeover-from ada)
library#7 taken over from ada by bob until …
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:library:comment item=7 author=bob Took over: the lease ran out and no branch was pushed.'
commented on library#7 (urn:iki:ledger:library:comment:…)
[uncacheable]
```

The other half matters as much. Ada's agent may not be dead, only slow, and it must find out before it
writes anything else:

<!-- transcript: run -->
```console
$ (cd library && python3 tools/gonk.py status)
process: solo, spec
no live claim: item library#7 is held by `bob`, not `ada`: the claim was released or taken over. Stop writing; claim another item
$ (cd library && python3 tools/gonk.py claim 7 --renew; echo "exit $?")
error: conflict: library#7 is held by bob, not ada: a renewal extends a claim its holder still holds, and this one changed hands
gonk.py: this working tree no longer holds item 7: stop writing, put what you have on a branch, say where in a comment, and claim something else
exit 3
```

A renewal never takes, and exit 3 is the ledger's conflict, which a script can act on. The item's
history keeps all of it:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:library:item:7:history' 2>/dev/null | grep -E 'claim|release'
… claim ada
… release ada (released)
… claim ada
… release ada (takeover)
… claim bob
```

| what happened | `next` says | a plain `claim` by someone else | what moves it |
| --- | --- | --- | --- |
| a live claim | `claimed by ada` | refused | the holder's `release`, or a dispatcher's |
| the lease ran out | `lease expired` | refused | the holder's renewal, or a `--takeover-from ada` |
| released, reported or closed | offered, unless it waits for a dispatcher or is closed | taken | a new claim |

## A person or a machine

Over the socket, the name on a claim is free text, and gonk records only that the owner made it. Over
QUIC, gonk also records the claim's **principal**, from the client's certificate. And every claim has a
**kind**, `person` or `machine`, which gonk stamps itself: a passkey sign-in makes a person's claim,
and every other door a machine's. The machine enrolled at the top of this page takes an item:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/build-box -c 'sink urn:iki:ledger:library:claim item=1 lease=30m build-box'
library#1 claimed by build-box until …
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:library:item:1' 2>/dev/null | grep 'principal'
  principal: urn:iki:host:gonk:client:…
```

A saved query reads the kind back:

<!-- transcript: file claim-kinds.rq -->
```sparql
# Who holds what, and what kind of claim gonk stamped.
PREFIX ledger: <https://ikigai-rs.dev/ns/ledger#>
SELECT ?number ?holder ?kind WHERE {
  GRAPH <urn:iki:ledger:graph:library> {
    ?item ledger:number ?number ; ledger:claimedBy ?holder ; ledger:claimKind ?kind .
  }
}
ORDER BY ?number
```

<!-- transcript: run -->
```console
$ cat claim-kinds.rq | ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:script:library-claim-kinds language=sparql'
urn:script:library-claim-kinds:version:sha256:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:script:library-claim-kinds:result as=text/csv'
number,holder,kind
1,build-box,https://ikigai-rs.dev/ns/ledger#machine
7,bob,https://ikigai-rs.dev/ns/ledger#machine
[uncacheable]
```

Both are machines. The caller has no say in it: gonk's doors overwrite whatever a request says about
who sent it, and its own test (`tests/claim_kind_775.rs`) sends a forged person's claim through each
door and reads back what was stored. The difference shows when a claim outlives its work. The doctor reports a machine's claim on an item not in flight as
**orphaned**, since a process that forgot to let go looks just like that, and leaves a person's hold
alone. A person claims through [the ledger in a browser](../ledger/browser.md), which is a passkey
ceremony this check cannot perform.

A machine's grant also sets what it may do at all. `build-box` was enrolled with `--worker`, so it can
claim and comment, and closing stays with its dispatcher:

<!-- transcript: run -->
```console
$ ikigai --connect quic://127.0.0.1:1070 --cert-dir .config/ikigai/gonk/quic/clients/build-box -c 'sink urn:iki:ledger:library:close item=1 done'
error: denied: this capability holds neither `urn:cap:ledger:close:library` nor `urn:cap:ledger:write:library`, and this resource takes either. A ledger grant names exactly one ledger, and a grant for another verb (a worker's claim and comment, say) does not reach this one
```

<a id="what-you-built"></a>

## What you have now

The claim's whole life: taken, refused to others, renewed, expired, taken over, and refused to the
holder who lost it. The [next chapter](work.md) does the work a claim is for.
