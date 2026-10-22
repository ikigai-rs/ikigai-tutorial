//! The two pictures "A space that changes while it runs" shows are `urn:diagram:kernel` as the
//! chapter's host answers it, before and after its first declaration, byte for byte. On a
//! mismatch the picture the host draws is written beside the build and the failure names the
//! command that commits it; nothing rewrites it from inside a test.

use live_space::{source, Host, FOUR_DOORS};

fn holds(file: &str, drawn: &str) {
    let committed = format!(
        "{}/../../books/ikigai/src/applied/{file}",
        env!("CARGO_MANIFEST_DIR")
    );
    if std::fs::read_to_string(&committed).ok().as_deref() == Some(drawn) {
        return;
    }
    let out = format!("{}/{file}", env!("CARGO_TARGET_TMPDIR"));
    std::fs::write(&out, drawn).expect("write the picture the host draws");
    panic!("the committed {file} is not what the host draws.\nIf the change is intended:  cp {out} {committed}");
}

#[test]
fn the_committed_pictures_are_what_the_host_draws() {
    let mut host = Host::start("workshop");
    let empty = host
        .read(source("urn:diagram:kernel", &[]))
        .expect("the picture");
    holds("live-space-empty.svg", &empty);

    host.write(FOUR_DOORS);
    let doors = host
        .read(source("urn:diagram:kernel", &[]))
        .expect("the picture");
    holds("live-space-doors.svg", &doors);
}
