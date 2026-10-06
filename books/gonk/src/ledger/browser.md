# The ledger in a browser

The last chapter reached gonk with `curl`, through paths that are a mechanical mapping of the
ledger's own resources. This one opens the same server the way a person does: pages, forms, a
sign-in. Everything a browser does here is still a request to a resource, so every step but
one can be checked from a shell, and this chapter does check them. The one that cannot is the
passkey ceremony, which needs a browser and an authenticator; it is marked, and described.

## Start one, and keep what it says

Start a scratch gonk as before, in an empty directory that is its home, and this time keep
what it prints in a file as well as on the screen:

<!-- transcript: serve -->
```console
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk --port 1070 --no-quic --no-backup 2>&1 | tee gonk.log
ikigai-gonk 0.1.0 — holding the store at …/.ikigai/store
  http    http://localhost:1070/ — loopback (127.0.0.1:1070); anonymous read+write: default; 0 passkey(s)
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
| `/sparql` | a query over one ledger's graph, rendered as a table |

Every one of those pages is a **transform of a graph face**. The ledger page is
`urn:iki:ledger:default:items` asked for as Turtle and rendered by one XSLT stylesheet,
server-side, so the page cannot show anything the graph does not say. The graph is right
there:

<!-- transcript: run -->
```console
$ curl -s -H 'Accept: text/turtle' http://127.0.0.1:1070/iki/ledger/items | grep dcterms:title
	dcterms:title "Write the gonk book" ;
```

and so is a query over it. The SPARQL page answers a program in the store's own formats
when it is asked for one, CSV among them:

<!-- transcript: run -->
```console
$ curl -s -G -H 'Accept: text/csv' --data-urlencode 'query=PREFIX dcterms: <http://purl.org/dc/terms/> SELECT ?title WHERE { ?item dcterms:title ?title }' http://127.0.0.1:1070/sparql
title
Write the gonk book
```

The query runs over the default ledger's graph and nothing else. The page sets that graph as
the query's whole dataset, under your grant, so a grant naming one ledger cannot read another
through it, and `FROM` is refused rather than quietly ignored.

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
signals take the grant away. A write whose `Origin` names another site gets **nothing**:

<!-- transcript: run -->
```console
$ curl -s -H 'Origin: https://elsewhere.example' -d _action=append -d _ledger=default -d _then=items --data-urlencode 'content=Not from here' http://localhost:1070/act
denied: capability does not grant `urn:cap:store:read:graph:*` (declared by `urn:iki:gonk:act`)
```

`curl` sends no `Origin` unless told to, and neither does a script, which is why the requests
before it were unaffected. A request whose `Host` is anything but `localhost`, `127.0.0.1` or
`[::1]` gets nothing too, reads included; that is the defense against DNS rebinding.

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
$ env HOME="$PWD" XDG_CONFIG_HOME="$PWD/.config" ikigai-gonk passkey invite ada --ledger book=write --browse read --port 1070
passkey invite for `ada` — grant `ada` (6 scopes) in …/.config/ikigai/gonk/grants.json
  valid for 30 minutes, once
  open  http://localhost:1070/#invite=…
  in a browser on this machine (localhost, not 127.0.0.1); the passkey is enrolled in …/.config/ikigai/gonk/clients.json
```

The same `HOME` again, so the invite is written into this gonk's config home rather than a
real one; and `--port 1070`, so the link it prints points at this gonk. Each flag is a
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
The repositories arrive in the repository browser part of this book.

Two rules sit behind the command. An identity must be **strictly stronger** than anonymous, so
`passkey invite ada --ledger default=write` is refused: signing in would change nothing. And
a grant name is shared by every passkey and certificate enrolled under it, so writing
different scopes under an existing name is refused unless `--force`, naming each scope the
rewrite would remove and add.

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
```

Read them with three things in mind:

- **The subject is the resource, not the URL.** `/l/default/item/1` was
  `urn:iki:gonk:page:item:default:1`; the form was `urn:iki:gonk:act`.
- **`principal` is who a WRITE was from.** The form's line says `anon`; a page read says `-`,
  because the HTTP library tells a door who a write is from and never a read. A signed-in
  write would name the passkey's IRI.
- **`outcome` is the kernel's, not HTTP's.** A refusal the resource makes (the `/l/book`
  page) is a line with `outcome=denied`. A refusal the door makes before it dispatches
  anything (the cross-site form) is no line at all: there was no request to log.

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
