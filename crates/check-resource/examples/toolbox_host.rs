//! A scratch host for the chapter's images (`a11y/capture-toolbox.mjs`, ledger #2980): the chapter's
//! own space with a toolbox build bound at `urn:iki:toolbox:dist:{path}`, and `/toolbox/{…}` routed
//! to `urn:toolbox:{…}` by the mechanical rule, each segment percent-decoded and joined by `:`, with
//! `TOOLBOX_CSP` on the route.
//!
//! cargo run -p check-resource --example toolbox_host -- <toolbox build directory> [<scope>…]
//!
//! It listens on a loopback port the system picks and prints its address. Every request runs under
//! one caller, who holds the scopes named: a host with one reader, or one stranger. It is not a
//! server to deploy: one thread, no TLS, no sign-in, nothing kept.

use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use futures::executor::block_on;
use ikigai_core::{
    ArgRef, Capability, EndpointSpace, Error, Fallback, FnEndpoint, Invocation, Iri, Kernel,
    ReprType, Representation, Request, Space, UriTemplate, Verb,
};

/// The build directory, at the name the toolbox reads it from. A name that is not one plain file
/// name is not found: the toolbox asks only for the files on its allowlist.
fn build(dist: PathBuf) -> EndpointSpace {
    let files = FnEndpoint::new("toolbox-build", move |inv: &Invocation<'_>| {
        let name = inv.bindings.get("path").unwrap_or_default();
        let missing = || Error::Endpoint(format!("the build has no `{name}`"));
        if Path::new(name).file_name().and_then(|n| n.to_str()) != Some(name) {
            return Err(missing());
        }
        let bytes = std::fs::read(dist.join(name)).map_err(|_| missing())?;
        Ok(Representation::new(
            ReprType::new("application/octet-stream"),
            bytes,
        ))
    });
    EndpointSpace::new().bind(
        UriTemplate::parse("urn:iki:toolbox:dist:{path}").expect("a valid URI template"),
        files,
    )
}

fn percent_decoded(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let hex = (bytes[at] == b'%')
            .then(|| text.get(at + 1..at + 3))
            .flatten()
            .and_then(|hex| u8::from_str_radix(hex, 16).ok());
        match hex {
            Some(byte) => {
                out.push(byte);
                at += 3;
            }
            None => {
                out.push(bytes[at]);
                at += 1;
            }
        }
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `/toolbox/check/{iri}?shapes={iri}` as the request the kernel is issued. `None` is outside the
/// route.
fn routed(target: &str) -> Option<Request> {
    let (path, query) = target.split_once('?').unwrap_or((target, ""));
    let rest = path.strip_prefix("/toolbox/")?;
    let segments: Vec<String> = rest.split('/').map(percent_decoded).collect();
    let name = Iri::parse(format!("urn:toolbox:{}", segments.join(":"))).ok()?;
    let mut request = Request::new(Verb::Source, name);
    for pair in query.split('&').filter(|pair| !pair.is_empty()) {
        let (arg, value) = pair.split_once('=').unwrap_or((pair, ""));
        request = request.with_arg(
            percent_decoded(arg),
            ArgRef::Inline(percent_decoded(value).into_bytes()),
        );
    }
    Some(request)
}

fn answer(stream: &mut TcpStream, status: &str, media_type: &str, body: &[u8]) {
    let head = format!(
        "HTTP/1.1 {status}\r\ncontent-type: {media_type}\r\ncontent-length: {}\r\n\
         content-security-policy: {}\r\nconnection: close\r\n\r\n",
        body.len(),
        ikigai_toolbox::TOOLBOX_CSP,
    );
    // A browser that went away is no failure of the host's.
    let _ = stream.write_all(head.as_bytes());
    let _ = stream.write_all(body);
}

fn serve(mut stream: TcpStream, kernel: &Kernel, caller: &Capability) {
    let Some(target) = read_target(&stream) else {
        return answer(&mut stream, "400 Bad Request", "text/plain", b"not a GET");
    };
    let Some(request) = routed(&target) else {
        return answer(
            &mut stream,
            "404 Not Found",
            "text/plain",
            b"not a toolbox path",
        );
    };
    match block_on(kernel.issue(request, caller)) {
        Ok(reply) => answer(
            &mut stream,
            "200 OK",
            &reply.repr_type.media_type,
            &reply.bytes,
        ),
        Err(error) => {
            let status = match error {
                Error::Denied(_) => "403 Forbidden",
                Error::Unresolved(_) => "404 Not Found",
                _ => "500 Internal Server Error",
            };
            let words = error.to_string();
            answer(
                &mut stream,
                status,
                "text/plain; charset=utf-8",
                words.as_bytes(),
            );
        }
    }
}

/// The request line's target, with the headers read off and ignored.
fn read_target(stream: &TcpStream) -> Option<String> {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).ok()?;
    let target = line.strip_prefix("GET ")?.split(' ').next()?.to_string();
    loop {
        let mut header = String::new();
        if reader.read_line(&mut header).ok()? == 0 || header.trim().is_empty() {
            return Some(target);
        }
    }
}

fn main() {
    let mut args = std::env::args().skip(1);
    let dist = args
        .next()
        .expect("usage: toolbox_host <toolbox build directory> [<scope>…]");
    let caller = Capability::scoped(args.collect::<Vec<_>>());
    let kernel = Kernel::new(Arc::new(Fallback::new(vec![
        Arc::new(check_resource::space()) as Arc<dyn Space>,
        Arc::new(build(PathBuf::from(dist))) as Arc<dyn Space>,
    ])));
    let listener = TcpListener::bind("127.0.0.1:0").expect("a loopback port");
    let address = listener.local_addr().expect("the port it took");
    println!("http://{address}");
    std::io::stdout().flush().expect("stdout");
    for stream in listener.incoming().flatten() {
        serve(stream, &kernel, &caller);
    }
}
