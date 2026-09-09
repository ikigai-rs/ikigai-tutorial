# Running it

Read a chapter, then have a kernel in front of you. Four ways, cheapest first.

## In this page

The published book carries its own kernel: Part I's space — `camel-case` and the endpoints
[What resolution buys you](payoff.md) adds — compiled to WebAssembly, under the same engine
the CLI uses. Where a chapter has a **Run** button, the lines beside it are real REPL lines
and the answer is a real resolution, in your browser, with nothing installed:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:camel-case in="resource oriented computing"'>
<pre class="ikigai-run-expected">resourceOrientedComputing
[computed]</pre>
</div>

Nothing runs until you press Run; until then a cell shows the output its listing
produces, marked as expected. One kernel serves the whole page and keeps its cache
between runs, so a second press answers `[cached]`, and every run is kept under the cell.
The command is yours to edit — Reset brings the chapter's back. The kernel binds Part I's
names and nothing else — the other parts' hosts, and your own crate, are not in the page
— and if it fails to load, a cell keeps the expected output and says so.

## The tutorial binary (this repo, nothing else)

```bash
cargo run -p hello-camel -- "resource oriented computing"
cargo run -p hello-camel -- --catalog
```

```text
in  resource oriented computing
out resourceOrientedComputing
```

That is a complete ikigai host: a root space, a kernel around it, one resolution. It is
about thirty lines and you will have read all of them by the end of
[Binding, and a host of your own](binding.md). The second form prints the host's catalog
— every endpoint it binds, describing itself — which [What resolution buys you](payoff.md)
reads through the same kernel.

## The CLI

The full host is the [`ikigai-cli`](https://crates.io/crates/ikigai-cli) crate, which
installs a binary called `ikigai`:

```bash
cargo install ikigai-cli --locked
```

> ⚠ The crate is `ikigai-cli`. Only the **binary** is called `ikigai` — and `cargo install
> ikigai` does not fail, it fetches an unrelated crate by another author and leaves you
> with something that is not this. `--locked` builds against the dependency versions the
> release was tested with rather than whatever the registry resolves today.

One-shot resolutions take `-c`, and `--plain` drops the decoration so output is pipeable:

```bash
ikigai --plain -c 'source urn:iki:fn:toUpper in="hello"'
```

Run it with no arguments and you get a REPL with the same grammar: pipes (`|`), map
(`..`), named arguments, `compose`, `cache`, `cap`, `trace`, and `list`. The REPL, the
one-shot flag and the page-assembling browser demo all drive *the same engine* — worth
knowing early, because anything you learn in one place transfers. One difference that
matters: the cache lives in the process, so a cache hit is something you see in a REPL
session and never across two `-c` runs.

Two commands worth running on your first day, because they show the system describing
itself rather than doing work:

```bash
ikigai --plain -c 'source urn:kernel:catalog'
ikigai --plain -c 'source urn:kernel:actions'
```

The first is everything resolvable. The second is everything *you* may invoke, given the
capability you are holding — the same list an agent would be handed.

## The browser demo

<https://ikigai-rs.github.io/ikigai-web-demo/> runs the kernel as WebAssembly, in the
page, with no server: the whole page is one resource that the in-browser kernel composed.
The **Control** tab shows the scheduler, the cache and its golden threads updating live;
the **Demo** tab is a set of runnable walkthroughs.

It is also the only host that loads **modules** rather than linking everything in (as of
2026-09-08) — see [What a module is](../modules/index.md).
