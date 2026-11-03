//! A kernel that logs, for Running ikigai's "Logging" chapter.
//!
//! `ikigai-log` decides nothing about when to log: binding its endpoints opens no segment, and
//! no released `ikigai` binary links it. A host opens the writer. This is the smallest such
//! host: the log's space, SPARQL (so a segment can be a `graph=` source) and the function library
//! (something to resolve), with [`LogTracer`] installed so that every resolution lands as an
//! entry.
//!
//! It has two doors. The socket `ikigai --connect` dials is the operator's: root, naming nobody.
//! The second, [`Door`], is a stand-in for the door a real service has: it takes a principal per
//! request and resolves under a per-call tracer, which is the point of it. See [`Door::ask`].

use std::io::{BufRead, BufReader, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, Clock, Fallback, Iri, Kernel, Request, Space, SystemClock, Time, Verb,
};
use ikigai_log::{Destination, LogConfig, LogHandle, LogTracer, Principal, Timestamp, Vocabulary};

/// The name this host's override layer goes by: `log-host.log.toml`, over `log.toml`.
pub const APP: &str = "log-host";

/// A clock that starts where it is told and advances one second per reading, so that what a
/// page prints (a segment's name, an entry's time) is the same on every run. Only a page wants
/// this; a real host runs on [`SystemClock`], which is what `log-host` uses unless asked.
#[derive(Debug)]
pub struct SteppingClock(AtomicU64);

impl SteppingClock {
    /// A clock whose first reading is `start`.
    pub fn from(start: Timestamp) -> SteppingClock {
        SteppingClock(AtomicU64::new(start.as_millis()))
    }
}

impl Clock for SteppingClock {
    fn now(&self) -> Time {
        Time::from_millis(self.0.fetch_add(1000, Ordering::Relaxed))
    }
}

/// What a host needs to be told.
#[derive(Debug, Clone)]
pub struct Options {
    /// The config home `log.toml` is layered from.
    pub config_home: PathBuf,
    /// Read what is there and open no writer, so nothing is appended (to verify a restore).
    pub read_only: bool,
    /// Start a [`SteppingClock`] here, rather than reading the system's.
    pub clock_from: Option<Timestamp>,
}

/// A composed kernel, and the log it writes.
pub struct Host {
    /// The kernel: the log's space, SPARQL and the function library.
    pub kernel: Arc<Kernel>,
    /// The log. Closed if the host is read-only or `log.toml` says `off`.
    pub handle: Arc<LogHandle>,
    /// The tracer installed on the kernel, whose per-call children [`Door`] hands out.
    pub tracer: Arc<LogTracer>,
    /// The clock the kernel and the tracer share.
    pub clock: Arc<dyn Clock>,
}

impl Host {
    /// Layer `log.toml` from the config home, open the writer unless told not to, and compose.
    pub fn open(options: &Options) -> Result<Host, String> {
        let clock: Arc<dyn Clock> = match options.clock_from {
            Some(start) => Arc::new(SteppingClock::from(start)),
            None => Arc::new(SystemClock),
        };
        // ANCHOR: open
        // The quiet default is `off`. A host that means to log says so, and a `log.toml` can
        // still say otherwise; that is the operator's call, not this crate's.
        let base = LogConfig::default().with_destination(Destination::File);
        let mut config = ikigai_log::load::complete_in(&options.config_home, Some(APP), base)
            .map_err(|e| e.to_string())?;
        // Left alone, segments would land in the data home (`~/.ikigai/log`). This host keeps
        // them beside the `log.toml` that governs them, so a config home is the whole of a log.
        if config.directory.is_none() {
            let home = std::env::current_dir()
                .map_err(|e| e.to_string())?
                .join(&options.config_home);
            config = config.with_directory(home.join("log"));
        }
        let handle = Arc::new(LogHandle::new(
            Some(options.config_home.clone()),
            Some(APP.to_string()),
            config,
        ));
        if !options.read_only {
            let now = Timestamp::from_millis(clock.now().as_millis());
            handle
                .open(Vocabulary::shared_builtin(), now)
                .map_err(|e| e.to_string())?;
        }
        // ANCHOR_END: open
        let space = Fallback::new(vec![
            Arc::new(ikigai_sparql::space()) as Arc<dyn Space>,
            Arc::new(ikigai_fn::space()) as Arc<dyn Space>,
            // Last: its segment template matches everything under `urn:log:`.
            Arc::new(ikigai_log::endpoints::space(handle.clone())) as Arc<dyn Space>,
        ]);
        let kernel =
            Kernel::with_meta_renderer(Arc::new(space), Arc::new(ikigai_vocab::TurtleRenderer))
                .with_clock(clock.clone());
        // ANCHOR: tracer
        // The one global tracer: the process's own, naming nobody. Every resolution that does
        // not come through `Door` lands here.
        let tracer = Arc::new(LogTracer::new(handle.clone(), clock.clone()));
        if !options.read_only {
            kernel.set_tracer(tracer.clone());
        }
        // ANCHOR_END: tracer
        Ok(Host {
            kernel: Arc::new(kernel),
            handle,
            tracer,
            clock,
        })
    }

    /// End the log in order: a `log:ProcessStop` and a final seal.
    pub fn close(&self) -> Result<(), String> {
        let now = Timestamp::from_millis(self.clock.now().as_millis());
        self.handle.close(now).map_err(|e| e.to_string())
    }
}

/// The door that names a party per request.
///
/// A real door takes the name from what authenticated the caller (a certificate, a passkey).
/// This one takes it from the request line, because the chapter is about what happens to the
/// name afterwards. Its socket is the owner's, like the operator's.
pub struct Door {
    /// The host behind the door.
    pub host: Arc<Host>,
}

impl Door {
    /// Resolve `iri` for `principal`, under a capability that holds no scope at all.
    ///
    /// The tracer is per CALL. [`Kernel::set_tracer`] is one slot for the whole kernel:
    /// installing Alice's tracer there would file Bob's concurrent request under Alice, and
    /// [`Kernel::issue_traced`] is exclusive of that slot, so each request lands exactly once,
    /// under its own name.
    // ANCHOR: ask
    pub fn ask(&self, principal: &str, iri: &str, args: &[(&str, &str)]) -> Result<String, String> {
        let principal = Principal::new(principal).map_err(|e| e.to_string())?;
        let tenant = Arc::new(self.host.tracer.on_behalf_of(principal));
        let target = Iri::parse(iri).map_err(|e| e.to_string())?;
        let request = args.iter().fold(
            Request::new(Verb::Source, target),
            |request, (name, value)| {
                request.with_arg(*name, ArgRef::Inline(value.as_bytes().to_vec()))
            },
        );
        let capability = Capability::scoped(Vec::<String>::new());
        let answer = block_on(self.host.kernel.issue_traced(request, &capability, tenant))
            .map_err(|e| e.to_string())?;
        Ok(String::from_utf8_lossy(&answer.bytes).into_owned())
    }
    // ANCHOR_END: ask

    /// Serve the door until a `stop` line arrives. One request per connection, one at a time:
    /// `ask <principal> <iri> [name=value…]`, or `stop`.
    pub fn serve(&self, listener: UnixListener) -> std::io::Result<()> {
        for stream in listener.incoming() {
            let mut stream = stream?;
            let mut line = String::new();
            BufReader::new(&stream).read_line(&mut line)?;
            let words: Vec<&str> = line.split_whitespace().collect();
            match words.as_slice() {
                ["stop"] => {
                    let answer = match self.host.close() {
                        Ok(()) => "log closed\n".to_string(),
                        Err(e) => format!("error: {e}\n"),
                    };
                    stream.write_all(answer.as_bytes())?;
                    return Ok(());
                }
                ["ask", principal, iri, args @ ..] => {
                    let args: Vec<(&str, &str)> =
                        args.iter().filter_map(|arg| arg.split_once('=')).collect();
                    let answer = match self.ask(principal, iri, &args) {
                        Ok(text) => text,
                        Err(e) => format!("refused: {e}"),
                    };
                    writeln!(stream, "{}", answer.trim_end())?;
                }
                _ => writeln!(
                    stream,
                    "usage: ask <principal> <iri> [name=value…], or stop"
                )?,
            }
        }
        Ok(())
    }
}

/// Send one line to a door and return what it answered.
pub fn call(door: &Path, line: &str) -> std::io::Result<String> {
    let mut stream = UnixStream::connect(door)?;
    writeln!(stream, "{line}")?;
    let mut answer = String::new();
    std::io::Read::read_to_string(&mut stream, &mut answer)?;
    Ok(answer)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A scratch config home that logs at `debug`, removed when dropped.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir = std::env::temp_dir().join(format!("log-host-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            std::fs::write(
                dir.join("log.toml"),
                "level = \"debug\"\ninstance = \"t\"\n",
            )
            .unwrap();
            Scratch(dir)
        }

        fn host(&self, read_only: bool) -> Arc<Host> {
            Arc::new(
                Host::open(&Options {
                    config_home: self.0.clone(),
                    read_only,
                    clock_from: Some(Timestamp::from_millis(0)),
                })
                .unwrap(),
            )
        }

        fn lines(&self) -> String {
            let mut text = String::new();
            for entry in std::fs::read_dir(self.0.join("log")).unwrap() {
                let path = entry.unwrap().path();
                if path.extension().is_some_and(|e| e == "log") {
                    text.push_str(&std::fs::read_to_string(path).unwrap());
                }
            }
            text
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn each_request_lands_once_under_its_own_principal() {
        let scratch = Scratch::new("principal");
        let door = Door {
            host: scratch.host(false),
        };
        let upper = "urn:iki:fn:toUpper";
        assert_eq!(door.ask("urn:agent:a", upper, &[("in", "x")]).unwrap(), "X");
        assert_eq!(door.ask("urn:agent:b", upper, &[("in", "y")]).unwrap(), "Y");
        let lines = scratch.lines();
        let attributed: Vec<&str> = lines.lines().filter(|l| l.contains(upper)).collect();
        assert_eq!(attributed.len(), 2, "{lines}");
        assert!(attributed[0].contains("principal=urn:agent:a "), "{lines}");
        assert!(attributed[1].contains("principal=urn:agent:b "), "{lines}");
    }

    #[test]
    fn the_door_holds_no_authority_and_the_refusal_is_attributed() {
        let scratch = Scratch::new("denied");
        let door = Door {
            host: scratch.host(false),
        };
        let refused = door
            .ask("urn:agent:a", "urn:log:segments", &[])
            .unwrap_err();
        assert!(refused.contains("urn:cap:log:read"), "{refused}");
        let lines = scratch.lines();
        assert!(
            lines
                .lines()
                .any(|l| l.contains("log:CapabilityDenied") && l.contains("principal=urn:agent:a ")),
            "{lines}"
        );
    }

    #[test]
    fn a_read_only_host_appends_nothing() {
        let scratch = Scratch::new("read-only");
        let writer = scratch.host(false);
        writer.close().unwrap();
        let before = scratch.lines();
        let reader = scratch.host(true);
        assert!(!reader.handle.is_open());
        let request = Request::new(Verb::Source, Iri::parse("urn:log:verify").unwrap());
        let report = block_on(reader.kernel.issue(request, &Capability::root())).unwrap();
        let report = String::from_utf8_lossy(&report.bytes).into_owned();
        assert!(report.contains("0 broken"), "{report}");
        assert_eq!(scratch.lines(), before);
    }
}
