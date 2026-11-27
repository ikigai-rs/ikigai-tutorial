# A game as a log: a fold

[Time is a resource](time.md) and the tic-tac-toe parts both hold state in a place that
gets written over. A `Sink` writes a mark into a cell, and the cell's old value is gone.
This chapter keeps the same game a different way. It keeps the **moves**, in order, and
never changes one. The board is then something you compute from them: start from an
empty board and apply each move in turn. That computation is a **fold** over the log,
and a resource built that way is called **event-sourced**.

The ikigai-core design note *Events, streams and resources*
(`docs/design/events-and-resources.md`, section 2.7) explains why this fits resources
well: "Because entries never change, the state after entry n is a pure function of an
immutable prefix." So the board after move 4 can be cached forever. Only the board *now*
changes, and only when a move is appended.

The code is the crate `crates/event-fold`, which needs nothing but `ikigai-core`.

## The model, on paper

As in the [tic-tac-toe](tic-tac-toe-1.md) parts, the names come first:

| the set | its name | what it is | how long an answer lasts |
|---|---|---|---|
| the **log** | `urn:iki:tutorial:fold:log` | `Sink` appends one move (`content` is a cell, `1` to `9`); `Source` answers the head, how many moves there are | until the next append |
| an **entry** | `urn:iki:tutorial:fold:entry:seq:{n}` | the move at position `n`, such as `X5`: an atom, with no `Sink` of its own | forever |
| a **checkpoint** | `urn:iki:tutorial:fold:checkpoint:seq:{n}` | for `n` a multiple of 3: the board from the checkpoint 3 moves earlier, with those 3 entries applied | forever |
| a **position** | `urn:iki:tutorial:fold:state:seq:{n}` | the board after `n` moves: the position before it with entry `n` applied, or the checkpoint itself when `n` is a multiple of 3 | forever |
| the **head** | `urn:iki:tutorial:fold:state` | the board now: the position at the head | until the next append |

Only the log moves. Everything with `seq:{n}` in its name reads only entries, earlier
positions and checkpoints, and none of those ever changes. So the only name an append
leaves stale, other than the log itself, is the head.

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:names}}
```

A board's representation is three rows of text and a line saying what comes next:

```text
O..
.X.
..X
next: O
```

## One step, and the only state

The fold repeats one pure step: apply a move to a board. Because it is pure, the board
after move `n` depends on moves `1` to `n` and nothing else.

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:apply}}
```

The crate's only mutable state is the list of moves. It also counts how often the fold's
step runs, so that a test can tell a read the cache answered from a read that computed:

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:store}}
```

Every step runs in one function. It reads a base board by name, so the kernel's cache
answers it when it can, and then applies the entries after it:

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:fold}}
```

## The log: the one name that moves

An append is checked against the board at the head before it is kept, so the log only
ever holds a game that can be played. A taken cell, a cell that is not `1` to `9`, or a
move after a win is refused, and nothing is appended. The kernel cuts the log's golden
thread when the `Sink` succeeds. That is the only write in the crate.

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:log}}
```

## Entries, positions and checkpoints

An entry answers the move at its position, and an entry is never rewritten:

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:entry}}
```

A position is the position before it with one more entry applied. At a multiple of 3 it is
the checkpoint, and that is where the recursion stops:

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:state_at}}
```

A checkpoint is the checkpoint before it with 3 entries applied. So reading a position
cold goes back at most 2 positions to a checkpoint, then from checkpoint to checkpoint. It
never goes back through every position.

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:checkpoint}}
```

The head reads the log, then the position the log's head names. Reading the log is what
hangs the head from the log's golden thread:

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:state}}
```

The space binds the five names over one log:

```rust,ignore
{{#include ../../../../crates/event-fold/src/lib.rs:space}}
```

## Playing it

This listing plays five moves through the log and reads the game back. `mdbook test` runs
it on every pull request:

```rust
# extern crate event_fold;
# extern crate ikigai_core;
# extern crate futures;
use std::sync::Arc;
use futures::executor::block_on;
use ikigai_core::{ArgRef, Capability, Iri, Kernel, Request, Verb};

let kernel = event_fold::kernel(Arc::default());
let source = |name: &str| {
    let request = Request::new(Verb::Source, Iri::parse(name).unwrap());
    let repr = block_on(kernel.issue(request, &Capability::root())).unwrap();
    String::from_utf8_lossy(&repr.bytes).into_owned()
};
let play = |cell: &str| {
    let request = Request::new(Verb::Sink, Iri::parse(event_fold::LOG).unwrap())
        .with_arg("content", ArgRef::Inline(cell.as_bytes().to_vec()));
    block_on(kernel.issue(request, &Capability::root())).unwrap();
};

for cell in ["5", "1", "9", "3", "2"] {
    play(cell);
}
assert_eq!(source(event_fold::LOG), "5");
assert_eq!(source("urn:iki:tutorial:fold:entry:seq:4"), "O3");
assert_eq!(source(event_fold::STATE), "OXO\n.X.\n..X\nnext: O");
// An earlier position can still be read by its name.
assert_eq!(source("urn:iki:tutorial:fold:state:seq:2"), "O..\n.X.\n...\nnext: X");
```

## What the tests show

`crates/event-fold/tests/fold.rs` checks each claim on this page against a kernel and a
log the test holds:

```text
cargo test -p event-fold
```

**One fold step per append.** Positions, checkpoints and entries are cached forever. An
append clears the log and the head from the cache. The position, checkpoint and entry
the test checks after it stay cached. The new head then
costs exactly one step: the kernel's cache answers the position before it, and one entry
is applied. Asked again, the head costs no step at all.

```rust,ignore
{{#include ../../../../crates/event-fold/tests/fold.rs:forever}}
```

**A cold read goes through checkpoints only.** A second kernel over the same 8-move log
starts with an empty cache. Reading the board after 8 moves runs 8 steps, one per entry.
The read goes from position 8 to position 7, then to the checkpoint at 6, then to the
checkpoint at 3. It does not read every position.

```rust,ignore
{{#include ../../../../crates/event-fold/tests/fold.rs:checkpoints}}
```

## What this example does not do

The design note asks for checkpoints "by spill" (`urn:stream:spill`). A spill turns a
body that can be read only once into one that can be read again, and it needs a spill
facility from the host and a disk-budget grant. Every answer in this crate is already a
buffer in the kernel's cache, so a checkpoint here is a composite cached forever. It
limits how far back a cold read goes, but it lives only as long as the kernel's cache.

A spill would not make a checkpoint outlast a restart either. ikigai-fs 0.6.0's
`FileSpill` unlinks its backing file when the last holder of the replay handle drops
it, so a spill keeps a body readable for that handle's lifetime. It does not make the
body durable. A checkpoint that survives a restart needs a durable store the host binds:
an atom written at the checkpoint. This example does not build one, so a kernel started
afresh folds again from the log.

This is also one fold, not a fold composer. The design note says to write a generic one
only after two real folds need it.
