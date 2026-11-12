# Accessibility, done and not bolted on

WCAG 2.2 AA is the floor. Start with the design review below, then read how criteria are kept
as data and how their checks are reported. The rest of this chapter is still to be written.

The planned sections cover the behavior that belongs to the shared shell: landmarks and a skip
link, one status region and what may speak in it, focus after a swap, keyboard paths, target
sizes and reduced motion. They will explain how each is checked: axe in every theme and media
setting, and browser checks for what axe cannot see.
[htmx: server-rendered HTML with push](../delivery/htmx.md) lists the checks the inspector
runs today.

## A design review, before code

Before implementation, attach an accessibility annotation layer to the design, including its
narrow layout, focus and error states. Answer each question below, or record why it does not
apply. Keep the answers with the design so the implementer and reviewer can check the same
intent. This is a short review of design decisions, not a complete WCAG audit or evidence that
the finished UI conforms.

The questions adapt [AtomicA11y's accessible-design entries](https://github.com/charlietriplett/atomica11y-skills/tree/4b8b1e712fd924517a2f2fd75b5479f123ba9704/atomica11y-skills/accessible-design)
(copyright 2026 Charlie Triplett LLC, [Apache-2.0](https://github.com/charlietriplett/atomica11y-skills/blob/4b8b1e712fd924517a2f2fd75b5479f123ba9704/LICENSE.txt))
into questions and annotations for this book. We select A and AA requirements from
[WCAG 2.2](https://www.w3.org/TR/WCAG22/); the source also includes stricter checks.
SC below means success criterion. The principle and disability groups help reviewers find
barriers; people can belong to several groups.

- **Structure:** What is the page title, page language, heading hierarchy and meaningful reading
  order? Mark heading levels and regions on the design, including how repeated navigation is
  bypassed. (Perceivable, Operable, Understandable; blindness, cognitive disabilities;
  SC 1.3.1, 1.3.2, 2.4.1, 2.4.2, 2.4.6, 3.1.1.)
- **Controls:** Which items navigate and which perform actions? Mark link or button roles,
  the accessible name of every icon button, and each custom component's name, role, state and
  group. For buttons, links and fields, attach the per-kind criterion below rather than copying
  its acceptance checks. (Operable, Robust; blindness, motor and speech disabilities;
  SC 2.4.4, 2.5.3, 4.1.2.)
- **Pictures and media:** What text alternative conveys each informative image's purpose, and
  which images are decorative? For media, specify captions, audio description or a transcript
  as applicable. (Perceivable; blindness, deafness and hearing loss; SC 1.1.1, 1.2.1–1.2.5.)
- **Appearance and layout:** Do the annotated color pairs meet text and non-text contrast, and
  do meaning and errors survive without color? Show text at 200%, text-spacing changes and
  reflow at 320 CSS pixels, retaining information and controls; document any permitted reflow
  exception. (Perceivable; low vision, color-perception and cognitive disabilities;
  SC 1.4.1, 1.4.3, 1.4.4, 1.4.10, 1.4.11, 1.4.12.)
- **Keyboard and pointer:** What is the focus order, visible focus treatment and keyboard path
  through each interaction, including dismissal and return from overlays? Show that focus is
  not entirely hidden by sticky content. Mark pointer targets of at least 24 by 24 CSS pixels
  or document the applicable [SC 2.5.8 exception](https://www.w3.org/WAI/WCAG22/Understanding/target-size-minimum.html),
  including spacing where used; provide a single-pointer alternative to dragging.
  (Operable; motor disabilities, blindness, low vision; SC 2.1.1, 2.1.2, 2.4.3, 2.4.7,
  2.4.11, 2.5.7, 2.5.8.)
- **Forms:** Does the narrow layout stack each field with its label, hint and error in a
  meaningful sequence? Annotate required fields, related groups, error identification and
  correction, and the outcome of submission using the field criterion below. Stacking is our
  design technique, not a WCAG requirement. (Perceivable, Understandable, Robust; blindness,
  low vision, cognitive disabilities; SC 1.3.1, 1.3.2, 3.3.1, 3.3.2, 3.3.3, 4.1.2.)
- **Changes over time:** What happens to focus after navigation or a swap, and which status
  messages should be announced without taking focus? Identify automatic movement or updates
  that need pause, stop or hide controls, and avoid flashing above the threshold.
  (Operable, Robust; blindness, motor and cognitive disabilities, photosensitivity;
  SC 2.2.2, 2.3.1, 2.4.3, 4.1.3.)

Use ikigai-ui's criterion resource for the delivery target being designed:

| Kind | HTML criterion | Source for HTML and GPUI criteria |
|---|---|---|
| Button | `urn:iki:ui:criterion:kind:button:target:html` | [button criteria](https://github.com/ikigai-rs/ikigai-ui/blob/main/assets/criterion/button.json) |
| Link | `urn:iki:ui:criterion:kind:link:target:html` | [link criteria](https://github.com/ikigai-rs/ikigai-ui/blob/main/assets/criterion/link.json) |
| Field | `urn:iki:ui:criterion:kind:field:target:html` | [field criteria](https://github.com/ikigai-rs/ikigai-ui/blob/main/assets/criterion/field.json) |

For GPUI, replace `target:html` with `target:gpui` and read the record's limitations as well as
its checks. A criterion describes intended behavior; it does not prove that a face delivers it.
The AA floor here does not adopt the source's 44 by 44 target size (SC 2.5.5, AAA) or Focus
Appearance (SC 2.4.13, AAA). Any stricter product decision must be explicit.

## Criteria as data

A criterion is a record, not a paragraph to copy. ikigai-ui names one per kind and delivery
target, `urn:iki:ui:criterion:kind:{kind}:target:{target}`, so a design note, a test and a
report can all cite the same record by name: `urn:iki:ui:criterion:kind:field:target:html` is
the field kind's HTML criterion, and the same kind with `target:gpui` is its desktop one. The
record for a kind and target says, in fields of their own, its semantics, its keyboard paths
(each key with its outcome), its name, role and state, the settings a reader may change, and
the checks that hold it. Resolve the record by its name when you review or test a face; do not
paraphrase it into the design, where it would drift.

The criterion records are on ikigai-ui's main branch
([PR 65](https://github.com/ikigai-rs/ikigai-ui/pull/65) for button, link and field,
[PR 67](https://github.com/ikigai-rs/ikigai-ui/pull/67) for every other kind and element). No
release on crates.io carries them yet (0.5.0 does not), and this book uses only published
crates, so no example here resolves one: the links in the table above open the records'
source.

## What a reader sees and what a reader hears

Each check in a record carries a mode. A **SEE** check is something a sighted reader, or a
script standing in for one, can confirm from the rendered face: a native `<button>`, a focus
ring of at least 2 px, a name that matches the visible text, text that grows at 200%. Many SEE
checks name a probe that a test can run against the face's HTML. A **HEAR** check is what a
screen reader announces: "WHEN I Tab to the button with a desktop screen reader I HEAR its
purpose and its role of button." Only someone running a screen reader, or a job that drives one,
can confirm it. Passing every SEE check says nothing about the HEAR checks; keep them apart in
any report.

## Pass, Fail or Blocked

Report every check a criterion or an accessibility file names, one row each, in a
`Check | Result | Evidence` table:

- **Pass**: the check ran on the code being reported, and held. The evidence names the test or
  command, the tool and its version, and what the check does not cover.
- **Fail**: the check ran and did not hold. The evidence says what failed and where.
- **Blocked**: the check did not run. The evidence says why: no tool, no screen-reader job, a
  dependency not released. A check nobody ran is Blocked, never left out and never "not
  applicable". A run from an earlier date that nothing repeats is Blocked too, with its date.

Until a screen-reader job runs, every HEAR check is Blocked. This repository's own
[ACCESSIBILITY.md](https://github.com/ikigai-rs/ikigai-tutorial/blob/main/ACCESSIBILITY.md)
reports its checks this way. A table of Pass rows is evidence about what those checks
exercise. It is not a conformance claim.

Carry these annotations into implementation review, then test the rendered UI, including with
assistive technology. The [htmx chapter's checks](../delivery/htmx.md) are evidence about what
they exercise, not a substitute for those tests.
