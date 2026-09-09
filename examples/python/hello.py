"""Hello, resource — in Python.

The smallest ikigai peer there is: one function, one decorator, one socket. Run it and
a Rust host can mount it; `--mount urn:py:=<socket>` in the chapter is the binding.

    python3 examples/python/hello.py /tmp/py-hello.sock

Modeled on ikigai-python's ``examples/endpoints.py`` (read-only for this book); kept
here so the book builds from a clone of this repository alone. Needs ``ikigai-python``
installed (``pip install <path to ikigai-python>``); it has no dependencies of its own.
"""

from __future__ import annotations

import sys
from typing import Annotated

from ikigai import endpoint, serve

# ANCHOR: endpoint
# The decorator is the description, and the signature is the contract. `who: str`
# becomes a required input of class xsd:string; the `Annotated` text becomes its
# summary; `cacheable=True` is the same claim `.cacheable()` makes in Rust — a pure
# function of its declared inputs — and it is the mounting kernel, not this process,
# that will honor it.
@endpoint("urn:py:hello", summary="Greet someone", cacheable=True)
def hello(who: Annotated[str, "the name to greet"]) -> str:
    return f"Hello, {who}!"


# A second one, so `list` has something to show and a pipe has something to feed.
@endpoint("urn:py:shout", summary="Uppercase a string, loudly", cacheable=True)
def shout(text: Annotated[str, "the text to shout"]) -> str:
    return text.upper() + "!"
# ANCHOR_END: endpoint


# ANCHOR: serve
if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/py-hello.sock"
    print(f"serving urn:py:hello and urn:py:shout on {path}", file=sys.stderr)
    serve([hello, shout], path)  # blocks; speaks the wire protocol
# ANCHOR_END: serve
