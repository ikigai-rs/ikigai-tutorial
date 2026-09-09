# The repositories

One repository per module, all under <https://github.com/ikigai-rs>. This map is derived
from the organization's repository list (`gh repo list ikigai-rs`) on 2026-09-08, grouped
the way the maintainers group them; a few private repositories (the site, the release
tooling, a secrets module and a set of Lisp programs) are omitted. If a name here does not
resolve on GitHub, the list moved before this page did — the organization is the truth.

## The kernel and the host

- [`ikigai-core`](https://github.com/ikigai-rs/ikigai-core) — the kernel workspace:
  `ikigai-core` (kernel, `Description`/`ArgSpec`, capabilities, golden threads, clocks),
  `ikigai-vocab` (the vocabulary and the Turtle renderer), `ikigai-store`. Everything
  else depends on it.
- [`ikigai-cli`](https://github.com/ikigai-rs/ikigai-cli) — the main host and a
  workspace of its own: the `ikigai` binary, the embedded host, the engine (the REPL
  grammar), resolve/wire/scheduler, the IPC and QUIC transports, the MCP projection, and
  the inbound-HTTP transport library.
- [`ikigai-module`](https://github.com/ikigai-rs/ikigai-module) — the loadable-module
  format: `ModuleSpace`, the transports, host-callback resolution. Part II.

## Compute and data

- [`ikigai-fn`](https://github.com/ikigai-rs/ikigai-fn) — the function library this book
  chains onto, and the smallest complete module crate: read it first if you are writing
  one.
- [`ikigai-fs`](https://github.com/ikigai-rs/ikigai-fs) — the file workspace,
  `urn:file:*`, natively and on a browser's storage.
- [`ikigai-http`](https://github.com/ikigai-rs/ikigai-http) — the outbound HTTP client
  as resources.
- [`ikigai-text`](https://github.com/ikigai-rs/ikigai-text) — Unix-like text endpoints,
  `urn:text:*`, pure pipeline citizens.
- [`ikigai-linkeddata`](https://github.com/ikigai-rs/ikigai-linkeddata) — `ikigai-rdf`
  (transreption between RDF syntaxes), `ikigai-sparql` (the graph face), `ikigai-sniff`.
- [`ikigai-jsonld`](https://github.com/ikigai-rs/ikigai-jsonld) — JSON-LD
  expand/compact/flatten, lazy-loadable as wasm.
- [`ikigai-xslt`](https://github.com/ikigai-rs/ikigai-xslt) and
  [`ikigai-xslt-module`](https://github.com/ikigai-rs/ikigai-xslt-module) — XSLT as a
  resource, linked or loadable.
- [`ikigai-shacl`](https://github.com/ikigai-rs/ikigai-shacl) — SHACL validation,
  native and in the browser.
- [`ikigai-sexpr`](https://github.com/ikigai-rs/ikigai-sexpr) — s-expressions to SPARQL
  and Turtle, no Lisp engine.
- [`ikigai-lisp`](https://github.com/ikigai-rs/ikigai-lisp) — the Lisp engine
  (`urn:lisp:eval`), whose builtins are the five verbs.

## Personal and platform

- [`ikigai-personal`](https://github.com/ikigai-rs/ikigai-personal) — calendar and
  contacts (macOS EventKit).
- [`ikigai-org`](https://github.com/ikigai-rs/ikigai-org) — the org-mode agenda as the
  same event graph.
- [`ikigai-meeting`](https://github.com/ikigai-rs/ikigai-meeting) — meeting scheduling
  over provider backends.
- [`ikigai-llm`](https://github.com/ikigai-rs/ikigai-llm) — `urn:llm:ask`, one facade
  over pluggable inference backends.

## Polyglot faces

- [`ikigai-python`](https://github.com/ikigai-rs/ikigai-python) — a pure-stdlib Python
  client and servable peer for the wire protocol.
- [`ikigai-deno`](https://github.com/ikigai-rs/ikigai-deno) — the same in
  zero-dependency TypeScript.

## Trust and governance

- [`ikigai-sign`](https://github.com/ikigai-rs/ikigai-sign) and
  [`ikigai-encrypt`](https://github.com/ikigai-rs/ikigai-encrypt) — signatures as RDF
  graphs, and their dual.
- [`ikigai-throttle`](https://github.com/ikigai-rs/ikigai-throttle) — rate limit, retry,
  circuit breaker, failover and timeout as space overlays; `Failover` is what a
  prefer-mount is made of.
- [`ikigai-a11y`](https://github.com/ikigai-rs/ikigai-a11y) — layered accessibility
  configuration and the WCAG contrast floor this book's gate measures with.
- [`ikigai-log`](https://github.com/ikigai-rs/ikigai-log) — the log's line grammar and
  its provenance vocabulary.

## Developer tooling and faces

- [`ikigai-repo`](https://github.com/ikigai-rs/ikigai-repo) — git, gh and cargo as
  capability-gated resources.
- [`ikigai-browse`](https://github.com/ikigai-rs/ikigai-browse) — repository browsing
  as resources, with text, HTML and Turtle faces.
- [`ikigai-dev-server`](https://github.com/ikigai-rs/ikigai-dev-server) — the standalone
  IPC server for those, whose `Cargo.toml` is its manifest.
- [`ikigai-web`](https://github.com/ikigai-rs/ikigai-web) — the standalone HTTP and
  SPARQL face (published as `ikigai-web-server`).
- [`ikigai-runbook`](https://github.com/ikigai-rs/ikigai-runbook) — guided, runnable
  demos as resources.
- [`ikigai-name`](https://github.com/ikigai-rs/ikigai-name) — persistent-identifier
  resolution, `urn:name:*`.
  <!-- urn-gate: illustration urn:name: — the family that repository binds; the CLI on
       this machine does not link it, so nothing here can vouch for a name under it. -->
- [`ikigai-emacs`](https://github.com/ikigai-rs/ikigai-emacs) — the editor as a client,
  its commands generated from the manifold.
- [`ikigai-web-demo`](https://github.com/ikigai-rs/ikigai-web-demo) — the kernel in the
  browser, a page assembled by resolution.
- [`ikigai-tutorial`](https://github.com/ikigai-rs/ikigai-tutorial) — this book, and the
  crates it teaches.

## The semantic CMS

- [`ikigai-cms`](https://github.com/ikigai-rs/ikigai-cms) — personal content as one RDF
  graph.
- [`ikigai-cms-web`](https://github.com/ikigai-rs/ikigai-cms-web) — the reading room
  over it.
