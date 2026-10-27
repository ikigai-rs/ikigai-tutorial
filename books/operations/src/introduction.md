# Introduction

This is the operator's book. You have the `ikigai` binary, or are about to, and you want to drive
it well: run it at the prompt and in scripts, serve a kernel to other processes and other
machines, decide who may call it and with what authority, put an HTTP face on it, leave it running
unattended, watch it, and fix it when it is wrong. Each chapter takes one of those jobs and shows
the commands that do it, with what they print.

It does not teach resources. What a resource is, how a kernel resolves one, what a capability or a
golden thread is: [The ikigai Book](../index.html) teaches all of that from the first endpoint up,
and this book links to it rather than saying it twice. gonk, the work ledger and review host, has
[a book of its own](../gonk/index.html), doors and QUIC client included; [The ikigai
Architecture](../architecture/index.html) is where the pieces are shown fitting together. Here the
subject is the binary, its flags and subcommands, and what it takes to run kernels for real.

## The binary this book was tested against

Every command in this book was run against one release of `ikigai-cli`, built with its `quic` and
`web` features:

<!-- transcript: manual — it compiles the whole binary from crates.io, minutes of build and a
     network fetch; the check installs exactly this release itself, the same way, before it
     runs anything else in the book. -->
```console
$ cargo install --locked ikigai-cli@{{#include ../ikigai-cli.version}} --features quic,web
```

The crate is `ikigai-cli`; the binary it installs is `ikigai`. `--locked` builds the dependency
versions the release was tested with, rather than whatever the registry resolves today; without
it, an install can link versions nobody has tested together. The two features add the QUIC
transport (`--connect quic://…`, `serve quic://…`, certificates) and the HTTP doors (`serve
--http`); the default build has neither, and its `--help` lists their flags anyway, so their
absence only shows when a command that needs them fails.

Ask the binary which release it is:

<!-- transcript: run -->
```console
$ ikigai --version
ikigai {{#include ../ikigai-cli.version}}
```

If yours prints another version, most of this book still applies, and where a release changed
something a chapter shows, the chapter's output is the release above's. [What's new](whats-new.md)
says when the book moves to a newer one.

## Every command here is run

A block like the one above is a **transcript**: the lines that start with `$ ` are what you type,
and the lines under each are what it prints. None of them is written by hand. Whenever the book is
built, a check installs the release above, runs every transcript in a fresh, empty directory of its
own, and fails the build if a command prints anything other than what its page says. So when a
release changes what a command prints, this book cannot go on showing the old output: the build
that would publish it fails first.

A few things legitimately differ between that run and yours, and the check is told exactly which,
block by block, rather than told to look away:

- **Your working directory.** Commands that need a file or a socket put it in the directory you
  run them from, and output that names that directory shows it as `$PWD`.
- **Your home directory**, shown as `~`.
- **A port a server chose.** A server started on port 0 picks a free one and prints it. The book
  shows the one it picked when the page was written; yours will differ, and the next command
  should use yours. (The check does the same: it reads the port from the server's first lines and
  uses it.)
- **Timestamps, durations and certificate fingerprints**, which are never the same twice.
- **A version number** in a line that is about something else.

A command that cannot run in that check (it needs a person at a terminal, a Mac, or the network)
is marked on its page, which says in one line why. The check counts those too, so the share of
this book that is not run is reported on every build.

Some chapters also have **Run** buttons: the lines beside one are real REPL lines, answered by a
kernel compiled into the page, with nothing installed. [The REPL and its
modes](binary/repl.md) is the first.

## Where to start

Read [The REPL and its modes](binary/repl.md) first: every later chapter drives the binary the ways
it shows. The other chapters are listed in the table of contents as they will be written; the ones
not yet written are shown there without a link.
