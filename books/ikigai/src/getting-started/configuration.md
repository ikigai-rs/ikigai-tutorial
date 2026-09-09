# Configuration

A host reads a config file. This chapter is about *where* it looks, *how* two files
combine, and — because the point of a book with tests is that the tests are the claim —
how an endpoint that reads configuration is tested against a directory the test owns.

The code is in `crates/building-endpoints`, one part ahead, because it is the first
endpoint in the book whose answer comes from a file. It is bound in that part's host at
`urn:iki:tutorial:banner`.

## Where files live

`ikigai-core` owns the answer, so every host agrees:

```rust
# extern crate ikigai_core;
use std::path::{Path, PathBuf};

// This machine's config home: `$XDG_CONFIG_HOME/ikigai`, else `~/.config/ikigai`.
let home: Option<PathBuf> = ikigai_core::config::config_home();

// The files that make up one logical configuration, in the order they are read.
let layers: Vec<PathBuf> = ikigai_core::config::layered_paths_in(
    Path::new("/tmp/example-home"), "tutorial.toml", Some("yours"),
);
assert_eq!(layers, vec![
    PathBuf::from("/tmp/example-home/tutorial.toml"),
    PathBuf::from("/tmp/example-home/yours.tutorial.toml"),
]);
# let _ = home;
```

`config_home()` is `None` when the machine has no home to offer. That is a legal state —
the process is under-configured, not broken — and it is `None` rather than a guess on
purpose: a guessed home reads as success.

`layered_paths_in(home, stem, app)` is the layering rule, and it is the same for every
crate in the ecosystem: the shared file, then an app-scoped **sibling** named
`<app>.<stem>` that overrides it key-wise. The stem is the whole file name, extension
included. A flat listing of `~/.config/ikigai/` then reads as what it is — `a11y.toml`
states what every front end shares, `cms-web.a11y.toml` the handful of keys one of them
differs on. The `_in` form takes the home as an argument; `layered_paths(stem, app)` is
the same rule over `config_home()`.

## A host that reads one

`[banner] text = "…"` is the whole schema. Small on purpose, so the layering is visible:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/config.rs:settings}}
```

Two rules in that loader. A **missing** file is a missing *layer* — skipped, silently,
because "no override here" is ordinary. A file that is **present and broken** stops the
read, because a config that is silently ignored is an operator believing one thing while
the process does another, and nothing anywhere disagreeing until an outage.

And the endpoint that serves it:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/config.rs:endpoint}}
```

## The home is taken at construction

`banner(home, app)` takes the home as an argument. It does not call `config_home()` in
its body, and this is the rule the chapter exists to state: **the injected form is the
real one, and the ambient read is sugar over it, done once, in the host.**

Three reasons, from `ikigai-core`'s design note on the subject. `config_home()` reads the
environment, which is process-global, so an endpoint calling it under `cargo test` reads
the *developer's* real `~/.config/ikigai` — and the tests that result assert only that a
value has a type, because they do not own the file it came from. A test cannot hand such
an endpoint a different home without `set_var`, which races the test beside it. And one
process legitimately has two answers: a test binary mounting a fixture home beside a
mount with none is the ordinary case.

So the host resolves the home once and hands it down, and a test hands down a directory
of its own:

```rust,ignore
{{#include ../../../../crates/building-endpoints/src/config.rs:layers}}
```

The last four lines of that test are the golden thread from [What resolution buys
you](payoff.md), meeting a file. The endpoint declared `.depends_on(<path>)` for each
candidate file, so the cached banner is valid until one of those threads is cut. Nothing
in this host watches the directory — a real one would — so the test cuts it by hand, and
the next read sees the override.

> ⚠ The endpoint holds the **home**, not the parsed settings, and re-reads per
> resolution. That is the right split for a value that lives in a file: the cache makes
> the re-read free until a thread is cut, and cutting recomputes. A handle that parsed
> the file at construction would serve the values the process started with, forever,
> however many times the thread was cut. The other choice is right for *process* state a
> `Sink` mutates in place — then the handle *is* the authority, not a stale copy of a
> file. Ask which one you have before you choose.

## Try it

Write a file into this machine's config home and ask the host for its banner:

```bash
mkdir -p "${XDG_CONFIG_HOME:-$HOME/.config}/ikigai"
printf '[banner]\ntext = "hello from a file"\n' > "${XDG_CONFIG_HOME:-$HOME/.config}/ikigai/tutorial.toml"
cargo run -p building-endpoints -- --banner
```

```text
hello from a file
```

Then write `yours.tutorial.toml` beside it with a different `text` and run again: the
app-scoped layer wins. Delete both and the default answers.

## The other channels

Command-line **flags** override the file; that is the second channel, and the two are the
ones this book uses. **Environment variables** are the awkward third: ambient influence
over a process, leaving no trace in any file, invisible to anyone reading the deployment,
inherited silently by children, undiffable. A system whose thesis is that behavior should
be nameable and inspectable is not comfortable taking instructions that way.

They are not absent, and a book that said so would be lying on your first day. The CLI
(`ikigai-cli` 0.1.18) reads a set of `IKIGAI_*` variables — `IKIGAI_FILES`, the one you
meet first, in [The file workspace](file-workspace.md), and `IKIGAI_GRANTS`,
`IKIGAI_SMTP_HOST`, `IKIGAI_PASSKEY_ORIGIN` and others carrying *deployment* facts. It
picks its scheduler through a three-way precedence, `decide(flag, config, env)`, and then
*says which one won*:

```bash
ikigai --plain -c 'source urn:kernel:scheduler'
```

```text
scheduler
  backend    single
  threads    1
  source     default
```

That `source` row is the shape of the argument rather than a settlement of it: what makes
an invisible channel expensive is that nothing afterwards can tell you it was used, and a
host that reports which channel decided has bought back the property the objection is
about. Configure what you build through the file and flags. If you find yourself wanting
an env var, treat that as a question worth raising rather than a pattern worth copying.

## Fail loud on missing configuration

> ⚠ The house rule, and the opposite of what most frameworks do: **something expected but
> unset must stop the program.** Not warn, not silently substitute, not carry on degraded.
> The loader above draws the line where it belongs — absent is legal, broken is not — and
> a required value with no default should refuse to start and say which key is missing.
> The worst version of getting this wrong is a service that blocks on a keychain prompt
> *before its first line of output*, so the log shows nothing at all: an empty log reads
> as "still starting" and means "blocked, forever". One `println!` before the call would
> have made it self-identifying.
