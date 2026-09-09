# The notebook: the catalog as a graph

Four cells, one story: the catalog is a list, then a card, then a graph, then a query.
`examples/python/catalog.ipynb` is the notebook; `examples/python/catalog.py` is the same
cells as a script, which is what was run to produce the output below (against a served
`ikigai-cli` 0.1.18 — the counts are that host's, and yours will differ).

```bash
pip install rdflib /path/to/ikigai-python
ikigai serve /tmp/ikbook-kernel.sock &
python3 examples/python/catalog.py /tmp/ikbook-kernel.sock
```

## 1. The catalog as a list

```python
{{#include ../../../../examples/python/catalog.py:entries}}
```

```text
261 bound names; the first three:
  urn:agent:select → agent-select
  urn:client:issue → client-issue
  urn:cms:bookmarks → bookmarks
```

`entries()` is what `list` prints at the shell, as data: pattern, endpoint id, and — for a
mounted peer — where it is served from. A client discovers what it can reach instead of
being told.

## 2. One card, as data

```python
{{#include ../../../../examples/python/catalog.py:describe}}
```

```text
Upper-case — Upper-cases the UTF-8 text supplied in the `in` argument.
  input in required
```

`describe` is the JSON face of `Meta`, parsed. This is the dictionary an agent's tool
definition is projected from — [Why an endpoint describes
itself](../getting-started/self-description.md), reached by `dict` lookup.

## 3. The whole catalog, as a graph

```python
{{#include ../../../../examples/python/catalog.py:graph}}
```

```text
3289 triples in the catalog
```

`urn:kernel:catalog` answers as Turtle, rdflib parses it, and from here on it is RDF: the
vocabulary at `https://ikigai-rs.dev/ns#` is the schema, every endpoint and every input is
a node with a stable IRI, and a graph library's whole toolkit applies.

## 4. One query

```python
{{#include ../../../../examples/python/catalog.py:sparql}}
```

```text
  decrypt
  encrypt
  eval
  rdf-from-sexpr
  sexpr-from-rdf
  sexpr-to-rdf
  sign
  sparql-from-sexpr
  tz-convert
  tz-now
  verify
```

The same question [The graph face](../building/graph-face.md) asked with
`urn:sparql:select` — which endpoints take a string? — asked here by a client with its own
query engine, over a catalog it fetched by name. Two things to notice in the answer.
`toUpper` is not in it, because `ikigai-fn` 0.2.0 declares no `class` on its `in` input; an
undeclared class is invisible to the question, which is [the argument for declaring
one](../getting-started/self-description.md). And the property path
`(ik:input | ik:action/ik:input)` is the one the graph-face chapter needed too: a
multi-verb endpoint's inputs hang off its action nodes.

> ⚠ The catalog you get is the catalog of the host you connected to. The CLI's is a few
> hundred endpoints from a dozen crates; the tutorial host's is eleven. Both are the same
> shape, and a query written against one runs against the other — but the answer is a fact
> about a host, not about ikigai.
