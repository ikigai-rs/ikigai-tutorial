# Exercises: code and language as resources

Four, and all four are written in **your** crate — `crates/your-endpoints/tests/code.rs` —
against the code this section quotes, `crates/script-host`, `crates/script-authority` and
`crates/sentence-draft`, which you use and do not edit. The file already holds the scaffolding:
`publish` (a SPARQL script, under a capability), `result` (an answer, and whether it came from the
cache), `person` (a capability naming its principal, as a door would mint it), `stub_model` (a stub
with drafts of your own), and one test proving all of it works. The loop is:

```bash
cargo test -p your-endpoints --test code
```

---

## 1. A query of your own, and the thread it hangs from

Publish a query called `closed-items` that lists the CLOSED items, by number and title. Read its
result twice. Then run `close-item` from [Scripts are resources](scripts.md) for item 4, and read it again.

- **File** — `crates/your-endpoints/tests/code.rs`; start from `script_host::kernel()`.
- **Run** — `cargo test -p your-endpoints --test code`
- **Right when** — the first read is computed and does not list "Answer the issue"; the second is
  answered from the cache; after the run, the read is computed again and lists it.
- **Read again** — [Scripts are resources](scripts.md), "Running is a read" and "Running is a
  write".

<details>
<summary>Hint</summary>

Alice holds grants for `open-items` and `close-item`, not for your script, so the first thing to
write is a person who may publish and run `closed-items`. Which grants? The publish needs
`urn:cap:script:write:closed-items`, and is refused unless your person holds what the text derives
(`urn:cap:store:read:graph:urn:iki:tutorial:items`). Running `close-item` needs its run grant and
both graph grants the UPDATE derives, because a run is the runner's capability, narrowed.
`close-item` must be published first; the host (`Capability::root()`) can do that.

You did not tell the result to recompute when an item closes, and you cannot: there is nowhere to
say it. If the third read comes back `[cached]`, look at whether your query really reads the graph
the update writes.

</details>

---

## 2. Narrow a ceiling, and find the read that should have changed

On `script_authority::kernel(&ceilings)`, publish `GRAPHS` as `every-graph` under the host, and
have Bob read it. Then change `every-graph`'s ceiling file to the **payroll** graph alone, and read
again, and again after the cut the chapter makes. Then mint Dave, who holds
`urn:cap:script:run:team-*` and the payroll graph, and have him run Alice's `team-open` (publish
it as Alice first, as [the chapter](authority.md#a-namespace-grant-and-an-exclusion-that-wins-over-it)
does).

- **File** — `crates/your-endpoints/tests/code.rs`; `Ceilings::new()` and `set` are
  `script_authority`'s.
- **Run** — `cargo test -p your-endpoints --test code`
- **Right when** — before the cut Bob still sees the items graph; after
  `kernel.cut("urn:script:every-graph:authority")` he sees the payroll and only the payroll; and
  Dave's run is refused with `Error::Denied`.
- **Read again** — [Who a script runs as](authority.md), "Nor more than the host's ceiling
  allows".

<details>
<summary>Hint</summary>

The assertion that the read did NOT change after the edit is the one worth writing on purpose. It
is the trap the chapter showed, and a test that asserts only the after-the-cut answer would pass
on a host that forgot the cut, so long as nothing else had been cached.

Dave holds the namespace grant to run `team-open`, so the run gate lets him in. What refuses him
is the graph: `team-open` is `OPEN_ITEMS`, which reads the items, and Dave's capability has only
the payroll in it. Narrowing can only keep what is already there.

</details>

---

## 3. A draft that never validates

Script a stub whose only draft, for the ask "delete everything", is an UPDATE:
`DELETE WHERE { GRAPH <urn:iki:tutorial:items> { ?s ?p ?o } }`. Draft with it, as a person who may
write the script `wipe` and holds both item grants and a network grant, with
`save=wipe`.

- **File** — `crates/your-endpoints/tests/code.rs`; `stub_model` gives you the model and the
  stub's record of its prompts. `sentence_draft::kernel(&model)` is the host.
- **Run** — `cargo test -p your-endpoints --test code`
- **Right when** — the draft's status (read its JSON face into `ikigai_nl::Draft`) is
  `Status::Failed`; there are exactly two attempts and the stub was asked twice; and
  `Exists urn:script:wipe` answers `false`.
- **Read again** — [A sentence becomes a draft](sentence.md), "The check" and "A sentence,
  drafted, checked, repaired and saved".

<details>
<summary>Hint</summary>

Your person holds the WRITE grant on the items graph, so this is not the capability check
refusing the draft: the check refuses any UPDATE, because a draft reads. Read both attempts'
errors. The second one is not about the UPDATE at all. Why did drafting stop at two when the bound
is three?

Nothing was saved, and nothing ran. A model that answers with a destructive query on every attempt
has produced, at the end, an answer saying so.

</details>

---

## 4. The style note ledger item 1242 left out, as a composition

The live demo's draft pairs `FROM <g>` with `GRAPH <g> { … }`. On this host that is harmless (the
script host runs a query over one set of graphs that serves as both its default and its named
graphs), so ledger item 1242 dropped it as a check, and `urn:nl:sparql:check` says nothing about
it. Against an ordinary SPARQL endpoint it finds nothing. Bind an endpoint of your own at
`urn:iki:tutorial:yours:checked` that takes `content`, asks `urn:nl:sparql:check` for its JSON
face, and answers that check with one more warning when the query names a graph both in a `FROM`
and in a `GRAPH`. Mount it with `sentence_draft::kernel_with`, and ask it about the live demo's
draft and about the corrected query.

<!-- urn-gate: illustration urn:iki:tutorial:yours:checked — the name the reader binds in this
     exercise, in their own test; no host this workspace builds binds it. -->

- **File** — `crates/your-endpoints/tests/code.rs`; `sentence_draft::{P2_DRAFT, P2_CORRECTED}` are
  the two queries, and `ikigai_nl::Check` is the type of the JSON face (its `graphs` are the graphs
  the text names).
- **Run** — `cargo test -p your-endpoints --test code`
- **Right when** — `P2_DRAFT` comes back with your warning beside the check's own (its preview is
  empty), and `P2_CORRECTED` comes back with neither.
- **Read again** — [A sentence becomes a draft](sentence.md), "A draft the check passed, and
  should not have"; and [Binding, and a host of your own](../getting-started/binding.md).

<details>
<summary>Hint</summary>

`AsyncFnEndpoint` is the shape: it issues a sub-request through `inv.issue(…)`, under the caller's
capability, so your endpoint can never show anybody a preview they could not have asked for. Give
it a `Description` with `content` as its input, or nothing can route an argument to it by name.
For each graph in the check's `graphs`, look for `FROM <g>` and `GRAPH <g>` in the text; a text
search is enough for this exercise, though a parser would not be fooled by a comment.

This is the resource-first version of a check, and it is worth asking what it does not do. It
warns a person who asks; the drafter never sees it, because the drafter asks `urn:nl:sparql:check`
by name, and your endpoint is a different name. That is the difference between this note and the
empty preview: `ikigai-nl` 0.3.0 made the empty preview a reason to repair by changing the check
and the drafter, which is where a finding has to live for a model to be told about it.

</details>
