# Logging

<!-- transcripts: normalize workdir, timestamp -->

A kernel's log answers the questions you ask after something happened: who asked for what, under
which authority, and whether the record has been touched since. This chapter turns one on, reads
it, backs it up, proves a restore, and queries it. What a
[resource](../../getting-started/hello-resource.html) is and what a [golden
thread](../../architecture/economics/economics.html) does are taught elsewhere; here they are only
used.

## The binary does not log; a host does

The log is a crate, [`ikigai-log`](https://github.com/ikigai-rs/ikigai-log). Binding its endpoints
opens nothing, because a module cannot tell a daemon from a one-shot command, and no released
`ikigai` links it at all, the release this book pins included:

<!-- transcript: run -->
```console
$ ikigai -c 'source urn:log:segments' < /dev/null; echo "exit $?"
error: no endpoint resolved for urn:log:segments
…
exit 1
```

So this chapter runs a host: `log-host`, in this book's repository, the smallest kernel that opens
a log. It composes `ikigai-log` 0.9.2, SPARQL and the function library, and serves them on a Unix
socket, which `ikigai --connect` then drives as [any served kernel](../binary/repl.md). Everything
it does is a few lines of library calls you would make in your own host, and they are shown below.

<!-- transcript: manual — it compiles the host from the repository, minutes of build and a
     network fetch; the check builds this same crate itself before it runs the page. -->
```console
$ cargo install --locked --git https://github.com/ikigai-rs/ikigai-tutorial --rev 6a3b2ccaf6f645cb33de9b802cde742d87a5e2e9 log-host
```

## What a log is here

A log is a directory of **segments**. A segment is a text file of one line per entry, so `grep`
reads it, and each one rotates into the next. Each is **sealed**: every few entries a `#seal` line
states a hash of everything so far, and a segment's `@prev` header names the last seal of the
segment before it, so the chain crosses rotations and restarts. And a segment is a resource: it
resolves to Turtle (PROV-O terms) at its own IRI.

Its settings are in `log.toml`, in the config home, with a `<app>.log.toml` layered over it. Save
this as `home/log.toml`:

<!-- transcript: file home/log.toml -->
```toml
level = "debug"
instance = "shop"

[seal]
every_entries = 4
every_millis = "never"

[rotation]
max_entries = 12
```

The numbers are small so that this page rotates in a few requests. The defaults are 1,000 entries
or 60 seconds a seal and 100,000 entries or a day a segment.

**Where segments go.** `directory` in `log.toml` says. With none, `ikigai-log` uses `log` in the
data home (`~/.ikigai/log`), not the config home; `log-host` overrides that to `log` beside its
`log.toml`, so that a config home is the whole of a log, which is convenient for the backups below.
Decide this in your own host, and write it down.

**Levels.** One dial, from the vocabulary, read when a segment opens:

| Level | Adds | Note |
|-------|------|------|
| always | process start and stop, seals, rotations, config changes, chain breaks, drops, capability refusals | written at every level, and cannot be turned off |
| error | `log:Error` | |
| warn | `log:Warning`, a request nested too deep | |
| info | `log:Message` | the default |
| debug | `log:Resolution`, one per request | what this page runs at |
| trace | `log:CacheHit` | |

A level change takes effect at the next segment and lands as an entry, so a gap has a reason on
the record.

## Examples, end to end

### Turn it on

The `--clock-from` flag is only for this page: a clock that starts at a stated time and advances a
second per reading, so that the names below are the ones you will see. Leave it off in real use.

<!-- transcript: serve -->
```console
$ log-host serve --config-home home --socket log.sock --door door.sock --clock-from 2026-11-02T09:00:00.000Z
log-host: log.sock (writing urn:log:shop:2026-11-02T09-00-00Z)
log-host: door on door.sock
```

The host opens the writer itself, and then installs one tracer on its kernel:

```rust,ignore
{{#include ../../../../crates/log-host/src/lib.rs:open}}
{{#include ../../../../crates/log-host/src/lib.rs:tracer}}
```

The segment is there at once, owner-only (mode 0600, created that way), with its header and its
first entry. The `.lock` beside it is how a second process under the same name knows to take a
different one:

<!-- transcript: run -->
```console
$ ls home/log
shop-2026-11-02T09-00-00Z.log
shop.lock
$ cat home/log/shop-2026-11-02T09-00-00Z.log
# ikigai-log v1
@prefix log:  <https://ikigai-rs.dev/ns/log#> .
@prefix prov: <http://www.w3.org/ns/prov#> .
@name     urn:log:shop:2026-11-02T09-00-00Z
@instance urn:ikigai:instance:shop
@level    log:debug
@started  2026-11-02T09:00:00.000Z
@prev     genesis
2026-11-02T09:00:00.000Z log:ProcessStart urn:ikigai:instance:shop seq=1 pid=…
```

`@prev genesis` says this is the first of its chain, in writing: a missing `@prev` is an error,
never a first segment.

### Who asked: a principal per request

One process serves many parties, and the log has a column for that. The second socket above is a
stand-in for the door of a real service, which would take the name from what authenticated the
caller (a certificate, a passkey). What matters is what it does with the name:

```rust,ignore
{{#include ../../../../crates/log-host/src/lib.rs:ask}}
```

The tracer is made per call and handed to `issue_traced`. It must never be handed to
`Kernel::set_tracer`: that is one slot for the whole kernel, so Alice's tracer there would file
Bob's concurrent request under Alice, with a `prov:Delegation` in the graph to prove it.
`issue_traced` is exclusive of that slot, so each request lands once, under its own name. (A door
that can mint `urn:cap:principal:<iri>` into the capability needs no per-call tracer, because the
global one reads it back. Where both name someone, the capability wins.)

Three requests: one answered, one missing an argument, one refused, since the door's capability
holds no scopes:

<!-- transcript: run -->
```console
$ log-host ask --door door.sock urn:agent:alice urn:iki:fn:toUpper in=hello
HELLO
$ log-host ask --door door.sock urn:agent:bob urn:iki:fn:toUpper
refused: missing required argument `in`
$ log-host ask --door door.sock urn:agent:bob urn:log:segments
refused: denied: capability does not grant `urn:cap:log:read` (declared by `urn:log:segments`)
$ grep principal= home/log/shop-2026-11-02T09-00-00Z.log
2026-11-02T09:00:01.000Z log:Resolution urn:iki:fn:toUpper seq=2 principal=urn:agent:alice worker=main span=0 dur=… answered-by=urn:iki:space:fn
2026-11-02T09:00:06.000Z log:Resolution urn:iki:fn:toUpper seq=3 principal=urn:agent:bob worker=main span=0 dur=… failed=missing-argument answered-by=urn:iki:space:fn
2026-11-02T09:00:10.000Z log:CapabilityDenied urn:log:segments seq=4 principal=urn:agent:bob worker=main span=0 denied=urn:cap:log:read
```

A principal is recorded, not authenticated: the log writes down the name the host gave it.

### Writing an entry from the REPL

`urn:log:write` takes a class, a subject and either a `msg` or the piped body. It answers the IRI
of the entry it wrote, which is a thing you can cite. (The bracketed line is on stderr, as in [the
REPL](../binary/repl.md); the pipes below drop it.)

<!-- transcript: run -->
```console
$ ikigai --connect log.sock -c 'sink urn:log:write class=log:Message subject=urn:agent:calendar msg="sync started"' < /dev/null
urn:log:shop:2026-11-02T09-00-00Z:…
[uncacheable]
```

A caller cannot speak for anyone. The columns the log writes itself are refused, and so are the
classes verify believes, because a forged `log:ProcessStop` would make a segment read as finished:

<!-- transcript: run -->
```console
$ ikigai --connect log.sock -c 'sink urn:log:write class=log:Message subject=urn:agent:calendar fields=principal=urn:agent:alice' < /dev/null
error: invalid argument `fields`: `principal=` is written by this log itself, never by a caller (reserved: seq, principal, cap, denied, failed, pid, configured, next)
…
$ ikigai --connect log.sock -c 'sink urn:log:write class=log:ProcessStop subject=urn:agent:calendar' < /dev/null
error: invalid argument `class`: https://ikigai-rs.dev/ns/log#ProcessStop is an always-land marker this log writes itself; a caller cannot append one
…
```

### What an escaped value looks like

An entry is one line, and that is what `grep`, the hash chain and your terminal all rely on. So a
value holding a control character, a line separator or a bidirectional formatting character is
quoted, with the character written as `\u{hex}`. This body carries an escape sequence that would
turn a terminal red, and a right-to-left override that would make a line read backward:

<!-- transcript: run -->
```console
$ printf 'red\033[31m\342\200\256evil' | ikigai --connect log.sock -c 'sink urn:log:write class=log:Message subject=urn:agent:calendar' 2>/dev/null
urn:log:shop:…
$ grep -h evil home/log/*.log
…log:Message urn:agent:calendar seq=… msg="red\u{1b}[31m\u{202e}evil"
$ cat home/log/*.log | grep -c "$(printf '\033')"
0
```

The parser reads `\u{hex}` back, so the value is not lost, and what you print to a terminal cannot
act on it. Segments written before 0.9.2 can hold raw characters. They still verify, so read old
logs with `cat -v`.

### Rotation, as it happens

Another few requests pass `max_entries`, and the entry that crosses it rolls the segment:

<!-- transcript: run -->
```console
$ log-host ask --door door.sock urn:agent:alice urn:iki:fn:toUpper in=two
TWO
$ log-host ask --door door.sock urn:agent:alice urn:iki:fn:toUpper in=three
THREE
$ log-host ask --door door.sock urn:agent:bob urn:iki:fn:toUpper in=four
FOUR
$ ls home/log | wc -l | tr -d ' '
3
$ grep -h -E 'log:Rotation|^#seal' home/log/shop-2026-11-02T09-00-00Z.log
#seal 1-4 sha256:…
#seal 5-8 sha256:…
#seal 9-12 sha256:…
…log:Rotation urn:log:shop:2026-11-02T09-00-00Z seq=13 next=urn:log:shop:…
#seal 13-13 sha256:…
```

The old segment ends on a marker naming its successor and a final seal, and will never change
again. The new one states that seal in its `@prev`:

<!-- transcript: run -->
```console
$ ls home/log/*.log | tail -1 | xargs grep -h '^@prev'
@prev     sha256:…
$ ls home/log/*.log | tail -1 | xargs grep -h log:ChainBroken | cut -d' ' -f2-3
log:ChainBroken urn:log:shop:…
```

**That second line is a false alarm you will see on every rotation (ledger #2951).** In
`ikigai-log` 0.9.2 the check a rotation runs reads the segment it just sealed and its predecessor,
before the successor has any lines, so it finds the rotation marker's `next=` naming a segment
"not here", and lands a
`log:ChainBroken` saying the chain was truncated. The chain is fine, and `urn:log:verify` says so
below. Until that changes, do not alert on `log:ChainBroken` alone: ask verify.

### Reading a segment, as text and as Turtle

A segment's IRI answers Turtle by default, and the file itself on request. Read a sealed one: the
open one is changing under you, so `cat` is the way to look at that. Here is one entry as a line:

<!-- transcript: run -->
```console
$ ikigai --connect log.sock -c 'source urn:log:shop:2026-11-02T09-00-00Z as=text/x-ikigai-log' < /dev/null 2>/dev/null | grep ' seq=2 '
2026-11-02T09:00:01.000Z log:Resolution urn:iki:fn:toUpper seq=2 principal=urn:agent:alice worker=main span=0 dur=… answered-by=urn:iki:space:fn
```

The default face makes a sealed segment a graph you can ask things of, even from the host still
writing the next one. Save this as `classes.rq`, and run it over the first segment:

<!-- transcript: file classes.rq -->
```sparql
PREFIX log: <https://ikigai-rs.dev/ns/log#>
SELECT ?class (COUNT(*) AS ?n) WHERE {
  GRAPH ?segment { ?e log:sequence ?seq ; a ?class }
} GROUP BY ?class ORDER BY ?class
```

<!-- transcript: run -->
```console
$ ikigai --connect log.sock -c 'source urn:sparql:select as=text/csv graph=urn:log:shop:2026-11-02T09-00-00Z' < classes.rq 2>/dev/null
class,n
https://ikigai-rs.dev/ns/log#CapabilityDenied,1
https://ikigai-rs.dev/ns/log#Message,2
https://ikigai-rs.dev/ns/log#ProcessStart,1
https://ikigai-rs.dev/ns/log#Resolution,8
https://ikigai-rs.dev/ns/log#Rotation,1
```

And here is the same entry in that graph:

<!-- transcript: run -->
```console
$ ikigai --connect log.sock -c 'source urn:log:shop:2026-11-02T09-00-00Z' < /dev/null 2>/dev/null | grep -A7 '^<urn:log:shop:2026-11-02T09-00-00Z:2> a '
<urn:log:shop:2026-11-02T09-00-00Z:2> a log:Resolution ;
	log:segment <urn:log:shop:2026-11-02T09-00-00Z> ;
	prov:startedAtTime "2026-11-02T09:00:01.000Z"^^xsd:dateTime ;
	prov:wasAssociatedWith <urn:ikigai:instance:shop> ;
	log:subject <urn:iki:fn:toUpper> ;
	log:resolved <urn:iki:fn:toUpper> ;
	log:sequence 2 ;
	log:onBehalfOf <urn:agent:alice> .
```

Each entry becomes its own IRI, `{segment}:{seq}`, never a blank node, since a blank node in a log
is a fact you cannot cite. The subject is stated twice, as `log:subject` always and as the class's
own property, here `log:resolved`, which PROV reads as `prov:used`. The principal becomes
`log:onBehalfOf`, a graph edge rather than a string.

### A dropped entry, and its marker

A tracer cannot fail upward: there is nowhere for the error to go. So what it cannot write it
counts, and the next entry lands a `log:Dropped` marker first, because a drop that leaves no mark
is the one thing this design forbids. A subject is an IRI column with no quoting, so a request for
a segment whose name holds a bidirectional character resolves, fails, and cannot be written down:

<!-- transcript: run -->
```console
$ ikigai --connect log.sock -c "source urn:log:a$(printf '\342\200\256')b" < /dev/null 2>&1 | cat -v
error: not found: no segment <urn:log:a\u{202e}b> in $PWD/home/log
$ log-host ask --door door.sock urn:agent:alice urn:iki:fn:toUpper in=five
FIVE
$ grep -h log:Dropped home/log/*.log
…log:Dropped urn:log:shop:… seq=… count=1 reason=unwritable
```

The marker names no principal, whichever tenant's entry happened to flush it: a drop is the
process's. There is no sampling, for the same reason.

## Backup and restore

Segments are files, and that is the whole of the backup story. What varies is when each is safe:

- **A sealed segment never changes.** It ends on `log:Rotation` or `log:ProcessStop`, and it can
  be copied whenever you like, once.
- **The open one is appended to.** A copy taken now is good up to the moment of copying, and may
  stop part way through a line, which verify will report. Take it for the recent past; take it
  again later.

### Copying while the writer runs

Which are sealed is a `grep`, and which is open is the config's to say:

<!-- transcript: run -->
```console
$ mkdir -p backup
$ grep -lE ' log:(Rotation|ProcessStop) ' home/log/*.log | xargs -I{} cp -p {} backup/
$ ls backup
shop-2026-11-02T09-00-00Z.log
$ ikigai --connect log.sock -c 'source urn:log:config' < /dev/null 2>/dev/null | grep -E '^# (open|segment)'
# open = true
# segment = urn:log:shop:…
```

### Offsite, encrypted

A segment holds principals, capability scopes and whatever prose a caller wrote, so it leaves the
machine encrypted. Use [age](https://age-encryption.org), which is what `ikigai-layers` seals
its own state with. The commands here are `rage`, a Rust implementation of the same format: `age` and
`age-keygen` take these flags too and read these files. Keep the identity somewhere other than
beside the backup. In real use, encrypt to its public key on the machine that holds the log, and
keep the identity off that machine.

<!-- transcript: run -->
```console
$ rage-keygen -o offsite.key 2>/dev/null
$ rage-keygen -y offsite.key > offsite.pub
$ tar czf - -C backup . | rage -R offsite.pub > offsite.tar.gz.age
$ mkdir fetched && rage -d -i offsite.key offsite.tar.gz.age | tar xzf - -C fetched
$ ls fetched
shop-2026-11-02T09-00-00Z.log
$ cmp fetched/shop-2026-11-02T09-00-00Z.log home/log/shop-2026-11-02T09-00-00Z.log && echo identical
identical
```

### Restoring, and proving it

End the writer in order, so that the last segment ends on a marker and a final seal. Finish the
backup with all the segments and the configuration, then encrypt and fetch that final archive.
The earlier archive held only the first sealed segment; its rotation marker names a successor
that must be backed up too before the whole chain can verify cleanly.

Restore into a fresh config home from the decrypted archive, and compare each restored segment
with that copy. The host runs `--read-only` over it, which opens no writer and installs no tracer,
so reading appends nothing:

<!-- transcript: run -->
```console
$ log-host stop --door door.sock
log closed
$ cp -p home/log/*.log backup/
$ cp home/log.toml backup/
$ tar czf - -C backup . | rage -R offsite.pub > offsite-final.tar.gz.age
$ mkdir recovered && rage -d -i offsite.key offsite-final.tar.gz.age | tar xzf - -C recovered
$ mkdir -p restored/log
$ cp recovered/log.toml restored/
$ cp -p recovered/*.log restored/log/
$ for segment in restored/log/*.log; do cmp "$segment" "recovered/$(basename "$segment")" || exit 1; done
$ ls restored/log | wc -l | tr -d ' '
3
```

<!-- transcript: serve -->
```console
$ log-host serve --config-home restored --socket restored.sock --read-only
log-host: restored.sock (read-only, no writer open)
```

Then ask `urn:log:verify` to walk it. It recomputes every hash, follows every `@prev`, and checks
that sequence numbers are dense, that seals cover what they say, and that nothing follows an
orderly end:

<!-- transcript: run -->
```console
$ ikigai --connect restored.sock -c 'source urn:log:verify' < /dev/null 2>/dev/null | sed -E 's/ (entries|seals|level|prev|signed|head)=[^ ]*//g; s/ states .* — / — /'
segment urn:log:shop:2026-11-02T09-00-00Z OK
segment urn:log:shop:… OK
segment urn:log:shop:… OK
verified 3 segment(s), 0 broken
```

**A tampered line.** Here someone rewrites who asked, in the first segment of the restored copy:

<!-- transcript: run -->
```console
$ sed 's/seq=3 principal=urn:agent:bob/seq=3 principal=urn:agent:alice/' recovered/shop-2026-11-02T09-00-00Z.log > restored/log/shop-2026-11-02T09-00-00Z.log
$ ikigai --connect restored.sock -c 'source urn:log:verify' < /dev/null 2>/dev/null | sed -E 's/ (entries|seals|level|prev|signed|head)=[^ ]*//g; s/ states .* — / — /'
segment urn:log:shop:2026-11-02T09-00-00Z BROKEN
  BROKEN seal 1-4 — the alteration is between sequence 1 and 4
  BROKEN seal 5-8 — the alteration is between sequence 5 and 8
  BROKEN seal 9-12 — the alteration is between sequence 9 and 12
  BROKEN seal 13-13 — the alteration is between sequence 13 and 13
segment urn:log:shop:… OK
segment urn:log:shop:… OK
verified 3 segment(s), 1 broken
```

The verify line is cut down here to fit: the real one also prints each seal's stated and computed
hash. The first seal to disagree says where to look. Every seal after it disagrees too, because the
chain is one hash carried forward. The next segments stay `OK`: the first of them states what the old
head *was*, and it is that statement which lets a good copy prove the tampered one wrong.

**A missing successor.** Put the good copy back and take away the newest segment, which the one
before it rotated into. Nothing else points forward, so this is the only check that can see the
chain cut at its newest end:

<!-- transcript: run -->
```console
$ cp -p recovered/shop-2026-11-02T09-00-00Z.log restored/log/
$ rm "$(ls restored/log/*.log | tail -1)"
$ ikigai --connect restored.sock -c 'source urn:log:verify' < /dev/null 2>/dev/null | sed -E 's/ (entries|seals|level|prev|signed|head)=[^ ]*//g; s/ states .* — / — /'
segment urn:log:shop:2026-11-02T09-00-00Z OK
segment urn:log:shop:… BROKEN
  BROKEN <urn:log:shop:…> rotated into <urn:log:shop:…>, which is not here: the chain was truncated at its head, where nothing points forward
verified 2 segment(s), 1 broken
```

**What an `OK` does not prove.** The chain is unkeyed SHA-256. Anyone who can write the files and
knows the format can recompute every hash, so resisting that writer takes signatures, which this
host does not set (`signed=none`; a host installs a `SealSigner`, and `urn:sign:verify` checks one),
or a head published somewhere the writer cannot reach. A crashed tail cannot be told from an
appended one: entries past the last seal of a segment that never ended are only `noted`.

### Retention

`ikigai-log` has no retention of its own and no tombstone yet, so removing old segments is `rm`,
and verify cannot tell that from a segment that was never there. It reads the oldest one it has as
the start of the chain, without checking the predecessor its `@prev` names:

<!-- transcript: run -->
```console
$ cp -p recovered/*.log restored/log/
$ rm restored/log/shop-2026-11-02T09-00-00Z.log
$ ikigai --connect restored.sock -c 'source urn:log:verify' < /dev/null 2>/dev/null | sed -E 's/ (entries|seals|level|prev|signed|head)=[^ ]*//g; s/ states .* — / — /'
segment urn:log:shop:… OK
segment urn:log:shop:… OK
verified 2 segment(s), 0 broken
```

So removing from the old end, and only from there, keeps verify green, and it also means that an
attacker removing from the old end looks just like you. To keep the reason on the record, write it
into the chain before you delete, naming the head you are about to lose, and copy the segment
offsite first. A log can be started again to do that: a writer opening in a directory finds the
tail of its own instance's chain and carries on from it, so restarts do not break it either.

<!-- transcript: serve -->
```console
$ log-host serve --config-home home --socket log.sock --door door.sock --clock-from 2026-11-02T10:00:00.000Z
log-host: log.sock (writing urn:log:shop:2026-11-02T10-00-00Z)
log-host: door on door.sock
```

<!-- transcript: run -->
```console
$ ikigai --connect log.sock -c "sink urn:log:write class=log:Message subject=urn:log:shop:2026-11-02T09-00-00Z msg=\"retention: removed, offsite in offsite.tar.gz.age\"" < /dev/null 2>/dev/null
urn:log:shop:2026-11-02T10-00-00Z:…
$ grep -c '^@prev     genesis' home/log/shop-2026-11-02T10-00-00Z.log
0
```

The entry is sealed with everything else. It is your record, not verify's: verify still only says
`OK`. Removing from the middle, or from the newest end after a rotation, is what the walks above
catch.

## Querying the logs

### grep and awk first

The format is meant for this. The time comes first, in fixed-width UTC, so it sorts as text and a
prefix is a window. The class is the second column and the subject the third:

<!-- transcript: run -->
```console
$ grep -h ' log:CapabilityDenied ' home/log/*.log | cut -d' ' -f2-
log:CapabilityDenied urn:log:segments seq=4 principal=urn:agent:bob worker=main span=0 denied=urn:cap:log:read
$ grep -h 'principal=urn:agent:alice' home/log/*.log | wc -l | tr -d ' '
4
$ grep -h '^2026-11-02T09:00:1' home/log/*.log | cut -d' ' -f2 | sort -u
log:CapabilityDenied
log:Message
log:Resolution
$ cat home/log/*.log | awk '/^[0-9]/ { n[$2]++ } END { for (c in n) print n[c], c }' | sort -k2
1 log:CapabilityDenied
2 log:ChainBroken
1 log:Dropped
3 log:Message
4 log:ProcessStart
1 log:ProcessStop
19 log:Resolution
2 log:Rotation
```

Start here, and stay here if it answers the question. No transreption, no kernel, and it works on
a dead host's files.

### Listing segments

`urn:log:segments` is a directory listing, in name order, and not a catalog. Keep it as a `graph=`
list:

<!-- transcript: run -->
```console
$ cp -p recovered/shop-2026-11-02T09-00-00Z.log restored/log/
$ ikigai --connect restored.sock -c 'source urn:log:segments' < /dev/null 2>/dev/null | grep '^urn:' | paste -sd, - > segments.txt
$ tr , '\n' < segments.txt | wc -l | tr -d ' '
3
```

### Transreption to Turtle

A segment's bytes transrept without a directory or a host's file access: `urn:log:transrept` is a
pure function of what you pipe in, which is how a browser or another machine reads one:

<!-- transcript: run -->
```console
$ cat home/log/shop-2026-11-02T09-00-00Z.log | ikigai --connect restored.sock -c 'source urn:log:transrept' 2>/dev/null | grep -c 'a log:Resolution'
8
```

### SPARQL across segments

Every segment is a graph, so a query lists them as `graph=` sources, and `GRAPH ?segment` names
where each row came from. Run these over a read-only host, or over sealed segments only, for a
reason shown below.

**Every request by one principal in a window.** `prov:startedAtTime` is a real `xsd:dateTime`:

<!-- transcript: file by-principal.rq -->
```sparql
PREFIX log:  <https://ikigai-rs.dev/ns/log#>
PREFIX prov: <http://www.w3.org/ns/prov#>
PREFIX xsd:  <http://www.w3.org/2001/XMLSchema#>
SELECT ?resource ?seq WHERE {
  GRAPH ?segment {
    ?e log:onBehalfOf <urn:agent:bob> ;
       prov:startedAtTime ?when ;
       log:subject ?resource ;
       log:sequence ?seq .
    FILTER (?when >= "2026-11-02T09:00:00Z"^^xsd:dateTime
         && ?when <  "2026-11-02T09:00:30Z"^^xsd:dateTime)
  }
} ORDER BY ?seq
```

<!-- transcript: run -->
```console
$ ikigai --connect restored.sock -c "source urn:sparql:select as=text/csv graph=$(cat segments.txt)" < by-principal.rq 2>/dev/null
resource,seq
urn:iki:fn:toUpper,3
urn:log:segments,4
```

**Errors by endpoint.** A failed invocation carries its kind in `log:failureKind`, and a refusal
is a class of its own. A query with no reasoner names both denial classes:

<!-- transcript: file errors.rq -->
```sparql
PREFIX log: <https://ikigai-rs.dev/ns/log#>
SELECT ?resource ?what (COUNT(*) AS ?n) WHERE {
  GRAPH ?segment {
    ?e log:subject ?resource .
    { ?e log:failureKind ?what }
    UNION { ?e a log:CapabilityDenied . BIND("denied" AS ?what) }
    UNION { ?e a log:EndpointDenied . BIND("denied" AS ?what) }
  }
} GROUP BY ?resource ?what ORDER BY ?resource ?what
```

<!-- transcript: run -->
```console
$ ikigai --connect restored.sock -c "source urn:sparql:select as=text/csv graph=$(cat segments.txt)" < errors.rq 2>/dev/null
resource,what,n
urn:iki:fn:toUpper,missing-argument,1
urn:log:segments,denied,1
urn:log:write,invalid-argument,2
```

**What a given request touched.** A resolution's sub-requests link to it by `log:invoked`, within
a segment and one process run. The query under [Reading a
segment](#reading-a-segment-as-text-and-as-turtle) is the one request on this log that had any: it
read the first segment as a graph. A sub-request answered from the cache is a `log:CacheHit`, which
lands only at `trace`, so at `debug` a request appears to touch only what it computed:

<!-- transcript: file touched.rq -->
```sparql
PREFIX log: <https://ikigai-rs.dev/ns/log#>
SELECT ?parent ?child WHERE {
  GRAPH ?segment {
    ?p log:invoked ?c ; log:subject ?parent .
    ?c log:subject ?child .
  }
} ORDER BY ?parent ?child
```

<!-- transcript: run -->
```console
$ ikigai --connect restored.sock -c "source urn:sparql:select as=text/csv graph=$(cat segments.txt)" < touched.rq 2>/dev/null
parent,child
urn:sparql:select,urn:log:shop:2026-11-02T09-00-00Z
```

### What it costs

Transreption is O(segment): every line is parsed on every read, and there is no index. `ikigai-log`
measured about 600 ms and 49 MB of Turtle for a default 100,000-entry segment, and a read budget of
16 MiB a segment and 64 MiB a verify walk refuses more rather than truncating. A finished segment
caches under a golden thread on its file, and a live one does not.

For a big log that means:

- Keep `grep` for the first cut, and list only the segments you need as `graph=`.
- Narrow a read by time, with `since=` and `until=`: the timestamp column is fixed-width, so a line
  outside the window is skipped before it is parsed, about twenty times cheaper than `from_seq=`.
- Set `max_entries` so that a segment is a size you are willing to transrept.
- Query sealed segments. A host whose tracer is writing cannot read its own open segment as a
  graph body, since each read logs a line that changes what it is reading:

<!-- transcript: run -->
```console
$ ikigai --connect log.sock -c 'source urn:sparql:select as=text/csv graph=urn:log:shop:2026-11-02T10-00-00Z' < by-principal.rq 2>&1
error: conflict: log segment changed while reading its body
```

## Limits, stated plainly

Each of these is a ledger item, and none is a promise:

- **No index.** Finding one tenant's work is `grep`, or a transreption of every segment.
- **No central collection** (ledger #1764). A log is per process, on one machine's disk; "what
  happened on this machine" is a union you assemble.
- **No heartbeats and no absence query yet** (ledger #292). The log can say what happened, not that
  something stopped happening.
- **gonk runs no log yet** (ledger #383). Nor does any released `ikigai`, as this page opened by
  showing.
- **C1 control characters pass through Turtle.** A line escapes them, but oxrdf's Turtle writer
  does not, so a graph face can hold what a line never does (ledger #2911). Here is U+0085, written
  and then counted raw in the file and in its Turtle:

<!-- transcript: run -->
```console
$ printf 'c1\302\205here' | ikigai --connect log.sock -c 'sink urn:log:write class=log:Message subject=urn:agent:calendar' > /dev/null 2>&1
$ grep -c "$(printf '\302\205')" home/log/shop-2026-11-02T10-00-00Z.log
0
$ cat home/log/shop-2026-11-02T10-00-00Z.log | ikigai --connect log.sock -c 'source urn:log:transrept' 2>/dev/null | grep -c "$(printf '\302\205')"
1
```

- **A false `log:ChainBroken` on every rotation** in `ikigai-log` 0.9.2 (ledger #2951), as shown under
  [Rotation](#rotation-as-it-happens).
