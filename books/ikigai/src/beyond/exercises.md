# Exercises

Three, and all three are written in **your** crate — `crates/your-endpoints/tests/beyond.rs`
— against the code this part quotes, `crates/two-hosts`, which you use and do not edit.
The file already holds the scaffolding an exercise would otherwise spend its time on: a
socket path nothing collides with, a dial loop that waits for a server thread to bind, a
peer that is always down, and one test proving the scaffolding works. The loop is:

```bash
cargo test -p your-endpoints --test beyond
```

> ⚠ Unix only. `ikigai-ipc` is `#![cfg(unix)]`, and so is that file; on another platform
> the whole test target compiles to nothing and passes vacuously, which is the one way a
> test file can lie to you. If `--test beyond` reports zero tests, that is why.

---

## 1. Serve Part I, and reach it

Serve `hello_camel::space()` behind a socket, connect, and resolve
`urn:iki:tutorial:camel-case` through the connection. Then ask the served kernel for a name
it does not bind.

- **File** — `crates/your-endpoints/tests/beyond.rs`; `serve_in_background` and
  `connect_when_up` are already there.
- **Run** — `cargo test -p your-endpoints --test beyond`
- **Right when** — the answer is `resourceOrientedComputing`, the same bytes the
  in-process test in Part I gets; and the unbound name fails with
  `ikigai_core::Error::Unresolved`, not with a transport error.
- **Read again** — [A kernel behind a socket](socket.md), both halves: the same
  `Resolver` trait, and failure arriving typed.

<details>
<summary>Hint</summary>

The connected peer is a `Resolver`, and a `Resolver`'s `issue` takes a `Request` and
gives back a `(Representation, CacheStatus)` pair — the second half is the cache verdict
the CLI prints as `[cached]` or `[computed]`, and you can ignore it here. Bring the trait
into scope (`use ikigai_resolve::Resolver;`) or the method will not be found, which is the
first thing that goes wrong.

The second assertion is the one worth writing carefully. `matches!(err,
Error::Unresolved(_))` is the claim; a looser `is_err()` would also pass if the socket had
simply not been there, and then the test would be asserting nothing this chapter said. The
error crossed a process boundary and kept its type — that is what makes the next exercise
possible.

</details>

---

## 2. Break the prefer-mount, and watch what falls through

Build a prefer-mount over your own space with a peer that is always down, and resolve
`urn:iki:tutorial:yours:word-count` through it. Then make the same mount an override. Then
make the peer *refuse* rather than fail.

- **File** — `crates/your-endpoints/tests/beyond.rs`; `dead(Error::Unavailable)` gives
  you the peer and a counter, and `two_hosts::mounts::{prefer, overriding, compose}` are
  the three shapes.
- **Run** — `cargo test -p your-endpoints --test beyond`
- **Right when** — the prefer-mount answers `3` for `"a b c"` *and the counter shows the
  peer was tried first*; the override fails with `Error::Unavailable`; and a prefer-mount
  over `dead(Error::Denied)` fails with `Error::Denied` — local never answers for a
  refusal.
- **Read again** — [Preferential resolution](preference.md), the three intentions and the
  rules for the third.

<details>
<summary>Hint</summary>

No socket is involved, which is the point of `DeadPeer`: every rule here is about what
happens when the answer is *not* a representation, so the only peer you need is one that
never gives one. Wrap the mount in a `Kernel::new(..)` and issue through it as Part I did.

The three outcomes are three different questions, and it helps to name them before you
assert them. `Unavailable` on a prefer-mount is "the peer is not there, so this machine
answers" — the counter proves *preferring* happened before *falling back*. `Unavailable` on
an override is "you named that machine, and it is not there" — a silent local answer would
be a different question answered. `Denied` on a prefer-mount is "the peer said you may
not" — and a fallback here would turn a capability boundary into a suggestion, which is
why it is the one failure the rule never swallows.

If your override test *passes* the resolution instead of failing it, check the order
`compose` put things in: an override composed after the local space overrides nothing.

</details>

---

## 3. Deny a callback, across the socket

Serve a host whose `urn:iki:tutorial:yours:name` declares `.requires("urn:cap:yours:name")`,
with your module routed under `urn:iki:tutorial:yours:module:`. Connect, and resolve the
module's `greeting` with `name` pointing at that resource — twice: under a capability that
grants the scope, and under one that grants something else.

- **File** — `crates/your-endpoints/tests/beyond.rs`. Build the host in the test rather
  than editing `module.rs`: `name()` there has no `requires`, and Part II's exercise 3
  is about adding one in-process.
- **Run** — `cargo test -p your-endpoints --test beyond`
- **Right when** — with `Capability::root().attenuate(["urn:cap:yours:name"])` the answer
  is `Hello, Ada!`; with `attenuate(["urn:cap:yours:other"])` the resolution fails with
  `Error::Denied`, and the message names both the scope the caller lacked and the resource
  that declared it.
- **Read again** — [Who is asking](identity.md) on carrying a capability and being
  clamped, and [Where this actually stands](../modules/status.md), exercise 3, which is
  this denial without the socket.

<details>
<summary>Hint</summary>

`issue` sends no capability, so it runs as the socket's principal — the owner, root. The
method that carries one is `issue_as(request, &capability)`, and the server resolves under
what you sent, clamped against what the connection is entitled to. Over a Unix socket
that entitlement is root, so the clamp is your attenuation and nothing else; over the QUIC
transport it is what the certificate was enrolled for.

Now look at *where* the denial happens. The module is reached — nothing about the
module's own endpoint requires anything. It is the module's **callback** to the host's
`name` that is refused, mid-invocation, because the capability the invocation is carrying
is the one you sent, narrowed on the way down, and it does not grant what that resource
declared. The message says so:

```text
denied: capability does not grant `urn:cap:yours:name` (declared by `urn:iki:tutorial:yours:name`)
```

Three things crossed the process boundary intact for that sentence to arrive: the
capability you carried, the attenuation down the call chain inside the served kernel, and
the typed error on the way back. That is Part II's claim — a module borrows the caller's
authority and can only narrow it — holding across a socket.

</details>
