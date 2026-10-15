# Working in this repository

`ikigai-tutorial` holds *The ikigai Book* and *The gonk Book* (mdbook, under `books/`) and the code
they teach (real crates, under `crates/`). These rules are for every agent and every person working
here. `CLAUDE.md` is a link to this file.

- **Read [ACCESSIBILITY.md](ACCESSIBILITY.md) before any change to a book's pages, styles, scripts
  or in-page kernels.** It says what is in scope, what the checks cover and do not, and the
  barriers we know about.
- Security reports go through the organization's security policy (GitHub private vulnerability
  reporting), never a public issue.
- Every change lands through a pull request with green CI (`.github/workflows/ci.yml` and
  `pages.yml`); a merge to main republishes the books. Run `./scripts/test-books.sh` before pushing
  a change to a book.
- A change to a book's published prose adds an entry to that book's `changes.toml` (its What's new
  page); the `whats-new` check refuses the pull request otherwise.
- Never invent testing, tool results or conformance evidence, and never weaken a check to make a
  change pass.
- Use American spelling in everything you write: prose, code comments, commit messages and pull
  requests. The game in the book is tic-tac-toe.
