# A week of earthquakes

The parts before this one wrote endpoints in Rust. This part writes none. Everything in it
is done with the `ikigai` command, the Run buttons on these pages, and files: a recorded
download, a jq program, a template. If you can use a shell, `curl` and `jq`, you have what
it needs.

It follows one set of data the whole way through: the U.S. Geological Survey's earthquake
feeds, which publish every earthquake on Earth as GeoJSON, with no key and no account. We
call it **Quakes**. Two copies of it run through the part:

- **A recorded week**: the 178 earthquakes of magnitude 4.5 and up between 2024-01-01 and
  2024-01-08, the largest the M7.5 Noto Peninsula earthquake in Japan on New Year's Day.
  It never changes, so every answer the book prints from it is the answer you get. The
  kernel in this page holds a copy, at `urn:file:quakes-2024-01-01.geojson`, so its Run
  buttons work with nothing installed.
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

Later in this part, the data gets names of its own, questions become files, and a page
built from them is served over HTTP.

## What you need

For the Run buttons, nothing. For the shell steps, the `ikigai` command at 0.2.4 or later
(`cargo install ikigai-cli --locked`; [Twenty minutes, nothing compiled](../front-door/cli.md)
has the details), because 0.2.4 is the first release whose command can run jq. Every
shell step in this part was run against `ikigai-cli` 0.2.4.

Work in a directory of your own, so nothing here touches the CLI's default workspace:

```bash
mkdir -p ~/quakes
cd ~/quakes
curl -sO https://raw.githubusercontent.com/ikigai-rs/ikigai-tutorial/main/crates/quakes/quakes-2024-01-01.geojson
ikigai --plain --files ~/quakes
```

`--files` makes `~/quakes` the file workspace, so `urn:file:quakes-2024-01-01.geojson`
names the file you just downloaded, as it does in this page. If you still have a REPL open
from the front door, quit it first: it is using the default workspace,
`~/.ikigai/workspace/`.
