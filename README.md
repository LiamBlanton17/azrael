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
--- Depth Limit: 3 ---
Score: 2821/6770 (0.42)
Best moves found: 187/677 (0.28)
Search time: 634.7564ms
Total nodes: 4,286,240
Avg. Depth: 3.00
MN/S: 6.75
EBF: 5.56

--- Depth Limit: 4 ---
Score: 3220/6770 (0.48)
Best moves found: 218/677 (0.32)
Search time: 3.1567776s
Total nodes: 21,080,477
Avg. Depth: 4.00
MN/S: 6.68
EBF: 6.48

--- Depth Limit: 6 ---
Score: 3456/6770 (0.51)
Best moves found: 251/677 (0.37)
Search time: 33.2961555s
Total nodes: 239,791,179
Avg. Depth: 5.57
MN/S: 7.20
EBF: 5.99


--- Depth Limit: 6 ---
Score: 3463/6770 (0.51)
Best moves found: 251/677 (0.37)
Search time: 17.2173085s
Total nodes: 130,456,099
Avg. Depth: 5.57
MN/S: 7.58
EBF: 5.40


--- Depth Limit: 6 ---
Score: 3040/6770 (0.45)
Best moves found: 210/677 (0.31)
Search time: 3.1336319s
Total nodes: 21,311,676
Avg. Depth: 5.94
MN/S: 6.80
ABF: 3.42