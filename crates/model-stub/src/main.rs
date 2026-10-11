//! `model-stub <socket>`: serve the stub model on a Unix socket until stopped.
//!
//! It prints one line once the socket accepts connections, so a transcript's `serve` block
//! knows it is up, and one line per ask after that (see the library's docs).

use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;
use std::time::{Duration, Instant};

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let (Some(socket), None) = (args.next(), args.next()) else {
        eprintln!("usage: model-stub <socket>");
        return ExitCode::from(2);
    };
    let socket = PathBuf::from(socket);
    let kernel = model_stub::kernel(Arc::new(AtomicUsize::new(0)));
    let path = socket.clone();
    let server = std::thread::spawn(move || ikigai_ipc::serve(kernel, &path));
    let deadline = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(&socket).is_err() {
        if server.is_finished() || Instant::now() > deadline {
            let why = match server.join() {
                Ok(Err(e)) => e.to_string(),
                _ => "it did not start listening".to_string(),
            };
            eprintln!("model-stub: could not serve {}: {why}", socket.display());
            return ExitCode::FAILURE;
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    println!(
        "model-stub: urn:llm:ask and urn:llm:config on {} (model {})",
        socket.display(),
        model_stub::MODEL
    );
    match server.join() {
        Ok(Ok(())) => ExitCode::SUCCESS,
        Ok(Err(e)) => {
            eprintln!("model-stub: {e}");
            ExitCode::FAILURE
        }
        Err(_) => ExitCode::FAILURE,
    }
}
