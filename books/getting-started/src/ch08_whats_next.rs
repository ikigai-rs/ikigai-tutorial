//! # 8 · What's next
//!
//! ## Loadable modules — the follow-on book
//!
//! Everything in this book was **linked in**: your crate exposes a `space()`, the host
//! depends on it at compile time, and the endpoints are in the binary. That is how nearly
//! all of ikigai works today, including every endpoint in the `ikigai` CLI.
//!
//! There is a second shape. A **module** is an independently-compiled `space()` that a
//! host routes some IRIs to at runtime, without linking it. Three crates support it —
//! `ikigai-xslt`, `ikigai-jsonld` and `ikigai-shacl` — and all three are *dual-mode*:
//! ordinary linked libraries by default, loadable WebAssembly with `--features module`.
//!
//! ### The idea worth understanding early
//!
//! A module is **not** a remote kernel. When you resolve against an IPC or QUIC peer,
//! that peer resolves every sub-request on its own side — it has its own catalog, its own
//! cache, its own spaces.
//!
//! A module is the opposite arrangement. Its endpoints must resolve *their* resource
//! references — an XSLT stylesheet, a SHACL shapes graph — against the **host's** kernel,
//! because the host owns the catalog and the cache. So a module has to call **back** into
//! the host in the middle of an invocation.
//!
//! That callback lands on a seam that already existed: an `Invocation` reaches the kernel
//! through the `Issuer` trait, so the module is simply handed the host as its issuer.
//! Nothing new had to be invented for it — which is the sort of thing that suggests the
//! decomposition was right.
//!
//! ### Where it actually stands
//!
//! `ikigai-module` describes itself as **"Phase 1: in-process proof."** The in-process
//! transport runs a module in the same process and exercises the full callback path; a
//! loopback transport runs the same session through the wire codec. Real out-of-process
//! transports — a second wasm instance, an embedded wasmtime, a socket — are Phase 2.
//!
//! So: the callback machinery is proven, the isolation is not there yet, and the only
//! host that loads modules today is the browser demo. Worth knowing before you plan
//! around it.
//!
//! ## Exercises
//!
//! 1. **Strict lowerCamelCase.** `toCamel` never lower-cases anything. Write `toLowerCamel`
//!    and decide what it should do with `"XMLHttpRequest"` — there is no obviously right
//!    answer, which is the point.
//! 2. **A second argument.** Add an optional `separator` so the caller can split on
//!    something other than whitespace. Declare it `optional()` with a `default`, then look
//!    at `urn:kernel:actions` and watch your own change appear in the catalog.
//! 3. **Take a resource, not a string.** Call your endpoint with `in=urn:fn:toUpper?in=x`
//!    and satisfy yourself that you did not have to write any code for that to work.
//! 4. **Break cacheability on purpose.** Make an endpoint that returns the current time,
//!    mark it `.cacheable()`, and observe how convincing a wrong answer looks.
//!
//! ## Where to read next
//!
//! - `ikigai-core` — the kernel, `Description`/`ArgSpec`, capabilities, golden threads
//! - `ikigai-fn` — the smallest complete example of a space, and the one this book chains
//! - `ikigai-cli` — a real host: transports, the engine grammar, MCP projection
//! - `ikigai-xslt` — one crate showing both the linked and the loadable shape side by side
//!
//! ## A closing note on style
//!
//! You will notice the comments in this codebase are unusually long, and that they argue
//! rather than describe. That is deliberate: the code says what it does, so a comment
//! that repeats it earns nothing. What a comment is *for* is the constraint the code
//! cannot state — why this is not cached, why this lock is dropped before that call, what
//! broke the last time somebody did the obvious thing.
//!
//! Where a comment states the *shape* of something that leaves the process, it wants a
//! doctest in the same edit — because a doctest is the only comment the compiler reads.
//! That rule is why this book is written in rustdoc.
