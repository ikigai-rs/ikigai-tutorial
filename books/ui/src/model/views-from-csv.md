# Views from CSV: one model, two sources

[Views from JSON](views-from-json.md) built a table of the planets over a JSON document, and its
view template named `records:{source}` instead of the document. This chapter supplies a second
source, a CSV table of the same planets, and shows the view that does not care which one it is
given.

## The data, as it is

The CSV table is an atom of its own, with a header row and one row per planet:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:data:planets.csv'>
<pre class="ikigai-run-expected">name,kind,year
Mercury,terrestrial,88
Venus,terrestrial,225
Earth,terrestrial,365
Mars,terrestrial,687
Jupiter,gas giant,4333
Saturn,gas giant,10759
Uranus,ice giant,30687
Neptune,ice giant,60190
[computed]</pre>
</div>

## One model

The view's model is the one the JSON chapter used: **an array of records**, each a set of named
fields. A JSON document of objects already is one. A CSV table becomes one by a transreption that
changes the form and not the data: one record per row, keyed by the header. That is a conversion
between two representations of the same thing, which is exactly what ikigai's transreptors are
for. The one needed takes a CSV table and gives a JSON array of objects keyed by its header row.

ikigai-json has exactly that: `urn:json:from-csv`, registered as the kernel's `text/csv` →
`application/json` transreptor, which reads the table with the `csv` crate (RFC 4180, so a quoted
field keeps its commas). You rarely name it. `urn:json:query` runs a jq program over a JSON
resource, and given a resource that is not JSON it plans the kernel's transreption to JSON first,
so the identity program `.` over the CSV atom is the table as records:

<div class="ikigai-run" data-cmd='source urn:json:query src=urn:iki:tutorial:inspect:data:planets.csv q=.'>
<pre class="ikigai-run-expected">[{"name":"Mercury","kind":"terrestrial","year":"88"},{"name":"Venus","kind":"terrestrial","year":"225"},{"name":"Earth","kind":"terrestrial","year":"365"},{"name":"Mars","kind":"terrestrial","year":"687"},{"name":"Jupiter","kind":"gas giant","year":"4333"},{"name":"Saturn","kind":"gas giant","year":"10759"},{"name":"Uranus","kind":"ice giant","year":"30687"},{"name":"Neptune","kind":"ice giant","year":"60190"}]
[computed]</pre>
</div>

Every value is a string, because CSV has no types: the years are `"88"`, where the JSON document
had `88`. The row template splices each as text either way, so the rows come out the same.

The kernel does not turn one representation into another just because a reader asks for it with
`as=` (ledger #2153), which is why the conversion is spelled as a query rather than as
`source …:planets.csv as=application/json`.

## Two sources, side by side

Each source is one template, and the two are the only place the view's sources differ. The CSV
source's records are that query over the table:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:template:records-csv'>
<pre class="ikigai-run-expected">$r{urn:json:query?src=urn:iki:tutorial:inspect:data:planets.csv&amp;q=.}
[computed]</pre>
</div>

and the JSON source's are the document itself:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:template:records-json'>
<pre class="ikigai-run-expected">$r{urn:iki:tutorial:inspect:data:planets.json}
[computed]</pre>
</div>

The view over the CSV table is the same `template:planets`, filled with `source=csv`:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:planets-csv'>
<pre class="ikigai-run-expected">&lt;section id="view" class="inspect"&gt;
&lt;h1&gt;Planets&lt;/h1&gt;
&lt;p&gt;One view, read from &lt;code&gt;urn:iki:tutorial:inspect:records:csv&lt;/code&gt;. The view does not know which source it was given: the same two templates show the JSON document and the CSV table.&lt;/p&gt;
&lt;table class="inspect-table"&gt;
&lt;caption&gt;The eight planets, from the csv source&lt;/caption&gt;
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

and the view over the JSON document, with `source=json`:

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

The two tables are the same, row for row. Only the sentence above each one names its source,
because the template prints its `source` argument there. The inspector's navigation has both
pages, `Planets (JSON)` and `Planets (CSV)`, so a browser shows the same thing.

## Two sources, two threads

The views share their templates, but each hangs from its own source's golden thread. Write the
JSON document, as the last chapter did:

<div class="ikigai-run" data-cmd='sink urn:iki:tutorial:inspect:data:planets.json [{"name":"Ceres","kind":"dwarf planet","year":1682}]'>
<pre class="ikigai-run-expected">written
[uncacheable]</pre>
</div>

The CSV view was read from nothing that write touched, so it is still cached:

<div class="ikigai-run" data-cmd='source urn:iki:tutorial:inspect:fragment:planets-csv'>
<pre class="ikigai-run-expected">&lt;section id="view" class="inspect"&gt;
&lt;h1&gt;Planets&lt;/h1&gt;
&lt;p&gt;One view, read from &lt;code&gt;urn:iki:tutorial:inspect:records:csv&lt;/code&gt;. The view does not know which source it was given: the same two templates show the JSON document and the CSV table.&lt;/p&gt;
&lt;table class="inspect-table"&gt;
&lt;caption&gt;The eight planets, from the csv source&lt;/caption&gt;
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

and the JSON view is recomputed from the new document:

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

A write to the CSV table works the same way the other way round; the inspector's tests write one
with a quoted comma and a field that holds markup, and check that the comma stays in its field,
the markup arrives as text and the JSON view does not move.

## No stopgaps left

While these two chapters were being written, the iteration and the CSV conversion were each a
labeled stopgap in the inspector, shaped like the resources this chapter needed and named
`stopgap:` so nobody mistook them for the design. Both gaps were filled where they belong, with the
same arguments: `urn:iki:fn:each` in ikigai-fn 0.6.0 and `urn:json:query` with its CSV transreptor
in ikigai-json 0.1.0. Replacing them changed two template lines and no output. The only code the
inspector has for these views is its two atoms.

## And the rest

JSON and CSV are two forms of many. The same pattern covers the others: find the model the view
needs, and the smallest step from the form the data has to that model.

- **XML** needs no step at all: an XSLT stylesheet reads it directly, as in the XSLT chapter,
  with no RDF anywhere.
- **SPARQL results** already have a JSON form (`application/sparql-results+json`); with
  `urn:json:query` to pick out the bindings, a query's answer is an array a row template can be
  filled from.
- **Plain lines of text**, a log or a listing, have ikigai-text (`grep`, `sort`, `head` and the
  rest, as resources), which a view can splice as they are.

## What was built, and what it rules out

One view and one row template over two sources, a CSV table and a JSON document, each source one
template naming where its records come from, and each with its own golden thread. Ruled out: a
second view for a second format, and a model the data has to be converted into before it can be
shown.

Next: [Live data and history](live.md).
