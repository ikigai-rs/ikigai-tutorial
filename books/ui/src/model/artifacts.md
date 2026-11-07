# Artifacts: from a directory or compiled in

*This chapter is not written yet.*

It will cover the inspector's **artifacts**: the XSLT stylesheets, templates, script and CSS
stylesheet used in [A page is a composed resource](composed.md). These are resources delivered
two ways under the same names: compiled into a binary, or read from a
directory a host binds, file by file, with the directory winning per artifact. The code is
already there and tested: `crates/inspector` reads `urn:iki:tutorial:inspect:dir:{file}` first
and falls back to its compiled-in bytes only on that file's absence, and a test renders both
deliveries and compares the bytes.

From the repository root, run `cargo run -p inspector-htmx -- --artifacts DIR`, replacing
`DIR` with the path to your artifact directory. The host prints `inspector: http://127.0.0.1:8071/`;
open that address to use the inspector. A directory can contain just the files you want to
override; the others stay compiled in.

The chapter will also cover what a directory may not do (change the door's CSP or headers),
and hot reload: a write to an artifact through the kernel cuts its golden thread, so the next
read of a page over it recomputes that page. It waits on a file watcher for changes made
outside the kernel; today the directory is read on every request instead, which is correct
and not cached.
