# Checkout

A browse root has to be on disk before gonk starts: gonk refuses to start on a root that names
a missing directory, because that is the shape of a typo. On a new machine that means cloning
every repository and writing one `gonk.browse.root` line for each. `ikigai-gonk checkout`
does both, and keeps the clones current afterwards.

It is a command, like `roborev file`: it opens no store and binds no door, and the server
itself has no code that fetches anything. So this page does not start a gonk at all.

## Two repositories to check out

A real checkout clones from GitHub or another server. This page makes two repositories of its
own to stand in for them, and clones them by `file://` URL, which git treats like any other
remote:

<!-- transcript: run -->
```console
$ git init -q -b main upstream/notes
$ printf '# Notes\n' > upstream/notes/README.md
$ git -C upstream/notes add README.md
$ git -C upstream/notes -c user.name='A Reader' -c user.email=reader@example.com commit -qm 'Start the notes'
$ git init -q -b main upstream/specs
$ printf '# Specs\n' > upstream/specs/README.md
$ git -C upstream/specs add README.md
$ git -C upstream/specs -c user.name='A Reader' -c user.email=reader@example.com commit -qm 'Start the specs'
```

## Clone, and write the roots

<!-- transcript: run -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk checkout "file://$PWD/upstream/notes" "file://$PWD/upstream/specs"
cloned     notes at …  ~/.ikigai/checkouts/notes
cloned     specs at …  ~/.ikigai/checkouts/specs

browse roots:
  gonk.browse.root = "notes=~/.ikigai/checkouts/notes"
  gonk.browse.root = "specs=~/.ikigai/checkouts/specs"

not written: add the lines above to …/.config/ikigai/config.toml, or run again with --write-config
```

Each URL is cloned into `~/.ikigai/checkouts/<name>`, where `<name>` is the URL's last path
segment without `.git`, and it is also the browse root's name. `name=url` sets a different one
(`kata=git@github.com:kenn-io/kata.git`), and `--dir` puts the clones somewhere else.
(`HOME` is this page's directory again, so `~` here is the page's.)

Without `--write-config` the lines are only printed. With it, the missing ones are appended to
the config home's `config.toml`:

<!-- transcript: run -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk checkout "file://$PWD/upstream/notes" "file://$PWD/upstream/specs" --write-config
current    notes at …  ~/.ikigai/checkouts/notes
current    specs at …  ~/.ikigai/checkouts/specs

browse roots:
  gonk.browse.root = "notes=~/.ikigai/checkouts/notes"
  gonk.browse.root = "specs=~/.ikigai/checkouts/specs"

…/.config/ikigai/config.toml:
  added      gonk.browse.root = "notes=~/.ikigai/checkouts/notes"
  added      gonk.browse.root = "specs=~/.ikigai/checkouts/specs"

gonk reads gonk.browse.root at startup only: restart it to serve the new root(s). Under launchd:
  launchctl kickstart -k gui/$(id -u)/dev.ikigai-rs.gonk
$ cat .config/ikigai/config.toml
gonk.browse.root = "notes=~/.ikigai/checkouts/notes"
gonk.browse.root = "specs=~/.ikigai/checkouts/specs"
```

The second run found both clones already there and current, so it cloned nothing, and only
wrote the config. The rules for that file are conservative: when it exists, it is copied beside
itself before anything is appended (a `backup` line names the copy), no other line is touched,
a root already configured for the same directory is left alone, and a name already configured
for a **different** directory is a conflict that writes nothing for that name.

## Keeping them current

Upstream moves on:

<!-- transcript: run -->
```console
$ printf 'A second line.\n' >> upstream/notes/README.md
$ git -C upstream/notes -c user.name='A Reader' -c user.email=reader@example.com commit -qam 'A second line'
```

Run the same command again, and each clone is fetched and its default branch
**fast-forwarded**:

<!-- transcript: run -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk checkout "file://$PWD/upstream/notes" "file://$PWD/upstream/specs"
updated    notes …..… (1 commit(s), fast-forward)  ~/.ikigai/checkouts/notes
current    specs at …  ~/.ikigai/checkouts/specs

browse roots:
  gonk.browse.root = "notes=~/.ikigai/checkouts/notes"
  gonk.browse.root = "specs=~/.ikigai/checkouts/specs"

not written: add the lines above to …/.config/ikigai/config.toml, or run again with --write-config
```

A root gonk already serves needs no restart after an update: gonk watches it, and a
fast-forward is a change on disk like any other (the watcher is shown working in
[Browse roots](../browse/roots.md)). Only a NEW root needs the restart the first run asked for.

### Every checkout, without the URLs

Repeating the URL list is fine once and tiresome on a timer. `--all` updates every checkout
already in the managed directory, each from its own `origin` and under its own directory name.
To see what else it reports, check out a third repository and do not write its root:

<!-- transcript: run -->
```console
$ git init -q -b main upstream/tools
$ printf '# Tools\n' > upstream/tools/README.md
$ git -C upstream/tools add README.md
$ git -C upstream/tools -c user.name='A Reader' -c user.email=reader@example.com commit -qm 'Start the tools'
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk checkout "file://$PWD/upstream/tools"
cloned     tools at …  ~/.ikigai/checkouts/tools

browse roots:
  gonk.browse.root = "tools=~/.ikigai/checkouts/tools"

not written: add the lines above to …/.config/ikigai/config.toml, or run again with --write-config
$ printf 'A third line.\n' >> upstream/notes/README.md
$ git -C upstream/notes -c user.name='A Reader' -c user.email=reader@example.com commit -qam 'A third line'
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk checkout --all
updated    notes …..… (1 commit(s), fast-forward)  ~/.ikigai/checkouts/notes
current    specs at …  ~/.ikigai/checkouts/specs
current    tools at …  ~/.ikigai/checkouts/tools

coverage of ~/.ikigai/checkouts by the roots in …/.config/ikigai/config.toml:
  unused     tools  ~/.ikigai/checkouts/tools  no gonk.browse.root points into it: gonk does not browse it (`checkout <url> --write-config` adds its line)
```

Each checkout gets its line, in the same words as before, and then a **coverage** report that
changes nothing: `unused` for a checkout no `gonk.browse.root` points into (gonk does not
browse it, so it is fetched for nothing), and `missing` for a root that points into the managed
directory at nothing (gonk will refuse to start until it exists). Coverage is information; it
does not change the exit status.

What `--all` never does is as deliberate: it clones nothing, writes no config (`--all
--write-config` is refused), and takes no URLs. And a bare `checkout` with no URLs is an
error that names `--all`, rather than quietly being it, because a command that fetches every
repository should say so on its own line.

## And nothing else

A fast-forward is all it does. A checkout with local changes (untracked files included), on
another branch, or with commits upstream does not have is **refused and left exactly as it
was**: nothing is reset, stashed, merged or discarded.

<!-- transcript: run -->
```console
$ printf 'work in progress\n' > .ikigai/checkouts/specs/wip.txt
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk checkout --all
current    notes at …  ~/.ikigai/checkouts/notes
REFUSED    specs  ~/.ikigai/checkouts/specs
  it has local changes (1 path(s); `git -C …/.ikigai/checkouts/specs status`). A managed checkout is fast-forwarded only; nothing was fetched or changed
current    tools at …  ~/.ikigai/checkouts/tools

coverage of ~/.ikigai/checkouts by the roots in …/.config/ikigai/config.toml:
  unused     tools  ~/.ikigai/checkouts/tools  no gonk.browse.root points into it: gonk does not browse it (`checkout <url> --write-config` adds its line)
$ cat .ikigai/checkouts/specs/wip.txt
work in progress
```

Each checkout is reported on its own and the rest still run. Any refusal or failure makes the
exit status 1, so a scheduled run that something has stopped is visible to whatever ran it, and
a refusal names the command that shows what is in the way.

git runs with its standard input closed and `GIT_TERMINAL_PROMPT=0`, and ssh in `BatchMode`
unless you have already said how to run ssh, so a repository that wants a password fails with
git's own message instead of waiting for one. Credential helpers and ssh agents work as
usual.

⚠ A managed clone shows its **default branch**. Work happening on other branches or in other
worktrees (an agent loop's worktrees, say) is not what gonk browses there; point a root at that
worktree by hand if that is what you want to read.

## On a timer

Nothing in the server fetches, so a periodic update is `checkout --all` run by the system's
scheduler, and gonk needs no restart for it. On macOS, a `launchd` agent beside gonk's own
(`~/Library/LaunchAgents/dev.ikigai-rs.gonk-checkout.plist`), every fifteen minutes:

```xml
<plist version="1.0"><dict>
  <key>Label</key><string>dev.ikigai-rs.gonk-checkout</string>
  <key>ProgramArguments</key><array>
    <string>/bin/sh</string>
    <string>-c</string>
    <string>date; exec /Users/you/.cargo/bin/ikigai-gonk checkout --all</string>
  </array>
  <key>StartInterval</key><integer>900</integer>
  <key>StandardOutPath</key><string>/tmp/ikigai-gonk-checkout.log</string>
  <key>StandardErrorPath</key><string>/tmp/ikigai-gonk-checkout.log</string>
</dict></plist>
```

<!-- transcript: manual — it registers a launchd agent, which a scratch check must not -->
```console
$ launchctl bootstrap gui/$(id -u) ~/Library/LaunchAgents/dev.ikigai-rs.gonk-checkout.plist
```

On Linux, the same as a crontab line (`crontab -e`):

```text
*/15 * * * * (date; $HOME/.cargo/bin/ikigai-gonk checkout --all) >> /tmp/ikigai-gonk-checkout.log 2>&1
```

The `date` is there because the command prints no time of its own and neither scheduler adds
one. Under either, git runs without a terminal: an HTTPS remote needs a credential helper and
an SSH remote a key the agent already holds, or that checkout is a `FAILED` line with git's
own message rather than a hang.
