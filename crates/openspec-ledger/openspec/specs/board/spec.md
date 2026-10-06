# board Specification

## Purpose

The tic-tac-toe board: nine squares, and how a game on them ends.

## Requirements

### Requirement: A new game has an empty board

The system SHALL start every game with all nine squares empty and X to move.

#### Scenario: Starting a game

- **WHEN** a player starts a new game
- **THEN** every square is empty
- **AND** it is X's turn

### Requirement: Three in a line wins

The system SHALL end the game as a win for a player who holds three squares in one row,
column or diagonal.

#### Scenario: A full row

- **WHEN** X holds the three squares of the top row
- **THEN** the game is over and X has won

#### Scenario: A full board with no line

- **WHEN** all nine squares are taken and no player holds a line
- **THEN** the game is over and it is a draw
