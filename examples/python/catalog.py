"""The notebook, as a script: catalog → describe() → as=text/turtle → rdflib → SPARQL.

`examples/python/catalog.ipynb` is the same cells with prose between them; this file is
what a test can run. Against a running kernel:

    ikigai serve /tmp/ikbook-kernel.sock &
    python3 examples/python/catalog.py /tmp/ikbook-kernel.sock

Needs ``ikigai-python`` and ``rdflib`` (``pip install rdflib <path to ikigai-python>``).
"""

from __future__ import annotations

import sys

import ikigai
import rdflib

IK = rdflib.Namespace("https://ikigai-rs.dev/ns#")


def main(path: str) -> None:
    k = ikigai.connect(path)

    # ANCHOR: entries
    # 1. The catalog as a list: every name the kernel binds, with the endpoint behind it
    #    and — for a mounted peer — where it is served from.
    entries = k.entries()
    print(len(entries), "bound names; the first three:")
    for entry in entries[:3]:
        print(" ", entry.pattern, "→", entry.endpoint)
    # ANCHOR_END: entries

    # ANCHOR: describe
    # 2. One endpoint's card, as data — the JSON Meta face, parsed.
    card = k.describe("urn:iki:fn:toUpper")
    print(card["title"], "—", card["summary"])
    for arg in card["inputs"]:
        print("  input", arg["name"], "required" if arg.get("required", True) else "optional")
    # ANCHOR_END: describe

    # ANCHOR: graph
    # 3. The whole catalog as a graph. `urn:kernel:catalog` answers as Turtle, and rdflib
    #    parses it; from here on it is RDF and the vocabulary is the schema.
    turtle = k.source("urn:kernel:catalog").text
    g = rdflib.Graph()
    g.parse(data=turtle, format="turtle")
    print(len(g), "triples in the catalog")
    # ANCHOR_END: graph

    # ANCHOR: sparql
    # 4. One SPARQL query over it: which endpoints take a string? The same question the
    #    graph-face chapter asked with `urn:sparql:select`, asked here by a client.
    query = """
        PREFIX ik: <https://ikigai-rs.dev/ns#>
        SELECT DISTINCT ?id WHERE {
          ?endpoint a ik:Endpoint ; ik:id ?id ; (ik:input | ik:action/ik:input) ?input .
          ?input ik:class <http://www.w3.org/2001/XMLSchema#string> .
        } ORDER BY ?id
    """
    for row in g.query(query):
        print(" ", row.id)
    # ANCHOR_END: sparql

    k.close()


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "/tmp/ikbook-kernel.sock")
