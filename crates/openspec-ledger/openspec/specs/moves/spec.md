# moves Specification

## Purpose

How players take turns, and which moves the game refuses.

## Requirements

### Requirement: Players alternate

The system SHALL give the turn to the other player after every accepted move.

#### Scenario: X then O

- **WHEN** X takes a square
- **THEN** it is O's turn

### Requirement: A taken square is refused

The system MUST refuse a move to a square that is already taken, and MUST NOT change the
turn when it does.

#### Scenario: Moving onto X

- **WHEN** O moves to a square X holds
- **THEN** the move is refused
- **AND** it is still O's turn
