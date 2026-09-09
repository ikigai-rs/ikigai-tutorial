"""Part I, from Python: resolve, describe, cache, trace, and be refused.

Against a running kernel — `ikigai serve <socket>` — this walks the same four things
"What resolution buys you" ran in Rust, over the wire, from a language with no kernel in
it at all.

    ikigai serve /tmp/ikbook-kernel.sock &
    python3 examples/python/client.py /tmp/ikbook-kernel.sock

Needs ``ikigai-python`` installed (``pip install <path to ikigai-python>``).
"""

from __future__ import annotations

import sys

import ikigai
from ikigai import Capability


def main(path: str) -> None:
    # ANCHOR: resolve
    # A name, resolved. `connect` speaks the wire protocol over the kernel's socket; the
    # answer is a representation — bytes with a type — exactly what a Rust caller gets.
    k = ikigai.connect(path)
    rep = k.source("urn:iki:fn:toUpper", **{"in": "resource oriented computing"})
    print(rep.text)            # RESOURCE ORIENTED COMPUTING
    print(rep.media_type)      # text/plain;charset=utf-8
    print(rep.cache_status.name)   # how the kernel's cache answered: MISS, HIT, UNCACHEABLE
    # ANCHOR_END: resolve

    # ANCHOR: describe
    # Self-description, as data. `describe` is the JSON face of Meta, parsed: the
    # ArgSpecs an endpoint declared, which is what an agent's tool definition is made of.
    card = k.describe("urn:iki:fn:toUpper")
    print(card["id"], [arg["name"] for arg in card["inputs"]])   # toUpper ['in']
    # ...and the same card as a graph, which the notebook chapter queries.
    turtle = k.meta("urn:iki:fn:toUpper", as_="text/turtle").text
    print(turtle.splitlines()[0])                                  # @prefix ik: <https://ikigai-rs.dev/ns#> .
    # ANCHOR_END: describe

    # ANCHOR: cached
    # Cached once: the probe says not yet; a resolution; the probe says served now.
    print(k.is_cached("urn:iki:fn:toUpper", **{"in": "a b"}))     # False
    k.source("urn:iki:fn:toUpper", **{"in": "a b"})
    print(k.is_cached("urn:iki:fn:toUpper", **{"in": "a b"}))     # True
    # ANCHOR_END: cached

    # ANCHOR: traced
    # Traced: the kernel records its own events and ships them back over the wire.
    rep, events = k.source_traced("urn:iki:fn:toUpper", **{"in": "a b"})
    for event in events:
        print(event.target, "cache_hit" if event.cache_hit else "computed")
    # ANCHOR_END: traced
    k.close()

    # ANCHOR: scoped
    # Capabilities: connect under a narrowed authority and the kernel enforces it — the
    # served kernel clamps what you carry to what the channel is entitled to, and a
    # write to the file workspace under a read-only scope is refused, typed.
    narrow = ikigai.connect(path, capability=Capability.scoped(["urn:cap:kernel:inspect"]))
    try:
        narrow.sink("urn:file:notes.txt", "nope")
    except ikigai.DeniedError as denied:
        print("denied:", denied.message)
    narrow.close()
    # ANCHOR_END: scoped


if __name__ == "__main__":
    main(sys.argv[1] if len(sys.argv) > 1 else "/tmp/ikbook-kernel.sock")
