# The ledger in a browser

In [A ledger of your own](your-own.md), you started a scratch gonk, read its ledger with
`curl`, and wrote to it with `ikigai` over the owner's socket. This chapter starts a fresh gonk
and opens its ledger in a browser: pages, forms, and a sign-in. Everything a browser does here
is still a request to a resource, so every step but one can be checked from a shell, and this
chapter does check them. The one that cannot is the
passkey ceremony, which needs a browser and an authenticator; it is marked, and described.

## Start one, and keep what it says

Start a scratch gonk as before, with both its homes in a new empty directory, and this time keep
what it prints in a file as well as on the screen. If the one from [A ledger of your
own](your-own.md) is still running, stop it first (Ctrl-C in its terminal): this one takes the same
port, and the page's answers count from an empty ledger ([A gonk per
chapter](../introduction.md#a-gonk-per-chapter)).

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
  instance … — new at every start; open pages compare it
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read: default; 0 passkey(s), sessions 12 h or 60 min idle; anonymous SPARQL budget 1000 ms
…
  log     segments in …/.config/ikigai/gonk/log (hash-chained; `urn:log:segments`, `urn:log:verify` over the socket only), not in the backup; one `log:Access` entry per request at each door, polls sampled (`gonk.log.access = false` turns them off); passkey sign-ins and sign-outs
…
```

gonk writes its banner and fatal errors to standard error; `2>&1 | tee gonk.log` keeps
those on screen and in a file. Access records go into hash-chained segments under
`.config/ikigai/gonk/log/`, which this chapter reads at the end. These segments are outside
the store and its backup. Leave gonk running, then open another terminal in the **same
directory** for the remaining commands.

File something, so the pages have an item to show. An anonymous caller at the HTTP door only
reads (the banner's `anonymous read: default`), so file it over the owner's socket, as in [the
last chapter](your-own.md#served-by-gonk):

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append Write the gonk book'
#1 urn:iki:ledger:default:item:…
[uncacheable]
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
and all and a title search; signed in under a grant that writes the ledger, it has a form that
files an item as well. A page is HTML, and HTML is something
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

## Render rules and CURIEs

The HTML results follow a rule table served at `/render-rules`. It matches a cell's term kind,
value and prefix, so an item IRI can link to its item regardless of the query's variable name.
The bare-integer rule also checks the column name: a count of three comments must not link to
item 3. Read the table the running server uses:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/render-rules | grep '^ *render:curie' | awk '{$1=$1; print}'
render:curie "ledger" .
```

Abbreviation is a rule too (gonk PR 357). A deployment can put this rule in its replacement table
at `<config home>/gonk/render-rules.ttl`:

```turtle
@prefix render: <urn:iki:gonk:render#> .
<urn:example:curie> a render:Rule ;
    render:matchKind "uri" ;
    render:matchPrefix "https://example.org/vocab#" ;
    render:curie "example" .
```

An IRI `https://example.org/vocab#open` then displays as `example:open`. Above the table, a
visible legend binds each prefix actually used once, including prefixes in linked cells.
The shipped table abbreviates the ledger vocabulary as `ledger:`. Display and control rules
match independently: links still match and substitute the full IRI, and their accessible names
include the visible CURIE. Each side orders rules by `render:order`, then the rule IRI.
A CURIE rule needs one namespace and `matchKind "uri"`; conflicting prefix bindings stop startup.
Replacing the table replaces abbreviation and controls together; an empty table leaves full IRIs
without controls. CSV and other machine results keep the original IRIs. Compare the same
constant in the two representations:

<!-- transcript: run -->
```console
$ curl -s -G --data-urlencode 'query=SELECT (<https://ikigai-rs.dev/ns/ledger#open> AS ?state) WHERE {}' http://localhost:1070/sparql > results.html
$ grep -o "<td class='uri'[^<]*</td>" results.html | sed 's/<[^>]*>//g'
ledger:open
$ grep -o 'ledger: is short for [^<]*' results.html
ledger: is short for https://ikigai-rs.dev/ns/ledger# — this table writes it that way.
$ curl -s -G -H 'Accept: text/csv' --data-urlencode 'query=SELECT (<https://ikigai-rs.dev/ns/ledger#open> AS ?state) WHERE {}' http://localhost:1070/sparql
state
https://ikigai-rs.dev/ns/ledger#open
```

## A form is a ledger action

The form a signed-in writer sees does not write anything gonk invented. It posts to `/act`,
which names an action the ledger already declares (`_action=append`), checks every field
against that action's contract, and issues it under your grant. This is the request your
browser sends when you file an item there, here sent with no one signed in:

<!-- transcript: run -->
```console
$ curl -s -w '\n%{http_code}\n' -d _action=append -d _ledger=default -d _then=items --data-urlencode 'content=Read it in a browser' http://localhost:1070/act
denied: capability does not grant `urn:cap:store:write:graph:*` (declared by `urn:iki:gonk:act`; any grant under `urn:cap:store:write:graph:` satisfies it)
403
```

Anonymous, the action is refused by the grant: the anonymous caller holds no write token at all
(gonk PR 186, ledger #1687; before it this request filed the item). Signed in under a grant that
writes `default`, the same request files the item, and the answer is the fragment of the page
that changed (`_then=items`), which the page swaps in place with [htmx](https://htmx.org). There
is no single-page application and no build step: the server renders, the browser shows.

A door that a browser writes through can be tried by any web page open on this machine, too: a
page on any site can post a form to `localhost`, and the browser would send it with your
sign-in cookie. So the browser's own signals are a **refusal**. A write whose `Origin` (or `Sec-Fetch-Site`) names another site is
answered `403` at the edge, before it reaches anything it names:

<!-- transcript: run -->
```console
$ curl -s -w '\n%{http_code}\n' -H 'Origin: https://elsewhere.example' -d _action=append -d _ledger=default -d _then=items --data-urlencode 'content=Not from here' http://localhost:1070/act
a write from another site is refused: its Origin or Sec-Fetch-Site names a page that is not this server's own
403
```

`curl` sends no `Origin` unless told to, and neither does a script, which is why the request
before it was refused by the grant rather than at the edge. A request whose `Host` is anything but `localhost`, `127.0.0.1` or
`[::1]` (with this port) is refused too, reads included, and the front page is no
exception; that is the defense against DNS rebinding, where a page on another site reaches a
loopback server under its own name:

<!-- transcript: run -->
```console
$ curl -s -w '\n%{http_code}\n' -H 'Host: elsewhere.example' http://127.0.0.1:1070/
this server answers only to its own loopback name (localhost, 127.0.0.1 or [::1], with its port); a request under another Host is refused — it is how a DNS rebinding page reaches a loopback server
421
```

The status is `421 Misdirected Request` (since gonk e45e5f9, `403` before): the request named a
server that is not this one, which is a different refusal from this server declining a request
made to it, and the cross-site write above stays `403`. Refused, rather than served with an empty grant: an empty grant is still offered everything
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

## When gonk restarts

Every full page carries the process instance id from the startup banner in a `gonk-instance`
meta tag. `/instance` answers that same id as plain text; it changes at every start, even when
the binary version and store stay the same (gonk PR 356):

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/instance > instance.txt
$ curl -s http://localhost:1070/ | grep -o "<meta name='gonk-instance' content='[^']*'>" | grep -F "$(cat instance.txt)"
<meta name='gonk-instance' content='…'>
```

A visible tab checks once a minute, and checks immediately when it becomes visible again or
returns from the browser's back/forward cache. When the id differs, it shows “gonk has restarted
since this page loaded, so what it shows may be out of date. Reload this page” and announces it
once through the page's status region. It stops polling and leaves focus where it was. Failed
checks stay silent; without JavaScript no restart notice appears. Reload to get the new page
and sign in again: sessions end on restart.

## Who you are, and what that grants

An anonymous caller on this machine reads the ledgers in `gonk.http.ledger` (`default`, unless
configured), and nothing else. A ledger outside that list is closed to it,
and the refusal says so in words before it says it in tokens:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/l/book
denied: Sign in to read the ledger "book". Sign in with passkey is in the header of every gonk page (the front page is /); then come back to this address.
The grant it needs: urn:cap:ledger:read:book (to read the ledger "book")
```

That is the notice a signed-out visitor reads, here and on every page that needs more than
anonymous authority: the one sentence that fits a signed-out caller, and where the **Sign in with
passkey** button is (a browser that navigates to the page gets gonk's page with the button on
it). A signed-in visitor whose grant does not reach the ledger is told that instead. Anything
more than anonymous, such as filing, delete, another ledger, or reading the repositories gonk
browses, belongs to an **identity**, and an identity is given by invitation:

<!-- transcript: run -->
```console
$ ikigai-gonk passkey invite ada --ledger book=write --browse read --port 1070 --config-home "$PWD/.config/ikigai"
passkey invite for `ada` — grant `ada` (7 scopes) in …/.config/ikigai/gonk/grants.json
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
    "urn:cap:store:read:graph:urn:collab:participants",
    "urn:cap:browse:read:*",
    "urn:cap:store:read:graph:urn:iki:browse:graph:default"
  ]
}
```

`--ledger book=write` is the writer of one ledger from
[the last chapter](your-own.md#whose-ledger), plus the read of `urn:collab:participants`, the graph
that says who may claim which author ([The three doors](../machines/doors.md#who-wrote-it)).
`--browse read` is the repositories as a role:
read every browse root (`--root <name>` names one instead), and the browse graph's quads. Its
stronger sibling, `--browse derive`, adds minting annotations and spending a mounted model's
inference; it is one word on purpose, and it is refused on a server with no model to spend.
The repositories arrive in [Browse roots](../browse/roots.md).

Two rules sit behind the command. An identity must be **strictly stronger** than anonymous, so
`passkey invite ada --ledger default=read` is refused: signing in would change nothing.
(`default=write` is stronger now that anonymous only reads, and is accepted.) And
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
   signed in. By default, the session ends when you sign out, after twelve hours, after
   sixty minutes with no request using that session, or when the server restarts.

The second click is deliberate: when the creation sheet closes it still holds the window's
focus, and a browser refuses a sign-in prompt from a page without it. The invite is
single-use and expires; gonk stores only its hash, and it is the whole trust anchor of the
registration, so no attestation is parsed. After signing in, `/l/book` is a ledger page
rather than a refusal, and every write `ada` makes carries her passkey as its author.

### Extra scopes and adding a grant

`passkey invite` also accepts repeatable `--scope <urn>` for tokens no role flag names (gonk
PR 348). They join the role tokens in one grant write, with duplicates stored once. A scope
must be a valid, permitted `urn:cap:` token; malformed values, owner tokens, door markers and
broad `exec:*` or `net:*` tokens are refused before anything is written. For example, a face
that reads a mounted socket peer needs its read token and `urn:cap:net:localhost`:

<!-- transcript: run -->
```console
$ ikigai-gonk passkey invite metrics --ledger book=read --scope urn:cap:metrics:read --scope urn:cap:net:localhost --port 1070 --config-home "$PWD/.config/ikigai"
passkey invite for `metrics` — grant `metrics` (4 scopes) in …/.config/ikigai/gonk/grants.json
  valid for 30 minutes, once
  open  http://localhost:1070/#invite=…
  in a browser on this machine (localhost, not 127.0.0.1); the passkey is enrolled in …/.config/ikigai/gonk/clients.json
```

With `--force`, the grant becomes exactly the role flags plus the `--scope` values, including
narrowing: leaving an old scope off removes it. Rewriting a shared grant affects every identity
using it. To widen just one enrolled passkey, the owner instead runs `ikigai-gonk passkey grant
add <label> <grant> --config-home <config home>` (or selects it with `--credential <id>`).
The grant must already exist with scopes and the passkey must not already hold it. The command
changes that passkey alone, records `PasskeyGrantAdded` with `by=owner`, and leaves its live
sessions signed in; their next request sees the union of its grants (gonk PR 349).
`passkey list` shows the grants, their scopes and the addition. `passkey grant remove` removes
one grant; removing the last is refused, so use `passkey revoke` to remove the passkey.

An invite redeemed while signed in can instead add its grant to that passkey after a fresh
passkey assertion, or create a new passkey. On the add path, gonk ends **all old sessions before
writing the wider grant**, then opens one new session from the assertion (gonk PR 352). An old
cookie cannot read the wider grant in between. If recording the widening fails, the grant change
is undone and the passkey stays signed out; sign in again. Owner `grant add` deliberately keeps
sessions; the owner can end them with `passkey logout --all`.

## What it wrote down

Each dispatched request gets a `log:Access` entry in a segment under
`.config/ikigai/gonk/log/`. The grammar is `ikigai-log`'s: when the request started, a class,
the resource, a sequence number, then the door's columns. `credential` says how the door
knew the principal; an error has no `bytes`, and an absent query has no `q`.

<!-- transcript: run -->
```console
$ grep ' urn:iki:gonk:act ' .config/ikigai/gonk/log/*.log
… log:Access urn:iki:gonk:act seq=… door=http verb=sink outcome=denied dur=0 principal=anon credential=https://ikigai-rs.dev/ns/log#anonymous
$ grep ' urn:iki:gonk:page:item:default:1 ' .config/ikigai/gonk/log/*.log
… log:Access urn:iki:gonk:page:item:default:1 seq=… door=http verb=source outcome=ok bytes=… dur=… principal=anon credential=https://ikigai-rs.dev/ns/log#anonymous q=as=text/html
… log:Access urn:iki:gonk:page:item:default:1 seq=… door=http verb=source outcome=denied dur=0 principal=anon credential=https://ikigai-rs.dev/ns/log#anonymous
… log:Access urn:iki:gonk:page:item:default:1 seq=… door=http verb=source outcome=ok bytes=… dur=… principal=anon credential=https://ikigai-rs.dev/ns/log#anonymous q=as=text/html
```

Read them with four things in mind:

- **The resource, not the URL, is the subject.** `/l/default/item/1` (asked three times: by
  `curl` as itself, as an image another page loads, and as another page's link) was
  `urn:iki:gonk:page:item:default:1`; the form was `urn:iki:gonk:act`.
- **`principal` is who the door could say was asking.** An anonymous read or write says
  `anon`; a signed-in request names the passkey's IRI. The door mints that identity into the
  request's capability (core's principal scope), which is where the log reads it.
- **`outcome` is the kernel's, not HTTP's.** A refusal the resource makes (the `/l/book`
  page, below) is a line with `outcome=denied`. A refusal at the edge, before the kernel (the
  cross-site form, the foreign `Host`, the loaded image), is a line too, written by the edge
  itself: `outcome=denied dur=0`, under the resource the path would have reached, so
  the one `act` line is the cross-site form and the middle item line is the image. Probing
  shows up in the log. (Before gonk PR 151 an edge refusal wrote no line at all.) The anonymous
  form post has no line: the anonymous grant does not even hold the family of token `act`
  declares, so the kernel refused it before dispatching it to anything that logs
  ([What gonk knew](../machines/doors.md#what-gonk-knew)).
- **`q` is the arguments the request ran with.** An edge refusal ran nothing, so `q` is absent;
  a page that was rendered says `as=text/html`, the representation the door asked the resource
  for. The kernel fills in every default a resource declares before dispatching it (`ikigai-core`
  0.1.104 and later), so a page over a listing carries the listing's defaults too:

<!-- transcript: run -->
```console
$ grep ' urn:iki:gonk:page:ledger:book ' .config/ikigai/gonk/log/*.log
… log:Access urn:iki:gonk:page:ledger:book seq=… door=http verb=source outcome=denied dur=… principal=anon credential=https://ikigai-rs.dev/ns/log#anonymous q=as=text/html&limit=50&offset=0&status=open
```

That is the `/l/book` refusal above: the first fifty open items, as a page, which is what the
page would have shown had the grant reached `book`.

`dur` is milliseconds, so `grep ' dur=[0-9]\{4,\} ' .config/ikigai/gonk/log/*.log` lists every request that took a
second or more. The first page this gonk rendered is usually one of them: it compiled the
stylesheet.

## What you have now

| resource | anonymous, on loopback | signed in as `ada` |
| --- | --- | --- |
| `default` | read | read (anonymous's grant; hers names only `book`) |
| `book` | the sign-in notice | read, write |
| delete, purge | refused | refused: not in her grant |
| the repositories | none | read, once a root is configured |
| a write's author | none | her passkey |

The pages added no authority and no state of their own. The ledger is the graph it was in the
last chapter; a page is a view of it, a form is one of its actions, and an identity is a name
for a list of tokens in `grants.json`. [Checking what you have](checking.md) checks what those
graphs say. Then [The Dashboard](dashboard.md)
reads every ledger at once, and [A lifecycle for your ledger](lifecycle.md) gives a ledger states
of its own. The following part, [An agent's ledger](../agent/socket.md), reaches the ledger from
another process, the way an agent does. (What a signed-out caller sees of a gonk that two teams share is in
[Two teams, one gonk](../teams/two-teams.md#what-a-signed-out-caller-sees).)
