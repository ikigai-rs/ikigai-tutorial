# A week of earthquakes

In [Twenty minutes with the CLI](../front-door/cli.md), you used the `ikigai` command
to read resources and inspect cached answers. [The REPL grammar](../front-door/grammar.md)
introduced pipes, maps and forks. This part uses those tools to explore a set of data,
without writing Rust. You will use the command, the Run buttons on these pages, and files:
a recorded download, a jq program, a template.

It follows one set of data the whole way through: the U.S. Geological Survey's earthquake
feeds, which publish every earthquake on Earth as GeoJSON, with no key and no account. We
call it **Quakes**. Two copies of it run through the part:

- **A recorded week**: the 178 earthquakes of magnitude 4.5 and up between 2024-01-01 and
  2024-01-08, the largest the M7.5 Noto Peninsula earthquake in Japan on New Year's Day.
  It never changes, so every answer the book prints from it is the answer you get. The
  kernel in this page holds a copy, at `urn:file:quakes-2024-01-01.geojson`, so its Run
  buttons work with nothing installed. [Composites, one layer at a
  time](composites.md) adds the week after it, recorded the same way, at
  `urn:file:quakes-2024-01-08.geojson`.
- **The live feed**: the same kind of document for the past seven days, which USGS updates
  every minute. Only the `ikigai` command can reach it, because this page cannot reach the
  network.

The data is USGS's, and so are the names of the earthquakes in it: each one has its own
page at `earthquake.usgs.gov`. Any name this part makes up for itself goes under
`urn:quakes:`, never under USGS's domain.

## What you will do

- [A lab with names in it](lab.md): ask the recorded week questions in the REPL's grammar
  (a pipe, a map, a fork), and watch the kernel describe, trace and cache what you asked.
- [Somebody else's resource](http.md): read the live feed through `urn:http:get`, see how
  long USGS says an answer stays fresh, and narrow what the session may reach to one host.
- [Reshaping JSON with jq](jq.md): ask sharper questions with `urn:json:query`, jq as a
  resource, and fill a template with the answers.
- [Giving the data names](names.md): give the feed's keys published meanings with a JSON-LD
  context, keep USGS's own address for each earthquake as its name, and ask the result
  questions in SPARQL, each question a resource.
- [Composites, one layer at a time](composites.md): build answers from answers, a filter,
  a sort, a grouping and a join of two weeks with notes of your own, each a name the next
  reads; see what a cut recomputes, and what joining the live feed costs.
- [Process scripts are files](scripts.md): keep REPL lines in a file and replay them, store
  a pipeline as a plan, ask what authority it needs before it runs, and write a script with a
  script.
- [A river of feeds](rss.md): turn a directory of RSS feeds into one graph with XSLT, read
  feeds by their publishers' rules, and keep questions about it, and the pages they make, as
  files.
- [Naming a composition](naming.md): bind a request with its arguments to a name, so the
  composites and the river's questions are asked by name, and see what a name fixes, what it
  follows, who may change it and when a name is somebody else's.
- [When a rule is not enough](beyond-rules.md): write a Lisp program, as a file, that decides
  which weeks to read, see it cached and recomputed when a file it read changes, and see what
  its sandbox can reach; then mount a few lines of Python as a peer for what the sandbox cannot
  do, and see what changes when the answer comes from another process.

The HTTP-view step is still a gap in this part: the chapters build HTML answers and save
them as files, but do not yet show how to serve those answers over HTTP. The walkthrough
stops at the saved page for that step.

## What you need

For the Run buttons, nothing. For the shell steps, use `ikigai-cli`
{{#include ../../ikigai-cli.version}}, the release the book is checked against. If you
installed it in [Twenty minutes with the CLI](../front-door/cli.md), check your version
with `ikigai --version`; otherwise, that chapter has the installation steps. To install
the version used here:

```bash
cargo install ikigai-cli --locked --version '={{#include ../../ikigai-cli.version}}'
```

The shell steps over the recorded week are replayed against that release every time the
book changes. The steps over the live feed cannot be, because the feed moves; each page
says which release printed them, and when.

If you still have a REPL open from [Twenty minutes with the CLI](../front-door/cli.md),
quit it first: it is using the default workspace, `~/.ikigai/workspace/`.

Work in a directory of your own, so nothing here touches the CLI's default workspace:

```bash
mkdir -p ~/quakes
cd ~/quakes
curl -sO https://raw.githubusercontent.com/ikigai-rs/ikigai-tutorial/main/crates/quakes/quakes-2024-01-01.geojson
ikigai --plain --files ~/quakes
```

`--files` makes `~/quakes` the file workspace, so `urn:file:quakes-2024-01-01.geojson`
names the file you just downloaded, as it does in this page.
