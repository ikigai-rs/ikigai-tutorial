# Configuration

## Two channels, and a third that is banned

Configuration reaches an ikigai host two ways: a **config file**, and **command-line
flags** that override it. That is the complete list.

**Environment variables are deliberately not a configuration channel.** Not "discouraged"
— excluded. The reasoning is worth understanding because it will look like an
inconvenience before it looks like a decision: an env var is ambient authority over a
process's behaviour that leaves no trace in any file, is invisible to anyone reading the
deployment, is inherited silently by child processes, and cannot be diffed. A system whose
whole thesis is that behaviour should be *nameable and inspectable* cannot then take its
instructions from an invisible channel.

(You will find `IKIGAI_FILES` in [chapter 7](ch07-file-workspace.md) and think you have
caught a contradiction. It is a *path root* for a sandbox, set once by the operator, not a
behaviour switch — but it is the honest edge of the rule and worth knowing it exists.)

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
