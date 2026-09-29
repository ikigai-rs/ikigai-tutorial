# Tic-tac-toe, V: a board you can play

Parts [I](tic-tac-toe-1.md) to [IV](tic-tac-toe-4.md) built tic-tac-toe out of
names: a stored cell, a platonic cell, lines, rows by alias, a board, the rules, and games
as corridors around them. Nobody could *play* it except by typing `sink` at a prompt. This
part gives it a face, and the face is resources too: HTML that the game serves under names
of its own, the same bytes whoever puts it on a screen. It covers the ground of the fifth
part of the series Part I credits, which starts from the edge a player touches, and of its
ninth, which draws the design together, and it ends by drawing this arc together.

Here are two games, `a` and `b`. Play them with the mouse or the keyboard: Tab to a square
and press Enter or Space.

<div class="ttt-boards">
<div class="ttt-play" data-game='a'><p class="ttt-unavailable">Game a's board appears here when the in-page kernel has loaded.</p></div>
<div class="ttt-play" data-game='b'><p class="ttt-unavailable">Game b's board appears here when the in-page kernel has loaded.</p></div>
</div>

Nothing on this page is a server. Each click is a request to the same kernel the runnable
cells use, in the game the board belongs to, and the cells below run in game `a`, the left
board. So you can play a move with the mouse and then ask the kernel what it did with it.
The expected output under each cell assumes you have not clicked yet. Once you have, the
answers are about your game instead, which is the point: press **New game** to get back to
the chapter's.

## The markup is a resource

The game's HTML lives in templates, and each template is a resource,
`urn:iki:tutorial:ttt:template:{name}`. There are thirteen, all small. Here is the one for a
square that has been played:

```text
{{#include ../../../../crates/tic-tac-toe/templates/square-taken.html}}
```

It is HTML with markers in it, and each marker is a request. `$h{urn:iki:tutorial:ttt:cell:{x}:{y}}`
asks the kernel for the cell at `(x, y)` and puts the answer into the page. No Rust fills this
template. It is written in the template language of `ikigai-fn`, the function library Part I's
host mounts, and that library fills it through the kernel, as it fills any composition.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:ttt:template:square-taken
source urn:iki:tutorial:ttt:template:status-turn'>
<pre class="ikigai-run-expected">&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-$h{{x}}-$h{{y}}&quot; aria-disabled=&quot;true&quot; aria-label=&quot;$h{urn:iki:tutorial:ttt:cell:{x}:{y}} at $h{{x}},$h{{y}}&quot;&gt;$h{urn:iki:tutorial:ttt:cell:{x}:{y}}&lt;/button&gt;
[computed]
$h{urn:iki:tutorial:ttt:turn} to play.
[computed]</pre>
</div>

A template is served as it is written, markers and all. It is filled only when a *view* asks
for it, and the next section is the whole language a view speaks.

## The template language

A marker is `$`, a letter, and a resource name in braces. The letter says how the answer goes
in:

| marker | puts in | markers in what it puts in |
|---|---|---|
| `$h{name}` | the answer as text, escaped for HTML | left alone |
| `$r{name}` | the answer as it is | left alone |
| `$a{name}` | the answer as it is | expanded, as part of this template |

`$$` is a literal `$`, so `$$h{…}` is the text `$h{…}`. Everything that is not a marker is
copied as it is.

**Why `$h` never expands.** The stored cell keeps any mark at all, and a mark written straight
into the store is text somebody else chose. `$h` escapes it: `&`, `<`, `>`, `"` and `'` become
`&amp;`, `&lt;`, `&gt;`, `&quot;` and `&#39;`, so the mark cannot become markup. And `$h` does
not look for markers in it, so the mark cannot become a request either. If it did, a player
could play a mark that is itself a marker, and the page would resolve whatever name they wrote
in it, with the view's authority. That is template injection, the same hole as SQL injection in
another language. So a value from an atom goes in with `$h`. `$r` is for markup that is trusted
and finished, such as another view, which goes in as it is and is not read again. Only `$a`
expands what it splices, and a template uses it only over templates of its own.

**Arguments.** `{x}` inside a marker is an argument: a variable the view's name captured, or an
argument of the request. In a resource name its value is percent-encoded, so an argument can
never add a segment or a query to the name it sits in. A marker that is only an argument,
`$h{{x}}`, splices the value and asks the kernel for nothing. The open square has no resource
in it but its own coordinates:

```text
{{#include ../../../../crates/tic-tac-toe/templates/square-open.html}}
```

**Fallbacks.** `$h{a || b}` tries `a`, and if `a` fails, `b`, and `$h{{title} || b}` stands in
for a missing argument. The game needs none, because a cell already answers `-` for an empty
square. The spreadsheet arc uses them, and its part V shows what the cache makes of one.

**A template bound at a name.** A view is a template bound at a name of its own.
`ikigai_fn::compose_over(src)` is the composer with its template fixed, and the variables the
name captures are the template's arguments:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:view}}
```

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:views}}
```

**The branch.** `urn:iki:fn:conditional` sources its `if` resource and returns only `then` or
only `else`. With `equals=`, the test is whether `if`'s text is that value. The side it does not
take is never asked for, so neither its work nor its golden threads enter the answer. A square
is two branches:

```text
{{#include ../../../../crates/tic-tac-toe/templates/square.html}}
{{#include ../../../../crates/tic-tac-toe/templates/square-empty.html}}
```

A square asks its cell whether it is `-`. If not, the square is `square-taken`. If it is,
`square-empty` asks the winner whether *it* is `-`: the square is `square-open` while the game
goes on, and `square-closed` once it is over. Each branch is a template rather than HTML, and
the square splices it with `$a`, so its markers are expanded as part of the square: the `{x}`
and `{y}` in `square-open` are the square view's own. This is the one place the game uses `$a`,
and what it expands is always one of its own templates, never a value.

**What stays code.** Every marker is a `Source`. A template can read and cannot write, so the
two views that write, a play and **New game**, are Rust, and a later section shows why.

## The board as a view

The board is a resource, `urn:iki:tutorial:ttt:view:board`: the `board` template bound at that
name. The template is the grid, with a square's view at each place:

```text
{{#include ../../../../crates/tic-tac-toe/templates/board.html}}
```

Each square's view is a resource too, `urn:iki:tutorial:ttt:view:square:{x}:{y}`, the `square`
template bound at a name with two variables. So `view:square:2:0` is the square template with
`x` = `2` and `y` = `0`:

<div class="ikigai-run" data-game='a' data-cmd='source urn:iki:tutorial:ttt:view:square:2:0'>
<pre class="ikigai-run-expected">&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-2-0&quot; hx-post=&quot;iki/tutorial/ttt/view/play/2/0&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;2,0: empty, play here&quot;&gt;&lt;/button&gt;
[cached]</pre>
</div>

It was already cached: the board above drew it when the page loaded.

The board is a composite like every other resource above the store. Every marker is a request
through the kernel, so the board is cacheable, it recomputes only when a move touches something
it read, and it shows whichever game it is asked in, because the corridor carries into every
one of those reads. And it has no code. The template decides which squares there are, and the
conditional in `square` decides which of three kinds each one is.

The status line is the same kind of thing, `urn:iki:tutorial:ttt:view:status`. The winner
chooses between `status-turn` and `status-over`, a second choice between `status-draw` and
`status-won`:

```text
{{#include ../../../../crates/tic-tac-toe/templates/status.html}}
{{#include ../../../../crates/tic-tac-toe/templates/status-over.html}}
{{#include ../../../../crates/tic-tac-toe/templates/status-turn.html}}
```

When this page loaded, the boards above asked for both views, in their own games. So in
game `a` they are already cached:

<div class="ikigai-run" data-game='a' data-cmd='cache urn:iki:tutorial:ttt:view:board
source urn:iki:tutorial:ttt:view:status'>
<pre class="ikigai-run-expected">cached
X to play.
[cached]</pre>
</div>

Is the HTML a transreption of the board or a projection of it? Measured against the bytes,
it looks like neither: the HTML is well over a kilobyte and the board is eleven characters. Measured against the resource, which is the definition this book uses, it is a
transreption, because every mark on the board can be read back out of it, so nothing about
the board is lost, even though the HTML text carries far more than the board does.

## Playing through the view

A square is a button, and an open square's button carries its move as `hx-post`. That is
htmx, a small library that turns attributes into requests: pressing the button sends a
`POST` to the path in `hx-post` and puts the answer where `hx-target` says. The path is
`iki/tutorial/ttt/view/play/1/1`, and the resource it names is the view's one write:

```rust,ignore
{{#include ../../../../crates/tic-tac-toe/src/lib.rs:view_play}}
```

`urn:iki:tutorial:ttt:view:play:{x}:{y}` is a `Sink` and nothing else. It sinks the move,
the same resource Part III built, through the kernel, so the rules are still the move's and
only the move's. Then it answers with `urn:iki:tutorial:ttt:view:reply`, a view like the
others, whose template puts what the write said, as the argument `message`, in front of the
status:

```text
{{#include ../../../../crates/tic-tac-toe/templates/reply.html}}
```

That answer is what the button's `hx-target` receives: the status line, which is a live
region, so a screen reader says what the move did. The board re-renders when the status has
settled, because its own markup asks it to (`hx-trigger` on the board, in the `game`
template).

This view is code, and it has to be. A template's markers are all reads, and a play is a write
and then a read, in that order: the reply must not be read until the move has been made.
Composition has no way to say "first", so a sequence stays a few lines of Rust. Everything it
shows is still a template.

<div class="ikigai-run" data-game='a' data-cmd='sink urn:iki:tutorial:ttt:view:play:1:1
cache urn:iki:tutorial:ttt:view:board
cache urn:iki:tutorial:ttt:view:status'>
<pre class="ikigai-run-expected">X plays 1,1. O to play.
[uncacheable]
not cached
cached</pre>
</div>

The move cut the stored cell's thread, and both views hung from it, so both were cut. Then
the reply *read* the status, to answer with it, which cached the new one. The board waits
until something asks for it. On this page something did: after a cell runs in a board's
game, the page asks that board to refresh, which is the next section's subject. So the
board view is cached again, with X in the center:

<div class="ikigai-run" data-game='a' data-cmd='source urn:iki:tutorial:ttt:view:board'>
<pre class="ikigai-run-expected">&lt;div class=&quot;ttt-grid&quot; role=&quot;group&quot; aria-label=&quot;The board: row 0 is the top row, column 0 the left&quot;&gt;
&lt;div class=&quot;ttt-row&quot;&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-0-0&quot; hx-post=&quot;iki/tutorial/ttt/view/play/0/0&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;0,0: empty, play here&quot;&gt;&lt;/button&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-1-0&quot; hx-post=&quot;iki/tutorial/ttt/view/play/1/0&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;1,0: empty, play here&quot;&gt;&lt;/button&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-2-0&quot; hx-post=&quot;iki/tutorial/ttt/view/play/2/0&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;2,0: empty, play here&quot;&gt;&lt;/button&gt;&lt;/div&gt;
&lt;div class=&quot;ttt-row&quot;&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-0-1&quot; hx-post=&quot;iki/tutorial/ttt/view/play/0/1&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;0,1: empty, play here&quot;&gt;&lt;/button&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-1-1&quot; aria-disabled=&quot;true&quot; aria-label=&quot;X at 1,1&quot;&gt;X&lt;/button&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-2-1&quot; hx-post=&quot;iki/tutorial/ttt/view/play/2/1&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;2,1: empty, play here&quot;&gt;&lt;/button&gt;&lt;/div&gt;
&lt;div class=&quot;ttt-row&quot;&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-0-2&quot; hx-post=&quot;iki/tutorial/ttt/view/play/0/2&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;0,2: empty, play here&quot;&gt;&lt;/button&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-1-2&quot; hx-post=&quot;iki/tutorial/ttt/view/play/1/2&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;1,2: empty, play here&quot;&gt;&lt;/button&gt;&lt;button type=&quot;button&quot; class=&quot;ttt-square&quot; id=&quot;ttt-square-2-2&quot; hx-post=&quot;iki/tutorial/ttt/view/play/2/2&quot; hx-target=&quot;previous .ttt-status&quot; aria-label=&quot;2,2: empty, play here&quot;&gt;&lt;/button&gt;&lt;/div&gt;
&lt;/div&gt;
[cached]</pre>
</div>

A move the rules refuse is *answered*, not failed. The view does not look inside the error
to decide how to show it. Whatever the move raised, the player reads the kernel's own
words, followed by whose turn it still is:

<div class="ikigai-run" data-game='a' data-cmd='sink urn:iki:tutorial:ttt:view:play:1:1'>
<pre class="ikigai-run-expected">invalid argument `x, y`: 1,1 is taken — X played there. O to play.
[uncacheable]</pre>
</div>

The boards never offer that move: a square that is taken is not an open button. It happens
when two people play one game from two screens, and the board one of them is looking at is
out of date.

A move made without the view recomputes the view the same way, because the view is only a
reader of what the move changed. Traced, the status shows the template language at work: the
`status` template, served from cache; the conditional, recomputed because the winner it asked
was; the `status-turn` template it chose, from cache; and the turn that template splices.
Under the winner and the turn, exactly the three lines through the corner and the board were
recomputed, and the other five lines were served from cache.

<div class="ikigai-run" data-game='a' data-cmd='sink urn:iki:tutorial:ttt:move:0:0
trace urn:iki:tutorial:ttt:view:status'>
<pre class="ikigai-run-expected">O plays 0,0
[uncacheable]
trace  urn:iki:tutorial:ttt:view:status
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process · chain urn:iki:tutorial:ttt:game:a root
&#32;
urn:iki:tutorial:ttt:view:status   composeOver · computed · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root   → 10b  X to play.
├─ urn:iki:tutorial:ttt:template:status   ttt-template · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
├─ urn:iki:fn:conditional   conditional · computed · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  ├─ urn:iki:tutorial:ttt:winner   ttt-winner · computed · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  ├─ urn:iki:tutorial:ttt:cells:0.0,1.1,2.2   ttt-cells · computed · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:diagonal:0 -&gt; urn:iki:tutorial:ttt:cells:0.0,1.1,2.2 scope=urn:iki:tutorial:ttt:game:a root
│  │  │  ├─ urn:iki:tutorial:ttt:cell:0:0   ttt-cell · computed · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  │  │  └─ urn:iki:tutorial:ttt:stored:0:0   ttt-stored · computed · ThreadId(1) · — · answered-by=urn:iki:tutorial:ttt:game:a scope=urn:iki:tutorial:ttt:game:a root
│  │  │  ├─ urn:iki:tutorial:ttt:cell:1:1   ttt-cell · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  │  └─ urn:iki:tutorial:ttt:cell:2:2   ttt-cell · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  ├─ urn:iki:tutorial:ttt:cells:2.0,1.1,0.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:diagonal:1 -&gt; urn:iki:tutorial:ttt:cells:2.0,1.1,0.2 scope=urn:iki:tutorial:ttt:game:a root
│  │  ├─ urn:iki:tutorial:ttt:cells:0.0,0.1,0.2   ttt-cells · computed · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:column:0 -&gt; urn:iki:tutorial:ttt:cells:0.0,0.1,0.2 scope=urn:iki:tutorial:ttt:game:a root
│  │  │  ├─ urn:iki:tutorial:ttt:cell:0:0   ttt-cell · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  │  ├─ urn:iki:tutorial:ttt:cell:0:1   ttt-cell · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  │  └─ urn:iki:tutorial:ttt:cell:0:2   ttt-cell · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  ├─ urn:iki:tutorial:ttt:cells:1.0,1.1,1.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:column:1 -&gt; urn:iki:tutorial:ttt:cells:1.0,1.1,1.2 scope=urn:iki:tutorial:ttt:game:a root
│  │  ├─ urn:iki:tutorial:ttt:cells:2.0,2.1,2.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:column:2 -&gt; urn:iki:tutorial:ttt:cells:2.0,2.1,2.2 scope=urn:iki:tutorial:ttt:game:a root
│  │  ├─ urn:iki:tutorial:ttt:cells:0.0,1.0,2.0   ttt-cells · computed · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:0 -&gt; urn:iki:tutorial:ttt:cells:0.0,1.0,2.0 scope=urn:iki:tutorial:ttt:game:a root
│  │  │  ├─ urn:iki:tutorial:ttt:cell:0:0   ttt-cell · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  │  ├─ urn:iki:tutorial:ttt:cell:1:0   ttt-cell · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  │  └─ urn:iki:tutorial:ttt:cell:2:0   ttt-cell · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
│  │  ├─ urn:iki:tutorial:ttt:cells:0.1,1.1,2.1   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:1 -&gt; urn:iki:tutorial:ttt:cells:0.1,1.1,2.1 scope=urn:iki:tutorial:ttt:game:a root
│  │  └─ urn:iki:tutorial:ttt:cells:0.2,1.2,2.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:2 -&gt; urn:iki:tutorial:ttt:cells:0.2,1.2,2.2 scope=urn:iki:tutorial:ttt:game:a root
│  └─ urn:iki:tutorial:ttt:template:status-turn   ttt-template · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
└─ urn:iki:tutorial:ttt:turn   ttt-turn · computed · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
   ├─ urn:iki:tutorial:ttt:winner   ttt-winner · cached · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
   └─ urn:iki:tutorial:ttt:board   ttt-board · computed · ThreadId(1) · — · scope=urn:iki:tutorial:ttt:game:a root
      ├─ urn:iki:tutorial:ttt:cells:0.0,1.0,2.0   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:0 -&gt; urn:iki:tutorial:ttt:cells:0.0,1.0,2.0 scope=urn:iki:tutorial:ttt:game:a root
      ├─ urn:iki:tutorial:ttt:cells:0.1,1.1,2.1   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:1 -&gt; urn:iki:tutorial:ttt:cells:0.1,1.1,2.1 scope=urn:iki:tutorial:ttt:game:a root
      └─ urn:iki:tutorial:ttt:cells:0.2,1.2,2.2   ttt-cells · cached · ThreadId(1) · — · alias=urn:iki:tutorial:ttt:row:2 -&gt; urn:iki:tutorial:ttt:cells:0.2,1.2,2.2 scope=urn:iki:tutorial:ttt:game:a root</pre>
</div>

Game `b` has not been touched. Its status still says X is to play, but it was computed
again rather than served:

<div class="ikigai-run" data-game='b' data-cmd='source urn:iki:tutorial:ttt:view:status'>
<pre class="ikigai-run-expected">X to play.
[computed]</pre>
</div>

That is Part IV's thread that is a name, now visible in a view: O's move on the top-left
corner of game `a` cut `stored:0:0` in every game, and game `b`'s status depended on
game `b`'s corner.

**New game** is the last write, `urn:iki:tutorial:ttt:view:reset`. It sinks
`urn:iki:tutorial:ttt:reset`, which deletes every square a line of the table passes through,
each one through the kernel, and answers like a play does:

<div class="ikigai-run" data-game='a' data-cmd='sink urn:iki:tutorial:ttt:view:reset'>
<pre class="ikigai-run-expected">The board is clear. X to play.
[uncacheable]</pre>
</div>

## One markup, three transports

Look at the paths in the markup: `iki/tutorial/ttt/view/board`,
`iki/tutorial/ttt/view/play/1/1`. They are relative, and they name no game and no host.
Every host that serves this markup maps a path to a name by one rule:

<!-- urn-gate: illustration urn:a:b:c — the path rule's placeholder: any relative path
     `a/b/c` names `urn:a:b:c`. Not a resource. -->
<!-- urn-gate: illustration urn:account:id:alice — ikigai-web's own documented example of its
     mechanical path mapping, quoted to show where the rule comes from. -->

> A request for the relative path `a/b/c` is a request for `urn:a:b:c`, in the game of the
> page it was made from. `GET` is `Source`, `POST` is `Sink`, `DELETE` is `Delete`.

That is the mechanical default of `ikigai-web`, the HTTP transport in the CLI's workspace,
which maps `/account/id/alice` to `urn:account:id:alice`, with one change: the path is
relative, so the part of the URL in front of it is the host's. A host that serves game `a`
at `/game/a/` and game `b` at `/game/b/` sends the same markup to both, and a click on game
`a`'s page arrives as `/game/a/iki/tutorial/ttt/view/play/1/1`. The host takes `/game/a/`
off the front and turns it into game `a`'s corridor. The game is where the page is, which is
Part IV's argument again: context, not a name.

This markup has three hosts in this arc, and it is the same markup in all three.

**This page.** There is no server. `js/ttt.js` listens for htmx's `htmx:beforeRequest`,
which fires before htmx sends anything, and for a request from inside a board it cancels the
request, maps the path by the rule above, and asks the page's kernel. The game comes from the
board's markup, `data-game='a'` on the element that holds it, exactly as a runnable cell's
game does, and never from the request. Then it hands the answer to `htmx.swap`, the function
htmx's own response handling calls, so everything htmx does after a response happens here
too. The kernel side is one call, beside the engine the cells use:

```rust,ignore
{{#include ../../../../crates/book-wasm/src/lib.rs:page_issue}}
```

The page's copy of the markup has one problem a server's does not. The kernel cuts every
cached answer a move depended on, but a board on the screen is HTML the kernel handed out
earlier, and nothing cuts that. A click keeps its own board current, because the markup asks
for the status and the board after every move. A runnable cell does not, so after a cell runs
in a board's game, the page asks that board to fetch its status again, and the board follows.
Across processes this is the same gap Part IV's store contract names: a write the kernel did
not see leaves a copy stale, and there is no golden thread over the wire yet to tell it.

**A host process.** `ttt-host` serves the same game over HTTP and over the kernel's IPC
socket, and can take a game's store from another process. The next section is about it.

**Other languages.** The Python and TypeScript faces of ikigai can serve the stored cell
already, under the contract Part IV wrote. They can also serve this markup: the templates are
resources they can read, the part of the template language these templates use is small
enough to write again in any language, and the path rule is one line. That is the next part
of the arc.

## The same board from a server

`ttt-host` is a crate in this repository and a program you can run. It holds the game in one
kernel, `space_with_store` as the root game, with one corridor per game named on its command
line, and it serves that kernel two ways.

Over HTTP, through `ikigai-web`, it serves a page per game, `/game/a/` for game `a`. The
page is a document around the game's own page shell, `urn:iki:tutorial:ttt:view:game:a`,
the same bytes this page shows,
with a `<base href="/game/a/">` so that the markup's relative paths arrive under the game.
The host does the rest: it takes `/game/a/` off the front of a request and turns it into
game `a`'s corridor, and maps what is left by the rule above.

```rust,ignore
{{#include ../../../../crates/ttt-host/src/lib.rs:gateway}}
```

<!-- urn-gate: illustration urn:game:a:iki:tutorial:ttt:board — ttt-host's edge name for game
     a's board, which the crate's tests resolve; no host this book builds binds it. -->

That is a space of its own because of who may inject a corridor. `ikigai-web` issues every
request with `Kernel::issue`, in the root's chain, and putting a corridor ahead of the root is
`Kernel::issue_in`, which only the holder of the kernel can call. The host holds it, so the
host's own small space in front of the game does the injecting. Over the IPC socket the same
space serves the same names, spelled as the HTTP paths are: game `a`'s board is
`urn:game:a:iki:tutorial:ttt:board` there. The pages, the files they load and the routes are
in the crate; the tests post a move to game `a`'s path and read game `a`'s board back, with
game `b` and the root untouched:

```rust,ignore
{{#include ../../../../crates/ttt-host/tests/host.rs:http_move}}
```

### A store in another process

A game's store can be another process: `--game py=/tmp/ttt-py.sock` makes game `py`'s store
whatever answers the stored cell's names on that socket. The host mounts it the way
`ikigai --override urn:iki:tutorial:ttt:stored:=/tmp/ttt-py.sock` would, names forwarded
unchanged, and before it accepts it, it runs Part IV's store contract against it:

```rust,ignore
{{#include ../../../../crates/ttt-host/src/lib.rs:peer}}
```

A store that breaks the contract is refused before anything listens, with the clause it
broke. The contract gained a clause in this part: `Exists` answers `true` or `false`, never
`NotFound`, and is cacheable like the mark. The Python and TypeScript stores already answered
that way; the Rust one used to answer `Exists` as if it were `Source`, and now does not.

Both of those stores are in their own repositories, as `examples/tictactoe_store.py` and
`examples/tictactoe_store.ts`. To play against one, start it on a socket, then start the host
with a game over that socket. From a clone of `ikigai-python`:

```sh
PYTHONPATH=src python3 -m examples.tictactoe_store /tmp/ttt-py.sock
```

or from a clone of `ikigai-deno`:

```sh
deno run -A examples/tictactoe_store.ts /tmp/ttt-ts.sock
```

and then, from this repository:

```sh
cargo run -p ttt-host -- --game a --game py=/tmp/ttt-py.sock --game ts=/tmp/ttt-ts.sock
```

and open `http://127.0.0.1:8070/game/py/`. The board is this page's board, the markup is the
same markup, and every mark on it lives in a Python dictionary. Stop the host and start it
again, and the Python store still has the game.

Two things to know. The mount is an override and not an alias: an alias mount at
`urn:iki:tutorial:ttt:stored:` would strip that prefix and forward names the Python and
TypeScript servers do not bind. And every write has to go through the host. The host cuts a
stored cell's thread when *its* kernel issues the write. A mark written to the Python process
by some other client is invisible to the host's cache, which goes on serving the old board
until something else cuts it, because there is no golden thread over the wire yet.

## What the board does for a reader who cannot see it

The board is buttons, so it works with a keyboard and a screen reader with no extra code.
Every square has a name that says where it is and what is on it: `2,0: empty, play here`,
`X at 1,1`. A square that cannot be played, taken or closed by the end of the game, is
`aria-disabled` rather than `disabled`: it stays in the tab order and keeps its name, so a
reader can still find out what is on it, and it does nothing when pressed. The status line
is a live region (`role="status"`), which a screen reader announces when it changes, and it
is the target of every play, so what a move did is said once, politely, after the move. And
the squares carry ids, because htmx puts focus back on the element that had it, by id, after
a swap: press a square and focus stays on that square while the board is rebuilt under it. On
this page, which has two boards, the shim prefixes each id with its game so that no id
appears twice.

## What was built

Five parts, in the order the series builds its own version, and the order is the lesson.

**The resource model first.** Part I named every set in the game before writing any code,
and found that only one of them holds anything: the stored cell. Everything else was a set
of cells, or a question about sets of cells.

**Composition over code.** Part II built lines, rows and the board out of names. A row is an
alias with no code behind it; a line is a list of cells; the board is three rows. The kernel
did the bookkeeping that code usually does: a move recomputes exactly what it touched and
serves the rest from cache, and no line of the game says so.

**Constraint at the edge.** Part III made the rules resources, and kept the model loose. The
stored cell still takes any mark anywhere. One resource, the move, refuses what the rules
forbid, and it refuses by reading resources the game already had.

**Context, not state.** Part IV played many games without changing a name, by making the
game a corridor around the names rather than a part of them, and it turned the store into a
seam any process in any language can fill.

**And this part** put a face on it without moving any of that. The views are more resources
of the same kind: composites, cached, per game, recomputed by the moves that touch them, and
written as templates with no view code at all. The markup is resources too, so a page, a
server and a program in another language show the same game from the same bytes.

Next, the arc crosses languages: the stored cell served from Python and from TypeScript, a
host that mounts it, and apps in both languages that serve this same board.
