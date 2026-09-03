//! # ikigai, Book 1 — Getting Started
//!
//! A resource-oriented computing kernel in Rust, taught from the outside in.
//!
//! ## What ikigai is, in one paragraph
//!
//! Everything is a **resource** named by a URI. You never call a function; you
//! **resolve** a name against a **kernel**, which routes it to a bound **endpoint** and
//! hands back a **representation** — bytes plus a media type. There are five verbs, not
//! a vocabulary of methods: `Source` (read), `Sink` (write), `Exists`, `Delete`, and
//! `Meta` (describe yourself). **Transreptors** convert one representation into another,
//! so the same resource can arrive as text, as Turtle, or as HTML. **Golden threads**
//! track what a result was derived from, so a write invalidates exactly what it should.
//! **Capabilities** gate authority and attenuate as they pass down a call chain.
//!
//! That is the whole model. The rest is consequences.
//!
//! ## Why bother
//!
//! Because naming a thing and *resolving* a thing are different acts, and separating
//! them buys you a lot at once. If a computation has a name, it can be cached, traced,
//! substituted, authorized, converted, and moved to another machine without its caller
//! knowing. A function call gives you none of that; it is a jump with arguments.
//!
//! The wager of this project is that a system where every step is a resolvable name is
//! *cheaper to reason about at scale* than one where the steps are opaque calls — and
//! that this matters most now that programs are being assembled by agents, which need
//! exactly the things resolution gives you: a machine-readable catalog of what can be
//! done, an enforceable boundary on what may be done, and provenance for what was done.
//!
//! ## How to read this book
//!
//! **The book is its own rustdoc.** Each chapter is a module; the prose is in the
//! module's docs; the examples are doctests. That means `cargo test` compiles and runs
//! every example in this book — a tutorial that lies stops building. Read it with:
//!
//! ```text
//! cargo doc -p ikigai-book-getting-started --open
//! ```
//!
//! or read the source files directly, which are ordered to be read top to bottom.
//!
//! ## Chapters
//!
//! 1. [`ch01_resolution`] — the model: names, verbs, representations
//! 2. [`ch02_running_it`] — get a kernel in front of you
//! 3. [`ch03_hello_resource`] — your first endpoint, `toCamel`
//! 4. [`ch04_self_description`] — why an endpoint describes itself
//! 5. [`ch05_binding`] — binding a name, and building a host of your own
//! 6. [`ch06_configuration`] — where configuration lives, and the channel that is banned
//! 7. [`ch07_file_workspace`] — the file workspace and its jail
//! 8. [`ch08_whats_next`] — loadable modules, and where to go from here
//!
//! ## Conventions
//!
//! Shell commands assume you are at the root of this repository. Code that appears in a
//! chapter is *the same code the crate compiles* — chapters do not paraphrase.

pub mod ch01_resolution;
pub mod ch02_running_it;
pub mod ch03_hello_resource;
pub mod ch04_self_description;
pub mod ch05_binding;
pub mod ch06_configuration;
pub mod ch07_file_workspace;
pub mod ch08_whats_next;

pub use ch03_hello_resource::to_camel;
pub use ch05_binding::space;
