# The stages viewer: a picture that changes as a chapter's steps run

`books/ikigai/js/stages.js` (styles: `books/ikigai/css/stages.css`) shows a picture that a
chapter's steps change as a row of **numbered tabs**, one per **stage**. A stage is the state
after a step that changed the picture or its words. Each tab holds the steps that make the
stage, what made it, what changed since the stage before (in words, with the changed words
marked), and the picture as it was then, kept and never redrawn. Once the page's kernel loads,
the open tab's stage is shown below the tabs beside an earlier one, **Before | After**, so a
reader compares two states without anything else moving. A reader's own steps add stages after
the chapter's, so they can compare any two states.

The viewer knows nothing about what it draws. A page names, in markup, the line that draws the
picture, the line that says it in words, and the steps. Any book's in-page kernel answers those
lines (`crates/book-wasm`, `crates/architecture-book-wasm`, `crates/gonk-book-wasm` and the rest,
through the `evalLineAsync` and `evalLineInChainAsync` exports `js/run.js` already uses).

Two chapters use it. The ikigai Architecture's "Spaces that wrap spaces" steps the resilience
harness with buttons and cells (`books/architecture/src/overlays/overlays.md`), and The ikigai
Book's "A space that changes while it runs" steps a live host's dynamic space with cells only,
its stages spanning the chapter's sections (`books/ikigai/src/applied/live-space.md`). Read either
page's markup beside this one.

## Using it in a book

One copy serves every book, the way `js/run.js` does: link the files into the book and list them
after `run.js`.

```bash
ln -s ../../ikigai/js/stages.js books/<book>/js/stages.js
ln -s ../../ikigai/css/stages.css books/<book>/css/stages.css
```

```toml
# books/<book>/book.toml
additional-css = [..., "css/stages.css"]
additional-js = [..., "js/run.js", "js/stages.js"]
```

The script does nothing on a page without a viewer.

## The markup contract

```html
<div class="ikigai-stages" data-chain='harness:overlays'
     data-picture='source urn:diagram:state scheme={scheme}'
     data-words='source urn:throttle:state'
     data-tabs-label='Stages of the harness' data-controls-label='Drive the harness'
     data-alt="What the picture draws, in words.">
<ul class="ikigai-stages-controls" hidden>
<li data-cmd='sink urn:iki:tutorial:harness:mode fail'>Set the upstream to fail</li>
<li data-cmd='source urn:iki:tutorial:harness:call' data-note='Optional: a sentence of yours.'>Call once</li>
</ul>
<ul class="ikigai-stages-facts">
<li data-cmd='source urn:iki:tutorial:harness:mode' data-label='upstream'>upstream: ok</li>
<li data-cmd='source urn:throttle:state' data-line='3' data-none='no circuit has failed yet'>no circuit has failed yet</li>
</ul>
<section class="ikigai-stage" data-label="closed">

#### Stage 1: the start

Prose, and the cells that make this stage.

<div class="ikigai-stage-result">
<p class="ikigai-stage-facts">upstream: ok · no circuit has failed yet</p>
<p class="ikigai-stage-picture"><a class="ikigai-scheme-light" href="p-1-light.svg"><img src="p-1-light.svg" alt="…"></a><a class="ikigai-scheme-dark" href="p-1-dark.svg"><img src="p-1-dark.svg" alt="…"></a></p>
<pre class="ikigai-stage-words">what data-words answers at this stage</pre>
</div>

More prose, and cells that change nothing.

</section>
<section class="ikigai-stage" data-label="open">
…
</section>
</div>
```

The viewer's attributes:

| attribute | required | what it is |
|---|---|---|
| `data-picture` | yes | A line whose answer is SVG: the picture. `{scheme}` in it is replaced by `light` and by `dark`, and both copies are kept; without it the line is asked once. |
| `data-words` | yes | A line whose answer says the stage in words (a text face). What changed is computed from it, and a stage's match is judged by it. |
| `data-before` | no | A line read after every step, after the words, whose answer is handed to the NEXT read of the picture as `before="…"` (quoted, `\` and `"` escaped): a picture that can mark what changed since an earlier state (ikigai-diagram 0.2.4's `urn:diagram:state`, with `urn:throttle:state as=text/turtle`) marks it. With it, a step makes a stage when the words or this answer moved, not when the picture did (a marked picture differs whenever the board does), and the picture is read last: a step that made a stage draws it marked against the last read, and a step that moved nothing draws it exactly as the current stage was drawn, so that read is the same request and a step that cut nothing finds it in the cache. The first step after the page loads has no earlier answer, so the first stage a reader makes is unmarked; the committed stages are drawn by the native test against the stage before. |
| `data-overview` | no | A line whose answer is SVG: the kernel's state above the tabs (`{scheme}` as for `data-picture`). It is read after every step, last, and drawn into the figure `data-overview-target` names, in place of the picture that figure commits, with a caption that says whether the read was a cache hit. After, below the tabs, stays the open tab's stage. |
| `data-overview-target`, `data-overview-alt` | with `data-overview` | The `id` of the figure above the tabs, and the alt text of the picture drawn into it (it describes the kernel as it is now, not as the committed picture was). |
| `data-chain` | no | The chain the lines run in, as a cell names it (`harness:overlays` for `data-harness='overlays'`, `live:workshop` for `data-live='workshop'`). Without it, the page's own kernel. Cells in the same chain are the viewer's steps. |
| `data-alt` | no | What the picture draws, in words: every picture's alt text is this, then "At this stage:" and the words. |
| `data-tabs-label`, `data-controls-label`, `data-compare-label` | no | The accessible names of the tab list, of the row of buttons and of the Before \| After region. |

The parts:

- **Stages** (`section.ikigai-stage`, direct children of the viewer, in order): `data-label` is
  the word on the tab ("2 · open"). The section holds the steps that make the stage (cells), prose,
  and one `div.ikigai-stage-result` with the committed facts, the picture(s) and the words. The
  viewer adds "Made by" (the last cell above the result, or "loading the page" for the first) and
  "What changed" at the top of the result. A heading inside a stage keeps its anchor: a link to it
  opens that tab.
- **Controls** (`ul.ikigai-stages-controls`, `hidden`), optional: each `li` becomes a button named
  by its text that runs its `data-cmd` lines. `data-note` adds a sentence to the stage it makes.
  Without the kernel the list stays hidden.
- **Facts** (`ul.ikigai-stages-facts`), optional: one line above the tabs, kept current after every
  step. Each `li` shows line `data-line` (default 1) of what its `data-cmd` answers, after
  `data-label: ` when there is one, or `data-none` when the answer has no such line. Its text in
  the markup is the facts before any step.

## What the viewer does after a step

After every control, and every cell that runs in its chain, the viewer reads the picture (each
scheme), the words and the facts through the kernel, as a reader's own lines (so a step that cuts
no thread reads the picture from the cache, and the viewer says so). Then:

- if the words and the picture are what it last saw, the status line says **No change** and no
  tab is added;
- if a control changed them, a new stage is added after the chapter's and selected, compared with
  the stage the kernel was at before the press;
- if a cell changed them to its own stage's words, that stage says the reader's kernel reached it;
  otherwise a new stage is added, and not selected, so the cell stays in view.

Whichever it was, a cell's own tab is opened and compared with the stage just before it.

Before the first step it compares with the first stage's committed words: nothing is read on load.

**Before | After, below the tabs** (Brian, 2026-10-26). Once the kernel loads, a region
(`div.ikigai-stages-compare`) sits between the tabs and the open tab's panel. Top to bottom:

- **Compare with**: one native `<select>` with its visible `<label>`, offering the stages before
  the open tab, numbered and named as the tabs are ("Stage 2 · open"), a reader's own stages
  among them once they exist. It starts at the stage just before, and opening a tab puts it back
  there. Picking another puts that stage in Before and redoes "What changed", asking the kernel
  nothing, and the status region says which stage Before now is. On the first tab there is
  nothing before it, and a sentence says so in place of the select. (Until ledger #2298 this was
  a row of `aria-pressed` buttons, one per stage, under a row of tabs that looked the same.)
- **Two pictures, side by side** (`figure.ikigai-stages-before`, `figure.ikigai-stages-after`), in
  a grid that stacks them on a narrow screen, Before on top. After is the open tab's stage, Before
  the stage picked. Each is the picture the page's kernel drew when that tab's step last ran (a
  solid frame, captioned "as your kernel drew it", with what that step did), or else the
  chapter's committed picture (a dashed frame, captioned "the chapter's picture", with what
  running the step would do). When a tab's step so far moved nothing (its first cell often sets
  something the picture does not show), its picture is the stage the kernel was still at, and the
  caption names that stage first and then says the tab's step has moved nothing yet ("stage 3 ·
  open, as your kernel drew it: the step of stage 4 · half-open has moved nothing yet"). A picture keeps 500 px and scrolls sideways in its slot on a phone;
  on a wide screen it grows to its natural 960 px.
- **What changed from stage A to stage B**: the comparison below, for the two shown. Its lines are
  never wrapped; the list scrolls sideways.

The panel below holds the tab's step as the chapter wrote it. Its commands and answers are never
wrapped either (a wrapped URI reads as two names): each scrolls sideways in its own box, and on a
page with a viewer the content may be 1680 px wide, so at 12 px the chapters' longest command (226
characters) fits on one line there. Switching tabs or "Compare with" asks the kernel nothing:
every picture shown is one a step already drew, or a committed file. Each tab's committed result
is hidden while After stands in for it, and a pointer ("The picture for this stage is below the
tabs, as After.") takes its place; until the kernel loads, without it, and in print, each tab
shows its own committed result instead.

A box that scrolls sideways takes a tab stop while it overflows, and Left and Right in it (and on
the tabs) are kept from mdbook, which binds them on the document to the previous and next chapter.

**What changed** compares a stage's words with the stage before's, line by line. A line that
replaced another is shown with the words that differ marked (`<mark>`) and the words they
replaced beside it; a new line and a line that is gone are said as such. A new line replaced
the gone line that begins with the same words before its first `: `, so a words line that says
where it is first (`holding: …`, `` circuit `…`: …``) is paired with its old self however many
lines came or went around it; the lines left over are paired in order when there are as many
on each side, and are otherwise new and gone. Write the words with that in mind: one line per
thing, the thing's name before the colon. It is real text in the
panel, never color alone. The same comparison runs on the committed stages, so their "what
changed" needs no kernel.

## Keeping the committed stages true

A chapter's stages are committed: the pictures as files beside the chapter, the words and facts
as text in the result block. **A native test holds them to the kernel**, and the part of it that
is the same for every book is `book_transcripts::stages` (`crates/book-transcripts/src/stages.rs`):
`viewers` reads the markup, `Viewer::read` reads the viewer's lines after a step the way the
script does, and `walkthrough` runs the page's cells in order on a fresh page, reading the
viewer's lines after each, and requires every cell that moves the words or the picture to sit in
the next stage's section, every result block to be what the kernel answers, and every picture
file to be the kernel's bytes. A book's test hands it a function that runs one line in the
viewer's chain on its own kernel: `crates/architecture-book-wasm/tests/pages.rs`
(`the_chapters_stages_are_the_walkthrough_as_the_kernel_draws_it`) for the harness,
`crates/book-wasm` (`the_live_space_stages_are_the_walkthrough_as_the_host_draws_it`) for the
live host. On a mismatch it prints the expected result blocks. **The pictures are written by the
code, never by hand**: run the test with `BOOK_PICTURES_WRITE=1` and every picture that differs
is written where the chapter keeps it; without it, they are written beside the build with the
`cp` command that commits them.

The no-kernel check (`a11y/no-kernel.mjs`) requires every viewer to keep its tabs and committed
pictures, offer no control, and say the kernel did not load.

## Accessibility

The tabs are the ARIA tabs pattern: a `tablist`, a `tab` per stage, a `tabpanel` per section, Left
and Right between tabs, Home and End to the ends, a visible focus ring. From a tab, Tab reaches
"Compare with" and the two pictures' links before the panel: the comparison is what the tab
changed. "Compare with" is a native select, 44 px high, in the theme's text
and link colors, and fits a 320 px screen. One polite status region says each step's
outcome once; Before | After is a labeled region, not a live one. In print, every stage shows,
one after another. See `ACCESSIBILITY.md` for what the checks cover.

`a11y/cells.mjs` holds the viewer to the reader who runs a chapter in order: after every cell in
a viewer's chain, its tab is open and After shows a picture the page's kernel drew (a `blob:`
image in the copy the theme shows), captioned as the kernel's. Then, once a page's cells have
run, it drives each viewer's "Compare with": on the last tab, the select offers exactly the stages
before it and starts at the one just before; picking stage 1 changes Before, "What changed" and
the status region; opening another tab puts it back; the first tab shows no select.
