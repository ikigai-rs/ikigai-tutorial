# The course model

In ikigai-learn, everything a course is made of has a name under `urn:learn:`. That one decision
carries most of the design: a course, a unit, a single slide, a question, a learner's answer and
the verdict on it are all resources, so each one can be cached, composed, read under a
capability, and cited by name.

**Status:** this chapter is the plan. The names below are ikigai-learn 0.1.0's resource map, and
only the rows marked *built* are implemented; the rest are designed and not yet built.

## Four ideas

- **A course is a list of names.** A course release is a manifest at a git tag: an ordered list of
  modules, each an ordered list of unit names. Reusing a unit in another course is naming it in
  that course's manifest too.
- **A learner's work is a gonk ledger.** Enrolling creates a ledger for the enrollment, with an
  opaque name, and seeds one item per exercise, so a learner's progress through a course is work
  on a ledger like any other ([The gonk Book](../../gonk/index.html) teaches the ledger).
- **Review is a tuplespace.** Grading is a layer where each submission is an offer, carrying the
  enrollment's pseudonym and never the learner's name, taken by one grader under a lease, so a
  grader who wanders off releases it.
- **Progress is a question the kernel answers.** Mastery and progress are compositions (stored
  SPARQL or s-expression programs over one learner's answers), never a stored counter. An answer
  recomputes that learner's progress and nobody else's.

## What the design encodes

ikigai-learn's README states each of these as a consequence in the resource map, with the
research behind it, not as a preference:

- **Retrieval, not rereading.** Questions sit inside every unit, and nothing marks a page as read.
- **Spaced review.** A review queue whose schedule is data (Leitner boxes first), and mastery that
  needs two sessions at least a day apart.
- **Mastery before progression.** The next thing offered is the first unit not yet mastered, not
  the next one in order.
- **Short units**, about six minutes each, and early exercises that are worked examples whose
  support fades.
- **A wrong answer is a value**: a verdict of `incorrect`, never a refusal, so a review stays
  cacheable. "I don't understand this yet" is excluded from every evaluation by construction.
- **No model verdict.** A verdict is a person's, then the deterministic checks', then `pending`. A
  model may draft feedback, shown as a draft in its own graph, never as a grade.

## Content: authored, and fixed at a release

A unit is an authored Markdown file, a [Lectern](../units/lectern.md) deck or a Book chapter like
the ones you are reading, that an author writes in git and a learner never writes. These are its
names and the names around it:

| Name | What it is |
|---|---|
| `urn:learn:unit:name:{u}` | a unit: an authored Lectern deck or Book chapter |
| `urn:learn:unit:name:{u}:release:{r}` | the unit at a release's git tag; cacheable forever |
| `urn:learn:unit:name:{u}:form:{f}` | the unit as `slides` (HTML), `page` (an article), `transcript` (prose, notes and diagram descriptions, for screen readers and audio) or `handout` (PDF) |
| `urn:learn:slide:unit:{u}:number:{n}` | one slide, citable by feedback |
| `urn:learn:slide:unit:{u}:range:{spec}` | a slide range, in Lectern's `#1-3,14` grammar |
| `urn:learn:lift:unit:{u}` | the unit's structure as RDF: ikigai-markdown's lift plus a Lectern mapping |
| `urn:learn:question:unit:{u}:id:{q}` | a question written inside the unit (choice, exact, short answer, or a check script) |
| `urn:learn:key:question:{q}`, `urn:learn:key:exercise:{e}` | an answer key; a reviewed exercise's key is sealed, readable only under the course's key capability |
| `urn:learn:exercise:name:{e}` | an exercise: the task, the prediction to write first, whether an agent may help (`allowed`, `draft-only`, `none`), the evidence that counts as done, the rubric |
| `urn:learn:check:exercise:{e}:name:{k}` | a check, as an ikigai-script resource |
| `urn:learn:course:name:{c}:release:{r}` | a course release: its manifest at a git tag |
| `urn:learn:rule:course:{c}:name:{n}` | a mastery rule, as data |
| `urn:learn:schedule:name:{s}` | a review schedule, as data |

Names are singular, the type first, and every partition is by a named key (`name:`, `release:`,
`number:`), so a name says what it is before it says which one.

## The learner's space

| Name | What it is |
|---|---|
| `urn:learn:learner:id:{l}` | a learner: a principal (a passkey for a person, a certificate for their agent) |
| `urn:learn:enroll:course:{c}` | the enroll Sink: writes an enrollment, creates a ledger with an opaque name, and seeds one item per exercise |
| `urn:learn:enrollment:id:{n}` | an enrollment |
| `urn:learn:response:enrollment:{n}:question:{q}:id:{r}` | a response, append-only |
| `urn:learn:result:submission:{s}:check:{k}` | a check's result, cached by the submission's digest and the check's version |
| `urn:learn:verdict:submission:{s}` | the verdict: a person's, then the checks', then `pending` |
| `urn:learn:review:enrollment:{n}:date:{d}` | the review queue for a day: a composition |
| `urn:learn:next:enrollment:{n}` | the next thing to work on: a composition |
| `urn:learn:release:enrollment:{n}` | **built.** The learner's list of their releases on this enrollment |
| `urn:learn:release:enrollment:{n}:party:{p}` | **built.** A release: part of the learner's record shared with one party |
| `urn:learn:credential:enrollment:{n}` | **built.** The `credential` view |
| `urn:learn:completion:enrollment:{n}` | **built.** The `completion` view: complete or not, and when |
| `urn:learn:progress:enrollment:{n}` | **built.** The `progress` view: mastery per unit and the verdict per exercise |
| `urn:learn:submission:enrollment:{n}:exercise:{e}` | **built.** The `submission` view: the work submitted for one exercise |

The built rows are the release grant, the subject of [the next chapter](release.md).

A learner's answers, submissions and notes are their own, read over the shared names of the
release they enrolled in, and their capability decides which graphs exist for them, so there is
nothing to filter afterward. A learner's record exports losslessly (Turtle and JSON-LD, the same
names). Counts about people are floored (`fewer than 5`), and the module keeps no per-learner
rows, no time on page, no per-person funnels, no cookies and no stored addresses.

## Errors are part of the contract

Each failure has one kind, and a caller can rely on it:

| Error | When |
|---|---|
| Denied | no capability, or a sealed name |
| Conflict | a second enrollment in one release, or an item already taken |
| NotFound | an unknown course, unit or release |
| InvalidArgument | a malformed answer |

A wrong answer is none of these. It is an answer, and its verdict is `incorrect`.
