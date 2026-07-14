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

## Last Strength Test Run (7/14/2026)
--- Time Limit: 10ms ---
Score: 3218/6770 (0.48)
Best moves found: 218/677 (0.32)
Search time: 4.5149094s
Total nodes: 33,121,527
Avg. Depth: 5.26
MN/S: 7.34
EBF: 4.51

--- Time Limit: 50ms ---
Score: 3508/6770 (0.52)
Best moves found: 254/677 (0.38)
Search time: 23.7357071s
Total nodes: 160,879,013
Avg. Depth: 6.50
MN/S: 6.78
EBF: 4.45

--- Time Limit: 250ms ---
Score: 3652/6770 (0.54)
Best moves found: 277/677 (0.41)
Search time: 94.1130628s
Total nodes: 734,715,366
Avg. Depth: 7.63
MN/S: 7.81
EBF: 4.48

--- Time Limit: 1s ---
Score: 3960/6770 (0.58)
Best moves found: 303/677 (0.45)
Search time: 456.0947882s
Total nodes: 3,237,091,037
Avg. Depth: 8.74
MN/S: 7.10
EBF: 4.47

--- Time Limit: 2.5s ---
Score: 4050/6770 (0.60)
Best moves found: 305/677 (0.45)
Search time: 1603.6033653s
Total nodes: 12,802,516,548
Avg. Depth: 9.75
MN/S: 7.98
EBF: 4.41
