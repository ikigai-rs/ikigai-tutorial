# Pages of your own

gonk's pages answer the questions every deployment has: what is open, what one item says, what
the Dashboard counts. Your team will have questions of its own. How far along is this epic? What
lands next? Who is sitting on an expired lease? A **face** is a page you write to answer one of
them, and gonk serves it from its own origin.

This part has three chapters. This one shows what a face is, what gonk refuses it, and builds a
first one that lists open items. [A board for an epic](board.md) and [Plans, projects and who has
what](plans.md) then read the starter's four samples.

## What a face is, and is not

A face is a directory of static files: an `index.html`, its scripts and its styles. gonk serves
them, and they run in the viewer's browser. Everything a face shows, it reads from gonk's
existing JSON routes, the same ones `curl` reads in [The ledger in a browser](../ledger/browser.md),
under the viewer's own session.

| A face is | A face is not |
| --- | --- |
| files you write, in a directory you name | a plugin: gonk runs none of its code |
| served by gonk, read-only, from that directory alone | a way to change gonk's own pages or their theme |
| a reader of gonk's routes, as the person viewing it | a new authority: it can read only what that person can |

So a face shows nothing its viewer could not read by hand. It can change nothing that gonk did not
already let that viewer change. The starter's faces only read.

## What you need

`ikigai-gonk` and `ikigai`, as [Which gonk](../introduction.md#which-gonk) installs them, plus
`git` and `curl`. The gonk this book pins is the first that serves faces (ikigai-gonk PR 295).
A browser is what a face is for. To check one from a shell, as this chapter does, you also need
[Deno](https://deno.com) 2, which runs the same JavaScript modules the browser loads.

## Get the template

The [gonk starter](https://github.com/ikigai-rs/ikigai-gonk-starter) carries a template and four
sample faces in `faces/`. In a new empty directory, clone it at the revision this book checks:

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ LC_ALL=C ls starter/faces starter/faces/template
starter/faces:
README.md
board
claims
flight
project
template

starter/faces/template:
README.md
app.css
app.js
index.html
kit.js
```

`template/` is the smallest face that works: a form that asks for an item and shows it. `kit.js`
holds what every face needs (an element builder that never parses markup, a same-origin JSON read,
gonk's paths), and every sample carries its own copy. Copy the template to a face of your own,
called `open`:

<!-- transcript: run -->
```console
$ mkdir faces
$ cp -R starter/faces/template faces/open
```

## The one config line

gonk learns about a face from one line in its config home: a name and a directory.

```toml
gonk.face = "open /path/to/faces/open"
```

The name becomes the face's address, so gonk is strict about it. A line that names no directory,
a name with a capital letter, or a line with no directory at all stops gonk at the start, before it
opens anything:

<!-- transcript: run -->
```console
$ mkdir refused
$ printf 'gonk.face = "open %s/faces/nowhere"\n' "$PWD" > refused/config.toml
$ ikigai-gonk --config-home "$PWD/refused" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk: gonk.face: face `open`: …/faces/nowhere is not a directory. Create it, or remove the line
  (config read from …/refused/config.toml)
$ printf 'gonk.face = "Open %s/faces/open"\n' "$PWD" > refused/config.toml
$ ikigai-gonk --config-home "$PWD/refused" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk: gonk.face: `Open` is not a face name: lowercase letters, digits and `-`, starting with a letter or digit
  (config read from …/refused/config.toml)
$ printf 'gonk.face = "open"\n' > refused/config.toml
$ ikigai-gonk --config-home "$PWD/refused" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk: gonk.face = "open" is not `<name> <directory>` (for example `gonk.face = "ops ~/ops-face"`)
  (config read from …/refused/config.toml)
```

Now write the real line. The line can be repeated, one per face.

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.face = "open %s/faces/open"\n' "$PWD" > .config/ikigai/config.toml
```

## Start gonk, and file something

If the gonk and the `model-stub` from [A query from a sentence](../project/sentence.md) are still
running, stop them first, since this gonk takes the same port ([A gonk per
chapter](../introduction.md#a-gonk-per-chapter)).

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

Over the owner's socket, file two items in `default` and one in a ledger called `notes`. The
anonymous caller at the HTTP door reads `default` and not `notes`, which matters below.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append labels=book Write the faces chapters'
#1 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=book,pin Move the gonk pin'
#2 urn:iki:ledger:default:item:…
[uncacheable]
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:notes:append A private note'
notes#1 urn:iki:ledger:notes:item:…
[uncacheable]
```

## Where a face lives

Each face gets a link in the header of every gonk page, named by its name:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/ | grep -o "href='/face/name/[^']*'"
href='/face/name/open/'
```

`/face/name/open/` serves the directory's `index.html`, and `/face/name/open/app.js` serves
`app.js`. Every other file is under the same prefix.

A face's address ends in `/`, so the template's relative `app.css` and `app.js` resolve
inside that face's directory. The header link includes the slash. A request without it
redirects to that address:

<!-- transcript: run -->
```console
$ curl -s -o /dev/null -w '%{http_code}\n' http://localhost:1070/face/name/open/
200
$ curl -s -o /dev/null -w '%{http_code}\n' http://localhost:1070/face/name/open
308
```

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open http://localhost:1070/face/name/open/
```

The template asks for an item. Give it `2` and it shows "Move the gonk pin", its state and its
labels, with a link to gonk's own page for the item.

## What the jail refuses

gonk reads the face's files through a file endpoint jailed to the directory. It serves only names
inside the directory, only a fixed list of file types, and nothing hidden. To see it, put a secret
beside the face, a hidden file and a shell script inside it, and a link that points out:

<!-- transcript: run -->
```console
$ echo 'not for the web' > faces/secret.txt
$ echo 'TOKEN=not-for-the-web' > faces/open/.env
$ echo 'echo hi' > faces/open/run.sh
$ ln -s ../secret.txt faces/open/secret.txt
```

None of them is served. A `..` is refused whether it is written plainly or percent-encoded. curl's
`--path-as-is` sends the path as typed, as an attacker would; without it, some curl versions remove
the `..` before sending:

<!-- transcript: run -->
```console
$ curl -s --path-as-is -w ' [%{http_code}]\n' http://localhost:1070/face/name/open/../secret.txt
not found: urn:face:name:open:..:secret.txt: a face serves no hidden name [404]
$ curl -s --path-as-is -w ' [%{http_code}]\n' http://localhost:1070/face/name/open/%2E%2E/secret.txt
not found: urn:face:name:open:..:secret.txt: a face serves no hidden name [404]
$ curl -s -w ' [%{http_code}]\n' http://localhost:1070/face/name/open/.env
not found: urn:face:name:open:.env: a face serves no hidden name [404]
$ curl -s -w ' [%{http_code}]\n' http://localhost:1070/face/name/open/secret.txt
invalid argument `path`: `secret.txt` is a symbolic link; this endpoint never follows links [400]
$ curl -s -w ' [%{http_code}]\n' http://localhost:1070/face/name/open/run.sh
not found: urn:face:name:open:run.sh: a face serves only .html, .css, .js, .mjs, .json, .jsonld, .ttl, .txt, .svg, .png, .jpg, .webp, .ico, .woff2 [404]
```

That last line is the whole list of file types a face may have at this pin. The starter's
`faces/README.md` also lists `.md`, which this gonk does not serve. And a face is read-only: nobody,
the owner included, can write into it through gonk.

<!-- transcript: run -->
```console
$ curl -s -X PUT --data 'alert(1)' -w ' [%{http_code}]\n' http://localhost:1070/face/name/open/app.js
method not allowed [405]
$ rm faces/secret.txt faces/open/.env faces/open/run.sh faces/open/secret.txt
```

## What the CSP refuses

A face is served under the HTTP door's default **Content Security Policy**, the same one gonk's own
pages carry, and it sets none of its own:

<!-- transcript: run -->
```console
$ curl -sI http://localhost:1070/face/name/open/ | grep -i '^content-security-policy'
Content-Security-Policy: default-src 'self'; script-src 'self'; style-src 'self'; object-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'
```

`script-src 'self'` means a script must be a file from gonk's own origin. A `<script>` element with
code inside it, or an `onclick=` attribute, does not run. `style-src 'self'` does the same for a
`<style>` element and a `style=` attribute. Write a page that tries it:

<!-- transcript: file faces/open/inline.html -->
```text
<!doctype html>
<html lang="en">
<head><meta charset="utf-8"><title>Inline</title></head>
<body>
  <main>
    <h1>Inline</h1>
    <p id="said">The inline script did not run.</p>
    <script>document.getElementById("said").textContent = "The inline script ran.";</script>
  </main>
</body>
</html>
```

gonk serves the file as written; the jail reads files, not HTML. The header that comes with it is
what makes the browser refuse the script:

<!-- transcript: run -->
```console
$ curl -s -o /dev/null -w '%{http_code} %{content_type}\n' http://localhost:1070/face/name/open/inline.html
200 text/html
$ curl -sI http://localhost:1070/face/name/open/inline.html | grep -io "script-src 'self'"
script-src 'self'
```

<!-- transcript: manual — it opens a browser; the console's message is the browser's, worded differently by each -->
```console
$ open http://localhost:1070/face/name/open/inline.html
```

The page says "The inline script did not run.", and the browser's developer console reports the
refused script as a Content Security Policy violation. This is why the starter's faces keep every
script in a file, attach handlers with `addEventListener`, and style only through classes in
`app.css`. Its `kit.js` throws if a face asks it for a `style` or `on…` attribute, rather than
letting the browser drop it silently. Remove the page:

<!-- transcript: run -->
```console
$ rm faces/open/inline.html
```

## Why it lives on gonk's origin

A face could be any static site. It lives on gonk's origin because of who is asking. A
person signed in to gonk with a passkey holds a session cookie for that origin, and only for it. A
page on gonk's origin reads gonk's routes with that session, so it sees what that person may see.
A page on another origin cannot borrow the session, and gonk refuses a read that another site's
page loads (gonk PR 100, ledger #880), as [The ledger in a
browser](../ledger/browser.md) showed with an image.

A browser says which kind of read it is making, in its `Sec-Fetch-*` headers. Here are the face's
read and the same read from another site's page:

<!-- transcript: run -->
```console
$ curl -s -o /dev/null -w '%{http_code}\n' -H 'Accept: application/json' -H 'Sec-Fetch-Site: same-origin' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Dest: empty' http://localhost:1070/iki/ledger/items
200
$ curl -s -o /dev/null -w '%{http_code}\n' -H 'Accept: application/json' -H 'Sec-Fetch-Site: cross-site' -H 'Sec-Fetch-Mode: cors' -H 'Sec-Fetch-Dest: empty' http://localhost:1070/iki/ledger/items
403
```

The face itself holds nothing. Its files are added to the anonymous loopback caller's grant, as the
toolbox's are, so anyone who can open gonk's pages on this machine can load the face. What the face
then reads is judged under the viewer's own grant. The anonymous viewer here cannot read `notes`,
so the face cannot either:

<!-- transcript: run -->
```console
$ curl -s -H 'Accept: application/json' http://localhost:1070/iki/ledger/notes/items | cut -d. -f1
denied: this capability does not hold `urn:cap:ledger:read:notes`
```

A viewer reaching gonk from another machine loads the face only if their passkey grant includes
reading its directory (`urn:cap:fs:read:{dir}`). The socket and QUIC doors serve no faces at all.

## A first page: the open items

Now turn the template into a page that answers one question: what is open in this ledger? Build
it in four steps.

### 1. Read the route the page will read

A face reads what `curl` reads. A ledger's listing is `/iki/ledger/items` for `default` and
`/iki/ledger/{name}/items` for any other, and `Accept: application/json` asks for its JSON face. It
answers open items, with a `total` to page by:

<!-- transcript: run -->
```console
$ curl -s -H 'Accept: application/json' 'http://localhost:1070/iki/ledger/items?status=open' | jq -c '{total, items: [.items[] | {number, title, priority, labels, claim, awaiting}]}'
{"total":2,"items":[{"number":2,"title":"Move the gonk pin","priority":1,"labels":["book","pin"],"claim":null,"awaiting":null},{"number":1,"title":"Write the faces chapters","priority":null,"labels":["book"],"claim":null,"awaiting":null}]}
```

`kit.js` already knows this route: `listAll(ledger, params)` reads every page of a listing, and
`stateOf(item)` turns `status`, `claim` and `awaiting` into words (open, claimed, expired,
awaiting, done).

### 2. Put the question in a module of its own

Keep the question apart from the page, in a module that touches no DOM. A browser and a shell can
then both run it. Save this as `faces/open/open.js`:

<!-- transcript: file faces/open/open.js -->
```js
// The open face's question, apart from the page: a ledger's open items, most urgent first.
// No DOM here, so a shell can run it as well as a browser.
import { listAll, paths, stateOf } from "./kit.js";

// Lower priority number first, no priority last, then the item number.
function rank(a, b) {
  return (a.priority ?? Infinity) - (b.priority ?? Infinity) || a.number - b.number;
}

// One row per open item: what the page shows, and where its link goes.
export async function openItems(ledger, now = Date.now()) {
  const items = await listAll(ledger, { status: "open" });
  return items.sort(rank).map((it) => ({
    label: `${ledger}#${it.number}`,
    title: it.title || "",
    state: stateOf(it, now),
    labels: it.labels || [],
    href: paths.itemPage(ledger, it.number),
  }));
}
```

### 3. Draw it

Replace the template's `app.js` with one that asks `open.js` and draws a list. Every element is
made by `el`, so a title holding `<script>` stays text, and the result is announced in the page's
one `role="status"` region:

<!-- transcript: file faces/open/app.js -->
```js
// The open face: a ledger's open items, each linked to gonk's own page for it.
import { el, announce, query, setQuery } from "./kit.js";
import { openItems } from "./open.js";

const form = document.getElementById("pick");
const out = document.getElementById("out");

function row(r) {
  const note = r.labels.length ? `${r.state}; ${r.labels.join(", ")}` : r.state;
  return el("li", {}, el("a", { href: r.href }, r.label), ` ${r.title} `, el("span", { class: "muted" }, `(${note})`));
}

async function show(ledger) {
  announce(`Reading ${ledger}…`);
  out.replaceChildren(el("h2", { id: "out-h" }, "Open items"));
  try {
    const rows = await openItems(ledger);
    out.append(rows.length ? el("ul", { class: "items" }, rows.map(row)) : el("p", { class: "muted" }, "Nothing is open."));
    announce(`${rows.length} open items in ${ledger}.`);
  } catch (error) {
    out.append(el("p", { class: "warn" }, `Could not read it: ${error.message}`));
    announce(`Could not read ${ledger}.`);
  }
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  const ledger = form.ledger.value.trim() || "default";
  setQuery({ ledger });
  show(ledger);
});

form.ledger.value = query().get("ledger") || "default";
show(form.ledger.value);
```

And replace `index.html`, so the form asks for a ledger alone. The page loads its one stylesheet
and its one module as files, which is all the CSP allows:

<!-- transcript: file faces/open/index.html -->
```html
<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Open items</title>
  <link rel="stylesheet" href="app.css">
  <script type="module" src="app.js"></script>
</head>
<body>
  <main>
    <h1>Open items</h1>
    <p class="muted">A ledger's open items, most urgent first, each linked to gonk's own page.</p>
    <form id="pick">
      <label>Ledger <input name="ledger" value="default" autocomplete="off"></label>
      <button type="submit">Show</button>
    </form>
    <p role="status" aria-live="polite"></p>
    <section id="out" aria-labelledby="out-h">
      <h2 id="out-h">Open items</h2>
    </section>
  </main>
</body>
</html>
```

gonk serves the new files at once, with no restart:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/face/name/open/ | grep -o '<h1>[^<]*</h1>'
<h1>Open items</h1>
$ curl -s -o /dev/null -w '%{http_code} %{content_type}\n' http://localhost:1070/face/name/open/open.js
200 text/javascript
```

### 4. Check it, from a shell and in a browser

Deno runs the same module the browser loads. `--location` gives it the face's address, so `kit.js`
reads gonk's origin as the page does, and `--allow-net` lets it reach that origin and nothing else.
`DENO_DIR` keeps Deno's cache in this directory, as `--data-home` does for gonk. Save the check as
`show.js`:

<!-- transcript: file show.js -->
```js
// What the open face shows, printed: the face's own module, run as the anonymous viewer.
import { openItems } from "./faces/open/open.js";

const ledger = Deno.args[0] || "default";
try {
  for (const r of await openItems(ledger)) console.log(`${r.label} ${r.title} (${r.state}) ${r.href}`);
} catch (error) {
  console.log(`Could not read it: ${error.message}`);
}
```

<!-- transcript: run -->
```console
$ DENO_DIR="$PWD/.deno" deno run --location http://localhost:1070/face/name/open/ --allow-net=localhost:1070 show.js
default#2 Move the gonk pin (open) /l/default/item/2
default#1 Write the faces chapters (open) /l/default/item/1
$ DENO_DIR="$PWD/.deno" deno run --location http://localhost:1070/face/name/open/ --allow-net=localhost:1070 show.js notes
Could not read it: /iki/ledger/notes/items: 403 Forbidden
```

The second run is the anonymous viewer asking for `notes`: the face says what failed, in words,
and shows nothing. Each link goes to gonk's own page for the item:

<!-- transcript: run -->
```console
$ curl -s -o /dev/null -w '%{http_code}\n' http://localhost:1070/l/default/item/2
200
```

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open 'http://localhost:1070/face/name/open/?ledger=default'
```

In the browser, check what the shell cannot. The starter's `faces/README.md` lists these checks,
and none of them has been run by this book's checks:

- what the page shows matches gonk's own ledger page;
- the developer console reports no CSP violation;
- Tab reaches the field, the button and every link, in reading order, with a visible focus ring;
- at 320 px wide nothing scrolls sideways;
- it reads in both light and dark mode;
- an accessibility audit (axe, or the browser's own) finds nothing.

A face's markup is yours, and so is its accessibility. gonk's checks cover gonk's pages and the
header link, not what a deployment serves.

## What you have

A face is a directory and one line. gonk serves it read-only and jailed, under a CSP that runs only
its files, and every read it makes is the viewer's own. Next, [A board for an
epic](board.md) reads the starter's first sample: an item's children, their `blocks` links, and
a column for each.
