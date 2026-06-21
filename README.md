# Azrael
A chess engine written in Rust.

## Project Layout
- Types: definitions and trait implementations of the custom types for the project
- State: implementation of board state representation
- Search: implementation of board state space searching
- Eval: implementation of static board state evaluation
- Interface: implementation of apis for the project

## Board State Design
6x 64-bit boards for piece positions (K, Q, R, B, N, P)
2x 64-bit boards for color positions (W, B)
1x 64-bit for Zobrist hash of the position
1x 6-bits for the half move counter (cannot exceed 50)
1x 4-bits for castling rights
1x 4-bits for en passent square (16 possible squares)
1x 1-bit for player turn

