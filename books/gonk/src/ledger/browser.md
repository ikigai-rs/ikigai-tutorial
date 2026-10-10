# The ledger in a browser

The last chapter reached gonk with `curl`, through paths that are a mechanical mapping of the
ledger's own resources. This one opens the same server the way a person does: pages, forms, a
sign-in. Everything a browser does here is still a request to a resource, so every step but
one can be checked from a shell, and this chapter does check them. The one that cannot is the
passkey ceremony, which needs a browser and an authenticator; it is marked, and described.

## Start one, and keep what it says

Start a scratch gonk as before, with both its homes in an empty directory, and this time keep
what it prints in a file as well as on the screen:

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s); anonymous SPARQL budget 1000 ms
…
  log     one `gonk:Access` line per request at each door, on stderr (`gonk.log.access = false` turns it off)
…
```

gonk writes its banner and its log to standard error, so `2>&1 | tee gonk.log` is how a
terminal keeps both: the screen shows them, and `gonk.log` is the record you will read at the
end of this chapter. (Run as a service, the same lines go wherever the service manager sends
standard error; gonk's README uses `/tmp/ikigai-gonk.log`.)

File something, so the pages have an item to show:

<!-- transcript: run -->
```console
$ curl -s -X POST --data-binary 'Write the gonk book' http://127.0.0.1:1070/iki/ledger/append
#1 urn:iki:ledger:default:item:…
```

## The pages

Open <http://localhost:1070/> in a browser. Use `localhost`, not `127.0.0.1`: a passkey is
bound to a host name, and a browser refuses an IP address as one, so a page opened by address
can never sign in.

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open http://localhost:1070/
```

The front page is the first ledger you may read, open items first, with filters for closed
and all, a title search, and a form that files an item. A page is HTML, and HTML is something
a shell can read as well as a browser. These are the lines that matter in it:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/ | grep -o "<title>[^<]*</title>"
<title>Ledger · gonk</title>
$ curl -s http://localhost:1070/ | grep -o "<span class='how-many'>[^<]*</span>"
<span class='how-many'>1 open items</span>
$ curl -s http://localhost:1070/l/default/item/1 | grep -o "<h1 class='item-title'>[^<]*</h1>"
<h1 class='item-title'>Write the gonk book</h1>
```

| path | what it shows |
| --- | --- |
| `/` | the first ledger you may read |
| `/l/{ledger}` | one ledger, with the status filters and the search |
| `/l/{ledger}/item/{n}` | one item: body, metadata, labels, links, comments, and the forms that work on it |
| `/sparql` | a query over one ledger's graph, rendered as a table; asked for a results format, the SPARQL 1.1 Protocol |

Every one of those pages is a **transform of a graph face**. The ledger page is
`urn:iki:ledger:default:items` asked for as Turtle and rendered by one XSLT stylesheet,
server-side, so the page cannot show anything the graph does not say. The graph is right
there:

<!-- transcript: run -->
```console
$ curl -s -H 'Accept: text/turtle' http://127.0.0.1:1070/iki/ledger/items | grep dcterms:title
	dcterms:title "Write the gonk book" ;
```

and so is a query over it. Asked for a results format rather than HTML, CSV among them,
`/sparql` answers a program instead of a person: it is the SPARQL 1.1 Protocol, run under your
grant. An anonymous caller's query also runs under a time budget, the banner's `anonymous SPARQL
budget 1000 ms`: it runs for at most a second, so one expensive query cannot hold the server
(gonk PR 105, ledger #964). gonk's own queries, the ones behind its pages, never run
under it.

<!-- transcript: run -->
```console
$ curl -s -G -H 'Accept: text/csv' --data-urlencode 'query=PREFIX dcterms: <http://purl.org/dc/terms/> SELECT ?title WHERE { ?item dcterms:title ?title }' http://127.0.0.1:1070/sparql
title
Write the gonk book
```

Naming no ledger, the query's dataset is the union of every graph your grant may read. For an
anonymous caller that is the ledgers the HTTP door grants, here `default`'s graph alone:

<!-- transcript: run -->
```console
$ curl -s -G -H 'Accept: text/csv' --data-urlencode 'query=SELECT ?g (COUNT(*) AS ?n) WHERE { GRAPH ?g { ?s ?p ?o } } GROUP BY ?g' http://127.0.0.1:1070/sparql
g,n
urn:iki:ledger:graph:default,…
```

`ledger=` narrows the dataset to one ledger's graph and `graph=` to the graphs named. A graph
the grant holds no token for refuses the whole query rather than answering over the part it
can see, and `FROM` is refused rather than quietly ignored, so a grant naming one ledger cannot
read another through this door:

<!-- transcript: run -->
```console
$ curl -s -w '\n%{http_code}\n' -G -H 'Accept: text/csv' --data-urlencode 'ledger=book' --data-urlencode 'query=SELECT ?s WHERE { ?s ?p ?o }' http://127.0.0.1:1070/sparql
denied: this capability does not hold `urn:cap:store:read:graph:urn:iki:ledger:graph:book`, so it cannot query `urn:iki:ledger:graph:book`
403
```

The HTML page is the same resource for a person: an editor over one ledger's graph, with the
answer rendered as a table.

## A form is a ledger action

The form on the front page does not write anything gonk invented. It posts to `/act`, which
names an action the ledger already declares (`_action=append`), checks every field against
that action's contract, and issues it under your grant. This is the request your browser
sends when you file an item there:

<!-- transcript: run -->
```console
$ curl -s -d _action=append -d _ledger=default -d _then=items --data-urlencode 'content=Read it in a browser' http://localhost:1070/act | grep -o "<span class='how-many'>[^<]*</span>"
<span class='how-many'>2 open items</span>
```

The answer is the fragment of the page that changed (`_then=items`), which the page swaps in
place with [htmx](https://htmx.org). There is no single-page application and no build step:
the server renders, the browser shows.

A door that lets anything on this machine write the ledger also lets any web page open on
this machine try to: a page on any site can post a form to `localhost`. So the browser's own
signals are a **refusal**. A write whose `Origin` (or `Sec-Fetch-Site`) names another site is
answered `403` at the edge, before it reaches anything it names:

<!-- transcript: run -->
```console
$ curl -s -w '\n%{http_code}\n' -H 'Origin: https://elsewhere.example' -d _action=append -d _ledger=default -d _then=items --data-urlencode 'content=Not from here' http://localhost:1070/act
a write from another site is refused: its Origin or Sec-Fetch-Site names a page that is not this server's own
403
```

`curl` sends no `Origin` unless told to, and neither does a script, which is why the requests
before it were unaffected. A request whose `Host` is anything but `localhost`, `127.0.0.1` or
`[::1]` (with this port) is refused the same way, reads included, and the front page is no
exception; that is the defense against DNS rebinding, where a page on another site reaches a
loopback server under its own name:

<!-- transcript: run -->
```console
$ curl -s -w '\n%{http_code}\n' -H 'Host: elsewhere.example' http://127.0.0.1:1070/
this server answers only to its own loopback name (localhost, 127.0.0.1 or [::1], with its port); a request under another Host is refused — it is how a DNS rebinding page reaches a loopback server
403
```

Refused, rather than served with an empty grant: an empty grant is still offered everything
that needs no grant (the pages, the passkey ceremonies, a resource's description), and gonk's
audit showed another site filling the passkey ceremony's table that way. A refused request
learns nothing, not even which methods a path accepts.

A read from another page is the subtler case, because a link to an item from somewhere else
is ordinary and ought to work. The browser labels every request with where it came from
(`Sec-Fetch-Site`) and what it is for (`Sec-Fetch-Mode`, `Sec-Fetch-Dest`), and gonk draws
its line at the page's **origin**, port included. The port matters: a site is a scheme and a
domain, so a page on `http://localhost:8090` is the *same site* as gonk, and the browser sends
gonk's sign-in cookie with everything that page asks for. Another page that **navigates**
here (a link, a typed address) is answered, under only the read half of whoever is signed in;
another page that **loads** something from here, as an image, a script, a frame or a
`fetch`, is refused at the edge. `curl` can send the labels a browser would:

<!-- transcript: run -->
```console
$ curl -s -w '\n%{http_code}\n' -H 'Sec-Fetch-Site: same-site' -H 'Sec-Fetch-Mode: no-cors' -H 'Sec-Fetch-Dest: image' http://localhost:1070/l/default/item/1
a page that is not this server's own may only NAVIGATE here (a link, or a typed address); it may not load this server's resources as an image, script, frame or fetch
403
$ curl -s -o /dev/null -w '%{http_code}\n' -H 'Sec-Fetch-Site: same-site' -H 'Sec-Fetch-Mode: navigate' -H 'Sec-Fetch-Dest: document' http://localhost:1070/l/default/item/1
200
```

Nothing gonk serves is meant to be embedded in another page, and a load is how a page spends
someone's authority without anyone clicking anything: an `<img>` pointing at a resource that
asks a model for an explanation or a review (the [browse](../browse/explain.md) family's)
would run the model and archive its answer under the signed-in identity's grant. Through gonk
`ed43af3` both kinds of read were answered under the whole grant, which is what that
`<img>` did (gonk PR 100, ledger #880). The read half, for a navigation, is the same caution
one step milder: following a link from elsewhere shows what the identity may read, and does
not spend on its behalf.

## Who you are, and what that grants

An anonymous caller on this machine reads and writes the ledgers in `gonk.http.ledger`
(`default`, unless configured), and nothing else. A ledger outside that list is closed to it,
and the refusal says so in words before it says it in tokens:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/l/book
denied: Sign in to read the ledger "book".
Signed in already? Your passkey's grant doesn't include the ledger "book".
The grant it needs: urn:cap:ledger:read:book (to read the ledger "book")
```

That is the notice a signed-out visitor reads, here and on every page that needs more than
anonymous authority, with a **Sign in with passkey** button beside it in a browser. Anything
more than anonymous, such as delete, another ledger, or reading the repositories gonk
browses, belongs to an **identity**, and an identity is given by invitation:

<!-- transcript: run -->
```console
$ ikigai-gonk passkey invite ada --ledger book=write --browse read --port 1070 --config-home "$PWD/.config/ikigai"
passkey invite for `ada` — grant `ada` (6 scopes) in …/.config/ikigai/gonk/grants.json
  valid for 30 minutes, once
  open  http://localhost:1070/#invite=…
  in a browser on this machine (localhost, not 127.0.0.1); the passkey is enrolled in …/.config/ikigai/gonk/clients.json
```

The same `--config-home` again, so the invite is written into this gonk's config home rather
than a real one; and `--port 1070`, so the link it prints points at this gonk. Each flag is a
**role**, and the command writes the tokens the role means:

<!-- transcript: run -->
```console
$ cat .config/ikigai/gonk/grants.json
{
  "ada": [
    "urn:cap:ledger:read:book",
    "urn:cap:store:read:graph:urn:iki:ledger:graph:book",
    "urn:cap:ledger:write:book",
    "urn:cap:store:write:graph:urn:iki:ledger:graph:book",
    "urn:cap:browse:read:*",
    "urn:cap:store:read:graph:urn:iki:browse:graph:default"
  ]
}
```

`--ledger book=write` is the four-token writer of one ledger from
[the last chapter](your-own.md#whose-ledger). `--browse read` is the repositories as a role:
read every browse root (`--root <name>` names one instead), and the browse graph's quads. Its
stronger sibling, `--browse derive`, adds minting annotations and spending a mounted model's
inference; it is one word on purpose, and it is refused on a server with no model to spend.
The repositories arrive in [Browse roots](../browse/roots.md).

Two rules sit behind the command. An identity must be **strictly stronger** than anonymous, so
`passkey invite ada --ledger default=write` is refused: signing in would change nothing. And
a grant name is shared by every passkey and certificate enrolled under it, so writing
different scopes under an existing name is refused unless `--force`, naming each scope the
rewrite would remove and add.

One role is deliberately missing from that grant. `ada` may write `book` through the ledger,
and the grant holds the store's write token for `book`'s graph because the ledger makes its own
writes under the caller's capability. That token is **not** authority to write the graph's
triples directly: gonk's HTTP and QUIC doors refuse a raw store write to a ledger graph, because
that is how an `author` naming somebody else would get in. An identity that really must write
raw (a repair tool, say) is given it by name, with `--ledger-graph book`, which the
[QUIC chapter](../machines/quic-client.md#writing-the-graph-raw) shows.

### The ceremony

<!-- transcript: manual — a passkey is created by a browser and an authenticator (Touch ID, a security key, a phone), which a check cannot be -->
```console
$ open "http://localhost:1070/#invite=…"
```

Open the link the invite printed, in a browser on the machine gonk runs on:

1. A panel, *Create a passkey for this server*, asks for a label (`laptop Touch ID`, say).
2. **Create passkey** brings up the browser's own passkey sheet. Approve it. gonk enrolls the
   credential's public key under the grant `ada`, and the page says so and asks you to sign in.
3. **Sign in with passkey**, in the header, and approve a second time. The page reloads
   signed in, and that browser holds the grant until it signs out, its session's twelve hours
   end, or the server restarts.

The second click is deliberate: when the creation sheet closes it still holds the window's
focus, and a browser refuses a sign-in prompt from a page without it. The invite is
single-use and expires; gonk stores only its hash, and it is the whole trust anchor of the
registration, so no attestation is parsed. After signing in, `/l/book` is a ledger page
rather than a refusal, and every write `ada` makes carries her passkey as its author.

## What it wrote down

Every door writes one line per request, and `gonk.log` has caught them all. The grammar is
`ikigai-log`'s: when the request started, a class, the resource, then the same columns on
every line.

<!-- transcript: run -->
```console
$ grep ' urn:iki:gonk:act ' gonk.log
… gonk:Access urn:iki:gonk:act door=http verb=sink outcome=ok bytes=… dur=… principal=anon q=-
$ grep ' urn:iki:gonk:page:item:default:1 ' gonk.log
… gonk:Access urn:iki:gonk:page:item:default:1 door=http verb=source outcome=ok bytes=… dur=… principal=- q=-
… gonk:Access urn:iki:gonk:page:item:default:1 door=http verb=source outcome=ok bytes=… dur=… principal=- q=-
```

Read them with three things in mind:

- **The subject is the resource, not the URL.** `/l/default/item/1` (read twice: once by
  `curl` as itself, once as another page's link) was
  `urn:iki:gonk:page:item:default:1`; the form was `urn:iki:gonk:act`.
- **`principal` is who a WRITE was from.** The form's line says `anon`; a page read says `-`,
  because the HTTP library tells a door who a write is from and never a read. A signed-in
  write would name the passkey's IRI.
- **`outcome` is the kernel's, not HTTP's.** A refusal the resource makes (the `/l/book`
  page) is a line with `outcome=denied`. A refusal at the edge, before the kernel (the
  cross-site form, the foreign `Host`, the loaded image), is no line at all: there was no
  request to log.

<!-- transcript: run -->
```console
$ grep ' urn:iki:gonk:page:ledger:book ' gonk.log
… gonk:Access urn:iki:gonk:page:ledger:book door=http verb=source outcome=denied bytes=- dur=… principal=- q=-
```

`dur` is milliseconds, so `grep ' dur=[0-9]\{4,\} ' gonk.log` lists every request that took a
second or more. The first page this gonk rendered is usually one of them: it compiled the
stylesheet.

## What you have now

| | anonymous, on loopback | signed in as `ada` |
| --- | --- | --- |
| `default` | read, write | read, write (anonymous's grant, plus her own) |
| `book` | the sign-in notice | read, write |
| delete, purge | refused | refused: not in her grant |
| the repositories | none | read, once a root is configured |
| a write's author | none | her passkey |

The pages added no authority and no state of their own. The ledger is the graph it was in the
last chapter; a page is a view of it, a form is one of its actions, and an identity is a name
for a list of tokens in `grants.json`. The next part reaches the same ledger from another
process, the way an agent does.
