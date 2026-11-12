# Accessibility, done and not bolted on

WCAG 2.2 AA is the floor. Start with the design review below; the rest of this chapter is
still to be written.

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

Carry these annotations into implementation review, then test the rendered UI, including with
assistive technology. The [htmx chapter's checks](../delivery/htmx.md) are evidence about what
they exercise, not a substitute for those tests.
