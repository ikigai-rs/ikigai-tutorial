//! `log-host`: the host "Logging" runs. See the library's docs.
//!
//! ```text
//! log-host serve --config-home DIR --socket PATH [--door PATH] [--read-only] [--clock-from TIME]
//! log-host ask --door PATH PRINCIPAL IRI [name=value…]
//! log-host stop --door PATH
//! ```

use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ikigai_log::Timestamp;
use log_host::{call, Door, Host, Options};

const USAGE: &str = "usage: log-host serve --config-home DIR --socket PATH [--door PATH] [--read-only] [--clock-from TIME]
       log-host ask --door PATH PRINCIPAL IRI [name=value…]
       log-host stop --door PATH";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let outcome = match args.first().map(String::as_str) {
        Some("serve") => serve(&args[1..]),
        Some("ask") => ask(&args[1..]),
        Some("stop") => stop(&args[1..]),
        _ => Err(USAGE.to_string()),
    };
    match outcome {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("log-host: {e}");
            ExitCode::FAILURE
        }
    }
}

fn wait_for(socket: &Path) -> Result<(), String> {
    let deadline = Instant::now() + Duration::from_secs(10);
    while UnixStream::connect(socket).is_err() {
        if Instant::now() > deadline {
            return Err(format!("{} did not start listening", socket.display()));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    Ok(())
}

fn serve(args: &[String]) -> Result<(), String> {
    let (mut config_home, mut socket, mut door) = (None, None, None);
    let (mut read_only, mut clock_from) = (false, None);
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        let mut value = || args.next().ok_or_else(|| format!("{arg} needs a value"));
        match arg.as_str() {
            "--config-home" => config_home = Some(PathBuf::from(value()?)),
            "--socket" => socket = Some(PathBuf::from(value()?)),
            "--door" => door = Some(PathBuf::from(value()?)),
            "--read-only" => read_only = true,
            "--clock-from" => {
                clock_from = Some(Timestamp::parse(value()?).map_err(|e| e.to_string())?)
            }
            _ => return Err(USAGE.to_string()),
        }
    }
    let config_home = config_home.ok_or(USAGE)?;
    let socket = socket.ok_or(USAGE)?;
    let host = Arc::new(Host::open(&Options {
        config_home,
        read_only,
        clock_from,
    })?);

    let kernel = host.kernel.clone();
    let path = socket.clone();
    std::thread::spawn(move || ikigai_ipc::serve_shared(kernel, &path));
    wait_for(&socket)?;
    let state = if read_only {
        "read-only, no writer open".to_string()
    } else {
        match host.handle.open_segment() {
            Some((segment, _)) => format!("writing {segment}"),
            None => "logging off".to_string(),
        }
    };
    println!("log-host: {} ({state})", socket.display());

    let Some(door) = door else {
        // Nothing to stop it but the process ending.
        loop {
            std::thread::park();
        }
    };
    let listener = std::os::unix::net::UnixListener::bind(&door).map_err(|e| e.to_string())?;
    println!("log-host: door on {}", door.display());
    let served = Door { host }.serve(listener).map_err(|e| e.to_string());
    // A socket file outlives its listener, and would refuse the next start.
    let _ = std::fs::remove_file(&door);
    served
}

fn door_flag(args: &[String]) -> Result<(&Path, &[String]), String> {
    match args {
        [flag, door, rest @ ..] if flag == "--door" => Ok((Path::new(door), rest)),
        _ => Err(USAGE.to_string()),
    }
}

fn ask(args: &[String]) -> Result<(), String> {
    let (door, rest) = door_flag(args)?;
    if rest.len() < 2 {
        return Err(USAGE.to_string());
    }
    let answer = call(door, &format!("ask {}", rest.join(" "))).map_err(|e| e.to_string())?;
    print!("{answer}");
    Ok(())
}

fn stop(args: &[String]) -> Result<(), String> {
    let (door, _) = door_flag(args)?;
    print!("{}", call(door, "stop").map_err(|e| e.to_string())?);
    Ok(())
}
