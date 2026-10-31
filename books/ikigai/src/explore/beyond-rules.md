# When a rule is not enough

Everything this part has asked so far was a **rule**: a jq program, a SPARQL question, an XSLT
stylesheet, a template, and in [Naming a composition](naming.md) a binding that gives one of
them a name. A rule answers from what it is handed. A jq query reads the one `src` its caller
names, and a SPARQL question reads the graphs in its `graph`. Neither decides what to read.

This chapter climbs past rules, one rung at a time, and stops on each rung to say where it ends.
The first rung is a **program**, in the Lisp the `ikigai` command carries, that decides which
weeks to read. The second is a **peer**: a few lines of Python in a process of their own, for a
job the program's sandbox does not do.

## What you need

The `~/quakes` directory from the earlier chapters, with its two recorded weeks and the names
file from [Naming a composition](naming.md), and a shell in it. Section 5 also needs `python3`
(3.11 or later, as `ikigai-python` requires) and the `ikigai-python` package.

```bash
cd ~/quakes
```

<!-- transcripts: workspace crates/quakes/quakes-2024-01-01.geojson crates/quakes/quakes-2024-01-08.geojson examples/quakes/near.py -->
<!-- transcripts: normalize workdir -->

The book's checks run every `$` line in sections 1 to 4 against `ikigai-cli`
{{#include ../../ikigai-cli.version}}, in order, in a fresh directory holding the two recorded
weeks. Section 5 starts a second process, which those checks cannot do, so its steps are replayed
by another check, `scripts/polyglot-tracks.sh`. It starts the peer from this page's own lines,
with `ikigai-python` at its `main`, and compares what every `ikigai` command prints. `$PWD` in an
answer stands for your own directory.

This page has no Run buttons. The kernel in the book's pages has no Lisp in it, because
`ikigai-lisp` runs only in a native process, and a page cannot start a Python process.

<!-- urn-gate: unbound urn:quakes:regions — a name the reader binds in their own names file on this page; no host binds it, and the replay binds it from the page. -->
<!-- urn-gate: illustration urn:quakes-py: — the prefix the reader mounts examples/quakes/near.py under, with their own `--mount urn:quakes-py:=`; not a name the CLI or this workspace binds. -->
<!-- urn-gate: illustration urn:regions — the name a mount at `urn:quakes:` forwards for `urn:quakes:regions`, shown as what goes wrong; nothing serves it. -->
<!-- urn-gate: illustration urn:quakes-py:near:35.68:139.69 — one point the peer in examples/quakes/near.py answers, under the reader's own mount; not a name the CLI or this workspace binds. -->

## 1. A question with steps in it

Ask the weeks **where** the earthquakes were. Every earthquake's `place` ends with its region,
after the last comma: `"39 km NE of Anamizu, Japan"` is in Japan. Some places have no comma
(`"Balleny Islands region"`) and are a region already. One place breaks the rule: the Noto
Peninsula earthquake's place is its name, `"2024 Noto Peninsula, Japan Earthquake"`, and what
follows its last comma is `Japan Earthquake`.

Count the earthquakes in each region, keep each region's strongest, and put the busiest region
first. Splitting a string is not what makes this hard to ask as a rule: jq's `split` does it,
and `split(", ") | last` gives a place's region. What does is that which weeks to count is
data, kept in a file you change, and a rule reads only the inputs its caller names. A program
can read the list, then read each week on it. And the Noto place needs one more step: when the
region ends in ` Earthquake`, take that off.

## 2. The program is a file

The `ikigai` command carries a Lisp: `ikigai-lisp`, built on the Steel Scheme engine and
served at `urn:lisp:eval`. Since `ikigai-cli` 0.2.5 it is `ikigai-lisp` 0.4.1, the first whose
sandbox has `sort` (with a comparator of your own) and `string-split`, and this program needs
both. A program is text, so it is a file in the workspace, and the CLI reads it like any other
resource.

Say which weeks to count first, in a file of their names. Save this as `weeks.txt`:

<!-- transcript: file weeks.txt -->

```text
urn:file:quakes-2024-01-01.geojson
```

Then the program. Save it as `regions.scm`:

<!-- transcript: file regions.scm -->

```scheme
; Which regions shook in the weeks `weeks.txt` lists: how many earthquakes each had, and
; the strongest of them, the busiest region first.

; One line per earthquake, "mag<TAB>place", picked out of a week by jq.
(define (rows week)
  (string-split
    (invoke 'source "urn:json:query" "src" week "as" "text/plain"
            "q" ".features | map(\"\\(.properties.mag)\t\\(.properties.place)\") | join(\"\n\")")
    "\n"))

; The region is what follows the place's last ", ", or the whole place when it has none.
; A named earthquake's place is its name ("2024 Noto Peninsula, Japan Earthquake"), so
; " Earthquake" comes off the end.
(define (last xs) (if (null? (cdr xs)) (car xs) (last (cdr xs))))
(define (region place)
  (let ((r (last (string-split place ", "))))
    (if (ends-with? r " Earthquake")
        (substring r 0 (- (string-length r) 11))
        r)))

; Count one earthquake into a tally of (region count strongest) entries.
(define (bump tally reg mag)
  (cond ((null? tally) (list (list reg 1 mag)))
        ((equal? (car (car tally)) reg)
         (let ((e (car tally)))
           (cons (list reg (+ 1 (cadr e)) (if (> mag (caddr e)) mag (caddr e)))
                 (cdr tally))))
        (else (cons (car tally) (bump (cdr tally) reg mag)))))

(define (count-rows tally rows)
  (if (null? rows)
      tally
      (let ((fields (string-split (car rows) "\t")))
        (count-rows (bump tally (region (cadr fields)) (string->number (car fields)))
                    (cdr rows)))))

(define (count-weeks tally weeks)
  (if (null? weeks)
      tally
      (count-weeks (count-rows tally (rows (car weeks))) (cdr weeks))))

; The busiest first; of two as busy, the one with the stronger earthquake.
(define (busier? a b)
  (or (> (cadr a) (cadr b))
      (and (= (cadr a) (cadr b)) (> (caddr a) (caddr b)))))

(define (first-n xs n)
  (if (or (null? xs) (= n 0)) (list) (cons (car xs) (first-n (cdr xs) (- n 1)))))
(define (csv-row e)
  (string-append (car e) "," (number->string (cadr e)) "," (number->string (caddr e))))
(define (join-lines xs)
  (if (null? (cdr xs)) (car xs) (string-append (car xs) "\n" (join-lines (cdr xs)))))

(cacheable
  (join-lines
    (cons "region,quakes,strongest"
          (map csv-row
               (first-n (sort (count-weeks (list) (split-whitespace (source "urn:file:weeks.txt")))
                              busier?)
                        8)))))
```

Read it from the bottom. The last form is the answer: `weeks.txt` read with `source`, its names
split apart, each week counted, the tally sorted with `busier?` and the first eight rows written
out as CSV. `rows` reads one week through `urn:json:query` with `invoke`, the verb that takes an
endpoint's named arguments. jq picks out two fields, and the program does the rest. Each
`(source …)` and `(invoke …)` is a request to the kernel, as a line at the prompt is.

`--load` reads a program resource and evaluates it:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --load urn:file:regions.scm
region,quakes,strongest
Japan,57,7.5
Indonesia,13,5.3
Philippines,7,5
Northern Mariana Islands,7,5
Papua New Guinea,6,5.2
Vanuatu,6,4.8
Japan region,5,4.8
Tonga,4,5.7
[2 computed]
```

Japan has 57 earthquakes, the Noto Peninsula earthquake among them, so its strongest is 7.5.
Without the line that takes ` Earthquake` off, Japan would have 56, and a region called
`Japan Earthquake` would have one. `Japan region` is a separate row: USGS names the sea off Japan
that way, and this program does not merge the two. Moving it into `Japan` would take one more
line in `region`.

The tally says two things were computed: reading `regions.scm` and evaluating it. The reads the
program made went through the kernel inside the evaluation.

## 3. Cached, and recomputed when what it read changes

An evaluation is uncacheable unless the program says otherwise, because a program may write.
This one wraps its answer in `(cacheable …)`. The kernel honors that only when the evaluation
wrote nothing, and the answer is never fresher than what it read. It hangs from the thread of
every resource the program read: the program file, `weeks.txt` and both weeks.

Give the program a name, the way [Naming a composition](naming.md#1-a-name-for-a-request) did.
`in` is `urn:lisp:eval`'s input for the program text, and `<urn:file:regions.scm>` is a
reference, so the kernel reads the file for it. Save these lines as `regions.iki`. In one REPL,
so the cache carries from line to line, they bind the name, ask it twice, add the second week to
`weeks.txt` and ask again:

<!-- transcript: file regions.iki -->

```text
bind urn:quakes:regions = source urn:lisp:eval in=<urn:file:regions.scm>
source urn:quakes:regions
cache urn:quakes:regions
sink urn:file:weeks.txt urn:file:quakes-2024-01-01.geojson urn:file:quakes-2024-01-08.geojson
cache urn:quakes:regions
source urn:quakes:regions
```

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names < regions.iki
bound in <urn:iki:host:names>:
urn:quakes:regions = source urn:lisp:eval in=<urn:file:regions.scm>
[uncacheable]
region,quakes,strongest
Japan,57,7.5
Indonesia,13,5.3
Philippines,7,5
Northern Mariana Islands,7,5
Papua New Guinea,6,5.2
Vanuatu,6,4.8
Japan region,5,4.8
Tonga,4,5.7
[computed]
cached
wrote 69 bytes to weeks.txt
[uncacheable]
not cached
region,quakes,strongest
Japan,63,7.5
Indonesia,21,5.3
Philippines,18,6.7
Tonga,13,5.7
Northern Mariana Islands,13,5
Papua New Guinea,10,5.2
Vanuatu,8,5.2
Japan region,8,4.9
[computed]
```

After the first answer the name was cached. The write to `weeks.txt` cut that file's thread, the
answer hung from it, and so the next read computed the tally again, now over both weeks: the
second week's M6.7 off the Philippines moved the Philippines up to third place. Nobody told
the kernel the program read `weeks.txt`. The program read it through the kernel, so the kernel
knew.

The name went into `names`, the file [Naming a composition](naming.md) wrote, beside the six
names already there.

## 4. What the program can reach

A program's only way out is the kernel's verbs: `source`, `sink`, `meta`, `exists`, `delete`
and `invoke`, each issued under the capability of the session that runs it. The sandbox has no
other door. Every name a program could use to open a file, print, read the environment or load
code is refused, and only a short list of pure functions is left (numbers, lists and strings).

Narrow a session to what the tally needs: reading this directory, and running Lisp
(`urn:cap:lisp`). Then ask for the tally, a write, a request to USGS, and a file opened without
the verbs. `"$PWD"` is in double quotes so the shell puts your directory in the scope:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . --names names -c "cap urn:cap:fs:read:$PWD urn:cap:lisp" -c 'source urn:quakes:regions' -e '(sink "urn:file:weeks.txt" "nothing")' -e '(source "urn:http:get" "https://earthquake.usgs.gov/")' -e '(open-input-file "weeks.txt")'; echo "exit $?"
narrowed — capability: urn:cap:fs:read:$PWD, urn:cap:lisp
region,quakes,strongest
Japan,63,7.5
Indonesia,21,5.3
Philippines,18,6.7
Tonga,13,5.7
Northern Mariana Islands,13,5
Papua New Guinea,10,5.2
Vanuatu,8,5.2
Japan region,8,4.9
[computed]
error: denied: capability does not grant `urn:cap:fs:write:*` (declared by `urn:file:weeks.txt`; any grant under `urn:cap:fs:write:` satisfies it)
error: denied: capability does not grant `urn:cap:net:*` (declared by `urn:http:get`; any grant under `urn:cap:net:` satisfies it)
error: denied: lisp: Error: Generic: `open-input-file` is not available to a program (ikigai-lisp's sandbox allows the kernel verbs and a short list of pure functions; see src/sandbox.rs)
— batch: 5 commands · computed
exit 1
```

The tally ran, because everything it reads is in this directory. The write and the request were
refused by the resources they asked, with the grant each needs. The file opened without a verb
was refused by the sandbox before anything was read. A program has the authority of the session
that runs it, and no more.

The short list of pure functions is also where the sandbox stops. It has `+`, `-`, `*`,
`quotient` and `modulo`, and no division, square root or trigonometry:

<!-- transcript: run -->

```console
$ ikigai --quiet --files . -e '(/ 1 2)' -e '(sqrt 2)'; echo "exit $?"
error: denied: lisp: Error: Generic: `/` is not available to a program (ikigai-lisp's sandbox allows the kernel verbs and a short list of pure functions; see src/sandbox.rs)
error: denied: lisp: Error: Generic: `sqrt` is not available to a program (ikigai-lisp's sandbox allows the kernel verbs and a short list of pure functions; see src/sandbox.rs)
exit 1
```

So the next question is not one for this program.

## 5. Where composition stops: a peer in Python

Name a point, and ask how far each earthquake was from it. Here the point is Tokyo, at
latitude 35.68 and longitude 139.69. The distance between two points on the Earth's surface is a
great circle, and the sum needs a sine, a cosine and an arcsine. jq has those, so a long `q=`
could hold the sum, and the Lisp sandbox has none of them. Neither is where this code belongs.
The day you want the Earth as an ellipsoid rather than a sphere, or a coastline from a shapefile,
you want a library, and the libraries are in languages like Python.

A **peer** is a process of its own that serves resources over the kernel's wire protocol, and
`ikigai-python` (pure Python, one dependency, `blake3`) is how a Python process serves them.
Install it, in a virtual environment if you keep one:

```bash
pip install git+https://github.com/ikigai-rs/ikigai-python
```

Save this as `near.py` (it is `examples/quakes/near.py` in this book's repository):

```python
{{#include ../../../../examples/quakes/near.py:near}}
```

`@endpoint` gives the function a name, and the signature is its contract. `{lat}` and `{lon}`
come from the name, so every point has a name of its own. `week` is the one input left, so a
piped value reaches it. The peer reads no file and asks the kernel for nothing: the week it
measures is the week it is handed.

Start the peer, and wait for its socket before you mount it (the `rm -f` clears a socket an
earlier run left behind):

<!-- demo: start -->
```bash
rm -f /tmp/quakes-near.sock
python3 near.py /tmp/quakes-near.sock &
until [ -S /tmp/quakes-near.sock ]; do sleep 0.2; done
```

`--mount urn:quakes-py:=/tmp/quakes-near.sock` mounts it: the CLI forwards every name under
`urn:quakes-py:` to that socket. Pipe each week into the point:

<!-- demo: run -->
```text
$ ikigai --quiet --files . --mount urn:quakes-py:=/tmp/quakes-near.sock -c 'source urn:file:quakes-2024-01-01.geojson | urn:quakes-py:near:35.68:139.69' -c 'source urn:file:quakes-2024-01-08.geojson | urn:quakes-py:near:35.68:139.69'
km,mag,place
168,4.5,"39 km ESE of Takahagi, Japan"
282,4.6,"55 km ENE of Anamizu, Japan"
283,5.2,"56 km ENE of Anamizu, Japan"
286,4.7,"66 km NNW of Itoigawa, Japan"
287,4.6,"71 km NNW of Itoigawa, Japan"
[1 computed · 1 uncacheable]
km,mag,place
280,4.5,"63 km NNW of Itoigawa, Japan"
291,4.5,"42 km NE of Anamizu, Japan"
296,5.9,"53 km WSW of Sado, Japan"
297,4.5,"60 km WSW of Sado, Japan"
426,4.6,"Izu Islands, Japan region"
[1 computed · 1 uncacheable]
— batch: 2 commands · 2 computed · 2 uncacheable
```

The nearest earthquake to Tokyo in the first week was 168 km away, off Takahagi; the Noto
aftershocks were nearly 300 km away, across the island.

## 6. Two processes

That was two processes: the CLI, and Python. The pipe crossed between them twice, the week out
and the CSV back. Three things follow from that, and they are why a peer is the last rung here
rather than a bigger rule.

**A peer has its own cache, or none.** Each stage's tally above says `1 uncacheable`, and the
peer says `cacheable=True`. Ask the same point twice, and ask the kernel why it kept nothing:

<!-- demo: run -->
```text
$ ikigai --quiet --files . --mount urn:quakes-py:=/tmp/quakes-near.sock -c 'source urn:file:quakes-2024-01-01.geojson | urn:quakes-py:near:35.68:139.69' -c 'source urn:file:quakes-2024-01-01.geojson | urn:quakes-py:near:35.68:139.69' -c 'source urn:kernel:uncached'
…
[1 computed · 1 uncacheable]
…
[1 cached · 1 uncacheable]
uncached (why a computed read was not cached; the last 64, most recent first)
  urn:quakes-py:near:35.68:139.69      ×2  declared  [root]

[uncacheable]
— batch: 3 commands · 1 cached · 1 computed · 3 uncacheable
```

The week came from the CLI's cache the second time; the peer was asked again. `ikigai-cli`
{{#include ../../ikigai-cli.version}} keeps no copy of what a mounted peer answers, whatever the
peer declares (`declared` is the mount's own declaration). A peer can change its answer without
the kernel hearing of it, because no thread crosses the socket, so a copy could be served stale
for as long as the CLI runs. The Lisp program's answer hung from every file it read; the peer's
answer hangs from nothing the kernel can cut.

**A peer is another authority.** The program in section 4 had exactly the session's capability,
and a narrowed session narrowed it. The peer runs with whatever its process may do, as the user
who started it, and the session's capability decides only whether the session may ask it. This
peer declares nothing it requires, so even a `cap read-only` session reaches it. That it reads
no files is a property of these lines of Python, not something the kernel enforces. It also has
no way back: `ikigai-python` has no channel back to the mounting kernel to read a name through,
so this peer could not read `urn:file:quakes-2024-01-01.geojson` if it tried, and the week has to
be piped in.

**A mount takes a whole prefix.** The CLI serves a mount ahead of its names layer, so a peer
mounted at `urn:quakes:` would take every name under it, `urn:quakes:regions` included, and
answer none of them. That is why this peer has a first segment of its own, `urn:quakes-py:`. (On
`ikigai-cli` 0.2.6, on 2026-10-30, mounting this peer at `urn:quakes:` made `source
urn:quakes:regions` fail: the CLI forwarded `urn:regions` to the peer, which serves no such name.)

Stop the peer when you are done (`kill %1` in the shell that started it), or leave it running:
nothing later in this part uses its socket.

## 7. Spaces that change

One more rung is not shown here: a **dynamic space**, whose doors are rebuilt from a
declaration while the kernel runs. You have used one already. The names layer of [Naming a
composition](naming.md) is `urn:iki:space:dynamic:names`, rebuilt each time `names` changes.
A dynamic space of your own binds its doors to endpoints a host registers, and this part writes
no host, and this page's kernel holds no dynamic space, so there is no Run cell for one here.
[A space that changes while it runs](../applied/live-space.md), later in the book, builds one
in Rust and runs it in its page.

## 8. The ladder

- **A rule** (jq, SPARQL, XSLT, a template) answers from the inputs its caller names. It stops
  where the question has to decide what to read.
- **A name** for a rule ([Naming a composition](naming.md)) fixes its inputs and keeps it
  cached. It is still one request.
- **A program** (Lisp, this chapter) reads what it decides to read, through the kernel, under
  the session's capability, and its answer is cached and recomputed by the threads of everything
  it read. It stops at the sandbox's list of pure functions.
- **A peer** (Python, this chapter) is any code you like, in a process of its own. It stops being
  the kernel's to govern: its own authority, and no copy of its answers in the CLI's cache.

Each rung up gives you more to say and gives the kernel less it can check for you.
