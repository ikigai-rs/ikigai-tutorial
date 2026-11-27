# Introduction

This book is for someone who wants to teach with ikigai: to write a course, publish it so that it
reads the same way for years, let people work through it at their own pace, review what they
submit, and give them a credential they can show to anyone. It covers the open source pieces that
do that, and only those, as their own repositories publish them:

- [`ikigai-learn`](https://github.com/ikigai-rs/ikigai-learn): courses as resources. A course is
  a list of names, a learner's work is a [gonk](../gonk/index.html) ledger, review is a
  tuplespace, and progress is a question the kernel answers, never stored state.
- [Lectern](https://github.com/bsletten/lectern-slides): Markdown slide decks, assembled from
  slide ranges and partials and rendered with reveal.js. A unit of a course can be a Lectern deck.
- [`ikigai-credential`](https://github.com/ikigai-rs/ikigai-credential): Open Badges 3.0
  credentials, issued and verified as resources, so a credential anyone can check is also a graph
  a kernel can query.

It does not teach resources. What a resource is, how a kernel resolves one, what a capability or a
golden thread is: [The ikigai Book](../index.html) teaches all of that from the first endpoint
up, and this book links there rather than saying it twice.

## What is built today, and what is planned

Most of what this book describes is a plan, and each chapter says which parts are which. As of
2026-11-27:

- **ikigai-learn 0.1.0** is its first release, on crates.io and tagged `v0.1.0`. What it builds is
  the *release grant*: a learner shares part of their own record with one other party, scoped,
  expiring and revocable, and four views answer that party and nobody else. The rest of the
  course model (units, questions, exercises, the review queue, mastery and progress) is designed,
  named in the crate's resource map, and not built.
- **ikigai-credential 0.1.0** is its first release too. It issues and verifies `eddsa-rdfc-2022`
  proofs over Open Badges 3.0 credentials and binds five resources: an issuer, an achievement, the
  credential, the issue Sink and the verifier. A badge it issues verifies in SpruceID's `ssi` and
  in the Digital Credentials Consortium's `verifier-core`, and one they sign verifies in it, all in
  that repository's CI with no network. Status lists, the credential's public page and several
  issuers in one host are planned.
- **Lectern 0.1.0** is on PyPI as `lectern-slides`. Slide ranges, partials, the accessibility
  audit and the text transcript are in that release. Two things this book's units need, a code
  fence rendered as a custom element and a reveal.js deck that loads nothing from a CDN, are on
  Lectern's `main` branch and listed as unreleased in its changelog; [Lectern decks as
  units](units/lectern.md) says so where it uses them.

The pages in the contents with no link are the chapters still to come. The chapters that run code
against `ikigai-learn` and `ikigai-credential` (issuing and verifying a credential, the interop,
and a course release served by ikigai-learn) are among them: every listing in these books is
compiled and run against published crates, and those two crates reached crates.io on the day this
book began, on a newer `ikigai-core` line than the one this repository's crates build on. Until
those chapters land, this book shows the crates' shapes as text: their names, their tables and
their rules, quoted from their own READMEs.

## How to read it

Read [The course model](course/model.md) first: every later chapter uses its names. [A course
release, pinned to a tag](course/release.md) is the part that is built, and [Lectern decks as
units](units/lectern.md) is where a unit comes from.

A name under `urn:learn:` or `urn:credential:` in this book is a name a kernel resolves once it
mounts the crate that binds it; none of them resolves on the open web.

The book changes as the crates do. [What's new](whats-new.md) lists each change, most recent
first, with the pages to read for it.
