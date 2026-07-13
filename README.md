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
1x 7-bits for the half move counter (cannot exceed 100)
1x 4-bits for castling rights
1x 4-bits for en passent square
1x 1-bit for player turn

With this in mind, it means the bit boards are 64-bytes, 8-bytes for Zobrist, so 72-bytes there.

Then an extra byte for half move counter, two more for the castling and en passent, and 1 more for the player turn.

This is a total of 76-bytes. The compiler will pad this out to 80-bytes anyway.

## Move Design
A move, just like a board, needs to have as small as a memory footprint as possible. The move structure will be 2 bytes.

6-bits for the "from square"
6-bits for the "to square"
2-bits for the promotion piece
2-bits for an optional move code (is capture, castle, etc)

## Move generation Design
When generating moves for the chess engine, it will be designed explicity from the start to allow generation of captures, non-captures, or both for each piece type and the position as a whole. This can dramatically speed up the engine, as we can position not generate a large portion of moves, where a capture causes a cutoff.

## Last Strength Test Run (7/13/2026)
- Score: 1067/6770
- Best moves found: 51/677
- Search time: 507.0123717s
- Total nodes: 11,497,840,224
- MN/S: 22.68
