# Spec Delta

## ADDED Requirements

### Requirement: The last move can be undone

The system SHALL let a player undo the most recent move of a game that is not over: its square
becomes empty and the turn returns to the player who made it.

#### Scenario: Undoing X's move

- **WHEN** X takes the center and then undoes it
- **THEN** the center is empty
- **AND** it is X's turn again

#### Scenario: Undo after a win

- **WHEN** the game is over and a player asks to undo
- **THEN** the undo is refused

## MODIFIED Requirements

### Requirement: Players alternate

The system SHALL give the turn to the other player after every accepted move, and back to the
player whose move was undone after an undo.

#### Scenario: X then O

- **WHEN** X takes a square
- **THEN** it is O's turn
