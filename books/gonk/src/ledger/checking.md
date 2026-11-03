# Checking what you have

The last chapter showed that every page gonk serves is a transform of a graph face. Now use that
face directly. Does what the ledger says parse? Does it fit the rules you hold it to? The **RDF
Toolbox** answers both in your browser tab, over a resource named in its address and read as you.
This chapter calls that a **check**.

Why a resource can be checked at all is the subject of [Checking a
resource](https://ikigai-rs.github.io/ikigai-tutorial/building/checking.html) in The ikigai Book,
which you do not need to have read. A check is not `urn:kernel:validate`, which asks whether a
*request* fits an action's contract before it runs. A check looks at what a resource already says.

## Build the toolbox

gonk serves the toolbox but does not contain it, because its engine is 8 MB of WebAssembly. In a
new empty directory, build it from
[ikigai-toolbox](https://github.com/ikigai-rs/ikigai-toolbox), at the commit this book is checked
against. The build needs git, rustup and node, and it compiles for a minute or more:

<!-- transcript: manual — scripts/test-transcripts.sh runs this build once per book run, and sets TOOLBOX_DIST as the last line does -->
```console
$ git clone https://github.com/ikigai-rs/ikigai-toolbox.git
$ git -C ikigai-toolbox checkout --quiet {{#include ../../../ikigai/toolbox.rev}}
$ git -C ikigai-toolbox submodule update --init --quiet
$ (cd ikigai-toolbox/vendor/turtle-lsp && rustup target add wasm32-unknown-unknown)
$ ./ikigai-toolbox/build.sh
$ export TOOLBOX_DIST="$PWD/ikigai-toolbox/dist"
```

`TOOLBOX_DIST` is only this chapter's shorthand, so that its commands read the same on any machine.
What gonk reads is one line in its config home, naming the build directory. On a machine you keep,
copy `dist/` somewhere that will not move and write that path, such as
`~/.local/share/ikigai-toolbox/dist`.

A line naming something that is not a directory stops gonk at the start, before it opens anything,
rather than leaving you a toolbox that answers `404`:

<!-- transcript: run -->
```console
$ mkdir refused
$ printf 'gonk.toolbox.dist = "%s"\n' "$PWD/no-such-build" > refused/config.toml
$ ikigai-gonk --config-home "$PWD/refused" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk: gonk.toolbox.dist: …/no-such-build is not a directory. Build it with ikigai-toolbox's build.sh and copy dist/ there, or remove the line
  (config read from …/refused/config.toml)
```

Now write the real line:

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.toolbox.dist = "%s"\n' "$TOOLBOX_DIST" > .config/ikigai/config.toml
```

## Start it, and file something

If the gonk from [The ledger in a browser](browser.md) is still running, stop it first, since this
one takes the same port ([A gonk per chapter](../introduction.md#a-gonk-per-chapter)). The banner
gains a line for the toolbox:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
  toolbox http://localhost:1070/toolbox — the RDF Toolbox over …
…
```

Over the owner's socket, as in [the last chapter](browser.md#start-one-and-keep-what-it-says), file
two items in `default`, the second with a label, and one item in a ledger of its own called `notes`.
The anonymous caller at the HTTP door reads `default` and not `notes`, which matters below.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append Write the gonk book'
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append labels=book Pin gonk to a git rev'
#2 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:notes:append A private note'
notes#1 urn:iki:ledger:notes:item:…
[uncacheable]
```

## The page

Every page's header now offers the toolbox, and with no `gonk.toolbox.dist` line it does not. Follow
that link, or type the address:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/ | grep -o "<a class='navlink' href='/toolbox'>[^<]*</a>"
<a class='navlink' href='/toolbox'>Toolbox</a>
```

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open http://localhost:1070/toolbox
```

The page opens on your own data, the default ledger's items, already in its editor. A shell can see
which resource it names and read the same Turtle the page reads:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/toolbox | grep -o 'about="[^"]*"'
about="urn:iki:ledger:items"
$ curl -s http://localhost:1070/toolbox/input/urn%3Aiki%3Aledger%3Aitems | grep dcterms:title
	dcterms:title "Write the gonk book" ;
	dcterms:title "Pin gonk to a git rev" ;
```

Everything from here happens in the tab, and nothing is uploaded. Validate parses the input, and two
findings should appear: the prefixes `sig:` and `prov:` are never used. The ledger writes those
prefixes on every answer, so they are warnings about style, not about what it says. Format and
SPARQL work over the same input. A caller who may not read the default ledger gets the empty toolbox
with its samples instead of an error, and those are always at `/toolbox/page`:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/toolbox/page | grep -o 'about="[^"]*"'
about="urn:toolbox:page"
```

## A check page for one item

Any resource with a Turtle face opens the same way, percent-encoded after `/toolbox/check/`. For
item `#2`, that is:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/toolbox/check/urn%3Aiki%3Aledger%3Aitem%3A2 | grep -o 'about="[^"]*"'
about="urn:iki:ledger:item:2"
$ curl -s http://localhost:1070/toolbox/input/urn%3Aiki%3Aledger%3Aitem%3A2 | grep -E 'title|label'
	dcterms:title "Pin gonk to a git rev" ;
	ledger:label "book" ;
```

The page's own address carries the check, so you can keep it or send it to someone who holds the
same read. Item pages do not link here yet, so you type this address yourself.

## It reads as you

The toolbox holds no authority over the ledger. `urn:toolbox:input:{iri}` reads the resource under
the caller's capability, so the anonymous caller's check of `notes` is refused, just as the ledger's
own address is:

<!-- transcript: run -->
```console
$ curl -s -o /dev/null -w '%{http_code}\n' http://localhost:1070/toolbox/input/urn%3Aiki%3Aledger%3Anotes%3Aitems
403
$ curl -s -o /dev/null -w '%{http_code}\n' http://localhost:1070/iki/ledger/notes/items
403
$ curl -s http://localhost:1070/toolbox/input/urn%3Aiki%3Aledger%3Anotes%3Aitems | cut -d. -f1
denied: this capability does not hold `urn:cap:ledger:read:notes`
```

The check *page* for `notes` still answers `200`, since the page itself holds nothing private: what
is refused is the read it then makes. Signed in under a grant that reads `notes`, as in [the last
chapter](browser.md), the same address shows that ledger's items.

Two more edges of the door are worth knowing. A link to the toolbox from another site arrives as a
foreign navigation, and that is refused, so open it from gonk's own pages or type the address. And
the toolbox is an HTML face of the HTTP door only: the socket and QUIC doors serve exactly what they
served before.

<!-- transcript: run -->
```console
$ curl -s -o /dev/null -w '%{http_code}\n' -H 'Sec-Fetch-Site: cross-site' -H 'Sec-Fetch-Mode: navigate' -H 'Sec-Fetch-Dest: document' http://localhost:1070/toolbox
403
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:toolbox:page' 2>&1 | grep '^error'
error: no endpoint resolved for urn:toolbox:page
```

## Check it against shapes

Parsing says the Turtle is well formed. **Shapes** say whether it is what you meant. Suppose your
team's rule is that every item carries a label. In [SHACL](https://www.w3.org/TR/shacl/), a graph
that describes other graphs, that rule is:

<!-- transcript: file item-shapes.ttl -->
```turtle
@prefix ledger: <https://ikigai-rs.dev/ns/ledger#> .
@prefix sh: <http://www.w3.org/ns/shacl#> .

<urn:example:triage:ItemShape> a sh:NodeShape ;
    sh:targetClass ledger:Item ;
    sh:property [
        sh:path ledger:label ;
        sh:minCount 1 ;
        sh:message "Every item carries a label."
    ] .
```

On the page, choose **Open files** and pick `item-shapes.ttl`. The file joins the input beside the
ledger's items, and a **SHACL** tab appears, because the input now has shapes. It should report one
violation, on item `#1`: "Every item carries a label." `#2` has one, so it passes: open
`/toolbox/check/urn%3Aiki%3Aledger%3Aitem%3A2`, add the same file, and the tab says the data conforms.

The check page can also name its shapes in the address, as a second input read exactly like the
first. So shapes that gonk serves, and that you may read, need no file. `shapes=` takes the
resource's name, and `#shacl` opens on that tab. This gonk serves no shapes, so the command below
names the render rules only to show the second input the page will fetch:

<!-- transcript: run -->
```console
$ curl -s 'http://localhost:1070/toolbox/check/urn%3Aiki%3Aledger%3Aitem%3A1?shapes=urn%3Aiki%3Agonk%3Arender-rules' | grep -o 'data-inputs="[^"]*"'
data-inputs="[&quot;/toolbox/input/urn%3Aiki%3Aledger%3Aitem%3A1&quot;,&quot;/toolbox/input/urn%3Aiki%3Agonk%3Arender-rules&quot;]"
```

To try the page with no gonk at all, The ikigai Book runs the same toolbox [in
place](https://ikigai-rs.github.io/ikigai-tutorial/building/checking.html#the-page-itself), over its
own small example.

## What you have now

| you want | ask |
| --- | --- |
| the toolbox in this gonk | a `gonk.toolbox.dist` line naming a build, and a restart |
| to check the default ledger's items | `/toolbox` |
| to check one resource | `/toolbox/check/{iri}` |
| to check it against shapes you may read | `/toolbox/check/{iri}?shapes={iri}#shacl` |
| to check it against shapes in a file | **Open files** on the page |
| the page with samples | `/toolbox/page` |

The toolbox added no authority and no state: gonk serves a page, the page reads Turtle faces as you,
and the checking runs in your tab. [The next chapter](dashboard.md) reads every ledger at once, on a page of charts and tables, and
[the one after it](lifecycle.md) gives a ledger states of its
own, and the rules a lifecycle holds are enforced on every write, where a check only tells you what
it finds.
