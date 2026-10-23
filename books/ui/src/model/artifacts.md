# Artifacts: from a directory or compiled in

*This chapter is not written yet.*

It will cover the inspector's **artifacts** (its stylesheets, templates, script and stylesheet)
as resources delivered two ways under the same names: compiled into a binary, or read from a
directory a host binds, file by file, with the directory winning per artifact. The code is
already there and tested: `crates/inspector` reads `urn:iki:tutorial:inspect:dir:{file}` first
and falls back to its compiled-in bytes only on that file's absence, and a test renders both
deliveries and compares the bytes. `cargo run -p inspector-htmx -- --artifacts DIR` runs it.

The chapter will also cover what a directory may not do (change the door's CSP or headers),
and hot reload: a write to an artifact cuts its golden thread, so every page over it
recomputes. It waits on a file watcher for changes made outside the kernel; today the
directory is read on every request instead, which is correct and not cached.
