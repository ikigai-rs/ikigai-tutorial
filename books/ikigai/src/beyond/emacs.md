# The editor as a client

Three chapters of plumbing, and the reasonable question at the end of them is who
benefits. This chapter and the next are the two answers, and they are the same answer twice:
a client that reads the manifold gets a command surface it did not have to write.

Start with the human one. `ikigai-emacs` is a thin client — it shells out to the `ikigai`
binary and does nothing heavier — and its most interesting command is a dozen lines long, half of
which are the message it prints afterwards.

## One function per resource you may call

`M-x ikigai-refresh-aliases` writes a file of elisp and loads it. What is in the file is
**one function per resource this capability may invoke, with the arguments that resource
declares**. Not a hand-written wrapper library: a projection of the live manifold, so
`(ikigai-fn-toUpper "hi")` exists because an endpoint says it takes an argument called
`in`, and it will keep existing in that shape for exactly as long as the endpoint does.

The whole of it is a resolution:

```bash
ikigai -c 'source urn:lisp:aliases as=text/x-emacs-lisp'
```

Read that line for what it is not. There is no code-generation tool, no schema file, no
build step, and nothing in the editor that knows what a calendar or a repository is. There
is a resource whose representation happens to be a program, and `as=` picks which language
it comes out in — the same projection has a Scheme face for the kernel's own Lisp. Two
representations of one resource, which is the oldest idea in this book applied to
tooling.

## Why it cannot drift

The generator does not read a config file listing what to emit. It resolves
`urn:kernel:actions` — the capability-scoped manifold, already narrowed to what the caller
may invoke — and joins it against `urn:kernel:catalog`, which knows each action's declared
arguments. Every function in the output is therefore backed by an endpoint's own
`Description`, the same one [Why an endpoint describes itself](../getting-started/self-description.md)
introduced.

Three consequences, and the third is the one that makes this more than convenience:

- A newly bound endpoint gets a function the next time you run the command. Nobody
  maintains a list.
- The functions take the arguments the endpoint declares, positionally for the required
  ones, with the optional ones still reachable — so a shortcut never narrows what the
  endpoint can do.
- **A scoped session gets a smaller file.** Capability filtering is not a filter bolted on
  top of the projection; it is the surface the kernel says you have. Reading the manifold
  is itself an act of inspection, which is why the command requires
  `urn:cap:kernel:inspect` — and why a narrowed session sees fewer functions rather than
  functions that fail when called.

## A refused name is a comment, not a gap

Projecting a URI into an identifier can fail. `urn:` names are not elisp symbols, and the
generator will not emit something it has not verified *reads* in the target language — not
"contains only legal characters", which proves nothing, but parses.

When it cannot, it emits a comment saying which name was skipped, why, and the generic call
that still reaches the resource. That choice is worth stealing. This file is `load`ed, and
`load` is all-or-nothing: one unreadable form takes the entire generated surface down with
it. A refused alias costs a shortcut; a refused *file* costs the user their editor
integration, and they will blame the editor.

It is the same rule the projection to agent tools follows, and the next chapter meets it
again: **certify the output, not the input.**

## The topology is a `defcustom`

The other half of the package is where the previous chapter shows up in a user's config.
`ikigai-mounts` is an ordinary Emacs customization variable holding a list of mounts —
prefix, target, and which of the three kinds — so a user writes their own topology in
their own init file, and every command the package runs composes it.

Sitting beside it is `ikigai-connect`, an IPC socket path, and the two are deliberately
mutually exclusive: when it is set, the package passes `--connect` and no mounts at all,
because **a connected host owns its own mounts**. The reasoning in that docstring is worth
lifting out, because it is a good architectural instinct in one sentence: a machine's
transport and topology are a property of *that machine*, so putting them in the host's
config rather than the editor's leaves one Emacs configuration that is identical
everywhere.

There is a second reason on macOS, and it is the practical one. Calendar access is granted
to the *launching application*, so a kernel spawned by Emacs is a different principal from
the daemon that holds the grant. Routing `urn:personal:` through that daemon is not an
optimization; it is the only way the resource resolves at all. A mount is doing something
here that no amount of local configuration could.

<!-- urn-gate: illustration urn:personal: — the calendar family on one operator's machine, named here to explain a real mount, not offered as a name a reader can resolve. -->

## What is genuinely small about this

It is worth being clear that the package is not much code. It runs a subprocess, it quotes
its arguments, it keeps cache tags out of stdout, and it loads a generated file. Every
interesting thing in this chapter is happening on the other side of that subprocess.

That is the claim, though: the client is small **because** the manifold is machine-legible.
An editor integration that has to know what your system offers is a project; one that asks
is an afternoon.
