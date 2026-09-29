# Tic-tac-toe, VI: the same game in Python and TypeScript

Five parts built one game in Rust, and nothing in the model said it had to be Rust. This
part moves it into two other languages, in two different ways.

**The atom in another language.** Only the stored cell holds a game. Part IV turned it
into a seam, the store contract, which any space in any language can fill. Here a Python
process and a TypeScript process each fill it. Everything above the atom stays in a Rust
kernel and does not change: the platonic cell, the lines, the rules, the cache and the
golden threads. The commands you ran in the earlier parts answer the same, word for word,
over a game whose marks live in a Python dictionary.

**A whole app in another language.** A Python app and a TypeScript app each serve Part V's
board to a browser. They do not compute the game. They fill Part V's templates from the
host's resources and send every write through the host. The HTMX board on this book's
pages, on `ttt-host`, and in each app is one markup.

The polyglot faces are the ones the [Python](../polyglot/python.md) and
[TypeScript](../polyglot/typescript.md) tracks teach. Each track's section on families
introduces the templated, multi-verb doors this part uses.

> **Nothing below runs in this page.** Both shapes need processes and sockets. The
> transcripts were captured from real runs against `ikigai-python` and `ikigai-deno` at the
> commits cited beside them, and `scripts/ttt-polyglot-demo.sh` in this repository runs the
> whole part again. It starts both stores, the host and both apps, reruns every command
> shown here, and compares the output with this page. It also checks that each code excerpt
> from those repositories still appears in them verbatim. CI does not run it, because it
> needs Python, Deno and two other checkouts. The two runnable cells are the exception:
> they ask this page's own kernel the same questions, so you can see that the answer does
> not depend on the language holding the store.

## Shape 1: the contract is the whole interface

Part IV wrote down what a store must do, and `check_store` checks it:

1. It binds `urn:iki:tutorial:ttt:stored:{x}:{y}` for any integer in its one plain
   spelling, and refuses `01`, `+1` and `-0`.
2. **Source** answers the mark, as `text/plain`, cacheable. An unplayed square is
   `NotFound`, not an empty answer.
3. **Sink** keeps any non-empty mark from `content`, trimmed.
4. **Exists** answers `true` or `false`, cacheable.
5. **Delete** clears the square, whether or not anything was there.

That is the entire interface between the game and whatever holds it. The game never asks
a store anything else, so a store never needs to know anything else. It does not know the
rules, which lines exist, or that anything is cached.

Both faces serve the stored cell with a **family**: one templated door answering four
verbs, which the language tracks introduce. In Python, `family(pattern)` gives a door, and
each verb is a decorated function whose parameters include the template's variables.

<!-- excerpt: ikigai-python examples/tictactoe_store.py @ 26e173a -->
```python
    cell = family(
        STORED,
        id="ttt-stored",
        title="Stored cell",
        summary="What has been played at (x, y): read it, play a mark, or clear it.",
    )

    @cell.source(cacheable=True, summary="the mark played at (x, y); NotFound if none has been")
    def read(x: Column, y: Row) -> str:
        at = _square(x, y)
        with store.lock:
            store.reads += 1
            mark = store.marks.get(at)
        if mark is None:
            raise NotFoundError(f"nothing has been played at {x},{y}")
        return mark
```

In TypeScript, `family(pattern, options)` is a builder, with one chained call per verb:

<!-- excerpt: ikigai-deno examples/tictactoe_store.ts @ d212d8c -->
```ts
  return family(STORED, {
    id: "ttt-stored",
    title: "Stored cell",
    summary:
      "What has been played at (x, y): read it, play a mark, or clear it.",
    bindings: coordinates,
  })
    .source({
      summary: "the mark played at (x, y); NotFound if none has been",
      output: TEXT_PLAIN_UTF8,
      cacheable: true,
    }, (args) => {
      const square = at(args);
      store.reads += 1;
      const mark = store.marks.get(square);
      if (mark === undefined) {
        throw new NotFoundError(`nothing has been played at ${square}`);
      }
      return mark;
    })
```

The Sink and the Delete follow in the same style in both files. Neither file declares an
Exists: both faces default it to "would Source succeed", which is clause 4. Neither file
has a line about caching beyond `cacheable`, or about invalidation at all, because the
host kernel does both.

### Start two stores

From a checkout of this repository, with `ikigai-python` and `ikigai-deno` checked out
beside it and `ttt-host` installed from here (`cargo install --path crates/ttt-host
--locked`):

<!-- demo: start -->
```sh
mkdir -p /tmp/ttt6demo
(cd ../ikigai-python && PYTHONPATH=src python3 -m examples.tictactoe_store /tmp/ttt6demo/py.sock) &
(cd ../ikigai-deno && deno run -A examples/tictactoe_store.ts /tmp/ttt6demo/ts.sock) &
```

The sockets live in `/tmp/ttt6demo` because macOS allows a socket path 104 bytes, and a
deep temporary directory does not leave room for the file name.

### The same commands, a Python-held game

`ttt-host --store <socket>` makes the root game's store the peer on that socket. Before it
accepts the peer, it runs `check_store` against it and refuses to start if a clause fails,
naming the clause. `-c` runs one REPL line against the host's kernel, in this process, and
prints what the line answers, the way the book's cells print it. These are Part I's,
Part II's and Part III's commands:

<!-- demo: run -->
```text
$ ttt-host --store /tmp/ttt6demo/py.sock \
    -c 'source urn:iki:tutorial:ttt:cell:1:1' \
    -c 'sink urn:iki:tutorial:ttt:move:1:1' \
    -c 'trace urn:iki:tutorial:ttt:cell:1:1' \
    -c 'source urn:iki:tutorial:ttt:board' \
    -c 'source urn:iki:tutorial:ttt:board' \
    -c 'sink urn:iki:tutorial:ttt:move:1:1' \
    -c 'sink urn:iki:tutorial:ttt:reset'
-
[computed]
X plays 1,1
[uncacheable]
trace  urn:iki:tutorial:ttt:cell:1:1
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process

urn:iki:tutorial:ttt:cell:1:1   ttt-cell · computed · main · —   → 1b  X
└─ urn:iki:tutorial:ttt:stored:1:1   ttt-stored · computed · main · —
   └─ urn:iki:tutorial:ttt:stored:1:1   ttt-stored · computed · Thread-2 (_handle) · 0ms
---
-X-
---
[computed]
---
-X-
---
[cached]
error: invalid argument `x, y`: 1,1 is taken — X played there
The board is clear
[uncacheable]
```

Every verdict is the Rust kernel's. The empty cell was `NotFound` in Python and `-` in
Rust, because the platonic cell turns exactly that refusal into the empty cell. The move
checked the rules in Rust and wrote one mark into Python. The board was computed once and
then served from cache, and the refusal came from rules that never saw a line of Python.

The trace is the part to look at. The cell's child is the stored cell, twice: once as the
host's kernel resolved it, on its `main` thread, and once more as a span from the other
process, on a thread called `Thread-2 (_handle)`, which is Python's name for the thread
serving that connection. The span came back over the wire and was stitched into the host's
tree. A resolution that crosses a language boundary can be traced across it.

Here are the same lines in this page's kernel, where the store is Rust:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:cell:1:1
sink urn:iki:tutorial:ttt:move:1:1
trace urn:iki:tutorial:ttt:cell:1:1
source urn:iki:tutorial:ttt:board
source urn:iki:tutorial:ttt:board
sink urn:iki:tutorial:ttt:move:1:1
sink urn:iki:tutorial:ttt:reset'>
<pre class="ikigai-run-expected">-
[computed]
X plays 1,1
[uncacheable]
trace  urn:iki:tutorial:ttt:cell:1:1
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:ttt:cell:1:1   ttt-cell · computed · ThreadId(1) · —   → 1b  X
└─ urn:iki:tutorial:ttt:stored:1:1   ttt-stored · computed · ThreadId(1) · —
---
-X-
---
[computed]
---
-X-
---
[cached]
error: invalid argument `x, y`: 1,1 is taken — X played there
The board is clear
[uncacheable]</pre>
</div>

The answers are the same. The trace is one node shorter, because the stored cell here is
an endpoint in the same kernel and has no process of its own to report a span from.

### The same commands, a TypeScript-held game

<!-- demo: run same-as-previous -->
```text
$ ttt-host --store /tmp/ttt6demo/ts.sock \
    -c 'source urn:iki:tutorial:ttt:cell:1:1' \
    -c 'sink urn:iki:tutorial:ttt:move:1:1' \
    -c 'trace urn:iki:tutorial:ttt:cell:1:1' \
    -c 'source urn:iki:tutorial:ttt:board' \
    -c 'source urn:iki:tutorial:ttt:board' \
    -c 'sink urn:iki:tutorial:ttt:move:1:1' \
    -c 'sink urn:iki:tutorial:ttt:reset'
…
   └─ urn:iki:tutorial:ttt:stored:1:1   ttt-stored · computed · deno-main · 0ms
…
```

The output is the Python run's, byte for byte, except for that one line: Deno serves the
connection on its main thread and says so. The demo script checks exactly that: the two
outputs are compared with the peer's thread name set aside, and must be identical.

So the store contract is the whole interface. Nothing above the atom changed, and nothing
above the atom could tell.

## Shape 2: the app in another language

The second shape keeps the whole game in the Rust host and moves the *face* instead. Part V
made the markup resources, `urn:iki:tutorial:ttt:template:{name}`, and wrote down what a
host of that markup implements: the template language the views are written in, and the path
rule that turns `iki/tutorial/ttt/view/board` into a name. Each app implements both in its
own language, as a client of the host.

Start the host with a game over each store, then the two apps. Each app is a client of the
host's IPC socket and a web server of its own:

<!-- demo: start -->
```sh
ttt-host --game py=/tmp/ttt6demo/py.sock --game ts=/tmp/ttt6demo/ts.sock --socket /tmp/ttt6demo/host.sock &
(cd ../ikigai-deno && deno run -A examples/tictactoe_app.ts --socket /tmp/ttt6demo/host.sock) &
(cd ../ikigai-python && PYTHONPATH=src python3 -m examples.tictactoe_app --socket /tmp/ttt6demo/host.sock) &
```

<!-- demo: log -->
```text
ttt-host: http://127.0.0.1:8070/ (the root game); games at /game/<id>/: ["py", "ts"]
ttt-host: IPC on /tmp/ttt6demo/host.sock
```

The host is on port 8070, the Deno app on 8071 and the Python app on 8072, so all three run
at once. Open `http://127.0.0.1:8072/game/py/` and you are playing game `py`, whose marks
are in the Python store, on a board the Python app rendered from resources the Rust host
computed. Open the same path on 8071 and it is the same game again.

### Filling the templates

In the Rust host a view is a template bound at a name, and `ikigai-fn` fills it. An app
could ask the host for the finished view, but then it would only pass the host's HTML
along, so each app composes the views itself. It reads the templates, `template:{name}`,
and the game's raw resources, `cell:{x}:{y}`, `winner` and `turn`, from the host, and fills
them with a small template engine of its own. The engine implements the part of the template language these templates
use, as the tic-tac-toe crate's README states it: the three markers, the `{x}` arguments,
the escape table and `conditional`.

It starts with a scan. In Python:

<!-- excerpt: ikigai-python examples/tictactoe_app.py @ 26e173a -->
```python
def scan(template: str) -> list[str | tuple[str, str]]:
    """``template`` as literal text and markers, each its letter and its trimmed body. `$$` is
    a literal `$`; a marker that never closes is literal text, `$` included."""
    out: list[str | tuple[str, str]] = []
    text, i = [], 0
    while i < len(template):
        if template.startswith("$$", i):
            text.append("$")
            i += 2
            continue
        if template[i] == "$" and template[i + 1 : i + 2] in ("a", "r", "h"):
            end = unquoted(template, i + 3) if template[i + 2 : i + 3] == "{" else -1
            if end >= 0:
                out += ["".join(text), (template[i + 1], template[i + 3 : end].strip(WHITESPACE))]
                text, i = [], end + 1
                continue
        text.append(template[i])
        i += 1
    return [*out, "".join(text)]
```

A template becomes literal text and markers, each marker its letter and its trimmed body.
`$$` is a `$`, and a marker that never closes stays text. `unquoted` finds the `}` that
closes the marker, counting the braces of any `{x}` inside it and skipping quoted values.

Each marker is then spliced by its letter:

<!-- excerpt: ikigai-python examples/tictactoe_app.py @ 26e173a -->
```python
def splice(mode: str, body: str, args: dict[str, str], source: Source, depth: int) -> str:
    """One marker: `$h` escapes what it names, `$r` splices it as it is, `$a` fills it."""
    if len(split(body, "||")) > 1:
        raise refused(f"marker `{body}`: this filler has no `||` fallbacks")
    name = body[1:-1] if body[:1] == "{" and body[-1:] == "}" else ""
    if ARG.fullmatch(name):
        if mode == "a":
            raise refused(f"marker `{body}`: an argument is a value and never a template")
        value = argument(args, name)
    else:
        iri, _, query = body.partition("?")
        given = {}
        for pair in filter(None, (pair.strip(WHITESPACE) for pair in split(query, "&"))):
            key, eq, word = pair.partition("=")
            if not eq:
                raise refused(f"marker argument `{pair}` is not key=value")
            given[key.strip(WHITESPACE)] = value_of(word.strip(WHITESPACE), args)
        request = substitute(iri.strip(WHITESPACE), args, encode=True)
        value = conditional(given, source) if request == CONDITIONAL else source(request, given)
        if mode == "a":
            return fill(value, args, source, depth + 1)
    return value.translate(ESCAPE) if mode == "h" else value
```

A body that is only an argument, `{x}`, is spliced as a value and resolves nothing. Anything
else is a request, with its arguments filled in, and every request is a read of the host but
one. `$a` fills what came back, `$r` splices it as it is, and `$h` escapes it through a
five-entry table: `&`, `<`, `>`, `"` and `'`, the last one as `&#39;`, which is what
`ikigai-fn` writes and not what Python's `html.escape` writes. That one character is the
difference between the same bytes and nearly the same bytes.

The one request the app answers itself is `conditional`, which the templates use to choose:
a square asks whether its cell is `-`, the status whether the winner is. The host's gateway
forwards only the game's names, so `urn:iki:fn:conditional` does not reach `ikigai-fn` from
an app, and the app has its own:

<!-- excerpt: ikigai-python examples/tictactoe_app.py @ 26e173a -->
```python
#: ``conditional``'s reading of ``if`` when there is no ``equals``.
BOOLEAN = {"true": True, "1": True, "yes": True, "on": True}
BOOLEAN |= {"false": False, "0": False, "no": False, "off": False, "": False}


def conditional(given: dict[str, str], source: Source) -> str:
    """``urn:iki:fn:conditional``: source ``if``; if its trimmed text is ``equals`` (or, with
    no ``equals``, a true boolean), source and answer ``then``, else ``else`` (or nothing).
    Only the chosen side is sourced."""
    test, then = argument(given, "if"), argument(given, "then")
    verdict = source(test, {}).strip(WHITESPACE)
    if "equals" in given:
        taken = verdict == given["equals"]
    elif (taken := BOOLEAN.get(verdict.lower())) is None:
        raise refused(f"conditional: `{test}` returned {verdict!r}, not a boolean")
    chosen = then if taken else given.get("else")
    return "" if chosen is None else source(chosen, {})
```

Only the chosen side is read, as in `ikigai-fn`, and what comes back is a template, which
the `$a` around the `conditional` then fills.

The TypeScript engine is the same design. Here is its fill, which parses every marker of a
template before it resolves any, so a malformed marker refuses the template whatever the
resources say, and then resolves the markers together:

<!-- excerpt: ikigai-deno examples/tictactoe_app.ts @ d212d8c -->
```ts
export async function compose(
  template: string,
  args: Args,
  resolve: Resolve,
  depth = 0,
): Promise<string> {
  if (depth >= MAX_DEPTH) {
    throw new EndpointError(`compose: recursion limit (${MAX_DEPTH}) exceeded`);
  }
  const segments = scan(template);
  // Every marker is parsed before any resolves: a malformed one fails the template.
  const markers = segments.map((s) =>
    typeof s === "string" ? null : parseMarker(s.mode, s.body)
  );
  const filled = await Promise.all(segments.map(async (s, n) => {
    const marker = markers[n];
    if (typeof s === "string" || marker === null) return s as string;
    let answer: string;
    if ("arg" in marker) {
      answer = argument(args, marker.arg);
    } else {
      const iri = fillArguments(marker.iri, args, true);
      const query: Args = {};
      for (const [key, { text, quoted }] of marker.args) {
        query[key] = quoted ? text : fillArguments(text, args, false);
      }
      answer = iri === CONDITIONAL
        ? await conditional(query, resolve)
        : await resolve(iri, query);
      if (s.mode === "a") {
        return compose(answer, args, resolve, depth + 1);
      }
    }
    return s.mode === "h" ? escapeHtml(answer) : answer;
  }));
  return filled.join("");
}
```

A view the board names, such as `view:square:1:0`, is composed the same way: the app
matches the name against the README's table of views and fills that view's template with
what the name captured. Any other name goes to the host, in the right game:

<!-- excerpt: ikigai-deno examples/tictactoe_app.ts @ d212d8c -->
```ts
  resolve: Resolve = (iri, args) => {
    const name = iri.startsWith(GAME) ? iri.slice(GAME.length) : null;
    for (const [pattern, template, vars] of VIEWS) {
      const m = name === null ? null : pattern.exec(name);
      if (m === null) continue;
      const captured = Object.fromEntries(vars.map((v, n) => [v, m[n + 1]]));
      return this.view(template, { ...args, ...captured });
    }
    const target = name === null ? iri : this.iri(name);
    return this.client.source(target, args).then((r) => r.text);
  };
```

Which template a view shows is not in either app. It is in the templates, as `conditional`
markers over `cell`, `winner` and `turn`, so the apps hold no rule of the game at all.

This costs something, and it is worth saying how much. Before Part V's views became
templates, each app filled simple `{{slot}}` placeholders, and its filler was about 70
lines. Now the TypeScript engine is about 225 lines and the Python one about 175, most of
it parsing: finding where a marker ends, splitting a request's arguments outside quotes,
checking that every `{` opens an argument. Each whole app grew less, by a fifth to a
quarter, because the view code that chose templates went away. And `conditional` now exists
three times: in Rust, in `ikigai-fn`, where the host uses it, and again in Python and in
TypeScript, because the gateway forwards only the game's names. The three copies are not
even the same. The Python one also reads a boolean, as `ikigai-fn`'s does, and the
TypeScript one implements only the `equals` form the templates use.

The README states the language as a list of cases, and a Rust test runs every case through
`ikigai-fn`, so the list cannot drift from the language. Each app copies the list into its
own tests. Where the apps found the language ambiguous, and they found seven places, the
README now says which way it goes, most of them as new cases.

Two things the apps do *not* do. They never compute the game: the winner is `winner`, read
from the host, not a function of the cells written again in Python. And they never write to
a store: a play is a `Sink` of the host's `move:{x}:{y}`, and a new game is a `Sink` of
`reset`. The next section is about why both matter.

### One markup, three backends

A move made through one app is a move in the host's game, so every face shows it:

<!-- demo: run -->
```text
$ curl -s -X POST http://127.0.0.1:8072/game/py/iki/tutorial/ttt/view/play/0/0
X plays 0,0. O to play.
$ curl -s -X POST http://127.0.0.1:8071/game/py/iki/tutorial/ttt/view/play/1/1
O plays 1,1. X to play.
$ curl -s http://127.0.0.1:8070/game/py/iki/tutorial/ttt/view/status
X to play.
$ for port in 8070 8071 8072; do curl -s http://127.0.0.1:$port/game/py/iki/tutorial/ttt/view/board | shasum -a 256 | cut -c1-16; done
80a8c36f3845618f
80a8c36f3845618f
80a8c36f3845618f
```

X played through the Python app, O through the TypeScript app, and the Rust host reports
the result. The last command hashes the board from each of the three HTTP faces: one
rendered in Rust, one in TypeScript, one in Python. The three digests are equal because the
three boards are the same bytes.

"The same bytes" is a claim each app's tests check, not a description. Both apps run their
parity tests against a real `ttt-host`. The tests play a won game, a refusal and a draw in
one game through the app and in another game through the host's own Rust views, and
compare the board, the status and every reply at each step. They also compare the pages
and the three static files, byte for byte, and the answer to every edge case the host's
HTTP face has: a wrong verb, a coordinate spelled `01`, a game the host does not have. An
app that drifted from the host by one character would fail its own tests.

## What does not cross, yet

Three limits, each of which the model predicts.

### A write the host does not see

The host's cache is correct because the host sees every write. When its kernel issues a
`Sink` or `Delete` to `stored:{x}:{y}`, it cuts that name's golden thread, and every cached
answer that read the square is recomputed on the next request. A write that goes to the
store *directly*, from some other client of the Python process, is not seen:

<!-- urn-gate: illustration urn:game:py:iki:tutorial:ttt:board — ttt-host's IPC name for game
     py's board; ttt-host binds it while it runs, and neither the CLI nor this page does. -->

<!-- demo: run -->
```text
$ ikigai --plain --override urn:iki:tutorial:ttt:stored:=/tmp/ttt6demo/py.sock -c 'sink urn:iki:tutorial:ttt:stored:2:2 O'
ok
[uncacheable]
$ ikigai --plain --override urn:iki:tutorial:ttt:stored:=/tmp/ttt6demo/py.sock -c 'source urn:iki:tutorial:ttt:stored:2:2'
O
[computed]
$ ikigai --plain --connect /tmp/ttt6demo/host.sock -c 'source urn:game:py:iki:tutorial:ttt:board'
X--
-O-
---
[uncacheable]
```

The Python store holds an O at `2,2`. The host still says the square is empty, and it will
go on saying so, because its cached cell at `2,2` hangs from a thread that nothing cut.
Worse, the host's rules would let X play there, and the Sink would overwrite the O, because
the move checks the square through that same cached cell. The fix is the rule the apps
follow: **every write goes through the host.** Here, a reset through the host deletes every
square through its kernel, which cuts every square's thread, and the host and the store
agree again:

<!-- demo: run -->
```text
$ curl -s -X POST http://127.0.0.1:8070/game/py/iki/tutorial/ttt/view/reset
The board is clear. X to play.
$ ikigai --plain --override urn:iki:tutorial:ttt:stored:=/tmp/ttt6demo/py.sock -c 'source urn:iki:tutorial:ttt:stored:2:2'
error: not found: nothing has been played at 2,2
```

The general fix would be a golden thread over the wire: a way for a peer to tell the host
that a name it serves has changed. There is none today, so a peer that can be written
behind the host's back is a correctness hazard, and the contract says so.

The `[uncacheable]` on the board is a second, smaller thing to notice. The IPC face of
`ttt-host` is a small kernel in front of the game's kernel, which injects the game's
corridor, and it marks every answer it forwards as uncacheable, because it never saw the
reads that answer depends on and could never cut a copy. The board's bytes still came from
the game kernel's cache. That is why the transcripts in shape 1 use `-c`, which runs in the
game's own kernel, where the verdicts are real.

### Composition stays in the kernel

A peer serves the atom and nothing else, and that is not an accident of this example. The
host knows what every cached answer was computed from because it resolved every piece of
it: the board read the rows, the rows read the cells, and each read was the kernel's. A
peer that did more, for example a Python `winner` that asked the host for the cells and
computed the answer itself, would make a request the host resolves as a new top-level
request from outside. The host would answer each cell correctly and learn nothing about
what the Python answer depended on. The Python `winner` would then be a *traccessor*:
something that accesses resources from across a boundary, in a fresh request, where
dependency tracking cannot follow. Its answer could not hang from any golden thread, so it
could never be cached safely or cut when a move changed it.

So the peers hold state, and the kernel composes. The apps follow the same rule from the
other side: they read `winner` rather than computing it, and render only what they read.

### A refusal is not typed yet

A move the rules refuse is answered in the kernel's words, and today those words say
`invalid argument`:

<!-- demo: run -->
```text
$ curl -s -X POST http://127.0.0.1:8072/game/py/iki/tutorial/ttt/view/play/0/0
X plays 0,0. O to play.
$ curl -s -X POST http://127.0.0.1:8071/game/py/iki/tutorial/ttt/view/play/0/0
invalid argument `x, y`: 0,0 is taken — X played there. O to play.
```

That is not really an invalid argument. The argument is well formed, and the refusal is
about the state of the game. The kernel has a better error for this now, `Conflict`, and the
wire can carry it: version 8 of the wire protocol gives it a tag in Rust, Python and
TypeScript together, and HTTP 409 at the web edges (ledger item 583). `ttt-host` and both
apps already speak it. What remains is the move, which still refuses with `InvalidArgument`
(ledger item 585). When it switches, a refused move will be answered with 409 and a message
that starts with `conflict:`, and a client that wants to treat "that square is taken"
differently from "that is not a coordinate" will be able to tell them apart by type.

## What was built

Across six parts: a game drawn as resources, then built as resources, played in many games
at once, given a face, and now held and served from three languages without its model
changing.

The last part shows the model's seam holding. The store contract was written in Part IV
for a store in *some* process. Two stores in other languages kept it on the first try, and
`check_store` said so before the host accepted them. The markup was written in Part V for
*some* host. Two apps in other languages serve it byte for byte, and their tests check
that. What stayed in one place was everything that needed to know what depended on what:
the composition, the cache and the rules, in the kernel.
