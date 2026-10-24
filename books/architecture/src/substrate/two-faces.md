# One mechanism, read two ways

Take one resource and hold it still: a **ledger item**. In
[The gonk Book](../../gonk/index.html) you built a whole host around items like this one; here it
is just a resource, and we are going to look at it from every side.

File one now. This page carries its own kernel — the same work ledger, compiled to WebAssembly —
so the item you file is real, and every later cell on this page reads the ledger you are building.

<div class="ikigai-run" data-cmd='sink urn:iki:ledger:append Translate the research note'>
<pre class="ikigai-run-expected">#1 urn:iki:ledger:default:item:…
[uncacheable]</pre>
</div>

It has a number for people and an IRI for machines. Read it back:

<div class="ikigai-run" data-cmd='source urn:iki:ledger:item:1'>
<pre class="ikigai-run-expected">   #1  open    p-  Translate the research note
  iri:      urn:iki:ledger:default:item:…
  filed:    …
  updated:  …
[computed]</pre>
</div>

Now look at what just happened, because the whole book is in it.

- You **operated** the item. `sink … append` is a thing a person does, and `source … item:1` is a
  page a person reads. This is the item's **user-experience face**: a URL you can link to, a verb
  you can invoke, an answer you can render.
- The kernel **authorized** you. The write was allowed because the capability you hold grants it;
  a reader who did not hold it would have been refused. This is the item's **security face**: a
  name resolved under an authority, an action admitted or denied.

Those are not two systems. They are one resolution, read two ways. The name you linked to is the
name the capability check keyed on. The verb you invoked is the unit the authority gates. The
representation you rendered is the representation a converter produced. There is no separate
"security object" behind the item that the page knows nothing about; the page and the check are
looking at the same resource.

## Why this is the architecture, and not a slogan

It would be easy to read "one substrate, two faces" as a tidy way to describe two things that are
really separate. It is not. The claim is concrete, and the rest of the book is six demonstrations
of it on this one item:

- The **capability** that lets you write the item is a resource, attenuated the same way down every
  call; the screen that asks a person to consent to a delegation reads the *same* grant the kernel
  will later check. One grant, a security artifact and a UX surface.
- The **action list** the kernel offers for the item under your capability is both the set of
  things you are authorized to do and the set of buttons a page would draw. An action you lack is
  not shown disabled; it is not shown. One list, an authorization and a menu.
- The **door** that serves the item to a browser is the one place that authenticates you, and the
  one place that sets the headers and escapes the output. One module, a boundary and a surface.
- The **cache** that makes a second read of the item cheap is keyed on *who asked*, so a cached
  answer computed under one authority is never handed to another. One index, a performance win and
  a confidentiality boundary.

Each of those has a chapter. What they have in common is the move you just watched: a person does a
thing, and the doing of it *is* the authorizing of it, because both are the kernel resolving a name
under a capability. Teaching the architecture means teaching that resolution once — which the next
chapter does — and then noticing, each time, that the thing you built for the person is already the
thing that confines them.

## A word on what you will and will not see run

The demonstrations in the capability and economics chapters run in this page, against this kernel,
for real: you will narrow your own authority and watch an action vanish, cut a golden thread and
watch a view recompute. A handful of patterns in the overlay and closing chapters depend on parts
of the system a browser cannot stand in for — a real network peer, a real passkey ceremony end to
end, the hash-chained log on disk. Those are drawn as pictures and labeled as pictures, with the
runnable version promised as the tooling that draws *state* (not just arrangement) lands. The book
never shows you a recording and calls it a run.
