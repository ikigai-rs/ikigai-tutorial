# The ikigai Book — design note

Not published: mdbook builds `src/`, and this file is beside it. It starts with the one decision
the next arc should not have to re-derive; the README's "The book is tested" says how each kind of
example is kept true.

## Serial-reader continuity (ledger #1623)

Every gate this book runs checks a page, or a block, on its own. A reader who goes front to back
does not arrive with nothing: a REPL still open from the front door, `ikigai serve` still on
`/tmp/ikbook-kernel.sock` from the Python track, `ttt-host` still on port 8070 from Tic-tac-toe V,
and a head full of the names earlier chapters used. Brian asked (2026-10-15) for the books to be
read that way, and for it to stay true on every change:

- **Setup, cleanup, names and state.** A change that adds, moves or removes any of them (an
  install, a REPL session, a server or socket, a certificate, a config home or `tutorial.toml`, an
  environment variable, a scratch directory) reads the chapters before and after it.
- **Say what the reader has.** "If you installed it in the front door, it is already there", "if
  the `ttt-host` from Part V is still running, stop it first: this one serves the same port",
  "start a fresh `ikigai` (quit a REPL still open from the front door)".
- **Standalone still.** A reader may open any chapter, so earlier setup is acknowledged, never
  deleted.
- **Order words are claims.** "The previous chapter", "the last two chapters", "three chapters of
  plumbing" and "six parts" all go stale when a chapter is inserted: "A query's authority is its
  dataset" (2026-10-15) made five of them wrong in Part III at once. Name the chapter with a link
  instead of counting. A forward reference says it is one ("later in the book"); a term (the
  manifold, a corridor) is defined before a chapter leans on it.
- **One claim, one answer.** When a later release changes a rule (core 0.1.73 made a cut of a pure
  function's name reach it), every chapter that stated the old rule changes with it, including the
  glossary.

`crates/book-urns/tests/book_continuity.rs` checks the part of this a parser can decide: a link
whose text says the next or previous chapter (or page, or part) points at the neighboring page in
`SUMMARY.md`. Its other rule, a page that starts a long-running piece links the page whose piece a
reader still has running, sees only `<!-- transcript: serve -->` blocks, and this book has none: its
servers start in `bash` and `sh` blocks that no runner classifies (the REPL replay and the
`cargo run` check do not start anything that stays up). Declaring them as `serve` would bring them
under that rule, and is the cheap next step if a server-starting chapter is added; until then this
part is review.
