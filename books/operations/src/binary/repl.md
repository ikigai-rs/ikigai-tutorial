# The REPL and its modes

<!-- transcripts: normalize workdir, home -->

`ikigai` is one engine with several ways in. The grammar is the same in all of them (`source`,
`sink`, `describe`, pipes, named arguments); what changes is who is typing, what else gets printed,
and how long the process lives. The process's lifetime matters more than it looks: the cache lives
in the process, so a REPL remembers what it resolved and a one-shot command does not.

| Mode | What you get | Use it for |
|------|--------------|------------|
| `ikigai` at a terminal | the full-screen REPL | exploring by hand |
| `ikigai --plain` | the line REPL: a prompt, a greeting, nothing drawn | a terminal that cannot take the full screen |
| `ikigai --quiet` | the line REPL with no greeting, and no prompt when stdin is a file or a pipe | replaying a session |
| `ikigai -c '…'` | one statement per `-c`, then exit | scripts, cron jobs, one-off reads |
| `ikigai -e '(…)'` | one Lisp form, then exit | a computation the grammar cannot say |
| `ikigai --load <iri>` | a Lisp script, read as a resource | a program kept beside its data |

Every command on this page ran in an empty directory, with an empty config home. If yours composes
mounts (a `mount` line in its `config.toml`), `ikigai` names them on stderr where this page shows
`none composed`, and nothing else here changes. *The config home*, a later chapter, is about that.

## At a terminal: the full-screen REPL

<!-- transcript: manual — the full-screen REPL draws on a terminal, and the check runs every
     command without one. -->
```console
$ ikigai
```

Type a line at the `ikigai>` prompt and press Return. `help` lists the grammar, `quit` (or Ctrl-D)
leaves. The REPL keeps its cache for as long as it runs, so the second read of the same thing is
answered from memory, and says so.

You can try the grammar here without installing anything. The lines beside each **Run** button are
REPL lines, answered by a kernel compiled into this page: the same engine, over a smaller set of
names than the full binary binds (the function library is here; files and servers are not).

<div class="ikigai-run" data-cmd='source urn:iki:fn:toUpper in="resource oriented"
source urn:iki:fn:toUpper in="resource oriented"'>
<pre class="ikigai-run-expected">RESOURCE ORIENTED
[computed]
RESOURCE ORIENTED
[cached]</pre>
</div>

The first line computed its answer; the second, the same request again, was served from the cache.
That second line is the whole difference between a REPL and the one-shot `-c` below: a `-c`
process exits when its statements are done, and its cache goes with it.

## `--plain`: the line REPL

`--plain` is the same REPL without the full screen: a prompt and plain lines, for a terminal that
cannot draw one, or a session you pipe in. It is chosen for you when stdin is not a terminal.

<!-- transcript: run; normalize version -->
```console
$ printf 'source urn:iki:fn:toUpper in="plain"\nquit\n' | ikigai --plain
ikigai 0.1.69 — REPL. Type `help`, or `quit` to exit.
ikigai: mount   none composed — no `mount` lines in the config home
ikigai> PLAIN
[computed]
ikigai>
```

It greets you with its version and says what it composed, on stderr, and prompts before every line
it reads, even when nobody is there to see the prompt.

## `--quiet`: a session that replays

`--quiet` is the line REPL with the greeting gone too, and, when stdin is not a terminal, no
prompts: it prints the answers and nothing else. That is what makes a session something you can
keep in a file and run again. Save three lines as `session.ik`:

<!-- transcript: file session.ik -->
```text
source urn:iki:fn:toUpper in="replay me"
source urn:iki:fn:toUpper in="replay me"
cache urn:iki:fn:toUpper in="replay me"
```

and replay it:

<!-- transcript: run -->
```console
$ ikigai --quiet < session.ik
REPLAY ME
[computed]
REPLAY ME
[cached]
cached
```

One process read all three lines, so the second read hit the cache, and `cache` (which asks
whether a request would be answered from the cache, without making it) says it would. Whatever the
REPL prints for you, a replay prints the same, in the same order, so a saved session is also a
test: run it again after an upgrade and compare.

## `-c`: a batch

`-c` runs one statement and exits. Repeat it for more; each `-c` is ONE statement, and a newline
inside it does not split it. The statements run in order in one process, so they share a cache, and
a batch of more than one ends with a line that counts them:

<!-- transcript: run -->
```console
$ ikigai -c 'source urn:iki:fn:toUpper in="a b"' -c 'source urn:iki:fn:toUpper in="a b"'
A B
[computed]
A B
[cached]
— batch: 2 commands · 1 cached · 1 computed
```

The answers go to stdout and the bracketed status lines to stderr, so a script that captures
stdout gets only the answers. A `;` does not separate statements, here or at the prompt; it
belongs to a fork (below), and outside one it is refused:

<!-- transcript: run -->
```console
$ ikigai -c 'source urn:iki:fn:toUpper in=one ; source urn:iki:fn:toUpper in=two'
error: `;` outside a `( … )` fork
```

A statement that fails does not stop the batch: the rest still run, and the batch exits with a
status of 1, so a script can tell.

<!-- transcript: run -->
```console
$ ikigai -c 'source urn:iki:fn:toUpper' -c 'source urn:iki:fn:toUpper in=second' < /dev/null; echo "exit $?"
error: missing required argument `in`
       `in`: the text to upper-case
       pipe a value in or pass in=…
SECOND
[computed]
— batch: 2 commands · computed
exit 1
```

## Stdin, in a batch

A `-c` batch whose stdin is not a terminal reads it ONCE, for the first statement that wants it,
and a statement wants it only when it is one of these:

- a `source` stage with no value piped or written after it, and exactly one REQUIRED argument left
  unnamed: stdin fills that argument;
- a `sink` given no content: stdin is the content, so a secret never has to appear on the command
  line;
- a `delete` given no content whose target declares one.

<!-- transcript: run -->
```console
$ printf 'from a pipe' | ikigai -c 'source urn:iki:fn:toUpper'
FROM A PIPE
[computed]
```

`toUpper` has one argument, `in`, and it is required, so stdin filled it. Name the argument
yourself and stdin is not read at all:

<!-- transcript: run -->
```console
$ printf 'never read' | ikigai -c 'source urn:iki:fn:toUpper in=named'
NAMED
[computed]
```

Read once means once: the first statement that wants stdin gets it, and a second one that wants it
finds nothing.

<!-- transcript: run -->
```console
$ printf 'piped' | ikigai -c 'source urn:iki:fn:toUpper' -c 'source urn:iki:fn:toUpper'
PIPED
[computed]
error: missing required argument `in`
       `in`: the text to upper-case
       pipe a value in or pass in=…
— batch: 2 commands · computed
```

A `sink` with no content takes stdin as its content. `--files` sets the directory `urn:file:` names
resolve in (by default `~/.ikigai/workspace`); here it is the directory you are in:

<!-- transcript: run -->
```console
$ printf 'remember the milk' | ikigai --files "$PWD" -c 'sink urn:file:notes.txt'
wrote 17 bytes to notes.txt
[uncacheable]

$ ikigai --files "$PWD" -c 'source urn:file:notes.txt'
remember the milk
[computed]
```

Two things about stdin cost operators time. First, a pipe nobody writes to (a supervisor's, or an
agent's harness) is not an empty stdin, it is a stdin that has not said anything yet, so a statement
that wants it waits: two seconds for a first byte, then it goes without and says so.

<!-- transcript: run -->
```console
$ sleep 3 | ikigai -c 'source urn:iki:fn:toUpper'
ikigai: nothing arrived on stdin within 2s, so the command that asked for it was sent without it (pipe the value in, or name it with content=…)
error: missing required argument `in`
       `in`: the text to upper-case
       pipe a value in or pass in=…
```

Second, an empty stdin is no value, never an empty one: a required argument is refused as missing
rather than filled with nothing. Redirect from `/dev/null` and the answer comes at once, without the
wait:

<!-- transcript: run -->
```console
$ ikigai -c 'source urn:iki:fn:toUpper' < /dev/null
error: missing required argument `in`
       `in`: the text to upper-case
       pipe a value in or pass in=…
```

So a batch run by cron, launchd or another program, which needs nothing on stdin, should be given
`< /dev/null`.

## Pipes inside a line: `|`, `..` and a fork

Inside one statement, `|` pipes the whole output of one stage into the next stage's unnamed
argument, `..` maps (runs the next stage once per line of the output, and joins the answers), and
`( a ; b )` forks (sends the same input to each branch, and joins their outputs in order). These are
the grammar's, not the shell's: quote the statement so the shell passes them through.

<!-- transcript: run -->
```console
$ ikigai -c 'source urn:demo:split "a,b,c" | urn:iki:fn:reverseList'
c
b
a
[2 computed]

$ ikigai -c 'source urn:demo:split "a,b,c" .. urn:iki:fn:toUpper'
A
B
C
[4 computed]
[fan-out 3 → 1 wide]

$ ikigai -c 'source urn:demo:split "a,b,c" | ( urn:iki:fn:toUpper ; urn:iki:fn:reverseList )'
A
B
C
c
b
a
[3 computed]
[fan-out 2 → 1 wide]
```

`[4 computed]` counts the resolutions a line made: the split, and `toUpper` once per item. The
`fan-out` line says how wide the map or fork ran: three items (or two branches), one at a time,
because the default scheduler runs them serially. The same three lines, in the page:

<div class="ikigai-run" data-cmd='source urn:demo:split "a,b,c" | urn:iki:fn:reverseList
source urn:demo:split "a,b,c" .. urn:iki:fn:toUpper
source urn:demo:split "a,b,c" | ( urn:iki:fn:toUpper ; urn:iki:fn:reverseList )'>
<pre class="ikigai-run-expected">c
b
a
[2 computed]
A
B
C
[1 cached · 3 computed]
A
B
C
c
b
a
[2 cached · 1 computed]</pre>
</div>

The answers are the same and the counts are not, for the reason this chapter opened with: the page
runs all three lines in one kernel, as a REPL would, so the split computed by the first line is
served from the cache to the second and third, and so is `reverseList` of it. Each `-c` above was a
fresh process with an empty cache. (The page's kernel has no scheduler, so it prints no `fan-out`
line.)

## `-e`: one Lisp form

`-e` evaluates one Lisp form and exits. The kernel's verbs are Lisp functions, so a form can read
resources:

<!-- transcript: run -->
```console
$ ikigai -e '(source "urn:iki:fn:toUpper" "from lisp")'
FROM LISP
[uncacheable]
```

There is no Run button for this one: the kernel in this page has no Lisp in it.

## `--load`: a script, read as a resource

`--load` reads a resource and evaluates it as Lisp: a script kept where your data is, read through
the kernel like anything else. Save this as `shout.scm` in the directory you are in:

<!-- transcript: file shout.scm -->
```scheme
(define (shout s) (source "urn:iki:fn:toUpper" s))
(shout "loaded")
```

and load it from there:

<!-- transcript: run -->
```console
$ ikigai --files "$PWD" --load urn:file:shout.scm
LOADED
[1 computed · 1 uncacheable]
```

`--cap` narrows the capability the script runs under before it starts. Under `read-only`, which may
read but holds no grant to evaluate Lisp, the script is read and then refused:

<!-- transcript: run -->
```console
$ ikigai --files "$PWD" --load urn:file:shout.scm --cap read-only
error: denied: capability does not grant `urn:cap:lisp` (declared by `urn:lisp:eval`)
[computed]
```

The `[computed]` is the read of the script, which the capability allowed.

## `describe` and `help`

`describe` asks a resource to describe itself (the `Meta` verb). With no type it answers Turtle;
`text/plain` is the face for people:

<!-- transcript: run -->
```console
$ ikigai -c 'describe urn:iki:fn:toUpper text/plain'
toUpper — Upper-case
Upper-cases the UTF-8 text supplied in the `in` argument.
verbs: Source, Meta
  input in [argument]: the text to upper-case
outputs: text/plain;charset=utf-8
[computed]
```

That description is how a value finds its argument: `toUpper` declares one input, `in`, required,
so a pipe, a positional word or stdin fills it. In the page:

<div class="ikigai-run" data-cmd='describe urn:iki:fn:toUpper text/plain'>
<pre class="ikigai-run-expected">toUpper — Upper-case
Upper-cases the UTF-8 text supplied in the `in` argument.
verbs: Source, Meta
  input in [argument]: the text to upper-case
outputs: text/plain;charset=utf-8
[computed]</pre>
</div>

`help` prints the grammar, the quoting rules and the stdin rules above, with examples; it is long,
and this is its start:

<!-- transcript: run -->
```console
$ ikigai -c help
commands:
  source <iri> [input]       SOURCE a resource; `input` is routed to its declared argument
  source <iri> key=value …   name arguments; positional/pipe fills the one left unnamed
  source a [input] | b | c   pipeline: `|` pipes the whole output into the next stage
  source a [input] .. b      map: run `b` per newline-item of `a`'s output, rejoin
…
```

`ikigai --help` is the other one: the binary's flags and subcommands, as opposed to the REPL's
grammar.

## `--connect`: the same REPL, another kernel

Every mode above builds a kernel inside the `ikigai` process. `--connect` attaches the same REPL,
or the same `-c` batch, to a kernel another process serves. Start one on a Unix socket in the
directory you are in, serving files from it, and leave it running:

<!-- transcript: serve -->
```console
$ ikigai serve k.sock --files "$PWD"
ikigai: mount   none composed — no `mount` lines in the config home
ikigai: watching $PWD (a file changed on disk is served fresh)
ikigai: serving on k.sock  (Ctrl-C to stop)
```

Then, from another terminal in the same directory:

<!-- transcript: run -->
```console
$ ikigai --connect k.sock -c 'source urn:file:notes.txt'
remember the milk
[computed]

$ ikigai --connect k.sock --quiet < session.ik
REPLAY ME
[computed]
REPLAY ME
[cached]
cached

$ ikigai --connect k.sock -c 'cache urn:iki:fn:toUpper in="replay me"'
cached
```

The first command read the note through the server's file root, not its own. The last is the one
to notice: a new process, with nothing cached, asked whether the read would be cached, and the answer
was yes, because the cache it asked about is the server's, which the replay before it filled. A
kernel served this way keeps its cache across every client that connects.

Stop the server with Ctrl-C in its terminal, or, from elsewhere, by its process ID (`kill <pid>`),
never by a pattern that could match another `ikigai` on the machine. *A kernel over a socket*, a
later chapter, is about serving.

## When it does not work

The refusals on this page, and what each means:

- ``error: missing required argument `in` ``: the statement reached a resource without a value for an
  argument it requires. Name it (`in=…`), pipe it, or check that stdin had something in it.
- `ikigai: nothing arrived on stdin within 2s`: a statement wanted stdin and the pipe was idle. Give
  the batch `< /dev/null`, or the value.
- ``error: `;` outside a `( … )` fork``: two statements in one `-c`. Use one `-c` each.
- `error: denied: capability does not grant …`: the capability the line ran under (here narrowed by
  `--cap`) holds no grant for what the resource declares it needs.
- `ikigai: connect …: No such file or directory`: `--connect` names a socket that is not
  there:

<!-- transcript: run -->
```console
$ ikigai --connect nobody.sock -c 'source urn:iki:fn:toUpper in=x'
ikigai: connect nobody.sock: No such file or directory (os error 2)
```
