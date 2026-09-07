//! A kernel behind a Unix socket — the smallest transport that is really a transport.
//!
//! Unix only, and the `#[cfg(unix)]` is not politeness: `ikigai-ipc` is itself
//! `#![cfg(unix)]`, so on any other platform the crate compiles to nothing and these
//! functions would not have a `serve` to call.

use std::path::Path;
use std::sync::Arc;

use ikigai_core::{Kernel, Space};

// ANCHOR: serve
/// Serve `space` as a kernel on the Unix socket at `path`, until an unrecoverable accept
/// error. **Blocks** — this is a server's main loop, not a spawn.
///
/// There is no authentication code here and that is the design: the socket is created
/// `0600` inside a `0700` directory, and `ikigai_ipc::serve` additionally refuses any
/// peer whose kernel-verified UID is not the server's own. On one machine, between one
/// user's processes, the operating system already knows who is asking.
pub fn serve(space: Arc<dyn Space>, path: &Path) -> std::io::Result<()> {
    ikigai_ipc::serve(Kernel::new(space), path)
}
// ANCHOR_END: serve

// ANCHOR: connect
/// Connect to a kernel server on `path`.
///
/// The returned [`IpcResolver`](ikigai_ipc::IpcResolver) implements the same `Resolver`
/// trait an embedded kernel does, which is the whole reason a mount can be pointed at
/// either without the caller knowing. It is also what makes the next chapter possible:
/// a thing that answers requests is a thing a local kernel can compose.
pub fn connect(path: &Path) -> std::io::Result<ikigai_ipc::IpcResolver> {
    ikigai_ipc::connect(path)
}
// ANCHOR_END: connect

#[cfg(test)]
mod tests {
    use super::*;
    use ikigai_core::{ArgRef, Iri, Request, Verb};
    use ikigai_resolve::Resolver;
    use std::time::{Duration, Instant};

    /// A socket path nothing else on this machine will collide with. Unix socket paths
    /// are short (about 100 bytes, kernel-enforced), so this deliberately does not nest.
    fn socket_path(label: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!("ikigai-book-{label}-{}.sock", std::process::id()))
    }

    /// Dial until the server has bound, or give up. `serve` binds on its own thread, so
    /// the first dial legitimately races it — a retry loop here is not flakiness
    /// papered over, it is the handshake with a thread that is starting.
    fn connect_when_up(path: &Path) -> ikigai_ipc::IpcResolver {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            match connect(path) {
                Ok(resolver) => return resolver,
                Err(e) if Instant::now() < deadline => {
                    std::thread::sleep(Duration::from_millis(20));
                    let _ = e;
                }
                Err(e) => panic!("the server never came up on {}: {e}", path.display()),
            }
        }
    }

    #[test]
    fn a_resource_resolves_through_the_socket_exactly_as_it_does_in_process() {
        let path = socket_path("resolves");
        let _ = std::fs::remove_file(&path);
        let server = path.clone();
        // The server thread outlives the test on purpose: `serve` never returns, and a
        // test that joined it would hang forever. Killing it is the process exiting.
        std::thread::spawn(move || {
            let _ = serve(Arc::new(hello_camel::space()), &server);
        });

        let resolver = connect_when_up(&path);
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:camel-case").expect("iri"),
        )
        .with_arg(
            "in",
            ArgRef::Inline(b"resource oriented computing".to_vec()),
        );
        let (representation, _status) = resolver.issue(request).expect("the peer answers");

        assert_eq!(
            String::from_utf8_lossy(&representation.bytes),
            "resourceOrientedComputing",
            "the same endpoint, one process away"
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn a_name_the_served_kernel_does_not_bind_fails_as_a_resolution_not_as_a_transport_error() {
        let path = socket_path("unbound");
        let _ = std::fs::remove_file(&path);
        let server = path.clone();
        std::thread::spawn(move || {
            let _ = serve(Arc::new(hello_camel::space()), &server);
        });

        let resolver = connect_when_up(&path);
        let request = Request::new(
            Verb::Source,
            Iri::parse("urn:iki:tutorial:not-bound-here").expect("iri"),
        );
        let error = resolver
            .issue(request)
            .expect_err("nothing binds that name");

        // The taxonomy crosses the wire intact: this is the server's `Unresolved`, not
        // some generic "the call failed". A client can tell "your kernel has no such
        // resource" from "your kernel is not there", which is the distinction the next
        // chapter's fallback rule is built on.
        assert!(
            matches!(error, ikigai_core::Error::Unresolved(_)),
            "expected the remote resolution failure to arrive typed, got {error:?}"
        );
        let _ = std::fs::remove_file(&path);
    }
}
