# Exercises: getting started

Four ways to find out whether Part I landed. None of them needs anything beyond this
repository.

## First: where your code goes, and how to run it

This book has shown you how to run *its* code four times and never once how to run yours.
So, before the exercises:

**Your endpoints go in `crates/your-endpoints/src/lib.rs`.** Not in `hello-camel` — that
is the file the chapters quote, and editing it would change what the book says. Nothing
quotes `your-endpoints`, and a test makes sure nothing starts to; it exists to be broken.
It starts with one worked endpoint, `word-count`, in the shape the book teaches, and a
`space()` that chains onto Part I's, so `camel-case` and `title` answer from your host
too. An endpoint is a function plus a description, and it reaches a name through one more
line there:

```rust,ignore
pub fn space() -> EndpointSpace {
    hello_camel::space()
        .bind(Exact::new("urn:iki:tutorial:yours:word-count"), word_count())
        .bind(Exact::new("urn:iki:tutorial:yours:your-thing"), your_thing()) // ← yours
}
```

The prefix `urn:iki:tutorial:yours:` is a stand-in for one you own. Rename it the day this
stops being a tutorial, and notice nothing else has to change.

One thing your crate cannot do is run in this page. The kernel the published book carries
is Part I's space compiled once, when the book was built; your code is compiled on your
machine, so the loop for it is `cargo test` and the binary, not a Run button.

**The fastest loop is a test**, because a test needs no wiring at all:

```bash
cargo test -p your-endpoints
```

The test module at the bottom of that file already contains the four lines that build a
kernel, resolve a name and hand back a `String` — the `text_of` helper. Copy it, change
the IRI, and you have a way to ask your endpoint a question and assert the answer.

**To see it from a shell**, `crates/your-endpoints/src/bin/yours.rs` resolves one name
and prints the result. Point its `NAME` at yours and run:

```bash
cargo run -p your-endpoints -- "a few words here"
```

**To watch the host describe itself**, ask for the catalog:

```bash
cargo run -p your-endpoints -- --catalog
```

Your endpoint appears in it the moment it is bound, described exactly as its `Description`
says — because the catalog is assembled from those descriptions rather than maintained
beside them. `urn:kernel:actions` lists the names alone. Both answer here because the
tutorial host was built with a Meta renderer; a bare `Kernel::new` has none, and would
answer `no Meta renderer configured` to either. [Binding, and a host of your
own](binding.md) shows the one line that makes the difference.

> ⚠ The catalog shows what you *declared*. A test that asserts `your_thing().describe()`
> is the same instrument pointed at one endpoint, and it is the one that runs on every
> `cargo test` — [Hello, resource](hello-resource.md) does exactly that with
> `camel_case().describe().id`.

<!-- urn-gate: illustration urn:iki:tutorial:yours:your-thing — a stand-in for whatever
     the reader binds; the point of the line is the shape, not the name. -->
<!-- urn-gate: illustration urn:iki:tutorial:yours:lower-camel-case — exercise 1 asks the
     reader to write this one; nothing in the repository binds it. -->
<!-- urn-gate: illustration urn:iki:tutorial:yours:now — exercise 4's deliberately wrong
     clock, written by the reader. -->

---

## 1. Strict lowerCamelCase

`camel-case` never lower-cases anything: `"Hello WORLD"` becomes `"HelloWORLD"`, because it
treats the input's own casing as meaningful. Write `lower-camel-case`, which does not — and
decide what it should do with `"XMLHttpRequest"`. There is no obviously right answer, which
is the point of the exercise.

- **File** — `crates/your-endpoints/src/lib.rs`; bind it at
  `urn:iki:tutorial:yours:lower-camel-case`. `hello_camel::camel` is the pure function
  under `camel-case`, if you want to start from it rather than from scratch.
- **Run** — `cargo test -p your-endpoints`
- **Right when** — your own test asserts the answer you chose for `"XMLHttpRequest"`, *and*
  `cargo test -p hello-camel` still passes untouched. If `camel-case`'s behavior changed,
  you edited the wrong crate.
- **Read again** — [Hello, resource](hello-resource.md) for the shape of an
  implementation, [Binding, and a host of your own](binding.md) for the bind line.

<details>
<summary>Hint</summary>

The mechanical part is small: `char::to_lowercase` returns an iterator for the same reason
`to_uppercase` does, so it composes the same way — `output.extend(first.to_lowercase())`.

The part that is actually the exercise is that `"XMLHttpRequest"` has no correct answer,
only a *stated* one. `"xMLHttpRequest"` (lower-case the first character), `"xmlHttpRequest"`
(treat a run of capitals as one word) and `"xmlhttprequest"` (lower-case everything before
re-humping) are all defensible, and they disagree about what a "word" is.

So write the assertion first, in a test named after the decision:

```rust,ignore
#[test]
fn a_leading_run_of_capitals_is_one_word() {
    assert_eq!(lower_camel("XMLHttpRequest"), /* the answer you are choosing */);
}
```

A test name is where a decision like this survives, because it is the only comment the
compiler reads. Correct output for the easy cases is unambiguous and worth pinning too:
`"resource oriented computing"` → `"resourceOrientedComputing"`, `"Hello WORLD"` →
whatever your rule says, and `"   "` → `""` rather than an error.

</details>

---

## 2. A second argument

Add an optional `separator`, so a caller can split on something other than whitespace.
Declare it in the description with `optional()` and a `default_value`.

- **File** — `crates/your-endpoints/src/lib.rs`, in both halves of your `lower-camel-case`
  from exercise 1 (or of `word-count`, if you skipped it — a `separator` makes sense
  there too): the `ArgSpec` list in the description *and* the implementation.
- **Run** — `cargo test -p your-endpoints`
- **Right when** — resolving with no `separator` gives the same answers as before, and
  resolving with `separator=","` camel-cases `"a,b,c"` into `"aBC"`. A test asserting your
  `ArgSpec` is present is worth writing too — `your_thing().describe().inputs` is a list
  you can look at.
- **Read again** — [Why an endpoint describes itself](self-description.md), the ArgSpecs
  section.

<details>
<summary>Hint</summary>

The trap here is the interesting part of the exercise, and it will not announce itself:
**a declared `default_value` is not injected into the invocation.** Nothing fills the
argument in for you. Declare `ArgSpec::new("separator").optional().default_value(" ")` and
then call `inv.inline_str("separator")` on a request that omitted it, and you get
`Err(MissingArgument("separator"))` — not `" "`.

So the description and the implementation each carry half of the same decision, and only
one of them is checked by a compiler:

```rust,ignore
let separator = inv.inline_str("separator").unwrap_or(/* your default */);
```

Which means the failure mode to watch for is the two halves disagreeing — a description
that promises one default while the code applies another. Nothing in the system catches
that; a test that resolves without the argument and asserts the declared default's
behaviour does.

`urn:kernel:actions` will *not* show your change, by the way: it lists names, and the name
was already there. It is the description that changed, and in this host you read that
through `describe()`.

</details>

---

## 3. Take a resource, not a string

Make your endpoint accept `in` as **a reference to another resource** as well as an inline
value: given `ArgRef::Reference(urn:iki:tutorial:title)`, resolve that name through the
kernel and camel-case whatever it returns.

- **File** — `crates/your-endpoints/src/lib.rs`. `urn:iki:tutorial:title` already
  answers from your host — it is the resource [What resolution buys you](payoff.md) writes
  to, and your `space()` chains onto the one that binds it — so the only new thing is a
  version of `camel-case` that can take a reference to it.
- **Run** — `cargo test -p your-endpoints`
- **Right when** — one test passes the text inline and gets the answer it always got, and
  a second passes `ArgRef::Reference` and gets the camel-cased contents of `title`. Then
  `Sink` a new title and resolve again: the by-reference answer follows the write, the
  inline one cannot.
- **Read again** — [Hello, resource](hello-resource.md) on `inline_str`,
  [What resolution buys you](payoff.md) for `camel-title`, which is this exercise with the
  reference hard-wired, and [The callback](../modules/the-callback.md), which is the same
  move seen from a module.

<details>
<summary>Hint</summary>

This is not free, and finding out why is the exercise. `inv.inline_str("in")` asks for an
*inline* value and refuses anything else — hand it a `Reference` and you get
`invalid argument in: expected an inline value`. An endpoint takes resources only by
opting in:

```rust,ignore
match inv.request.args.get("in") {
    Some(ArgRef::Reference(iri)) => { let repr = inv.source(iri).await?; /* … */ }
    Some(ArgRef::Inline(bytes)) => { /* what you do today */ }
    _ => return Err(Error::MissingArgument("in".into())),
}
```

Two consequences fall straight out of that shape.

`inv.source(…)` is **async**, so the endpoint is no longer an `FnEndpoint` — it becomes an
`AsyncFnEndpoint`, authored as `|inv| Box::pin(async move { … })`. The type change is the
whole cost, and it is why the book's first endpoint is not written this way.

A `Reference` carries **no arguments of its own** — it is a name, not a call — so the
resource it names has to resolve on its own. `urn:iki:tutorial:title` returning a constant
works. `urn:iki:fn:toUpper` does not: it needs an `in` argument, and there is nowhere in a
reference to put one. (`urn:iki:fn:toUpper?in=hello` does not resolve either; that binding
matches the name exactly, query string included.)

Correct output looks like this: with `title` still at its starting text, resolving your
endpoint with `in` as a reference to it returns `"resourceOrientedComputing"` — the same
bytes the inline call gives, produced without the caller ever holding the text. And
because `inv.source` recorded the dependency, the answer is cached under `title`'s golden
thread: write to `title` and it recomputes, exactly as `camel-title` did.

</details>

---

## 4. Break cacheability on purpose

Write an endpoint that returns the current time, mark the representation `.cacheable()`,
and watch how convincing a wrong answer looks.

- **File** — `crates/your-endpoints/src/lib.rs`; bind it at `urn:iki:tutorial:yours:now`.
- **Run** — `cargo test -p your-endpoints`
- **Right when** — a test resolves it twice through **one** kernel, with a sleep in
  between, and the two answers are byte-identical. Remove `.cacheable()`, run again, and
  they differ. The test that proves the bug is the one that passes.
- **Read again** — [Resolution](resolution.md) on golden threads, and
  [Hello, resource](hello-resource.md) on what `.cacheable()` asserts.

<details>
<summary>Hint</summary>

The cache lives in the `Kernel`, keyed on the request and the capability — so build the
kernel **once** and issue twice against it. Two `Kernel::new` calls give you two empty
caches and a test that passes for the wrong reason.

```rust,ignore
let kernel = kernel();
let first = resolve(&kernel, "urn:iki:tutorial:yours:now");
std::thread::sleep(Duration::from_millis(50));
let second = resolve(&kernel, "urn:iki:tutorial:yours:now");
assert_eq!(first, second); // ← the bug, asserted
```

Sit with what that assertion means for a moment. Nothing failed. No warning was printed,
no type was wrong, the endpoint is three lines of obviously correct code, and the answer is
a perfectly well-formed timestamp — just not the current one. That is the whole reason
`.cacheable()` is a claim you make rather than an optimization the kernel infers: *same
inputs, same answer, forever*. A clock has no inputs and a different answer every time, so
it is the one thing that can never be it.

Two things worth carrying out of the exercise. **When in doubt, do not cache** — an
uncacheable resource is slow, and a wrongly cached one is quietly incorrect, which is much
harder to notice. And in real code the time should come from `inv.now()`, the clock the
kernel injects, rather than `SystemTime::now()` — a kernel does not only run on your
machine, and `std::time` is not available everywhere it goes.

</details>
