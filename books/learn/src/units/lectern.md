# Lectern decks as units

A unit of a course is an authored Markdown file. One kind is a
[Lectern](https://github.com/bsletten/lectern-slides) deck: Markdown slides that Lectern assembles
into one deck and renders with reveal.js. ikigai-learn's plan is to wrap Lectern rather than
rewrite it: Lectern stays the authoring tool and the renderer, and the course model gives what it
builds names. This chapter is what an author does on the Lectern side, and what the course model
makes of it.

**Status:** Lectern 0.1.0 is on PyPI as `lectern-slides`. Assembling, slide ranges, partials, the
accessibility audit and the transcript below are in that release. The sections marked
*unreleased* describe features on Lectern's `main` branch, at commit `ffde44f` (2026-11-27), which
its changelog lists as unreleased. Nothing on the ikigai-learn side of this chapter is built.

## A deck

A deck is a directory with a manifest, `deck.toml`, and its slide files. Slides are separated by
`---`, and Lectern's own directives are HTML comments, so the raw Markdown stays valid CommonMark.
Every relative path in the manifest resolves against the deck's own directory. The commands, from
Lectern's README:

```text
pip install lectern-slides

lectern new ./unit                    scaffold deck.toml and a couple of slides
lectern check ./unit                  validate includes, ranges and partials; run the audit
lectern build ./unit                  reveal.js HTML, into dist/index.html
lectern build ./unit -f outline       a text transcript, dist/outline.md
lectern watch ./unit                  a preview that rebuilds and reloads on every save
```

## Slide sub-selection

A deck takes exactly the slides it wants from a larger file. In the manifest, an entry is a path
with optional ranges, and inside a slide the same thing is an include directive:

```toml
slides = [
  "slides/00-title.md",
  "_partials/principles.md#2-3,6",   # slides 2 to 3, and 6, of that file
  "slides/80-closing.md",
]
```

```markdown
<!-- include: partials/rdf-intro.md#1-3,14 -->
```

The ranges count from 1, within the included file: `N` is one slide, `A-B` an inclusive range,
`A-` from A to the end, `-B` from the start to B, and a comma-separated list combines them. An
index past the end is an error that names the file and how many slides it has. An include path is
looked for beside the including file first, then in each of the deck's `partials` directories.

This is how one slide is reused across units, and it is where the course model's slide names come
from. `urn:learn:slide:unit:{u}:range:{spec}` takes the same grammar, and
`urn:learn:slide:unit:{u}:number:{n}` counts from 1 the same way. `lectern assemble` writes a
`<!-- @from PATH slide=N -->` comment before every slide it contributes, so a build can say which
source file and which slide of it each assembled slide came from, and so which units cite a
partial that changed.

## The unit's forms

Each of the course model's four forms of a unit has a Lectern output behind it:

| Form | From Lectern |
|---|---|
| `slides` | `lectern build`: reveal.js HTML |
| `transcript` | `lectern build -f outline`: each slide's heading (or its label), its prose, the speaker notes as prose, and each Mermaid diagram as its description |
| `handout` | `lectern build -f pdf`: by default two slides to a page with their speaker notes; `--layout 1up` gives one slide a page, as a tagged PDF a screen reader can read (needs the `lectern-slides[pdf]` extra) |
| `page` | an article; ikigai-learn's design note names no Lectern output for it |

`lectern check` runs an accessibility audit by default, and each warning names its source file
and line: every slide needs a name (a heading or a `label`), images need alt text (an empty alt
marks one as decorative), an `<iframe>` needs a `title`, a Font Awesome icon needs
`aria-hidden="true"` or a name, a Mermaid diagram needs `accTitle` and `accDescr`, and a theme's
text colors are checked against 4.5:1 and its accent color against 3:1.

## Code that runs on a slide (unreleased)

A deck can render a code fence as a custom element, so a slide can carry a runnable cell. The
manifest maps a fence's language to an element and names the element's files; Lectern knows
nothing about the element itself. Lectern's README shows it with an ikigai cell:

```toml
[fences.ikigai]
element = "ikigai-cell"
scripts = ["vendor/ikigai/ikigai-cell.<hash>.js"]          # loaded as module scripts
files   = ["vendor/ikigai/ikigai-cell.<hash>.css",          # copied, not linked
           "vendor/ikigai/ikigai_web_demo.<hash>.js",
           "vendor/ikigai/ikigai_web_demo_bg.<hash>.wasm"]

[fences.ikigai.data]          # optional: a block the element reads, passed through untouched
type = "application/ikigai-kernel+json"
file = "kernel.json"
```

A fence in the `ikigai` language then renders as `<ikigai-cell>` with the fenced code as its text
content; every other fence renders as before. Every file is local to the deck (a URL is refused),
and the files are copied together under their own names into one content-hashed directory, so an
element's files still find each other. `lectern check --built` builds the deck and checks each
element in headless Chromium: its Run button needs an accessible name, and its output must be a
live region. Fence elements are rendered by the reveal.js renderer and its PDF; other renderers
show the fence as ordinary code.

## A deck that loads nothing from another host (unreleased)

By default a reveal.js deck loads reveal.js from a CDN. A course release has to be
self-contained (see [A course release, pinned to a tag](../course/release.md)), and one setting
makes the deck so:

```toml
[reveal]
assets = "vendored"        # "cdn" (the default) | "vendored" | "local"
# path = "vendor/reveal"   # with assets = "local": a directory holding reveal.js
```

`vendored` copies the reveal.js 5.1.0 files that ship inside Lectern into the build; `local`
copies them from a directory of your own, and refuses one with files missing, naming them. Either
way each file lands in `assets/` under a content-hashed name, and the page loads only those
relative paths. Mermaid, math and the free Font Awesome kit still load from a CDN, so a
self-contained deck that uses them is refused with a sentence saying so. `lectern check` flags any
load from another host that remains, such as a remote image, and `lectern check --built` flags
every request the built page makes to another host.

## What stays out of a deck

ikigai-learn's design note records one rule from a measurement: an HTML-comment directive in a
slide is copied verbatim into the built HTML, even though the preview does not show it. So **an
answer key is never written into a deck, not even as a comment**, because the comment ships in the
published page. Keys go in a separate file per unit, outside every path Lectern reads, served only
under the course's key capability, and a key found in a deck is to fail the release build.

Questions are to be written inside the unit, as directives that a Lectern deck and a Book chapter
share in one syntax. Lectern has no question syntax of its own, so until one renders, an author
previews the slides in Lectern and the questions through ikigai. Both are planned; the chapter on
questions and exercises will show them once they are built.
