//! `ttt-host`: serve the tutorial's tic-tac-toe over HTTP and IPC. See the crate docs.

use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("ttt-host: {message}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let options = ttt_host::parse_args(std::env::args().skip(1))?;
    // Every peer store is connected to and checked against the contract HERE, before any
    // listener opens: a host with a broken store does not start.
    let host = std::sync::Arc::new(ttt_host::Host::build(&options)?);
    if !options.commands.is_empty() {
        print!("{}", host.run(&options.commands));
        return Ok(());
    }
    let socket = match &options.socket {
        Some(socket) => socket.clone(),
        None => ttt_host::default_socket()?,
    };
    let http = match options.http {
        Some(addr) => addr,
        None => ttt_host::DEFAULT_HTTP.parse().expect("a constant address"),
    };

    let ipc_host = std::sync::Arc::clone(&host);
    let ipc_socket = socket.clone();
    std::thread::spawn(move || {
        if let Err(e) = ttt_host::serve_ipc(&ipc_host, &ipc_socket) {
            eprintln!("ttt-host: the IPC socket {}: {e}", ipc_socket.display());
        }
    });

    let runtime = tokio::runtime::Runtime::new().map_err(|e| format!("a runtime: {e}"))?;
    runtime.block_on(async {
        let listener = tokio::net::TcpListener::bind(http)
            .await
            .map_err(|e| format!("--http {http}: {e}"))?;
        // The address the listener GOT, not the one asked for: `--http 127.0.0.1:0` binds a
        // port the kernel chooses, and this line is how a script finds out which.
        let http = listener
            .local_addr()
            .map_err(|e| format!("--http {http}: {e}"))?;
        let games: Vec<&str> = host.games().collect();
        eprintln!("ttt-host: http://{http}/ (the root game); games at /game/<id>/: {games:?}");
        eprintln!("ttt-host: IPC on {}", socket.display());
        ttt_host::serve_http(&host, listener)
            .await
            .map_err(|e| format!("serving HTTP: {e}"))
    })
}
