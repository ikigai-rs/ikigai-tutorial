# Appendix: the kernel in the browser

Every Run button in this book resolves in a real ikigai kernel. It is compiled to
WebAssembly and runs in your browser tab, under the same engine as the `ikigai` command.
This appendix covers how that kernel is put together. It compares what the browser gives a
kernel with what a native host gives one, and explains why the same crates serve both.

It assumes [Binding, and a host of your own](getting-started/binding.md): a host is the
code that builds a kernel, binds its spaces and hands it what it needs. The page is one
more host.

## The kernel brings nothing from the platform

`ikigai-core` reads no clock, opens no socket, starts no thread and touches no file.
Everything that comes from outside the program comes from the host that built the kernel,
through a seam the host fills:

- a space to resolve against (`Kernel::new`);
- a Meta renderer, so `describe` and the catalog answer (`Kernel::with_meta_renderer`);
- a clock, so the kernel can say what time it is (`Kernel::with_clock`).

The kernel uses only what it was handed. That is what lets one kernel crate run in a
command-line tool, in a long-running service, under a test that stops time, and in a web
page.

## How the page builds its kernel

The page's host is the `book-wasm` crate in this repository. The book's publishing
workflow compiles it for `wasm32-unknown-unknown` and binds it with
`wasm-bindgen --target web`. The output lands beside the book in `wasm/` and is never
committed. The first time a page needs it, `js/run.js` loads that module, and the module
builds the page's kernel:

```rust,ignore
{{#include ../../../crates/book-wasm/src/lib.rs:page_kernel_over}}
```

A few things follow from this:

- **One kernel per page.** Every cell on a page runs against the same kernel, so a value
  cached by one cell is cached for the next. Reloading the page builds a new kernel, so
  nothing you did on one page is on the next.
- **The same spaces the chapters test.** The page's space is Part I's, the tic-tac-toe
  game's, the spreadsheet's and the rest, in one fallback. These are the same spaces, with
  the same bindings and renderer, that the chapters' Rust tests run against. Nothing is
  mocked and nothing is replayed.
- **The CLI's engine.** A cell's line goes through `ikigai-engine`, the REPL grammar the
  `ikigai` command uses, so `source a | b`, `cache`, `trace` and `describe` mean exactly
  what they mean at a shell. The engine's `eval_async` runs the line on the page's event
  loop rather than blocking it.

## What the browser provides, and what a native host provides

A browser is a capable host, but a different one. The table compares the two for each thing
a kernel or its endpoints might need:

| What it needs | In the browser | On a native host | How the kernel gets it |
| --- | --- | --- | --- |
| The time | `Date.now()` | the system clock | `Kernel::with_clock`; an endpoint asks `Invocation::now` |
| A wake-up later (timers) | `setTimeout`, `setInterval` | threads and native timers | a timer the host injects |
| Running async work | the page's event loop | an executor on threads | the host drives the kernel's futures |
| The network | `fetch`, WebSocket, WebTransport, under the page's origin rules | TCP, UDP, Unix sockets, QUIC | an endpoint the host binds |
| Files | none, beyond what the page fetches | the filesystem | a space the host binds (`ikigai-fs` natively) |
| Keeping data | the page's memory, until a reload | databases and files on disk | a store the host binds |
| Threads | one for the page's kernel (the book uses no Web Workers) | many | none needed: the kernel is async |
| Other programs | none | processes (`git`, `cargo`) | an endpoint the host binds |
| Platform services | none | the keychain, calendars, contacts | an endpoint the host binds |

None of the right-hand column is something the kernel asks the platform for. A browser
host and a native host fill the same seams, each from what it has. Where the browser has
nothing (files, processes, the keychain), the page binds no space for it, and a name in
those families is simply not found there. Three names under `urn:file:` are the exception,
and they are not a file system.

## One recorded file

The part "Exploring without Rust" runs on one recorded week of earthquakes, as USGS
published it: 178 events of magnitude 4.5 and up, from 2024-01-01 to 2024-01-08, in
GeoJSON. At a shell it is a file you save in the CLI's file workspace, where it is
`urn:file:quakes-2024-01-01.geojson`. The page binds that same name to the same bytes,
compiled into the module by the `quakes` crate, so a cell and a command name one resource.
It answers only reads, and it never changes. The JSON-LD context that
[Giving the data names](explore/names.md) reads beside it is bound the same way, at
`urn:file:quakes.context.jsonld`, the name the copy you download there has, and so is the
week after that [Composites, one layer at a time](explore/composites.md) joins, at
`urn:file:quakes-2024-01-08.geojson`.

The first line below asks `urn:json:query`, a jq program run as a resource, how many events
the file holds. The query reads the file through the kernel, so the second line finds the
file's own answer already cached. The third asks the file to describe itself:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:file:quakes-2024-01-01.geojson q=".features|length"
cache urn:file:quakes-2024-01-01.geojson
describe urn:file:quakes-2024-01-01.geojson text/plain'>
<pre class="ikigai-run-expected">178
[computed]
cached
quakes-window — A recorded week of earthquakes
USGS earthquakes of M4.5 and up from 2024-01-01 to 2024-01-08, 178 events, as GeoJSON, recorded once and never changed: the window &quot;Exploring without Rust&quot; runs on.
verbs: Source, Meta
[computed]</pre>
</div>

Every other name under `urn:file:` is not found in this page: a `sink` to one has nowhere
to go. The CLI binds the whole family to a directory on your disk.

## Rust's standard library cannot see the browser

The browser has a clock and timers, but a Rust crate compiled for
`wasm32-unknown-unknown` cannot reach them on its own. That target has no operating
system, so the standard library has nothing to call. `std::time::Instant::now()`,
`SystemTime::now()` and `std::thread::spawn` all **compile** for it, and then fail **when
they are called**. Everything the browser offers is reached through JavaScript imports,
which is what `wasm-bindgen`, `js-sys` and `web-sys` provide.

The same target also runs outside a browser: under a WebAssembly runtime such as
wasmtime, where there is no JavaScript at all. So a library that wants to run in both
places must not import JavaScript itself. It takes what it needs from its host, the way
the kernel does. That is why a crate in this ecosystem says "the host supplies a clock"
rather than calling `Date.now()` itself.

The page supplies its clock like this:

```rust,ignore
{{#include ../../../crates/book-wasm/src/lib.rs:browser_clock}}
```

Only a page that asks for a clock gets one, as [Time is a resource](applied/time.md)
explains, and this page does not ask. Try it: the time endpoint refuses, and a trace has no
durations to print.

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:time:now'>
<pre class="ikigai-run-expected">error: endpoint error: this kernel has no clock: its host did not give it one (Kernel::with_clock)</pre>
</div>

<div class="ikigai-run" data-cmd='trace urn:iki:tutorial:title'>
<pre class="ikigai-run-expected">trace  urn:iki:tutorial:title
  client      ikigai repl  ·  capability: root (full authority)
  transport   embedded · in-process
&#32;
urn:iki:tutorial:title   title · computed · ThreadId(1) · —   → 27b  resource oriented computing</pre>
</div>

The `—` in the trace is where a duration would go. With no clock, the kernel does not
guess.

Timers work the same way. A timer is not a clock: a clock answers "what time is it", and a
timer answers "wake me when". A native host's timer runs on a thread. In the browser, a
host builds one on `setTimeout` or `setInterval`, and hands it to whatever needs to wait.
`ikigai-web-demo` does this for its scheduled jobs, and the `Retry` overlay in
`ikigai-throttle` takes a timer the same way.

## Bridging the browser's async world

The kernel's futures are `Send`: they may be moved between threads, which a native host
needs. A future that waits on a browser promise is not, because JavaScript values stay on
the page's one thread. So the page cannot simply await `fetch` inside an endpoint.

The bridge is small. The endpoint starts the browser work as a task of the page's own,
with `spawn_local`, and waits on a channel that the task completes. The channel's future
is `Send`, so the kernel never holds anything that is not. This is how the page fetches the
RDF Toolbox's build in [Checking a resource](building/checking.md):

```rust,ignore
{{#include ../../../crates/book-wasm/src/lib.rs:dist}}
```

A browser timer uses the same bridge: a task waits on the `setTimeout` promise, then
completes the channel the endpoint is waiting on.

The other direction never happens: the kernel does not call into the page. In
[A spreadsheet, VII: push](applied/spreadsheet-7.md) the page awaits a future that a cut
completes, and then decides on its own turn of the event loop what to redraw. In
[A space that changes while it runs](applied/live-space.md) the live host's rebuild loop
is itself a `spawn_local` task.

## What fails, and how

- **A call that compiles but cannot run.** Reading the platform's time or starting a
  thread compiles for this target and fails when it runs, so no build catches it. A crate
  that must run here keeps those calls behind `#[cfg(not(target_arch = "wasm32"))]`, or
  takes them from its host, and checks for them in its tests.
- **A panic stops the page's kernel.** A panic in WebAssembly is a trap: the module stops,
  and the page's kernel with it, until you reload. The page installs
  `console_error_panic_hook`, so the reason appears in the browser's console. The book's
  programs stay far inside every bound for this reason.
- **The JavaScript and the WebAssembly must agree.** The generated JavaScript and the
  compiled module share an interface version. The book pins `wasm-bindgen` exactly, and
  pins its command-line tool to the same version, because a mismatch fails in the page,
  not in any build.

## The same code natively

`book-wasm` also builds as an ordinary Rust library, which is how this repository's
gates lint and test it. Natively, the page's clock is the system's, and the toolbox's
build is not fetched at all. The chapters' cells are run by those tests too: each one is
read from its page and resolved against a kernel built exactly as the page builds it. So a
cell whose answer changes fails the build, not your reading.

[A kernel behind a socket](beyond/socket.md) is the opposite end: a native host that
serves a kernel to other processes, with every seam in the table above filled from the
operating system.
