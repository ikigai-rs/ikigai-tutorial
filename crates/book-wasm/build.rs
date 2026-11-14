fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var("CARGO_CFG_TARGET_ARCH").as_deref() == Ok("wasm32") {
        // XSLT runs inline on wasm: there is no worker thread whose stack ikigai-xslt
        // can size. Its 0.8 release stack budget is about 45 MiB (limits::xslt_stack_size),
        // so reserve 48 MiB here instead of wasm-ld's 1 MiB default. Without this, the
        // RSS page traps in xrust's XPath parser and leaves subsequent cells running.
        // Keep this local to the kernel that binds XSLT, including its operations copy.
        println!("cargo:rustc-link-arg-cdylib=-zstack-size=50331648");
    }
}
