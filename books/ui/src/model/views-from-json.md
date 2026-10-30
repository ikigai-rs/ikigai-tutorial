# Views from JSON

[Graph to page with XSLT](xslt.md) turned the kernel's readouts into pages with a stylesheet,
because the kernel already publishes those readouts as graphs. Most data you will want to show
is not a graph. It arrives as a JSON document from an HTTP API, as a CSV export from a
spreadsheet, as rows from a database. This chapter and the next build a view over data like that,
in the form it already has. Nothing in either chapter turns the data into RDF.

The data is a table everyone knows: the eight planets, each with its name, its kind and the
length of its year in Earth days. The kernel inspector serves it twice, as a JSON document and as
a CSV table, so that the next chapter can show one view over both.

## The data, as it is

The JSON document is an **atom**, like the note in [A page is a composed
resource](composed.md): the one kind of name in this chapter that holds state. Read it:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:data:planets.json'>
<pre class="ikigai-run-expected">[
  { "name": "Mercury", "kind": "terrestrial", "year": 88 },
  { "name": "Venus", "kind": "terrestrial", "year": 225 },
  { "name": "Earth", "kind": "terrestrial", "year": 365 },
  { "name": "Mars", "kind": "terrestrial", "year": 687 },
  { "name": "Jupiter", "kind": "gas giant", "year": 4333 },
  { "name": "Saturn", "kind": "gas giant", "year": 10759 },
  { "name": "Uranus", "kind": "ice giant", "year": 30687 },
  { "name": "Neptune", "kind": "ice giant", "year": 60190 }
]
[computed]</pre>
</div>

That is the whole input: an array of records, each one an object. The view will be a table with
one row per record.

## A row is a template

The card in the first chapter was a template whose markers named resources. A row needs
something slightly different: its values are not resources, they are the fields of one record.
Compose (`ikigai-fn`'s template language) already has a marker for that. `$h{{name}}` splices the
template's **argument** `name`, escaped as text, the same way `$h{urn:…}` splices a resource. So
the row template is three arguments in a table row:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:template:planet-row'>
<pre class="ikigai-run-expected">&lt;tr&gt;&lt;th scope="row"&gt;$h{{name}}&lt;/th&gt;&lt;td&gt;$h{{kind}}&lt;/td&gt;&lt;td&gt;$h{{year}}&lt;/td&gt;&lt;/tr&gt;
[computed]</pre>
</div>

Fill it with `name=Earth kind=terrestrial year=365` and you have Earth's row. What is missing is
the step in between: take each record of the array and fill the template with that record's
fields as its arguments.

## Once per record

Compose fills one template once. A table wants the row template filled once for every record, and
compose has no "for each": a template can splice a resource or an argument, but nothing in it
repeats. Writing this chapter is how that gap was found, and it was filled where it belongs, in
ikigai-fn beside compose. Since ikigai-fn 0.6.0 there is a resource for exactly this step:

> `urn:iki:fn:each?src=<a JSON array>&template=<a row template>`: the template filled once for
> each element of the array, with the element's fields as its arguments, concatenated in order.

An element's fields reach the template only through an escaping marker (`$h`), so a field that
holds markup arrives as text, and a template that tried to splice a field as it is would be
refused. A string field is its text and a number its shortest decimal form, which is why the
years arrive as `88`:

<div class="ikigai-run" data-cmd='source urn:iki:fn:each src=urn:iki:tutorial:inspect:data:planets.json template=urn:iki:tutorial:inspect:template:planet-row'>
<pre class="ikigai-run-expected">&lt;tr&gt;&lt;th scope="row"&gt;Mercury&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;88&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Venus&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;225&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Earth&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;365&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Mars&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;687&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Jupiter&lt;/th&gt;&lt;td&gt;gas giant&lt;/td&gt;&lt;td&gt;4333&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Saturn&lt;/th&gt;&lt;td&gt;gas giant&lt;/td&gt;&lt;td&gt;10759&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Uranus&lt;/th&gt;&lt;td&gt;ice giant&lt;/td&gt;&lt;td&gt;30687&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Neptune&lt;/th&gt;&lt;td&gt;ice giant&lt;/td&gt;&lt;td&gt;60190&lt;/td&gt;&lt;/tr&gt;
[computed]</pre>
</div>

That is a composition, not code the inspector carries: the inspector binds ikigai-fn's `each` at
its conventional name and writes nothing for it.

## The view

The view is one more template, with `each` as a marker. Read it, and notice that it never
names the JSON document. It names `records:{source}`, so the same template serves every source:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:template:planets'>
<pre class="ikigai-run-expected">&lt;section id="view" class="inspect"&gt;
&lt;h1&gt;Planets&lt;/h1&gt;
&lt;p&gt;One view, read from &lt;code&gt;urn:iki:tutorial:inspect:records:$h{{source}}&lt;/code&gt;. The view does not know which source it was given: the same two templates show the JSON document and the CSV table.&lt;/p&gt;
&lt;table class="inspect-table"&gt;
&lt;caption&gt;The eight planets, from the $h{{source}} source&lt;/caption&gt;
&lt;thead&gt;&lt;tr&gt;&lt;th scope="col"&gt;Planet&lt;/th&gt;&lt;th scope="col"&gt;Kind&lt;/th&gt;&lt;th scope="col"&gt;Orbit (Earth days)&lt;/th&gt;&lt;/tr&gt;&lt;/thead&gt;
&lt;tbody&gt;
$r{urn:iki:fn:each?src=urn:iki:tutorial:inspect:records:{source}&amp;template=urn:iki:tutorial:inspect:template:planet-row}&lt;/tbody&gt;
&lt;/table&gt;
&lt;/section&gt;
[computed]</pre>
</div>

`records:json` is a composition too. Its template is one marker that splices the JSON document,
so for this source the records ARE the document. The inspector binds the view at
`fragment:planets-{source}`, with the same `composeOver` the card used, and ikigai-ui's shell turns
it into a page at `page:planets-json`, in the inspector's navigation beside the graph views:

```rust,ignore
{{#include ../../../../crates/inspector/src/data.rs:bindings}}
```

Here is the view, in this page's kernel:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:planets-json'>
<pre class="ikigai-run-expected">&lt;section id="view" class="inspect"&gt;
&lt;h1&gt;Planets&lt;/h1&gt;
&lt;p&gt;One view, read from &lt;code&gt;urn:iki:tutorial:inspect:records:json&lt;/code&gt;. The view does not know which source it was given: the same two templates show the JSON document and the CSV table.&lt;/p&gt;
&lt;table class="inspect-table"&gt;
&lt;caption&gt;The eight planets, from the json source&lt;/caption&gt;
&lt;thead&gt;&lt;tr&gt;&lt;th scope="col"&gt;Planet&lt;/th&gt;&lt;th scope="col"&gt;Kind&lt;/th&gt;&lt;th scope="col"&gt;Orbit (Earth days)&lt;/th&gt;&lt;/tr&gt;&lt;/thead&gt;
&lt;tbody&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Mercury&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;88&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Venus&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;225&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Earth&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;365&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Mars&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;687&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Jupiter&lt;/th&gt;&lt;td&gt;gas giant&lt;/td&gt;&lt;td&gt;4333&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Saturn&lt;/th&gt;&lt;td&gt;gas giant&lt;/td&gt;&lt;td&gt;10759&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Uranus&lt;/th&gt;&lt;td&gt;ice giant&lt;/td&gt;&lt;td&gt;30687&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Neptune&lt;/th&gt;&lt;td&gt;ice giant&lt;/td&gt;&lt;td&gt;60190&lt;/td&gt;&lt;/tr&gt;
&lt;/tbody&gt;
&lt;/table&gt;
&lt;/section&gt;
[computed]</pre>
</div>

Every part of it is cached, so asking again computes nothing:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:planets-json'>
<pre class="ikigai-run-expected">&lt;section id="view" class="inspect"&gt;
&lt;h1&gt;Planets&lt;/h1&gt;
&lt;p&gt;One view, read from &lt;code&gt;urn:iki:tutorial:inspect:records:json&lt;/code&gt;. The view does not know which source it was given: the same two templates show the JSON document and the CSV table.&lt;/p&gt;
&lt;table class="inspect-table"&gt;
&lt;caption&gt;The eight planets, from the json source&lt;/caption&gt;
&lt;thead&gt;&lt;tr&gt;&lt;th scope="col"&gt;Planet&lt;/th&gt;&lt;th scope="col"&gt;Kind&lt;/th&gt;&lt;th scope="col"&gt;Orbit (Earth days)&lt;/th&gt;&lt;/tr&gt;&lt;/thead&gt;
&lt;tbody&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Mercury&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;88&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Venus&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;225&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Earth&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;365&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Mars&lt;/th&gt;&lt;td&gt;terrestrial&lt;/td&gt;&lt;td&gt;687&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Jupiter&lt;/th&gt;&lt;td&gt;gas giant&lt;/td&gt;&lt;td&gt;4333&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Saturn&lt;/th&gt;&lt;td&gt;gas giant&lt;/td&gt;&lt;td&gt;10759&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Uranus&lt;/th&gt;&lt;td&gt;ice giant&lt;/td&gt;&lt;td&gt;30687&lt;/td&gt;&lt;/tr&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Neptune&lt;/th&gt;&lt;td&gt;ice giant&lt;/td&gt;&lt;td&gt;60190&lt;/td&gt;&lt;/tr&gt;
&lt;/tbody&gt;
&lt;/table&gt;
&lt;/section&gt;
[cached]</pre>
</div>

## A write is a cut, here too

The JSON document is an atom, so writing it cuts its golden thread, and everything made from it
is recomputed on the next read: the records, each row, the view. Replace it with a one-record
document:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:inspect:data:planets.json [{"name":"Ceres","kind":"dwarf planet","year":1682}]'>
<pre class="ikigai-run-expected">written
[uncacheable]</pre>
</div>

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:planets-json'>
<pre class="ikigai-run-expected">&lt;section id="view" class="inspect"&gt;
&lt;h1&gt;Planets&lt;/h1&gt;
&lt;p&gt;One view, read from &lt;code&gt;urn:iki:tutorial:inspect:records:json&lt;/code&gt;. The view does not know which source it was given: the same two templates show the JSON document and the CSV table.&lt;/p&gt;
&lt;table class="inspect-table"&gt;
&lt;caption&gt;The eight planets, from the json source&lt;/caption&gt;
&lt;thead&gt;&lt;tr&gt;&lt;th scope="col"&gt;Planet&lt;/th&gt;&lt;th scope="col"&gt;Kind&lt;/th&gt;&lt;th scope="col"&gt;Orbit (Earth days)&lt;/th&gt;&lt;/tr&gt;&lt;/thead&gt;
&lt;tbody&gt;
&lt;tr&gt;&lt;th scope="row"&gt;Ceres&lt;/th&gt;&lt;td&gt;dwarf planet&lt;/td&gt;&lt;td&gt;1682&lt;/td&gt;&lt;/tr&gt;
&lt;/tbody&gt;
&lt;/table&gt;
&lt;/section&gt;
[computed]</pre>
</div>

Every value reached the page through `$h`, so a record whose name holds markup shows the markup
as text, as the note did. A document that is not an array of objects fails the view with the
parser's reason, and never renders half a table.

## Reaching into a larger document

The planets document is the array itself. A real API answer usually is not: the array sits inside
an envelope, `{"count": 8, "planets": [ … ]}`, and the view wants `.planets`. Compose cannot reach
into a JSON document by itself. ikigai-json can: `urn:json:query` runs a jq program (through jaq,
sandboxed and bounded) over a JSON resource. Ask it for the first record's name in the document
you just wrote:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:iki:tutorial:inspect:data:planets.json q=.[0].name as=text/plain'>
<pre class="ikigai-run-expected">Ceres
[computed]</pre>
</div>

So `records:json` for an envelope is one marker,
`$r{urn:json:query?src=urn:…:planets.json&q=.planets}`, and nothing else in this chapter changes.
ikigai-fn 0.6.0 has a marker for the other common case: `$j{<src> <program>}` splices the result of
a jq program as escaped text, so a template can show one field of a document
(`$j{urn:…:planets.json .[2].name}`) without a resource of its own.

⚠ If you edit that cell, keep the program small. The sandbox bounds what a program may do, and in
a native kernel a program past a bound is refused with a reason; in this page's WebAssembly kernel
the refusal stops the kernel instead, and the page has to be reloaded.

<a id="and-when-a-graph-helps"></a>

## When a graph helps

JSON can become a graph. ikigai-jsonld reads JSON-LD, and a JSON-LD context maps plain JSON's
keys to vocabulary terms, after which everything in the XSLT chapter applies, and SPARQL too. That
is worth doing when you want what a graph gives you: joins with other graphs, shapes to validate
against, a query language. It is not the price of a view. The table above needed none of it.

## What was built, and what it rules out

A view over a JSON document: an atom, two templates and three compositions (`urn:iki:fn:each`,
the records, the view), and no code of the inspector's own beyond the atom. Ruled out: converting the data to
another model before it can be shown, and parsing JSON by hand.

Next, the same view from a CSV table: [Views from CSV: one model, two sources](views-from-csv.md).
