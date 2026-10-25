# The stages viewer: a picture that changes as a chapter's steps run

`books/ikigai/js/stages.js` (styles: `books/ikigai/css/stages.css`) shows a picture that a
chapter's steps change as a row of **numbered tabs**, one per **stage**. A stage is the state
after a step that changed the picture or its words. Each tab holds the steps that make the
stage, what made it, what changed since the stage before (in words, with the changed words
marked), and the picture as it was then, kept and never redrawn. A reader's own steps add stages
after the chapter's, so they can flip between any two states.

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
| `data-chain` | no | The chain the lines run in, as a cell names it (`harness:overlays` for `data-harness='overlays'`, `live:workshop` for `data-live='workshop'`). Without it, the page's own kernel. Cells in the same chain are the viewer's steps. |
| `data-alt` | no | What the picture draws, in words: every picture's alt text is this, then "At this stage:" and the words. |
| `data-tabs-label`, `data-controls-label` | no | The accessible names of the tab list and of the row of buttons. |

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
- if a control changed them, a new stage is added after the chapter's and selected;
- if a cell changed them to its own stage's words, that stage says the reader's kernel reached it;
  otherwise a new stage is added, and not selected, so the cell stays in view.

Before the first step it compares with the first stage's committed words: nothing is read on load.

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
and Right between tabs, Home and End to the ends, Tab into the panel, a visible focus ring. One
polite status region says each step's outcome once. In print, every stage shows, one after
another. See `ACCESSIBILITY.md` for what the checks cover.
