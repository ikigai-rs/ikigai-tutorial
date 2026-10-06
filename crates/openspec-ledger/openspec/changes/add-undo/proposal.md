# Proposal

## Why

A misclick ends a game of tic-tac-toe. Players want to take back the last move, and only the
last one, so that undo cannot be used to replay a lost game from the start.

## What Changes

- A player can undo the most recent move, which empties its square and gives the turn back.
- Undo after the game is over is refused.

## Capabilities

### New Capabilities

### Modified Capabilities

- `moves`: adds undo, and says what alternation means once a move can be taken back.

## Impact

The move history becomes state the game keeps; the board view gains an Undo control.
