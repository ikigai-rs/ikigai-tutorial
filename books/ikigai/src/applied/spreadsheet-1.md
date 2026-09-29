# A spreadsheet, I: cells are atoms

Tic-tac-toe built an application out of names, and every answer in it changed for one
reason: somebody played a move. This arc builds something a little bigger, a spreadsheet,
and it brings two things tic-tac-toe did not have. A cell can hold **code**: a formula, which
has to be compiled before it can run. And one edit ripples: change one number and every
total that reads it, and every total that reads those, is wrong until it is computed again.
Spreadsheet engines are built around that second problem. This arc's claim is that the
kernel already solves it, and that an application built as resources never writes a
recalculation engine at all.

It comes in parts, as tic-tac-toe did. This one draws the model on paper and builds the
atoms: the cells as a person types into them, the values of those that hold no formula, and
the grid that shows them. [Part II](spreadsheet-2.md) makes a formula a resource, compiled
and cached like any other answer. [Part III](spreadsheet-3.md) edits one cell and watches
exactly the cells that depend on it compute again.

## The model, on paper

As before, the names come first. A cell is named by its column and row, `A1` to `D6`: a
small sheet, four columns by six rows, which is enough to show everything and small enough
to see all of it at once.

| the set | its name | what it is | built in |
|---|---|---|---|
| an **input** | `urn:iki:tutorial:sheet:input:{ref}` | what a person typed: a number, some text, or a formula starting with `=` | this part |
| a **value** | `urn:iki:tutorial:sheet:cell:{ref}` | what the cell shows: a literal as typed, or its formula's result | this part (literals), part II (formulas) |
| the **grid** | `urn:iki:tutorial:sheet:view:grid` | every value, as an HTML table, from a template that is a resource | this part |
| the **compiler** | `urn:iki:tutorial:sheet:compile` | a transreptor from formula text to an s-expression | part II |
| a **compiled formula** | `urn:iki:tutorial:sheet:formula:{ref}` | the cell's formula, compiled: code, as a resource | part II |
| a cell's **references** | `urn:iki:tutorial:sheet:refs:{ref}` | the cells its formula reads, by name | part II |
| a **range** | `urn:iki:tutorial:sheet:range:{from}:{to}` | the cells in a rectangle, by name, for `SUM` | part II |
| a cell's **precedents** | `urn:iki:tutorial:sheet:precedents:{ref}` | every cell its value depends on, however far back | part II |
| what a redraw **costs** | `urn:iki:tutorial:sheet:view:cost` | which values drawing the grid again will compute, and which the cache will serve | part III |

Only the input holds state. Everything below it in the table is a question about inputs:
what does this one compile to, which cells does it read, what is its value. That is the
same bet tic-tac-toe made, with one difference that matters later. There, the questions were
about marks, and a mark is data. Here, some inputs are programs.

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:names}}
```

There is no name for "the dependents of a cell", the list a spreadsheet engine keeps so it
knows what to recompute after an edit. That is not an oversight. It is the subject of Part
III.

## A cell has one name

A cell is written `A1`: a capital letter for the column, then the row, with no leading zero.
Only that spelling names a cell, and only the cells on the sheet have names:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:cell_binding}}
```

The strictness is the one tic-tac-toe had, for the same reason. The kernel caches an answer
under the name it was asked by, and cuts a golden thread by name. If `a1` and `A1` both named
the top-left cell, a write to one would leave a cached answer under the other, still wrong,
with nothing to cut it. So there is one spelling, and the rest are refused with a word on
what to fix:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:sheet:cell:a1
source urn:iki:tutorial:sheet:cell:E1'>
<pre class="ikigai-run-expected">error: invalid argument `ref`: `a1` is not a cell: a capital letter for the column, then the row (e.g. B3)
error: invalid argument `ref`: E1 is off the sheet, which is columns A–D and rows 1–6</pre>
</div>

The page's formula bar, further down, is more forgiving: it is the edge where a person types,
so it takes `a1` and writes to `A1`. The names themselves are not.

## The input: the atom

What was typed into each cell is kept in memory, one string per cell, in a map the host
holds:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:store}}
```

The endpoint over it takes three verbs. `Sink` stores what was typed, trimmed, and answers
what it stored; `Source` reads it back; `Delete` clears it. A cell nobody has typed into is
`NotFound`, the way an unplayed square is:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:input}}
```

One detail is new. The answer's **media type** says what kind of input it is: a formula,
anything starting with `=`, is served as `text/x-formula`, and everything else as
`text/plain`. Nothing in this part reads that, but Part II's compiler is found by it.

Type into a cell, and read it back:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A1 42
source urn:iki:tutorial:sheet:input:A1
source urn:iki:tutorial:sheet:input:A1
source urn:iki:tutorial:sheet:input:B1'>
<pre class="ikigai-run-expected">42
[uncacheable]
42
[computed]
42
[cached]
error: not found: nothing has been typed into B1</pre>
</div>

## The value, and the empty cell

The input is what a person typed. The **value** is what the cell shows, and it is the name
everything else reads, just as tic-tac-toe's platonic cell was. For a cell holding a number
or some text, the value is the input, passed through. For a cell nobody has typed into, the
value is empty: every cell of the sheet has a value before anything is typed, so nothing has
to create a sheet before reading one.

The value endpoint is one function, and most of it belongs to Part II, which gives the
formula branch its meaning. The first two arms are this part's:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:cell}}
```

Type text into a cell, and ask for its value twice:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:A2 Rent
source urn:iki:tutorial:sheet:cell:A2
source urn:iki:tutorial:sheet:cell:A2'>
<pre class="ikigai-run-expected">Rent
[uncacheable]
Rent
[computed]
Rent
[cached]</pre>
</div>

Now ask for a cell nobody has touched:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:sheet:cell:C5'>
<pre class="ikigai-run-expected">[cached]</pre>
</div>

No text, since the value is empty, and `[cached]` the first time you asked. Nothing on this page had asked for
`C5`, as far as the cells go. The grid at the bottom of the page had: it shows every value
on the sheet, and it asked for all twenty-four when the page loaded. The grid and the cells
are one kernel with one cache, so the grid's answer is the cell's.

## A write cuts what was built on it

The value of `C5` is cached, and it is empty. What happens when somebody types into `C5`?
Nothing in the value endpoint names a golden thread, and nothing in the input endpoint cuts
one. Ask:

<div class="ikigai-run" data-cmd='cache urn:iki:tutorial:sheet:cell:C5
sink urn:iki:tutorial:sheet:input:C5 hello
cache urn:iki:tutorial:sheet:cell:C5
source urn:iki:tutorial:sheet:cell:C5'>
<pre class="ikigai-run-expected">cached
hello
[uncacheable]
not cached
hello
[computed]</pre>
</div>

The kernel did both halves. When the value was computed, it read the input through the
kernel, and the kernel recorded that read as a dependency, a *failed* one: the input was
`NotFound`. A failed read counts, because an answer built on a resource's absence is as
wrong as one built on its content once it appears. So the empty value hung from the
thread named after the input, and the `Sink` to the input cut that thread.

That is the same thing tic-tac-toe's empty square did, and it is the rule the whole arc
rests on: **a value depends on exactly what it read, and the kernel keeps the list.**

`Delete` is a write too, so clearing a cell brings the empty value back, computed again:

<div class="ikigai-run" data-cmd='delete urn:iki:tutorial:sheet:input:C5
source urn:iki:tutorial:sheet:cell:C5'>
<pre class="ikigai-run-expected">ok
[uncacheable]
[computed]</pre>
</div>

## The grid

The grid is HTML, and it is resources in the way tic-tac-toe's board was: templates, and views
that fill them. The grid's template is a table with a marker in each cell:

```text
{{#include ../../../../crates/spreadsheet/templates/grid.html:1:6}}
…
```

Each marker names one cell's **view**, `view:cell:{ref}`, and that view is a template too, of one
marker:

```text
{{#include ../../../../crates/spreadsheet/templates/cell.html}}
```

The markers are the language of `urn:iki:fn:compose`, the generic composer from `ikigai-fn`. A
marker names a resource; compose resolves it and splices the answer in, and there are three ways
to splice. `$h{…}` splices the answer as text, escaped for HTML. `$r{…}` splices it as it is, as
trusted markup: here, a view the kernel has already composed. `$a{…}` splices it as it is and then
expands any markers in it, which is how one template includes another. The [time
chapter](time.md)'s clock is one marker of the first kind, and [Tic-tac-toe,
V](tic-tac-toe-5.md#the-template-language) teaches the whole language.

A view is a template bound at a name. `compose_over` makes the binding, and the variables the name
captures are the template's arguments, so `view:cell:B2` is the `cell` template with `{ref}` = `B2`:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:view}}
```

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:views}}
```

That is all the code the grid has. The template says which cells there are and where they go, and
each cell's view says how a value is shown, so there is no four and no six in any code, and no
code that reads a cell.

How a value is shown is the part that matters, because a value is text a person typed. `$h`
escapes what it splices, so a cell holding `<b>` shows `<b>` rather than turning the rest of the
row bold. And `$h` never expands what it splices, so a cell holding a marker shows the marker as
typed. If it did, anybody who could type into a cell could make the page resolve any name they
liked, and show the answer: `$a{urn:kernel:cache}` in a cell would print the kernel's cache into
the grid. Only `$a` expands what it splices, and the sheet never uses it: every marker in its
templates is `$h` or `$r`.

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:sheet:input:B2 <b>not bold</b> $a{urn:kernel:cache}
source urn:iki:tutorial:sheet:view:cell:B2'>
<pre class="ikigai-run-expected">&lt;b&gt;not bold&lt;/b&gt; $a{urn:kernel:cache}
[uncacheable]
&amp;lt;b&amp;gt;not bold&amp;lt;/b&amp;gt; $a{urn:kernel:cache}
[computed]</pre>
</div>

A view reads a value and nothing else, so it is cached with the value and cut with it, like any
other composite.

## A sheet you can type into

Here is the sheet. The grid is the view above; the formula bar under it names a cell and what
to type into it, and **Enter** writes it. Empty input clears the cell.

<div class="sheet-play" data-shell='urn:iki:tutorial:sheet:template:page'><p class="sheet-unavailable">The sheet appears here when the in-page kernel has loaded.</p></div>

The formula bar is a form, and the form's write is one more resource, `view:edit`. It takes
the cell and the input as named arguments, the way a server would read a form body, and it
writes the input with a `Sink` through the kernel, or clears it with a `Delete`:

```rust,ignore
{{#include ../../../../crates/spreadsheet/src/lib.rs:view_edit}}
```

The page's markup is one more template, `template:page`. It asks for the grid when it loads,
and posts the form to `iki/tutorial/sheet/view/edit`, by the same rule as tic-tac-toe's
board: a relative path's segments, joined by `:`, after `urn:`. After an edit, and after any
cell on this page runs, the page draws the grid again, because a grid on the screen is HTML
the kernel handed out earlier and nothing cuts that. Drawing it again is cheap: every value
the edit did not touch is served from the cache.

Type a number into `A3`, and then a formula into `A4`: `=A3*2`. It works, and why it works
is the next part.

## What was built

| the set | the name | what it is |
|---|---|---|
| an input | `urn:iki:tutorial:sheet:input:{ref}` | code: the one atom |
| a value | `urn:iki:tutorial:sheet:cell:{ref}` | code: a small function of the input (and, in Part II, of other values) |
| a value, as HTML | `urn:iki:tutorial:sheet:view:cell:{ref}` | a template bound at a name: no code |
| the grid | `urn:iki:tutorial:sheet:view:grid` | a template bound at a name: no code |
| the formula bar's write | `urn:iki:tutorial:sheet:view:edit` | code: the human edge, a write through the kernel |

No part of it caches anything or invalidates anything, and the empty cell went stale and
came back without being told. The input is the only thing that holds state. The next part
puts code in it.
