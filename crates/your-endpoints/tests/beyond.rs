//! Part III's exercises are done here — against `crates/two-hosts`, which this file uses
//! and does not edit.
//!
//! What is here is the scaffolding an exercise would otherwise spend its time on: a
//! socket path nothing collides with, a dial loop that waits for a server thread to bind,
//! a peer that is always down, and one test proving the scaffolding works. The
//! assertions the exercises ask for are yours to write, below the line.
//!
//! Unix only, and not out of politeness: `ikigai-ipc` is itself `#![cfg(unix)]`, so on any
//! other platform there is no `serve` to call.
#![cfg(unix)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use ikigai_core::{Capability, Error, Representation, Request, SpaceEntry};
use ikigai_resolve::{CacheStatus, Resolver};

/// A socket path nothing else on this machine will collide with. Unix socket paths are
/// short (about 100 bytes, kernel-enforced), so this deliberately does not nest.
pub fn socket_path(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!("ikigai-yours-{label}-{}.sock", std::process::id()))
}

/// Dial until the server has bound, or give up. `serve` binds on its own thread, so the
/// first dial legitimately races it — this is the handshake with a thread that is
/// starting, not flakiness papered over.
pub fn connect_when_up(path: &Path) -> ikigai_ipc::IpcResolver {
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match two_hosts::socket::connect(path) {
            Ok(resolver) => return resolver,
            Err(_) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(20)),
            Err(e) => panic!("the server never came up on {}: {e}", path.display()),
        }
    }
}

/// Serve `space` on a fresh socket from a thread that outlives the test (`serve` never
/// returns; the process exiting is what stops it), and hand back the path.
pub fn serve_in_background(label: &str, space: Arc<dyn ikigai_core::Space>) -> PathBuf {
    let path = socket_path(label);
    let _ = std::fs::remove_file(&path);
    let server = path.clone();
    std::thread::spawn(move || {
        let _ = two_hosts::socket::serve(space, &server);
    });
    path
}

/// A peer that is always down, or always refuses. Every mount rule is about what happens
/// when the answer is *not* a representation, so the only peer those exercises need is
/// one that never gives one. `calls` counts how often it was asked.
pub struct DeadPeer {
    pub error: fn(String) -> Error,
    pub calls: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl Resolver for DeadPeer {
    fn issue(&self, _request: Request) -> Result<(Representation, CacheStatus), Error> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Err((self.error)("the peer".to_string()))
    }

    fn is_cached(&self, _request: &Request, _capability: &Capability) -> bool {
        false
    }

    fn entries(&self) -> Option<Vec<SpaceEntry>> {
        None
    }
}

/// A dead peer failing with `error`, and the counter it bumps on every call.
pub fn dead(error: fn(String) -> Error) -> (Arc<dyn Resolver>, Arc<AtomicUsize>) {
    let calls = Arc::new(AtomicUsize::new(0));
    (
        Arc::new(DeadPeer {
            error,
            calls: Arc::clone(&calls),
        }),
        calls,
    )
}

/// The scaffolding works: a server comes up and a client reaches it. Resolving something
/// through it is exercise 1.
#[test]
fn the_server_comes_up_and_a_client_reaches_it() {
    let path = serve_in_background("scaffold", Arc::new(your_endpoints::space()));
    let peer = connect_when_up(&path);
    assert!(
        peer.entries().is_some(),
        "a connected peer lists what it serves"
    );
    let _ = std::fs::remove_file(&path);
}

// ── Your exercises go below this line ──────────────────────────────────────────
