# Plans, projects and who has what

The starter carries three more faces, each answering a question gonk's own pages leave to you:

- **flight**: in what order can this plan's steps land, and which one lands next?
- **project**: what is open across this project's ledgers, with these labels and not those?
- **claims**: who holds what, and whose lease has run out?

This chapter serves all three from one gonk, runs each one's reads, and ends with when a question
belongs in a face and when it belongs in gonk.

## Three faces, three lines

In a new empty directory, generate a repository from the starter as in [the last
chapter](board.md). This time give gonk three lines, one per face:

<!-- transcript: run -->
```console
$ git clone --quiet https://github.com/ikigai-rs/ikigai-gonk-starter starter
$ git -C starter checkout --quiet {{#include ../../starter.rev}}
$ (cd starter && python3 tools/new.py ../site > /dev/null)
$ mkdir -p .config/ikigai
$ printf 'gonk.face = "flight %s/site/faces/flight"\n' "$PWD" > .config/ikigai/config.toml
$ printf 'gonk.face = "project %s/site/faces/project"\n' "$PWD" >> .config/ikigai/config.toml
$ printf 'gonk.face = "claims %s/site/faces/claims"\n' "$PWD" >> .config/ikigai/config.toml
```

If the gonk from [A board for an epic](board.md) is still running, stop it first, since this one
takes the same port.

<!-- transcript: serve -->
```console
$ ikigai-gonk --config-home "$PWD/.config/ikigai" --data-home "$PWD/.ikigai" --port 1070 --no-quic --no-backup
ikigai-gonk 0.1.1 — holding the store at …/.ikigai/store
…
```

Each face gets its own header link:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/ | grep -o "href='/face/name/[^']*'"
href='/face/name/flight'
href='/face/name/project'
href='/face/name/claims'
```

## A flight plan

A **record** item stands for a plan, such as a release. Its steps are its children, plus any item
it links to as `related`. The `blocks` links between the steps say what must land first.

File a release, three steps as its children, and a fourth item it only relates to. Migrating the
store blocks the export route, and the export route blocks its documentation:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:append priority=1 labels=release Release 0.2' -c 'sink urn:gk:iki:ledger:append priority=2 labels=api Add the export route' -c 'sink urn:gk:iki:ledger:append priority=1 labels=store Migrate the store' -c 'sink urn:gk:iki:ledger:append priority=2 labels=docs Document the export' -c 'sink urn:gk:iki:ledger:append priority=3 labels=docs Fix the README typo' 2>/dev/null
#1 urn:iki:ledger:default:item:…
#2 urn:iki:ledger:default:item:…
#3 urn:iki:ledger:default:item:…
#4 urn:iki:ledger:default:item:…
#5 urn:iki:ledger:default:item:…
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:link item=2 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=3 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=4 type=parent 1' -c 'sink urn:gk:iki:ledger:link item=1 type=related 5' -c 'sink urn:gk:iki:ledger:link item=3 type=blocks 2' -c 'sink urn:gk:iki:ledger:link item=2 type=blocks 4' 2>/dev/null
linked #2 parent #1
linked #3 parent #1
linked #4 parent #1
linked #1 related #5
linked #3 blocks #2
linked #2 blocks #4
```

### The landing order

`plan.js` orders the steps the way a release lands them:

1. A step comes after every step in the plan that `blocks` it.
2. Among the steps free to go, lower priority numbers come first, then no priority, then the item
   number.
3. The **next to land** is the first step in that order that is open, not handed back, and blocked
   by nothing open.
4. Steps that block each other in a circle can never land. They are listed apart, under "In a
   blocking cycle".

As in the board, the steps come from the record's own JSON face, so a step that has landed stays
in the plan. Run the face's modules from a shell, saved as `flight-text.js`:

<!-- transcript: file flight-text.js -->
```js
// What the flight face shows, printed: its own modules, run as the anonymous viewer.
import { item, pool, stateOf } from "./site/faces/flight/kit.js";
import { plan, stepNumbers } from "./site/faces/flight/plan.js";

const record = await item("default", Number(Deno.args[0]));
const steps = (await pool(stepNumbers(record), (n) => item("default", n))).filter((it) => !it.error);
const { order, cycle, next } = plan(steps);
console.log(`#${record.number} ${record.title}: ${order.length} steps`);
order.forEach((it, i) => console.log(`${i + 1}. #${it.number} ${it.title} (${stateOf(it)}, priority ${it.priority})${it === next ? " <- next to land" : ""}`));
for (const it of cycle) console.log(`in a cycle: #${it.number} ${it.title}`);
```

<!-- transcript: run -->
```console
$ DENO_DIR="$PWD/.deno" deno run --location http://localhost:1070/face/name/flight/ --allow-net=localhost:1070 flight-text.js 1
#1 Release 0.2: 4 steps
1. #3 Migrate the store (open, priority 1) <- next to land
2. #2 Add the export route (open, priority 2)
3. #4 Document the export (open, priority 2)
4. #5 Fix the README typo (open, priority 3)
```

`#5` blocks nothing and nothing blocks it, so it could land at any time. It comes last only because
its priority is the lowest. Land the migration, and the export route is next. The migration stays
in the plan, now done:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:close item=3 reason=done author=ada Migrated.' 2>/dev/null
closed #3 (done)
$ DENO_DIR="$PWD/.deno" deno run --location http://localhost:1070/face/name/flight/ --allow-net=localhost:1070 flight-text.js 1
#1 Release 0.2: 4 steps
1. #3 Migrate the store (done, priority 1)
2. #2 Add the export route (open, priority 2) <- next to land
3. #4 Document the export (open, priority 2)
4. #5 Fix the README typo (open, priority 3)
```

In the browser, the page marks the next step in words, "Next to land:", as well as with a border:

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open 'http://localhost:1070/face/name/flight/?ledger=default&record=1'
```

gonk's own `next`, from [A lifecycle for your ledger](../ledger/lifecycle.md), answers a different
question: what can anyone take now, across the whole ledger. The flight plan answers it for one
plan, in landing order, with what has landed still shown.

## Projects, as the face's own data

gonk has ledgers. It has no idea of a **project** that spans several of them. The project face adds
one without asking gonk for anything: the projects are a file in the face's own directory,
`projects.json`, which maps each project's name to its ledgers. The sample's names are
placeholders, and gonk serves the file like any other:

<!-- transcript: run -->
```console
$ curl -s http://localhost:1070/face/name/project/projects.json
{
  "example-web": ["web-frontend", "web-api"],
  "example-tools": ["default", "build-tools"]
}
```

Write your own. This deployment's release work is in `default`, and its operations work in `ops`:

<!-- transcript: file site/faces/project/projects.json -->
```json
{
  "release": ["default", "ops"]
}
```

File one item in `ops`, over the owner's socket:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:ops:append labels=docs Rotate the backup key' 2>/dev/null
ops#1 urn:iki:ledger:ops:item:…
```

### The filter is in the address

The face's form takes a project, labels to require (`labels`) and labels to exclude (`without`). It
writes all three into the address, so the address is the view. Bookmark it, or send it to a
colleague, and it reopens the same filter:

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open 'http://localhost:1070/face/name/project/?project=release&labels=docs&without=api'
```

For each ledger of the project, the page reads one listing, with the labels as the listing's own
filter. Those are reads you can make yourself:

<!-- transcript: run -->
```console
$ curl -s -H 'Accept: application/json' 'http://localhost:1070/iki/ledger/items?status=open&labels=docs' | jq -c '[.items[] | {number, title}]'
[{"number":4,"title":"Document the export"},{"number":5,"title":"Fix the README typo"}]
$ curl -s -o /dev/null -w '%{http_code}\n' -H 'Accept: application/json' 'http://localhost:1070/iki/ledger/ops/items?status=open&labels=docs'
403
```

The anonymous viewer cannot read `ops`, so the page shows the two items from `default` and says
"Could not read: ops." above them. Signed in under a grant that reads both ledgers, the same
address shows three. The file names ledgers. Whether you can read them is still up to gonk.

The `without` filter is applied in the page, after the reads. It drops any item carrying an
excluded label, then the page counts what is left by ledger, by priority and by label.

## Who has what

The claims face lists every claimed open item in a ledger, grouped by holder. Holders with an
expired lease come first, and so do the expired leases within each group. It then lists every
handed-back item, grouped by whose turn it is.

Claim two steps. `bob` takes one on a one-second lease and lets it run out:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'sink urn:gk:iki:ledger:claim item=2 lease=PT1S bob' -c 'sink urn:gk:iki:ledger:claim item=4 lease=2h ada' 2>/dev/null
#2 claimed by bob until …
#4 claimed by ada until …
$ sleep 2
```

The face reads one listing and judges each claim with `kit.js`'s `stateOf`: a claim whose
`expires` has passed is `expired`. Those are the face's own reads, printed. Save this as
`claims-text.js`:

<!-- transcript: file claims-text.js -->
```js
// The claims the claims face groups, printed: kit.js's own reads, as the anonymous viewer.
import { listAll, stateOf } from "./site/faces/claims/kit.js";

for (const it of await listAll(Deno.args[0] || "default", { status: "open" })) {
  if (it.claim) console.log(`${it.claim.holder} #${it.number} ${it.title} (${stateOf(it)})`);
}
```

<!-- transcript: run -->
```console
$ DENO_DIR="$PWD/.deno" deno run --location http://localhost:1070/face/name/claims/ --allow-net=localhost:1070 claims-text.js | sort
ada #4 Document the export (claimed)
bob #2 Add the export route (expired)
```

On the page, `bob` comes first, with "1 held, 1 with an expired lease", and the item says "lease
expired" with the time. gonk's own doctor finds the same lease, as [Claiming: lease, expiry,
takeover](../project/claim.md) showed, and names the remedy:

<!-- transcript: run -->
```console
$ ikigai --mount "urn:gk:=$PWD/.ikigai/gonk.sock" -c 'source urn:gk:iki:ledger:doctor' 2>/dev/null | grep -A1 '^  lease-expired'
  lease-expired  #2's lease for bob expired at …
                 remedy: take it over (`sink urn:iki:ledger:claim item=2 takeover=true from=bob`), or release it (`delete urn:iki:ledger:claim item=2`)
```

<!-- transcript: manual — it opens a browser, which a check cannot look at -->
```console
$ open 'http://localhost:1070/face/name/claims/?ledger=default'
```

## A face, or gonk's own pages?

A face is the right place for a question that is yours: your team's columns, your projects, your
landing rules. gonk's own pages are the right place for what every deployment needs, and for
anything that changes the ledger.

| The question | Where it belongs | Why |
| --- | --- | --- |
| What does item `#2` say, and what happened to it? | gonk's item page | every deployment asks it, and the page also lets you act |
| What is open with label `docs`? | gonk's ledger page, `/l/default?labels=docs` | gonk filters it, and every face can link to it |
| Is anything stuck or inconsistent? | gonk's doctor | it knows the lifecycle, and names a remedy |
| How far along is this epic? | a face (the board) | its five columns are one team's way of seeing |
| What lands next in this release? | a face (flight) | the landing rule is yours: priority first, or something else |
| What is open in the projects we define? | a face (project) | gonk has no projects. The face's own file does |
| Who should I chase about an expired lease? | a face (claims), then gonk's item page to act | the face groups by person. gonk takes the item over or releases it |

When a face's question turns out to be one every deployment asks, it is a candidate for gonk's own
pages. When a face wants to change something, it links to gonk's page for the item and lets the
viewer act there, under the same grant.

## Live updates, later

Every face in this part reads when its page loads or its form is sent, and not again. Reload to see
a change. gonk's own Dashboard polls, as [The Dashboard](../ledger/dashboard.md) showed.

gonk's push stream is designed (ledger #2968, gonk's `docs/design/push-at-the-door.md`) and not
yet built (ledger #3209 and #3210). When it lands, a page will be told when what it reads has
changed, instead of asking again. The design says that signed-in viewers will get the stream,
anonymous viewers will keep polling, and a face will opt in by declaring what it reads. None of
that is in the gonk this book pins. Until it is, a face that wants to stay current reloads on a
timer of its own, and pauses that timer when the tab is hidden.

## What you have

Four faces, each a directory and a line: a board for an epic, a flight plan, a project view and a
claims page. Each reads only gonk's JSON routes as the person viewing it, and each keeps its own
question in a module you can run from a shell. A face is where your team's questions live. gonk's
pages are where everyone's questions live, and where the work is changed.
