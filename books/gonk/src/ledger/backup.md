# Backing up and restoring

Everything so far lives in gonk's store, and gonk holds the store's only writer lock. A copy of
the store's directory taken while gonk runs is torn: its files change under the copy, and the
result may not open, or may open and be quietly short. So gonk takes its own backups. It takes one
every 24 hours, keeps the last five, and you can ask for one at any time over the owner's socket. A
restore always builds a **new** store beside the live one and checks it before it says done.

This chapter takes both kinds of backup gonk writes, a single archive and a snapshot in pieces,
restores each into a directory of its own, and starts a second gonk on what it restored.

## A store worth keeping

In a new empty directory, as in [every chapter](../introduction.md#a-gonk-per-chapter): if the gonk
from [A lifecycle for your ledger](lifecycle.md) is still running, stop it first, since this one
takes the same port. `--no-backup` turns off the schedule only. The backup resources stay bound,
and the banner says so:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.3 — holding the store at …/.ikigai/store
…
  backup  no schedule (--no-backup or gonk.backup.every = "off") — urn:iki:gonk:backup is still bound; this process is the only thing that can export the dataset
…
```

In another terminal, in the same directory, file three items in `default` and close one, and file
one more in a second ledger, `ops`, so the store holds two ledgers' graphs:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append Rotate the backup key' -c 'sink urn:gk:iki:ledger:append priority=1 Copy backups off the host' -c 'sink urn:gk:iki:ledger:append Practice a restore' -c 'sink urn:gk:iki:ledger:close item=1 reason=done'
#1 urn:iki:ledger:default:item:…
[uncacheable]
#2 urn:iki:ledger:default:item:…
[uncacheable]
#3 urn:iki:ledger:default:item:…
[uncacheable]
closed #1 (done)
[uncacheable]
— batch: 4 commands · request outcomes: 4 uncacheable
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:ops:append Check the backup disk'
ops#1 urn:iki:ledger:ops:item:…
[uncacheable]
```

## A backup, now

`urn:iki:gonk:backup` takes one. It is bound behind the owner's socket and nowhere else: the HTTP
door grants it to no one, and a QUIC client's grant cannot name it.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:gonk:backup'
backup gonk-store-….nq.gz
  43 quads in 2 graph(s), … bytes of N-Quads -> … bytes gzipped
        34  urn:iki:ledger:graph:default
         9  urn:iki:ledger:graph:ops
  sha256 (n-quads) …
  sha256 (archive) …
  scripts gonk-scripts-….json.gz: 0 script(s), 0 file(s), … bytes gzipped
  kept 5, pruned nothing
[uncacheable]
$ ls .ikigai/backups
gonk-scripts-….json.gz
gonk-store-….nq.gz
gonk-store-….nq.gz.meta.json
```

The archive is N-Quads, gzipped, so every quad keeps its graph. That is what keeps each ledger's
grants meaningful after a restore. The scripts from [Your first saved query](../scripts/first.md)
are files beside the store, not quads in it, so every backup writes them as a second archive under
the same stamp. The sidecar `.meta.json` records both, with their digests and the per-graph
counts. `backup:status` reads it back:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:gonk:backup:status'
backups in …/.ikigai/backups
  schedule  OFF (gonk.backup.every = "off") — keep 5
  newest    gonk-store-….nq.gz
    taken_at       …
    quads          43
    nquads_bytes   …
    archive_bytes  …
    nquads_sha256  …
    graphs         "urn:iki:ledger:graph:default" 34, "urn:iki:ledger:graph:ops" 9
    scripts        gonk-scripts-….json.gz: 0 script(s), 0 file(s)
  rotation  1 archive(s)
…
  timer     not scheduled in this process
[uncacheable]
```

## A restore is a new store

Loading an archive into a store that already holds quads would be a **merge**: the union brings
back everything either side deleted. So a restore builds a store in a directory that does not
exist yet, and refuses any other. Read the archive out with `backup:archive:{name}` and pipe it
into `restore`:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c "source urn:gk:iki:gonk:backup:archive:$(cd .ikigai/backups && ls gonk-store-*.nq.gz) | sink urn:gk:iki:gonk:restore into=$PWD/restored"
restored 43 quads into …/restored (gzip, sha256 …)
        34  urn:iki:ledger:graph:default
         9  urn:iki:ledger:graph:ops
  graph set and per-graph counts match the archive exactly
  this server still holds the LIVE store; to adopt this one, stop gonk and swap the directories
[2 uncacheable]
```

The restore counts the quads in each graph it loaded and compares them with what the archive
recorded. Only a match leaves a store behind. Adopting it is your step, with gonk stopped, so the
one destructive move is the one you make by hand.

Since gonk ledger-v0.22.2, every backup gonk writes can also be restored (gonk PR 374). A restore
expands the whole archive in memory, and the most a run may hold is now the restore's own bound,
1 GiB. Before that, the limit was 64 MiB, so a backup past 64 MiB of N-Quads was taken and then
refused by its own restore.

## A snapshot in pieces

The single archive has a ceiling: 1 GiB of N-Quads and 10 million quads. A store past it is backed
up as a **snapshot in pieces** instead. The scheduled backup switches to pieces when it reaches the
ceiling, and `backup:chunks` takes one directly at any size, so you can see one here:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:gonk:backup:chunks'
backup gonk-chunks-…
  43 quads in 2 graph(s), 2 bounded pieces
  manifest sha256 …
[uncacheable]
$ ls .ikigai/backups/gonk-chunks-*
00000000.nq.gz
00000001.nq.gz
manifest.json
scripts.json.gz
$ jq -c '.format, (.pieces[] | {bytes, graphs}), .graphs' .ikigai/backups/gonk-chunks-*/manifest.json
"gonk-chunks/1"
{"bytes":…,"graphs":{"urn:iki:ledger:graph:default":34}}
{"bytes":…,"graphs":{"urn:iki:ledger:graph:ops":9}}
{"urn:iki:ledger:graph:default":34,"urn:iki:ledger:graph:ops":9}
```

The snapshot is one directory. Each piece is gzipped N-Quads from a single graph, at most 64 MiB
and 100,000 quads, so a large graph takes as many pieces as it needs. The manifest records each
piece's size, digest and per-graph counts, and the counts for the whole snapshot. The pieces are
not separate backups. Copy the **whole directory**, manifest included, when you move one off the
host.

Restore it by name, into a new directory again:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c "sink urn:gk:iki:gonk:restore:chunks content=$(cd .ikigai/backups && ls -d gonk-chunks-*) into=$PWD/restored-pieces"
restored 43 quads in 2 graph(s) into …/restored-pieces
  every piece digest and graph count verified
  graph set and per-graph counts match the snapshot exactly
  this server still holds the LIVE store; stop gonk before swapping directories
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c "sink urn:gk:iki:gonk:restore:chunks content=$(cd .ikigai/backups && ls -d gonk-chunks-*) into=$PWD/restored"
error: invalid argument `into`: …/restored is not empty. Restoring into a non-empty store is a MERGE, not a restore: the union of two datasets brings back everything either of them deleted. Name a directory that does not exist yet.
       `into`: a NEW store directory, absent or empty
```

Each piece is expanded and checked against its digest and counts before its quads are loaded into
a staged store. At the end, the restored graphs and their counts are compared with the manifest.
A missing piece, a digest, length or count that does not match, bad gzip, or a quad repeated
across pieces each stops the restore before it publishes anything. The target is left absent or
empty, and you can try again. The second command shows the merge refusal, with the first
restore's directory as its target.

What this looks like at scale is gonk's own drill, `examples/backup-memory` in its repository:
10 million quads take about 5 minutes to back up in pieces and about 4.5 minutes to restore. It
has not been run for this book.

## The restored store, served

A restored directory is a complete store. Start a second gonk on it, with its own data home and
another port, and read it the way you read the first:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai-restored" --store "$PWD/restored-pieces" --port 1071 --no-quic --no-backup
ikigai-gonk 0.1.3 — holding the store at …/restored-pieces
…
```

<!-- transcript: run -->
```console
$ curl -s http://127.0.0.1:1071/iki/ledger/items
   #3  open    no priority set  Practice a restore
   #2  open    priority 1  Copy backups off the host
2 item(s)
$ ikigai --mount "urn:gk:=$PWD/.ikigai-restored/gonk.sock" -c 'source urn:gk:iki:ledger:ledgers'
default                       2 open      3 total  urn:iki:ledger:graph:default
ops                           1 open      1 total  urn:iki:ledger:graph:ops
2 ledger(s)
[uncacheable]
```

Both ledgers came back, with the closed item still closed. Stop both gonks before the next
chapter.

In resource terms: a backup is a **view** of the whole dataset at one snapshot, written as files.
A restore is the one write that makes a new store, and it is checked against the counts that view
recorded. [Over the socket](../agent/socket.md), the next chapter, reaches the ledger from another
process, the way an agent does.
