//! The code taught by **Part III — Beyond one host**.
//!
//! Part II left a module talking to its host through an in-process transport, and ended
//! by saying that a real one is on the other side of *something*. This crate is the
//! other side of something: a kernel behind a Unix socket, the authority a certificate
//! mints when the socket is a network, and the three relationships a mounted peer can
//! have with the local namespace.
//!
//! As in the earlier parts, the book pulls these listings in by anchor, so there is one
//! copy and it is the one that compiles. What is different here is how much of it is
//! *tested*: the socket module really serves a kernel and really resolves through it,
//! and the mount module asserts each of the three failure rules that make a degrading
//! mount honest. Those are the claims a reader has no way to check by reading.

pub mod identity;
pub mod mounts;
#[cfg(unix)]
pub mod socket;
