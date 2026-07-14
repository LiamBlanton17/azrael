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
Score: 2611/6770 (0.39)
Best moves found: 172/677 (0.25)
Search time: 4.6848328s
Total nodes: 34,899,480
Avg. Depth: 5.66
MN/S: 7.45
EBF: 4.29

--- Time Limit: 50ms ---
Score: 2633/6770 (0.39)
Best moves found: 178/677 (0.26)
Search time: 27.0238681s
Total nodes: 170,751,775
Avg. Depth: 7.01
MN/S: 6.32
EBF: 4.39

--- Time Limit: 250ms ---
Score: 2689/6770 (0.40)
Best moves found: 185/677 (0.27)
Search time: 90.970239s
Total nodes: 748,214,524
Avg. Depth: 8.13
MN/S: 8.22
EBF: 4.12

--- Time Limit: 1s ---
Score: 2707/6770 (0.40)
Best moves found: 187/677 (0.28)
Search time: 481.7248105s
Total nodes: 3,181,591,202
Avg. Depth: 9.19
MN/S: 6.60
EBF: 4.37

--- Time Limit: 5s ---
Score: 2716/6770 (0.40)
Best moves found: 187/677 (0.28)
Search time: 1901.0237717s
Total nodes: 15,430,901,906
Avg. Depth: 10.32
MN/S: 8.12
EBF: 4.15
