# OpenSpec in one chapter

[A ledger of your own](../ledger/your-own.md) gave you a place to record work and ask what to
do next. A ledger item alone does not say what the software is *supposed to do*, so an agent
needs a spec to work from. [OpenSpec](https://github.com/Fission-AI/OpenSpec)
(MIT, by Fission AI; this book follows the layout in version 1.14.1) is a small
convention for writing it down where the code is: Markdown files in an `openspec/` directory,
with just enough structure that a tool — or a SPARQL query — can find the parts.

This chapter is the convention, on a tree this book commits; the next two lift that tree into
a graph and keep its work in a ledger. Nothing here needs OpenSpec's own CLI, which drafts these
files with an AI assistant and validates and archives them; what follows is only the format it
reads and writes.

## Two halves

```text
crates/openspec-ledger/openspec/
├── specs/                      what the system does NOW, one folder per capability
│   ├── board/spec.md
│   └── moves/spec.md
└── changes/                    what someone PROPOSES to change, one folder per change
    └── add-undo/
        ├── proposal.md         why, and what changes
        ├── specs/moves/spec.md the change as a delta against specs/moves/spec.md
        └── tasks.md            the checklist that implements it
```

**Specs** are the source of truth: they describe how the system behaves today, grouped by
**capability**. **Changes** are proposals, kept apart until they are done, so several can be in
flight without editing the truth underneath each other. The game is tic-tac-toe, as in
[The ikigai Book](../../index.html), because everybody already knows its rules and the spec can be
short.

## A spec: requirements and scenarios

```markdown
{{#include ../../../../crates/openspec-ledger/openspec/specs/moves/spec.md}}
```

A `### Requirement:` is one behavior, stated with an RFC 2119 keyword — **SHALL** or **MUST**
for an absolute, **SHOULD** for a recommendation, **MAY** for an option. Each has at least one
`#### Scenario:`, a concrete case written as **WHEN** / **THEN** (and **AND**) steps that a test
could check. That is all the structure there is: headings with fixed prefixes, and bullets under
them. A spec says what is observable, never how it is built; class names and libraries belong in
a change's design, not here.

## A change: a proposal, a delta, and tasks

The proposal says why, and which capabilities the change touches:

```markdown
{{#include ../../../../crates/openspec-ledger/openspec/changes/add-undo/proposal.md}}
```

The **delta spec** says what will be different, by section — `ADDED`, `MODIFIED` and `REMOVED
Requirements` — rather than restating the whole spec, so a reviewer reads only the change:

```markdown
{{#include ../../../../crates/openspec-ledger/openspec/changes/add-undo/specs/moves/spec.md}}
```

A `MODIFIED` requirement is written out in full, as it will read afterwards, and is matched to
the current one by its name. And the **tasks** are an ordinary Markdown checklist, numbered by
group:

```markdown
{{#include ../../../../crates/openspec-ledger/openspec/changes/add-undo/tasks.md}}
```

⚠ **One convention here is this book's, not OpenSpec's.** A task that ends in a requirement's
name in parentheses — `(The last move can be undone)` — says which requirement it implements.
OpenSpec does not require this link in a task; this book needs the link to file each task as a
ledger item *about* its requirement, so it adds the parenthesis, and a task without one (1.1
here) has no requirement link. A project adopting this would agree on the same convention, or
put the link somewhere else and adjust the [mapping query](graph.md#the-mapping-by-name) that
reads tasks.

## Archiving

When a change is done, OpenSpec **archives** it: the delta is merged into the current specs (the
added requirement appended to `specs/moves/spec.md`, the modified one replaced), and the change's
folder **moves** to `changes/archive/<date>-<id>/`, intact, as the record of why the specs say
what they say. The active change folders sit directly under `changes/`; completed changes sit under
`changes/archive/`. [Changes and tasks as ledger items](ledger.md) uses that move to close the
remaining items for a change.

<a id="where-this-goes-next"></a>

## Where to go next

The files above let a person read one spec or change at a time. Questions across files take
more work: *which requirements does this change touch, which of its tasks are still open, which
requirement has the most work against it, which items in the ledger are about a requirement that
no longer exists?* [An openspec/ tree as a graph](graph.md) turns the tree into a graph you can
query, without writing a parser for any of it.
