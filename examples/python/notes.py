"""A family of resources, in Python: one template, four verbs.

`hello.py` served two exact names. This serves a FAMILY — every IRI one template
matches — and answers more than one verb on it: read a note, write it, delete it, ask
whether it is there. Run it and mount it the way the chapter mounts `hello.py`:

    python3 examples/python/notes.py /tmp/py-notes.sock
    ikigai --plain --mount urn:py:=/tmp/py-notes.sock -c 'sink urn:py:note:milk "2 liters"'

Modeled on ikigai-python's ``examples/tictactoe_store.py`` (read-only for this book);
kept here so the book builds from a clone of this repository alone. Needs
``ikigai-python`` installed (``pip install <path to ikigai-python>``).
"""

from __future__ import annotations

import sys
import threading
from typing import Annotated

from ikigai import NotFoundError, endpoint, family, serve

NOTES: dict[str, str] = {}
LOCK = threading.Lock()  # one thread per connection

# ANCHOR: index
# An exact name, declared FIRST: the family below would match `urn:py:note:index` too
# (its `{name}` would capture `index`), and the first declared door that matches wins.
# Not cacheable: it reads every note, and a write cuts only the note it wrote.
@endpoint("urn:py:note:index", summary="The notes there are, one name per line")
def index() -> str:
    with LOCK:
        return "\n".join(sorted(NOTES))
# ANCHOR_END: index


# ANCHOR: family
# One door for every `urn:py:note:{name}`: `{name}` is a Level 1 variable, matched the
# way ikigai_core::UriTemplate matches it, and it reaches each handler as a parameter.
note = family("urn:py:note:{name}", id="py-note", summary="A note, by name")

Name = Annotated[str, "the note's name"]


@note.source(cacheable=True, summary="the note's text; NotFound if there is none")
def read(name: Name) -> str:
    with LOCK:
        if name not in NOTES:
            raise NotFoundError(f"no note called {name}")
        return NOTES[name]


@note.sink(summary="write the note")
def write(name: Name, content: Annotated[str, "the note's text"]) -> str:
    with LOCK:
        NOTES[name] = content
    return f"wrote {name}"


@note.delete(summary="forget the note")
def forget(name: Name) -> str:
    with LOCK:
        NOTES.pop(name, None)
    return f"forgot {name}"


# No `@note.exists`: Exists defaults to "would Source succeed", so a NotFoundError is
# `false` — cacheable exactly when Source is.
# ANCHOR_END: family


if __name__ == "__main__":
    path = sys.argv[1] if len(sys.argv) > 1 else "/tmp/py-notes.sock"
    print(f"serving urn:py:note:index and urn:py:note:{{name}} on {path}", file=sys.stderr)
    serve([index, note], path)  # the order is the lookup order
