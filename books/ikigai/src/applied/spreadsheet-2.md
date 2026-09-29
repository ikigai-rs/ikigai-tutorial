# A spreadsheet, II: a formula is code as a resource

[Part I](spreadsheet-1.md) stored what a person typed and passed it through as the cell's
value. That is enough for `42` and for `Rent`. It is not enough for `=A1*2`, which is not a
value but a program: it has to be parsed, checked and run, and what it produces depends on
other cells.

The usual way to handle that is to parse the formula when it is typed and keep the parse tree
in the cell, beside the text. This part does something else. The compiled form of a formula is
a **resource** of its own, with a name, and compiling is a **transreption**: the kernel finds
the compiler by what it converts, runs it, and caches what it produces like any other answer.
NetKernel treated code this way (its Java and Groovy were compiled by transreptors, and the
compiled classes were cached representations), and the reason it matters is the same here:
when a formula changes, exactly that formula is compiled again, and the kernel works out
which one without being told.

## The formula language

The sheet's formulas are small on purpose: numbers, cells, `+ - * /`, parentheses, a minus
sign in front of anything, and `SUM` over any mix of cells, numbers and ranges (`A1:B3`).
That is enough to show a formula reading other cells, a range reading a rectangle of them,
and every kind of error a formula can make.

A formula compiles to an expression:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/formula.rs:expr}}
```

and the expression is written as an **s-expression**, in the one spelling this crate reads
back:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/formula.rs:print}}
```

S-expressions are the ecosystem's way of writing code down as data (`ikigai-sexpr` compiles
them to SPARQL), and they suit a compiled formula because they make the structure visible: the
precedence of `*` over `+`, which the text only implies, is the nesting of the lists. The
sheet carries its own small reader rather than `ikigai-sexpr`'s, because a formula needs
decimals and that datum has only integers.

The compiler itself is a short recursive-descent parser, and its one entry point never
fails:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/formula.rs:compile}}
```

The section on errors says why it does not.

## The compiler is a transreptor

Wrapped as an endpoint, the compiler takes formula text as `content` and answers the
s-expression. What makes it a transreptor is its description:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:compile}}
```

`.transreptor([FORMULA_TYPE], [SEXPR_TYPE])` says it converts `text/x-formula` into
`text/x-sexpr`, and that declaration is what the kernel selects on, as in
[Transreption](../building/transreption.md). `describe` shows it, as the description's Turtle;
the lines to look for are `ik:transreptsFrom`, `ik:transreptsTo` and `ik:lossless`:

<div class="ikigai-run" data-cmd='describe urn:iki:tutorial:sheet:compile'>
<pre class="ikigai-run-expected">@prefix ik: &lt;https://ikigai-rs.dev/ns#&gt; .
&#32;
&lt;urn:ikigai:endpoint:sheet-compile&gt; a ik:Endpoint, ik:Transreptor ;
    ik:id &quot;sheet-compile&quot; ;
    ik:title &quot;Compile a formula&quot; ;
    ik:summary &quot;Compiles a formula (=A1*2) to an s-expression ((* A1 2)).&quot; ;
    ik:verb &quot;Source&quot;, &quot;Meta&quot; ;
    ik:output &quot;text/x-sexpr&quot; ;
    ik:transreptsFrom &quot;text/x-formula&quot; ;
    ik:transreptsTo &quot;text/x-sexpr&quot; ;
    ik:lossless false ;
    ik:input &lt;urn:ikigai:endpoint:sheet-compile:input:content&gt; ;
    ik:input &lt;urn:ikigai:endpoint:sheet-compile:input:as&gt; ;
    ik:action &lt;urn:ikigai:endpoint:sheet-compile:action:source&gt; .
&#32;
&lt;urn:ikigai:endpoint:sheet-compile:input:content&gt; ik:inputName &quot;content&quot; ;
    ik:source &quot;argument&quot; ;
    ik:required true ;
    ik:summary &quot;the formula, starting with =&quot; ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#string&gt; .
&#32;
&lt;urn:ikigai:endpoint:sheet-compile:input:as&gt; ik:inputName &quot;as&quot; ;
    ik:source &quot;argument&quot; ;
    ik:required false ;
    ik:summary &quot;the target type; only text/x-sexpr&quot; ;
    ik:class &lt;http://www.w3.org/2001/XMLSchema#string&gt; .
&#32;
&lt;urn:ikigai:endpoint:sheet-compile:action:source&gt; a ik:Action ;
    ik:verb &quot;Source&quot; ;
    ik:output &quot;text/x-sexpr&quot; ;
    ik:input &lt;urn:ikigai:endpoint:sheet-compile:input:content&gt; ;
    ik:input &lt;urn:ikigai:endpoint:sheet-compile:input:as&gt; .
[computed]</pre>
</div>

`.lossy()` is the honest half. A transreption proper changes form without changing what is
represented, so the original could be recovered from what comes out. Compiling cannot promise
that: `=a1 +2` and `=A1+2` compile to the same `(+ A1 2)`, and nothing in the output says which
was typed. So the compiler declares itself a projection, and the kernel will not route through
it unless the caller consents.

## The compiled formula is a resource

`formula:{ref}` is the cell's formula, compiled. It reads the cell's input, asks the kernel
for a plan from the input's type to `text/x-sexpr`, and runs the plan:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:formula}}
```

This is where Part I's media type earns its place. The input says it is `text/x-formula`, the
compiler says it converts `text/x-formula`, and the kernel joins the two. `formula:{ref}` never
names the compiler, and a different one bound in its place would be found the same way. The consent to the lossy step is given here, by the one endpoint that
knows losing the spelling is what compiling means.

Type a formula, and ask for its compiled form twice:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A1 5
sink urn:iki:tutorial:sheet:input:A2 =a1 * 2
source urn:iki:tutorial:sheet:formula:A2
source urn:iki:tutorial:sheet:formula:A2'>
<pre class="ikigai-run-expected">5
[uncacheable]
=a1 * 2
[uncacheable]
(* A1 2)
[computed]
(* A1 2)
[cached]</pre>
</div>

The second time, nothing was compiled. Now give the sheet a second formula, and read both
values:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A3 =A2+1
source urn:iki:tutorial:sheet:cell:A2
source urn:iki:tutorial:sheet:cell:A3'>
<pre class="ikigai-run-expected">=A2+1
[uncacheable]
10
[cached]
11
[computed]</pre>
</div>

`A2` was cached before this cell asked for it. The grid at the bottom of the page draws itself
again after every cell runs, so it read `A2`'s value, and compiled `A2`'s formula, as soon as
the last cell had typed it. `A3` is new, so it was computed here.

## Editing a formula compiles exactly that formula

Both formulas are compiled and cached now (the grid below read every value after that cell
ran, and a value reads its formula). Three edits, three different things to change:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A1 7
cache urn:iki:tutorial:sheet:formula:A2
sink urn:iki:tutorial:sheet:input:A2 =A1*3
cache urn:iki:tutorial:sheet:formula:A2
cache urn:iki:tutorial:sheet:formula:A3'>
<pre class="ikigai-run-expected">7
[uncacheable]
cached
=A1*3
[uncacheable]
not cached
cached</pre>
</div>

Changing `A1`, a cell the formula *names*, leaves the formula compiled: `=A1*2` is the same
program whatever `A1` holds, and the compiled form never read `A1`. Changing `A2`'s own input
cuts `A2`'s compiled form, and only that one: `A3`'s formula names `A2` too, and is still
cached, because `=A2+1` did not change either.

Nothing here decides any of that. The compiled form read the input, and only the input, so it
hangs from the input's thread and nothing else. Which is the whole point of making the compiled
form a resource: "recompile when the source changes" is not code anybody wrote, it is what
caching a transreption means.

## Code as data: which cells a formula reads

A compiled formula is data, and the next question is one a spreadsheet asks constantly: which
cells does this formula read? The text could be scanned for things that look like cells, but
the compiled form already knows, so `refs:{ref}` reads it:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:refs}}
```

A range is listed cell by cell, and the listing is a resource too, `range:{from}:{to}`, a pure
function of its own name:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:range}}
```

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:B1 =SUM(A1:A3) * 2
source urn:iki:tutorial:sheet:formula:B1
source urn:iki:tutorial:sheet:refs:B1
source urn:iki:tutorial:sheet:range:A3:A1'>
<pre class="ikigai-run-expected">=SUM(A1:A3) * 2
[uncacheable]
(* (sum (range A1 A3)) 2)
[computed]
urn:iki:tutorial:sheet:cell:A1
urn:iki:tutorial:sheet:cell:A2
urn:iki:tutorial:sheet:cell:A3
[computed]
error: invalid argument `from, to`: A3:A1 is spelled top-left first: A1:A3</pre>
</div>

A range has one spelling, top-left first, for the reason a cell does: one name, one cache
entry, one thread. The compiler writes that spelling whatever order the formula used, so
`SUM(A3:A1)` and `SUM(A1:A3)` read the same range.

## Evaluating: a value reads the values it names

Part I showed the value endpoint and skipped its formula branch. Here it is again, and now
the branch is the interesting part:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:cell}}
```

For a formula, the value reads the cell's precedents (the next section but one), then the
compiled formula, reads it back into an expression, and evaluates it:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:eval}}
```

Every cell the expression names is read through the kernel, as `cell:{ref}`. So a value is a
composite over other values, the way tic-tac-toe's line was a composite over cells, and its
dependencies are exactly the cells the evaluation read, recorded by the kernel as it read them.
An empty cell is 0 in arithmetic; `SUM` skips text and empty cells, as spreadsheets do.

## Errors are values

A formula can go wrong five ways, and each is a value the cell shows, never a failed request
or a crashed page:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:errors}}
```

An error is also passed on: a formula that reads a cell showing `#DIV/0` shows `#DIV/0`.

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:B2 Rent
sink urn:iki:tutorial:sheet:input:C1 =A1/0
sink urn:iki:tutorial:sheet:input:C2 =C1+1
sink urn:iki:tutorial:sheet:input:C3 =E1+1
sink urn:iki:tutorial:sheet:input:C4 =B2*2
source urn:iki:tutorial:sheet:cell:C1
source urn:iki:tutorial:sheet:cell:C2
source urn:iki:tutorial:sheet:cell:C3
source urn:iki:tutorial:sheet:cell:C4'>
<pre class="ikigai-run-expected">Rent
[uncacheable]
=A1/0
[uncacheable]
=C1+1
[uncacheable]
=E1+1
[uncacheable]
=B2*2
[uncacheable]
#DIV/0
[computed]
#DIV/0
[computed]
#REF
[computed]
#VALUE
[computed]</pre>
</div>

`E1` is `#REF` because the sheet has no column `E`, so no name answers for it. The evaluator
knows that without asking, and asks for nothing.

The fifth error is the reason the compiler never fails. A formula with a mistake in it still
compiles, to an expression that says what was wrong:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:C5 =A1+
source urn:iki:tutorial:sheet:formula:C5
source urn:iki:tutorial:sheet:cell:C5
source urn:iki:tutorial:sheet:cell:C5'>
<pre class="ikigai-run-expected">=A1+
[uncacheable]
(syntax-error &quot;expected a number, a cell or (, and the formula ended&quot;)
[computed]
#SYNTAX
[computed]
#SYNTAX
[cached]</pre>
</div>

A compiler that *refused* `=A1+` would be the more obvious design, and it would cost the cache.
The kernel never caches an answer built on a refusal of that kind, because it cannot name what
would make the refusal go away: it records a failed read as a dependency that has already
expired. So every read of a cell with a typo in it would compile the typo again, and every
value that read that cell would be computed again with it. Compiled to a value instead,
`(syntax-error …)` is cached, hangs from the input like any other compiled form, and goes away
when the input is fixed.

## A cycle is named, not chased

A formula can read itself, the long way round: `D1` reads `D2`, and `D2` reads `D1`. Evaluated
naively, `D1` asks for `D2`, which asks for `D1`, which asks for `D2`, until something stops it.
Here something would: the kernel bounds how deep one resolution may go (64 levels, by default),
and refuses past it. But that refusal is a kernel error with the whole chain in it, the kind of
answer the last section said cannot be cached, and it would arrive as a failed page rather
than a cell saying what is wrong.

So a value checks first, with one more resource. `precedents:{ref}` is every cell a cell's
value depends on, however far back, found by walking `refs:{ref}`:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:precedents}}
```

It reads formulas and never values, and it visits each cell once, so it ends even when the
formulas go round in a circle. A cell that turns up among its own precedents is in one, and its
value is `#CYCLE`:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:D1 =D2+1
sink urn:iki:tutorial:sheet:input:D2 =D1*2
source urn:iki:tutorial:sheet:cell:D1
source urn:iki:tutorial:sheet:precedents:D1'>
<pre class="ikigai-run-expected">=D2+1
[uncacheable]
=D1*2
[uncacheable]
#CYCLE
[computed]
urn:iki:tutorial:sheet:cell:D2
urn:iki:tutorial:sheet:cell:D1
[cached]</pre>
</div>

Break the cycle by giving `D2` a number, and `D1` is computed again from it. Its precedents were
built from `D2`'s old formula, so `D2`'s input cut them too:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:D2 4
source urn:iki:tutorial:sheet:precedents:D1
source urn:iki:tutorial:sheet:cell:D1'>
<pre class="ikigai-run-expected">4
[uncacheable]
urn:iki:tutorial:sheet:cell:D2
[computed]
5
[computed]</pre>
</div>

## The sheet

The same sheet as Part I, on this page's kernel, with everything the cells above typed into it.
Try a formula of your own, a range, and a mistake.

<div class="sheet-play" data-shell='urn:iki:tutorial:sheet:template:page'><p class="sheet-unavailable">The sheet appears here when the in-page kernel has loaded.</p></div>

## What was built

| the set | the name | what it is |
|---|---|---|
| the compiler | `urn:iki:tutorial:sheet:compile` | code: a pure function, declared as a lossy transreptor |
| a compiled formula | `urn:iki:tutorial:sheet:formula:{ref}` | glue: the input, run through whatever plan the kernel finds |
| references | `urn:iki:tutorial:sheet:refs:{ref}` | code: a small function of the compiled form and the ranges it names |
| a range | `urn:iki:tutorial:sheet:range:{from}:{to}` | code: a pure function of its own name |
| precedents | `urn:iki:tutorial:sheet:precedents:{ref}` | code: a walk over `refs`, the question "is this a cycle?" |
| a value | `urn:iki:tutorial:sheet:cell:{ref}` | code: the evaluator, reading every value it needs by name |

`formula:{ref}` is glue for the same reason the time chapter's view was: a name cannot carry
an argument. If a name could mean "this resource, transrepted to that type", the compiled
formula would be a rule, `formula:{ref}` = `input:{ref}` as `text/x-sexpr`, and no code at all.

Every one of these is cacheable, and not one names a golden thread or lists a dependency. The
next part edits a cell and asks what that cost.
