# A board for an epic

An **epic** is an item whose children are the work: five chapters of a book, three arcs of a
release. gonk's own item page lists an epic's children. It does not say which of them can be worked
now, which wait on a sibling, and how far along the whole is. The starter's `board` face answers
that, in five columns.

This chapter runs the sample as it ships. It assumes [the last chapter](own.md): what a face is,
the config line, and why the address ends in a slash.

## What it reads

The board reads one item, the epic, and then each of its children, all through gonk's JSON item
route. Two fields of an item's JSON face carry the structure:

- **`children`**: every item whose `parent` link names this one, open and closed alike.
- **`links`**: this item's own links, each a `type` (`blocks`, `parent` or `related`) and a
  `target`. "A blocks B" is a link on A.

It never lists the ledger to find the children. A listing answers open items, so a child that is
already done would be missing, and the board would show no progress.

## The sample, and its line

In a new empty directory, clone the starter at the revision this book checks. Its `tools/new.py`
generates a repository that carries `faces/`, and `--face board` prints the line for that
repository's board:

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ (cd starter && python3 tools/new.py ../site --face board | tail -5)
add to gonk's config.toml (ikigai-gonk … or later), then restart gonk:
  gonk.face = "board …/site/faces/board"
  (or later: python3 tools/gonk.py init --face board)
AGENTS.md holds this process's rules; README.md walks through the sample.
$ LC_ALL=C ls site/faces/board
README.md
app.css
app.js
board.js
index.html
kit.js
```

The revision it names is the one this book pins. `board.js` holds the board's logic and touches no DOM, the way `open.js` did in the last chapter.
`app.js` draws it. Write the line into gonk's config home:

<!-- transcript: run -->
```console
$ mkdir -p .config/ikigai
$ printf 'gonk.face = "board %s/site/faces/board"\n' "$PWD" > .config/ikigai/config.toml
```

If the gonk from [Pages of your own](own.md) is still running, stop it first, since this one takes
the same port.

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

## An epic and its children

File an epic and five children over the owner's socket, then link each child to the epic with
`parent`. One child `blocks` another: the last chapter cannot be written before the pin moves.

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=epic Faces in the gonk Book' -c 'sink urn:gk:iki:ledger:append labels=docs Pages of your own' -c 'sink urn:gk:iki:ledger:append labels=docs A board for an epic' -c 'sink urn:gk:iki:ledger:append labels=docs,pin Plans, projects and who has what' -c 'sink urn:gk:iki:ledger:append labels=pin Move the gonk pin' -c 'sink urn:gk:iki:ledger:append labels=docs Write the release note' 2>/dev/null
#1 urn:iki:ledger:default:item:…
#2 urn:iki:ledger:default:item:…
#3 urn:iki:ledger:default:item:…
#4 urn:iki:ledger:default:item:…
#5 urn:iki:ledger:default:item:…
#6 urn:iki:ledger:default:item:…
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:link item=2 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=3 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=4 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=5 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=6 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=5 type=blocks 4' 2>/dev/null
linked #2 parent #1
linked #3 parent #1
linked #4 parent #1
linked #5 parent #1
linked #6 parent #1
linked #5 blocks #4
```

Now move the children along, as [Hub and spoke](../ways/hub-and-spoke.md) did by hand. `#2` is
closed. `ada` claims `#3` on a lease. `bob` claims `#6`, hands it back with a comment, and
releases it, so it waits on its dispatcher:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=2 reason=done author=ada Written and merged.' -c 'sink urn:gk:iki:ledger:claim item=3 lease=2h ada' -c 'sink urn:gk:iki:ledger:claim item=6 lease=2h bob' -c 'sink urn:gk:iki:ledger:comment item=6 author=bob handback=true Note written; please check it.' -c 'delete urn:gk:iki:ledger:claim item=6 from=bob' 2>/dev/null
closed #2 (done)
#3 claimed by ada at epoch 1 until …
#6 claimed by bob at epoch 1 until …
commented on #6 (urn:iki:ledger:default:comment:…) awaiting-dispatcher
#6 released
```

## Read what the board reads

The epic's JSON face names all five children, the closed `#2` among them:

<!-- transcript: run -->
```console
$ curl -s -H 'Accept: application/json' http://localhost:1070/iki/ledger/item/1 | jq -c '[.item.children[].number]'
[2,3,4,5,6]
$ curl -s -H 'Accept: application/json' 'http://localhost:1070/iki/ledger/items?status=open' | jq -c '[.items[].number] | sort'
[1,3,4,5,6]
```

The second read is the listing, and `#2` is not in it. That is why the board asks the epic.

Each child's own face carries its links, its claim and whether it is awaiting someone. `#5` blocks
`#4`, `ada` holds `#3`, and `#6` is the dispatcher's turn:

<!-- transcript: run -->
```console
$ curl -s -H 'Accept: application/json' http://localhost:1070/iki/ledger/item/5 | jq -c '[.item.links[] | {type, target: .target.number}]'
[{"type":"blocks","target":4},{"type":"parent","target":1}]
$ curl -s -H 'Accept: application/json' http://localhost:1070/iki/ledger/item/3 | jq -c '.item.claim | {holder, lease}'
{"holder":"ada","lease":"PT2H"}
$ curl -s -H 'Accept: application/json' http://localhost:1070/iki/ledger/item/6 | jq -c '.item | {status, awaiting, claim}'
{"status":"open","awaiting":"dispatcher","claim":null}
```

## The columns

`board.js` puts each child in one column, in this order of questions:

| Column | When |
| --- | --- |
| Done | the child is closed |
| Awaiting (handed back) | `awaiting` names someone: a worker handed it back |
| In flight | someone holds a claim on it, live or with its lease run out |
| Blocked by an open sibling | it is open and unclaimed, and an open sibling `blocks` it |
| Ready | it is open and unclaimed, and nothing open blocks it |

A blocker that is closed no longer blocks, and a blocker outside the epic is not counted: the board
judges the epic's children among themselves.

A column is a heading and a list, and an empty one says "None.", so the board reads in order
without color. Progress is a `<progress>` element with a label ("1 of 5 done"), not a bar whose
width a script sets. The CSP would refuse that style.

## The board, from a shell

Run the board's own modules with Deno, as the anonymous viewer, and print what each column holds.
Save this as `board-text.js`:

<!-- transcript: file board-text.js -->
```js
// What the board face shows, printed: its own modules, run as the anonymous viewer.
import { item } from "./site/faces/board/kit.js";
import { COLUMNS, board, readChildren } from "./site/faces/board/board.js";

const epic = await item("default", Number(Deno.args[0]));
const { items: kids, unreadable } = await readChildren(epic);
if (unreadable.length) throw new Error(`${unreadable.length} children could not be read`);
const { by, done, tally } = board(kids);
console.log(`#${epic.number} ${epic.title}: ${done} of ${kids.length} done`);
for (const [key, name] of COLUMNS) {
  console.log(`${name} (${by[key].length})`);
  for (const it of by[key]) console.log(`  #${it.number} ${it.title}`);
}
console.log(`Labels: ${tally.map(([label, n]) => `${label} ${n}`).join(", ")}`);
```

<!-- transcript: run -->
```console
$ DENO_DIR="$PWD/.deno" deno run --location http://localhost:1070/face/name/board/ --allow-net=localhost:1070 board-text.js 1
#1 Faces in the gonk Book: 1 of 5 done
Blocked by an open sibling (1)
  #4 Plans, projects and who has what
Ready (1)
  #5 Move the gonk pin
In flight (1)
  #3 A board for an epic
Awaiting (handed back) (1)
  #6 Write the release note
Done (1)
  #2 Pages of your own
Labels: docs 4, pin 2
```

Close the blocker, and `#4` moves to Ready on the next read:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=5 reason=done author=ada Pinned.' 2>/dev/null
closed #5 (done)
$ DENO_DIR="$PWD/.deno" deno run --location http://localhost:1070/face/name/board/ --allow-net=localhost:1070 board-text.js 1 | sed -n '1,5p'
#1 Faces in the gonk Book: 2 of 5 done
Blocked by an open sibling (0)
Ready (1)
  #4 Plans, projects and who has what
In flight (1)
```

The board does not update itself. It reads when the page loads or its form is sent, so reload it to
see a change. The [next chapter](plans.md) says what gonk's push stream will add.

## The label tally

Under the columns, the board counts the children's labels, most first. Each label links into gonk's
own ledger page, filtered by that label (`paths.labelPage` in `kit.js`):

<!-- transcript: run -->
```console
$ DENO_DIR="$PWD/.deno" deno run --location http://localhost:1070/face/name/board/ --allow-net=localhost:1070 board-text.js 1 | tail -1
Labels: docs 4, pin 2
$ curl -s 'http://localhost:1070/l/default?labels=pin' | grep -o 'Move the gonk pin\|Plans, projects and who has what'
Plans, projects and who has what
```

`/l/default?labels=pin` is gonk's page, not the face's, and it lists open items only. The closed
`#5` is counted on the board and not listed there. The face asks a question, and gonk's own page is
where you go to act on the answer.

## Open it

The board takes its ledger and epic from the address, so a bookmark reopens the same board:

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open 'http://localhost:1070/face/name/board/?ledger=default&epic=1'
```

Each card links to gonk's page for that item, and a claimed card names its holder ("ada"). A lease
that has run out says so in words, "lease expired", not only in a warning color. In the browser, run
the checks from [the last chapter](own.md#4-check-it-from-a-shell-and-in-a-browser).

## What you have

The board reads one epic and its children, never a listing, and judges each child by its own
state and its siblings' `blocks` links. Its logic is a module you can run without a browser. [Plans,
projects and who has what](plans.md) reads the other three samples, and says when a face is the
right place for a question and when gonk's own pages are.
