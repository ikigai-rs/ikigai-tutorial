//! The code taught by "A space that changes while it runs": a **dynamic space**, built and
//! rebuilt from a declaration resource while the kernel runs (ikigai-core 0.1.109, ledger
//! #2006).
//!
//! [Tic-tac-toe, VII](../../books/ikigai/src/applied/tic-tac-toe-7.md) built a space from a
//! declaration once, at start. This crate keeps the same two halves, a [`registry`] of
//! endpoints the host wrote and a declaration that only arranges them, and adds the third
//! thing a running host needs: a place the declaration lives, which a write changes.
//!
//! * [`DECLARATION`] — the declaration, as a resource. `Sink` an arrangement to it (the
//!   s-expression Tic-tac-toe VII's file is written in, or Turtle); `Source` answers it as
//!   the `ik:` Turtle core builds from. Until something is written, it is not found.
//! * [`registry`] — four endpoints the book already has, by the names they give themselves:
//!   Part I's `camel-case`, `title` and `camel-title`, and ikigai-fn's `toUpper`. Nothing is
//!   bound to them until a declaration says where.
//! * [`kernel`] — a host: the declaration and `urn:diagram:kernel` in a fixed space of its
//!   own, then the dynamic space, behind it. The root never changes; the dynamic space's
//!   doors do.
//!
//! * [`Host`] — the three, started: the space named, the host's loop (`DynamicSpace::drive`)
//!   running on a [`LocalPool`], and a write that lets the loop catch up before it returns,
//!   so the chapter's listings read like a REPL session.

use std::sync::{Arc, Mutex};

use futures::executor::{block_on, LocalPool};
use futures::task::LocalSpawnExt;
use ikigai_core::{
    ArgRef, Capability, DynamicSpace, EndpointSpace, Error, Exact, Fallback, FnEndpoint,
    Invocation, Iri, Kernel, Registry, ReprType, Representation, Request, Space, Verb,
};
use ikigai_sexpr::arrangement_to_topology;
use ikigai_vocab::TurtleRenderer;

pub use hello_camel::TitleState;

// ANCHOR: names
/// Where the dynamic space's declaration lives. A write to it is a change to the space.
pub const DECLARATION: &str = "urn:iki:tutorial:live:declaration";

/// `view.ttl`: a declaration holding one view, `urn:iki:tutorial:live:picture`, onto
/// `urn:diagram:kernel`. Turtle, because the s-expression form cannot say `resolveAt` yet.
pub const VIEW: &str = include_str!("../view.ttl");

/// The first declaration the chapter writes: all four endpoints, bound.
pub const FOUR_DOORS: &str = r#"(endpoints
  (door "urn:iki:tutorial:camel-case" camel-case)
  (door "urn:iki:tutorial:title" title)
  (door "urn:iki:tutorial:camel-title" camel-title)
  (door "urn:iki:tutorial:upper" toUpper))"#;
// ANCHOR_END: names

/// What the declaration holds: the text last written, if any.
#[derive(Debug, Default)]
pub struct DeclarationState {
    text: Mutex<Option<String>>,
}

/// `live-declaration`: the dynamic space's declaration. `Sink` replaces it with `content`;
/// `Source` answers it as Turtle, reading an s-expression (its first character is `(`)
/// through ikigai-sexpr and passing anything else through as Turtle already. Not found
/// until it is first written.
///
/// The read is cacheable, so it hangs from its own name, and the kernel cuts that thread
/// after every write: the host's loop hears the write as that cut.
pub fn declaration(state: Arc<DeclarationState>) -> FnEndpoint {
    FnEndpoint::new("live-declaration", move |inv: &Invocation<'_>| {
        let mut text = state.text.lock().expect("declaration lock");
        match inv.request.verb {
            Verb::Sink => {
                *text = Some(inv.inline_str("content")?.to_string());
                Ok(Representation::new(
                    ReprType::new("text/plain"),
                    b"ok".to_vec(),
                ))
            }
            _ => {
                let Some(written) = text.as_deref() else {
                    return Err(Error::NotFound(inv.request.target.as_str().to_string()));
                };
                let turtle = if written.trim_start().starts_with('(') {
                    arrangement_to_topology(written)
                        .map_err(|e| Error::Endpoint(format!("not an arrangement: {e}")))?
                        .to_turtle()
                } else {
                    written.to_string()
                };
                Ok(Representation::new(
                    ReprType::new("text/turtle").with_param("charset", "utf-8"),
                    turtle.into_bytes(),
                )
                .cacheable())
            }
        }
    })
}

// ANCHOR: registry
/// Every endpoint the dynamic space may bind, by the name each gives itself. A
/// declaration that names anything else is refused, and binds nothing.
pub fn registry(title: Arc<TitleState>) -> Registry {
    let mut registry = Registry::new();
    for endpoint in [
        Arc::new(hello_camel::camel_case()) as Arc<dyn ikigai_core::Endpoint>,
        Arc::new(hello_camel::title(title)),
        Arc::new(hello_camel::camel_title()),
        Arc::new(ikigai_fn::to_upper()),
    ] {
        registry.register(endpoint).expect("four distinct names");
    }
    registry
}
// ANCHOR_END: registry

// ANCHOR: kernel
/// The root of a host for `dynamic`: the declaration, then the picture of the host's own arrangement,
/// then the dynamic space. The layers are consulted in order, so a door the declaration
/// binds can never shadow the declaration itself.
pub fn root(
    declaration_state: Arc<DeclarationState>,
    dynamic: Arc<DynamicSpace>,
) -> Arc<dyn Space> {
    let fixed = EndpointSpace::new().bind(Exact::new(DECLARATION), declaration(declaration_state));
    Arc::new(Fallback::new(vec![
        Arc::new(fixed),
        Arc::new(ikigai_diagram::space()),
        dynamic,
    ]))
}

/// The kernel over [`root`], with the Meta renderer Part I's host uses.
pub fn kernel(root: Arc<dyn Space>) -> Kernel {
    Kernel::with_meta_renderer(root, Arc::new(TurtleRenderer))
}
// ANCHOR_END: kernel

/// A `Source` of `iri`, with `args` inline: the request every listing reads with.
pub fn source(iri: &str, args: &[(&str, &str)]) -> Request {
    let mut request = Request::new(Verb::Source, Iri::parse(iri).expect("an IRI"));
    for (name, value) in args {
        request = request.with_arg(*name, ArgRef::Inline(value.as_bytes().to_vec()));
    }
    request
}

/// The chapter's host, started: a dynamic space, the kernel around it, and the loop that
/// rebuilds the space whenever its declaration is written.
pub struct Host {
    /// The kernel every listing asks.
    pub kernel: Arc<Kernel>,
    /// The kernel's root space: the declaration, the picture, and the dynamic space.
    pub root: Arc<dyn Space>,
    /// The dynamic space, for its status and its thread.
    pub space: Arc<DynamicSpace>,
    /// `title`'s state, so a listing can count the reads that really ran.
    pub title: Arc<TitleState>,
    pool: LocalPool,
}

impl Host {
    // ANCHOR: start
    /// Make an empty dynamic space called `name`, put it in a host, and start the loop
    /// that rebuilds it from [`DECLARATION`].
    pub fn start(name: &str) -> Host {
        let title = Arc::new(TitleState::default());
        let space = Arc::new(DynamicSpace::new(
            name,
            Iri::parse(DECLARATION).expect("a constant IRI"),
            registry(Arc::clone(&title)),
        ));
        let root = root(Arc::default(), Arc::clone(&space));
        let kernel = Arc::new(kernel(Arc::clone(&root)));
        // The host's loop: read the declaration, bind it, and wait for its thread to be
        // cut, forever. Core has no runtime, so the host spawns it on one.
        let pool = LocalPool::new();
        let (looping, on) = (Arc::clone(&space), Arc::clone(&kernel));
        pool.spawner()
            .spawn_local(async move {
                let stopped = looping.drive(&on, &Capability::root()).await;
                panic!("the rebuild loop stopped: {stopped}");
            })
            .expect("spawn the rebuild loop");
        let mut host = Host {
            kernel,
            root,
            space,
            title,
            pool,
        };
        host.settle();
        host
    }
    // ANCHOR_END: start

    // ANCHOR: write
    /// Write `declaration` to [`DECLARATION`], as any client with the authority to write it
    /// could, then let the loop run until it has nothing left to do.
    pub fn write(&mut self, declaration: &str) {
        let write = Request::new(Verb::Sink, Iri::parse(DECLARATION).expect("a constant IRI"))
            .with_arg("content", ArgRef::Inline(declaration.as_bytes().to_vec()));
        block_on(self.kernel.issue(write, &Capability::root())).expect("the write");
        self.settle();
    }
    // ANCHOR_END: write

    /// Run the host's loop until it waits for the next cut.
    pub fn settle(&mut self) {
        self.pool.run_until_stalled();
    }

    /// `request` under root, its answer as text.
    pub fn read(&self, request: Request) -> Result<String, Error> {
        self.read_as(request, &Capability::root())
    }

    /// `request` under `capability`, its answer as text.
    pub fn read_as(&self, request: Request, capability: &Capability) -> Result<String, Error> {
        let answer = block_on(self.kernel.issue(request, capability))?;
        Ok(String::from_utf8_lossy(&answer.bytes).into_owned())
    }

    /// Whether `request`, under root, would be served from the cache now.
    pub fn is_cached(&self, request: &Request) -> bool {
        self.kernel.is_cached(request, &Capability::root())
    }
}
