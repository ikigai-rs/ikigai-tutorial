# The aliases, day to day

[The editor as a client](emacs.md) explained why the generated aliases exist and why they
cannot drift. This page is the other half: how you get them, how you find out what you
have, how you call them, and when to regenerate.

Every transcript here is from one scratch run: `ikigai-cli` 0.1.41, `ikigai-emacs` at
`6f03b98`, Emacs 30.2 started with `-Q`, and an empty config home, so no mounts. Your
counts will differ, because they depend on what your kernel offers you, and that is the
point of the whole exercise.

## Setting up

`ikigai.el` needs the `ikigai` binary on `exec-path` and nothing else. Three lines in your
init file:

```elisp
(add-to-list 'load-path "~/src/ikigai-emacs")   ; wherever you cloned it
(require 'ikigai)
(load ikigai-aliases-file t t)                  ; the last refresh, if there has been one
```

`ikigai-aliases-file` is where the generated file lives. It defaults to
`ikigai-aliases.el` in your Emacs directory; customize it if you want the file somewhere
else. The `load` line passes `noerror`, so a machine that has never refreshed starts
cleanly, and every later session gets its functions without asking the kernel again.

The first time, run `M-x ikigai-refresh-aliases`. It resolves the alias resource, writes
the file, loads it, and says how many functions it defined:

```text
ikigai: 129 aliases loaded from ~/.emacs.d/ikigai-aliases.el
```

The top of the file it wrote:

```elisp
;;; -*- lexical-binding: t -*-
;;; Generated from an ikigai kernel's manifold — do not edit.
;;;
;;; One function per resource this capability may invoke, with the arguments that
;;; resource declares. Regenerate with:
;;;   ikigai -c 'source urn:lisp:aliases as=text/x-emacs-lisp' < /dev/null
;;;
;;; Transport, mounts and quoting come from ikigai.el.

(require 'ikigai)
```

and one of its 129 definitions:

```elisp
(defun ikigai-fn-toUpper (in &rest args)
  "Upper-cases the UTF-8 text supplied in the `in` argument."
  (apply #'ikigai-invoke 'source "urn:iki:fn:toUpper" "in" in args))
```

Each alias is that same shape: a docstring and one call to `ikigai-invoke`. So an alias
goes wherever every other call from `ikigai.el` goes: through `ikigai-connect` when it is
set, otherwise through an embedded kernel composed with `ikigai-mounts`.

## Finding what you have

### From a name to a function

The function name is the resource's name, made into an elisp symbol by fixed rules:

1. Drop the prefix: `urn:iki:` if the name has it, otherwise `urn:`.
2. Replace every character that is not an ASCII letter, digit or hyphen with `-`. Case is
   kept, so `toUpper` stays `toUpper`.
3. Mark the verb: Source is bare, Sink adds `!`, Exists adds `?`, Delete adds `-delete!`.
4. Put `ikigai-` in front. The few names `ikigai.el` already defines (`ikigai-eval`,
   `ikigai-invoke`, `ikigai-repl` and the like) get `-resource` added, so a resource can
   never replace the function every alias calls through.

From the file above:

| Resource | Verb | Function |
|---|---|---|
| `urn:iki:fn:toUpper` | Source | `ikigai-fn-toUpper` |
| `urn:text:grep` | Source | `ikigai-text-grep` |
| `urn:host:history` | Source and Sink | `ikigai-host-history`, `ikigai-host-history!` |
| `urn:httpHead` | Exists | `ikigai-httpHead?` |
| `urn:httpDelete` | Delete | `ikigai-httpDelete-delete!` |

The first row is also why `urn:fn:toUpper` and `urn:iki:fn:toUpper` give one function and
not two. The catalog lists only the canonical name, and dropping `urn:iki:` means the
function keeps its name while the namespaces move under `urn:iki:`.

### Looking them up

The aliases are ordinary functions. **They are not commands**: no generated definition is
`interactive`, so `M-x` does not offer them. `M-x ikigai-` completes to the package's own
commands and stops there:

```text
ikigai-eval-dwim  ikigai-eval-elisp-dwim  ikigai-org-email-invite
ikigai-org-schedule-zoom  ikigai-refresh-aliases  ikigai-repl  ikigai-repl-mode
```

To browse the aliases, use the help commands that work on functions. `C-h f ikigai-text-`
then `TAB` completes over every alias in that family, and `M-x apropos` with `^ikigai-text-`
lists them with their docstrings:

```text
ikigai-text-grep
  Function: Keeps the input lines that contain ‘pattern‘ (a literal
            substring, not a regex). ‘i=true‘ matches case-insensitively;
            ‘v=true‘ inverts the match (keep non-matching lines).
ikigai-text-head
  Function: Returns the first ‘n‘ lines of the input (default 10).
ikigai-text-nl
  Function: Numbers every input line from 1: a right-aligned (6-wide) number,
            a tab, then the line.
```

`C-h f ikigai-text-head` shows one in full:

```text
ikigai-text-head is a interpreted-function in
‘~/.emacs.d/ikigai-aliases.el’.

(ikigai-text-head IN &rest ARGS)

Returns the first ‘n‘ lines of the input (default 10).
```

Read the two parts of that for what each one tells you, because they come from different
places. The **argument list** is the endpoint's declared contract: `IN` is there because
`urn:text:head` declares a required input called `in`. The **docstring** is the first line
of the endpoint's own summary, copied in, and nothing more. It is not generated from the
arguments, so an optional argument appears there only when the endpoint's author mentioned
it in the summary: `n` above, `i` and `v` for grep. For the full contract, ask the
resource itself: `describe urn:text:head`, run with `ikigai-eval-dwim` (below), prints
every declared argument with its type and whether it is required, which is how you learn
that `n` is an optional integer that defaults to 10.

### Whose functions these are

The file is generated from `urn:kernel:actions`, the manifold *under the capability you
called with*, so two people refreshing against the same kernel can get different files.
The narrowing is easiest to see from the shell, where `cap` narrows a session:

```bash
ikigai -c 'cap urn:cap:kernel:inspect' -c 'source urn:lisp:aliases as=text/x-emacs-lisp'
```

Holding only the right to inspect, the same kernel gave 78 functions instead of 129. The
51 that went are the resources that need some other grant: the repository facades, the
HTTP verbs, the language models, the secret store and the scheduler among them. Narrow further, below
the inspection right itself, and there is no file at all:

```text
narrowed — capability: urn:cap:fn
error: denied: capability does not grant `urn:cap:kernel:inspect` (declared by `urn:lisp:aliases`)
```

`ikigai.el` has no command to narrow a session, so from Emacs the capability is whatever
the kernel you reach gives you. Change which kernel that is, and refresh: a different
`ikigai-connect` host gives that host's surface. A mount in `ikigai-mounts` adds the
mounted kernel's resources under the mount's prefix, so mounting a peer at `urn:narrow:`
gave functions like `ikigai-narrow-kernel-catalog`.

<!-- urn-gate: illustration urn:narrow: — a mount prefix from the scratch run, not a name the CLI binds. -->

> ⚠ As of `ikigai-cli` 0.1.41, a mount also *removes* functions. When the mounted peer
> binds an endpoint the local kernel also binds (both had `toUpper` and the kernel's own
> resources), the generated file keeps only the mounted copy, so in that run
> `ikigai-fn-toUpper` was gone and `ikigai-narrow-iki-fn-toUpper` took its place. Check
> the count after you refresh with a new mount.

## Calling them

### From Lisp, or from `M-:`

An alias is a function, so you call it as one: from your own code, from `M-:`
(`eval-expression`), or from a scratch buffer. The value is a string, the resource's
representation with surrounding whitespace trimmed, so `M-:` shows it in the echo area:

```elisp
(ikigai-fn-toUpper "hello, resource")
⇒ "HELLO, RESOURCE"
```

### Arguments

The **required** arguments are positional, in the order the endpoint declares them:

```elisp
(ikigai-text-grep "apple\nbanana\ncherry" "an")
⇒ "banana"
```

Everything else goes after them as **name and value pairs, both strings**. This is how you
reach an optional argument, `as=` for a different representation, or the content of a Sink:

```elisp
(ikigai-text-head "one\ntwo\nthree" "n" "2")
⇒ "one
two"

(ikigai-host-history! "content" "on")
⇒ "history on (0 entries)"
```

The pairs are not keyword arguments, and the values are not converted for you. Both of
these fail in Emacs, before the kernel is asked:

```elisp
(ikigai-text-head "one\ntwo\nthree" :n "2")    ; signals (wrong-type-argument sequencep :n)
(ikigai-text-head "one\ntwo\nthree" "n" 2)     ; signals (wrong-type-argument sequencep 2)
```

And spell the names exactly. A pair whose name the endpoint does not declare is ignored
on this path rather than refused: `(ikigai-text-head "one\ntwo\nthree" "lines" "2")`
returns all three lines.

### Where the result goes

The return value is all you get: nothing is written to a buffer. Two commands in
`ikigai.el` do the rest.

- **`ikigai-eval-elisp-dwim`** evaluates the Lisp form before point (or the region),
  shows the result, and puts it on the kill ring, so `C-y` pastes it. With `C-u` it also
  inserts the result after point. This is the one to bind for alias calls. A buffer
  holding the grep call above, after `C-u M-x ikigai-eval-elisp-dwim` with point at the
  end of the form:

  ```text
  (ikigai-text-grep "apple\nbanana\ncherry" "an")
  banana
  ```

- **`ikigai-eval-dwim`** is its sibling for the REPL grammar instead of Lisp: it runs the
  region or the current line as an `ikigai` command, so a line reading
  `source urn:fn:toUpper in=hello` shows `HELLO  (copied)`. Reach for it when you want a
  resource there is no alias for, or a `describe`.

### When it fails

A failure is a `user-error` whose message is the kernel's own error, prefixed with
`ikigai:`. A name nothing binds:

```elisp
(ikigai-invoke 'source "urn:fn:nothing-here")
;; signals (user-error "ikigai: error: no endpoint resolved for urn:iki:fn:nothing-here")
```

<!-- urn-gate: unbound urn:fn:nothing-here — a deliberately missing name, to show how NotFound reads in Emacs. -->
<!-- urn-gate: unbound urn:iki:fn:nothing-here — the canonical spelling of the same missing name, as the error reports it. -->

A refusal reads the same way. Here one resource was routed by an override mount to a
second kernel on the same machine, serving under a ceiling of `urn:cap:kernel:inspect`
alone (`ikigai serve quic://127.0.0.1:47946 --cap urn:cap:kernel:inspect`):

```elisp
(setq ikigai-mounts
      '((:prefix "urn:host:posture" :target "quic://127.0.0.1:47946" :mode override)))

(ikigai-host-posture)
;; signals (user-error "ikigai: error: denied: capability does not grant `urn:cap:host:posture` (declared by `urn:host:posture`)")
```

That example also shows the limit of "a narrowed session sees fewer functions". The file
describes the kernel you generated it against; an override sends one call somewhere else,
and the function stays in the file because the local kernel would have answered it.

Two failures never reach the kernel at all: a value that is not a string (above), and a
required argument left out, which Emacs reports as `wrong-number-of-arguments`.

## Keeping them current

The file is a snapshot of one manifold at one moment. Run `M-x ikigai-refresh-aliases`
again when that manifold changes:

- you changed `ikigai-connect` or `ikigai-mounts`;
- your grant changed, wider or narrower;
- you installed a new `ikigai`, or a module changed what it binds.

A refresh does not *remove* a function from the running Emacs. `load` defines what is in
the new file and leaves alone what is not, so an alias that has gone from the file stays
defined until you restart Emacs (or `fmakunbound` it), and calling it still sends the
request to the kernel. And the refresh message counts only what was defined.

What it did not define is in the file as a comment. A name is refused when it would not
read back as an elisp symbol or, the case you are likeliest to meet, when two resources
project to the same function. The scratch run below bound two endpoints at
`urn:text:head` and `urn:text.head` with a small `--arrangement` file. Both project to
`ikigai-text-head`. The file defines the first in sorted order and records the second:

<!-- urn-gate: illustration urn:text.head — bound only by the scratch arrangement in this example, to force two names onto one function. -->

```elisp
(defun ikigai-text-head (in &rest args)
  "Returns the last `n` lines of the input (default 10)."
  (apply #'ikigai-invoke 'source "urn:text.head" "in" in args))

;; SKIPPED urn:text:head (Source): `ikigai-text-head` is already defined above — two IRIs, one name.
;; Still reachable as (ikigai-invoke 'source "urn:text:head").
```

Nothing is lost: the comment gives the generic call that still reaches the resource. But
the refresh message does not mention it, so after a refresh that changed your topology,
search the file for `SKIPPED`.

Long-lived sessions, org-babel scripting and an editor workbench over the kernel are
material for a developer-oriented book, not this one.
