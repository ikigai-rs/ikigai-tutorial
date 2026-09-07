# Configuration

## Two channels, and a boundary still being drawn

Configuration reaches an ikigai host two ways: a **config file**, and **command-line
flags** that override it. Those are the channels this book uses, and the ones to reach for
when you configure something you built.

Environment variables are the awkward third, and it is worth knowing why they are awkward
before you meet one. An env var is ambient influence over a process's behaviour: it leaves
no trace in any file, is invisible to anyone reading the deployment, is inherited silently
by child processes, and cannot be diffed. A system whose thesis is that behaviour should be
*nameable and inspectable* is not comfortable taking instructions through a channel with
none of those properties.

They are not absent, though, and a book that told you they were would be lying to you on
your first day. A host reads a set of `IKIGAI_*` variables today — `IKIGAI_FILES`,
`IKIGAI_GRANTS`, `IKIGAI_SMTP_HOST`, `IKIGAI_PASSKEY_ORIGIN` and others — and they have a
shape: most carry a *deployment* fact (where this process's mail relay is, which origin its
passkeys are scoped to, where its grants file lives) rather than a behaviour switch for a
resource. `IKIGAI_FILES` is the one you meet first, in
[The file workspace](file-workspace.md).

And one of them is deliberate rather than residual. The CLI picks its scheduler through
`decide(flag, config, env)` — a precedence function with three arguments, written that way
on purpose — and then *says which one won*:

```bash
ikigai --plain -c 'source urn:kernel:scheduler'
```

```text
scheduler
  backend    single
  threads    1
  source     default
```

That `source` row is the interesting part, and it is the shape of the argument rather than
a settlement of it: what makes an invisible channel expensive is that nothing afterwards
can tell you it was used, and a host that reports which channel decided has bought back
the property the objection is about.

So: the boundary is being tidied, not settled, and this book is not where it gets settled.
Configure what you build through the config file and flags. If you find yourself wanting an
env var instead, treat that as a question worth raising rather than a pattern worth copying.

## Where files live

`ikigai-core` owns the answer, so every host agrees:

```rust,no_run
# extern crate ikigai_core;
use std::path::PathBuf;

let home: Option<PathBuf> = ikigai_core::config::config_home();
let candidates: Vec<PathBuf> = ikigai_core::config::layered_paths("cms", None);
# let _ = (home, candidates);
```

`config_home()` resolves the XDG config directory — `$XDG_CONFIG_HOME/ikigai`, else
`~/.config/ikigai`.

`layered_paths()` is the interesting one. It returns the *ordered list* of files that make
up one logical configuration, so a host can read shared defaults and let a more specific
file override them, rather than every crate inventing its own precedence rules.

In practice you will see files like `~/.config/ikigai/cms.toml`, one per host.

## Fail loud on missing configuration

> ⚠ The house rule, and it is the opposite of what most frameworks do: **something
> expected but unset must stop the program.** Not warn, not silently substitute a default,
> not carry on degraded.

A silent default is a lie the system tells itself — the operator believes one thing is
configured, the process is doing another, and nothing anywhere disagrees until an outage.
If a value is genuinely optional, model it as optional. If it is required, refuse to start
without it and say which key is missing.

## A worked example of getting this wrong

The reading room's own service binary reads its passkey store from the OS keychain
*before it prints its first line of output*. When that call blocks — which it does in any
context that cannot answer the keychain prompt — the log shows **nothing at all**. No
error, no partial banner.

An empty log reads as "still starting". It actually means "blocked, forever". One
`println!` before the call would have made it self-identifying. That is what failing loud
buys, and what its absence costs.
