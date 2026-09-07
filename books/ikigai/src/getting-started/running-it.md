# Running it

Read a chapter, then have a kernel in front of you. Three ways, cheapest first.

## The tutorial binary (this repo, nothing else)

```bash
cargo run -p hello-camel -- "resource oriented computing"
```

```text
in  resource oriented computing
out resourceOrientedComputing
```

That is a complete ikigai host: a root space, a kernel around it, one resolution. It is
about thirty lines and you will have read all of them by the end of
[chapter 5](binding.md).

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
(`..`), named arguments, `compose`, `cache`, `cap`, and `list`. The REPL, the one-shot
flag and the page-assembling browser demo all drive *the same engine* — worth knowing
early, because anything you learn in one place transfers.

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

It is also the only host that loads **modules** rather than linking everything in — see
[chapter 8](../modules/index.md).
