//! `inspector-htmx`: serve the kernel inspector over HTTP. See the library's docs.

#[tokio::main]
async fn main() {
    let options = match inspector_htmx::parse_args(std::env::args().skip(1)) {
        Ok(options) => options,
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(2);
        }
    };
    let listener = match tokio::net::TcpListener::bind(options.http).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("inspector-htmx: cannot listen on {}: {e}", options.http);
            std::process::exit(1);
        }
    };
    match listener.local_addr() {
        // The banner names the bound address, so a test that asked for port 0 reads it here.
        Ok(addr) => println!("inspector: http://{addr}/"),
        Err(e) => eprintln!("inspector-htmx: {e}"),
    }
    if let Err(e) = inspector_htmx::serve(listener, options.artifacts).await {
        eprintln!("inspector-htmx: {e}");
        std::process::exit(1);
    }
}
