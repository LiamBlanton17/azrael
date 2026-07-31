# Azrael

A UCI chess engine written from scratch in Rust — bitboard move generation, an alpha-beta search with modern pruning, and a tapered evaluation built on PeSTO's tuned piece-square tables.

[![CI](https://github.com/LiamBlanton17/azrael/actions/workflows/ci.yml/badge.svg)](https://github.com/LiamBlanton17/azrael/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Lichess bullet rating](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Flichess.org%2Fapi%2Fuser%2Fazrael17&query=%24.perfs.bullet.rating&label=lichess%20bullet&logo=lichess&logoColor=white&color=5b8ac6)](https://lichess.org/@/azrael17)
[![Lichess blitz rating](https://img.shields.io/badge/dynamic/json?url=https%3A%2F%2Flichess.org%2Fapi%2Fuser%2Fazrael17&query=%24.perfs.blitz.rating&label=lichess%20blitz&logo=lichess&logoColor=white&color=5b8ac6)](https://lichess.org/@/azrael17)

**Play it online:** [lichess.org/@/azrael17](https://lichess.org/@/azrael17)

> When I got into programming in 5th grade, I always thought if someone could write a chess engine they must be really smart. Well, now I am a CS graduate and a SWE, I decided it was time for me to make 11 year-old me proud.

## Features

**Board representation**
- Bitboard board state (6 piece boards + 2 color boards) — see [Design](#design) for the full memory layout
- [Magic bitboards](https://www.chessprogramming.org/Magic_Bitboards) for sliding-piece (rook/bishop/queen) attack generation
- [Zobrist hashing](https://www.chessprogramming.org/Zobrist_Hashing) for fast position identity and repetition detection
- Staged move generation — captures, quiets, or both, per piece type, to maximize cutoffs

**Search**
- Iterative deepening negamax with [alpha-beta](https://www.chessprogramming.org/Alpha-Beta) pruning
- [Principal Variation Search](https://www.chessprogramming.org/Principal_Variation_Search) with null-window re-search
- [Null-move pruning](https://www.chessprogramming.org/Null_Move_Pruning)
- [Aspiration windows](https://www.chessprogramming.org/Aspiration_Windows)
- [Static Exchange Evaluation](https://www.chessprogramming.org/Static_Exchange_Evaluation)
- Razoring at low depths
- Check and root depth extensions
- [Quiescence search](https://www.chessprogramming.org/Quiescence_Search) with delta pruning and check-evasion handling
- [Transposition table](https://www.chessprogramming.org/Transposition_Table) with depth-preferred replacement
- Move ordering: TT move, [killer moves](https://www.chessprogramming.org/Killer_Heuristic), and a [history heuristic](https://www.chessprogramming.org/History_Heuristic)

**Evaluation**
- Material and [piece-square tables](https://www.chessprogramming.org/Piece-Square_Tables) from [PeSTO](https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function) (Texel-tuned by Ronald Friederich), with material folded into the tables and evaluated incrementally
- Hand-tuned pawn structure, king safety, and open/semi-open file terms
- Bishop-pair and tempo bonuses
- Tapered evaluation across game phases, with lazy evaluation
- Mobility evaluation based on the [Aconcagua](https://github.com/gabtar/aconcagua) chess engine

**Interface**
- Speaks the [UCI protocol](https://en.wikipedia.org/wiki/Universal_Chess_Interface) — drops into any UCI GUI or tournament runner (tested with `fastchess`)

## Build

Requires a recent stable Rust toolchain.

```sh
cargo build --release
```

> Azrael builds with `target-cpu=native` (see [`.cargo/config.toml`](.cargo/config.toml)) so it uses your CPU's native
> bit-manipulation instructions (e.g. `popcnt`). If you plan to run the binary on a different machine than you built it
> on, remove that flag first.

The binary is produced at `target/release/azrael`.

## Usage

```sh
# Run as a UCI engine (for a GUI or a tournament runner like fastchess)
azrael uci

# Find the best move in a position, limited by depth (ply) or time (ms)
azrael search "<FEN>" depth 9
azrael search "<FEN>" time 1000

# Verify move generation and measure raw search speed
azrael perft

# Run the tactical/positional strength suite (EPD files in test_files/)
azrael strength
```

## Correctness (perft)

Move generation is validated with [perft](https://www.chessprogramming.org/Perft) node counts against known-good
references, which also doubles as a raw nodes-per-second benchmark.

| Position      | Depth | Nodes         | Matches reference | MN/s  |
|---------------|-------|---------------|-------------------|-------|
| startpos      |     6 |   119,060,324 |         ✅        | 40.23 |
| kiwipete      |     5 |   193,690,690 |         ✅        | 42.49 |
| Rook Pawn     |     7 |   178,633,661 |         ✅        | 36.23 |
| Chaos         |     6 |   706,045,033 |         ✅        | 38.20 |
| Engine Killer |     5 |    89,941,194 |         ✅        | 38.84 |
| Standard      |     5 |   164,075,551 |         ✅        | 43.06 |

## Strength testing

`azrael strength` runs Azrael over annotated [EPD](https://www.chessprogramming.org/Extended_Position_Description)
suites (in [`test_files/`](test_files/)) — themed positions covering center control, recapturing, undermining,
simplification, and more — and scores how often it finds the annotated best move at a range of depth/time limits.

| Depth | Score               | Best move found  | Time     | Nodes         | MN/s | ABF  |
|-------|---------------------|------------------|----------|---------------|------|------|
|     3 | 2,643 / 6,770 (39%) | 174 / 677 (26%)  |   0.3 s  |       911,459 | 3.46 | 8.26 |
|     5 | 2,857 / 6,770 (42%) | 197 / 677 (29%)  |   1.0 s  |     3,857,620 | 3.90 | 4.79 |
|     7 | 3,217 / 6,770 (48%) | 226 / 677 (33%)  |   3.8 s  |    15,834,044 | 4.17 | 3.78 |
|     9 | 3,649 / 6,770 (54%) | 273 / 677 (40%)  |  12.2 s  |    51,463,637 | 4.21 | 3.21 |
|    13 | 4,143 / 6,770 (61%) | 319 / 677 (47%)  | 121.0 s  |   485,241,733 | 4.01 | 2.66 |
|    15 | 4,416 / 6,770 (65%) | 343 / 677 (51%)  | 362.3 s  | 1,424,570,275 | 3.93 | 2.51 |

_Score is total points earned across the suite's weighted candidate moves; **Best move found** counts positions where Azrael played the top-rated move. **MN/s** is millions of nodes searched per second; **ABF** is the effective branching factor (lower means more aggressive pruning)._

## Design

Azrael is built around a compact, cache-friendly board representation. Everything below is packed to keep the working
set small and the hot paths allocation-free.

**Board state** — the pieces are stored in two representations, kept in sync on every move:
- **Bitboards** — 6 piece boards (K, Q, R, B, N, P) + 2 color boards (W, B), for fast set-wise operations and attack generation.
- **Mailbox** — a 64-byte array mapping each square directly to the piece on it. This gives O(1) "what's on this square?" lookups, which are hot in the make/unmake-move routines and in move ordering — places where reading it off the bitboards would otherwise cost a scan.

Alongside those sit a 64-bit Zobrist hash and a handful of packed state fields:
- 7 bits — halfmove clock (capped at 100)
- 4 bits — castling rights
- 4 bits — en-passant square
- 1 bit — side to move

The mailbox is a deliberate space-for-speed trade: it adds 64 bytes and makes the struct less cache-friendly, but the constant-time lookups it enables in the hottest paths more than pay for themselves.

**Move encoding** — each move is packed into 2 bytes:
- 6 bits — from square
- 6 bits — to square
- 2 bits — promotion piece
- 2 bits — move code (capture, castle, etc.)

**Staged move generation** — generation is designed from the ground up to emit captures, quiets, or both, per piece
type and for the position as a whole. When a capture produces a cutoff, the quiet moves are never generated at all.

## Project layout

| Module        | Responsibility                                                        |
|---------------|-----------------------------------------------------------------------|
| `types/`      | Core types (bitboards, squares, pieces, moves) and their trait impls  |
| `state/`      | Board state representation — FEN, Zobrist, SAN/long-algebraic parsing |
| `search/`     | Move generation, magics, and the search stack (negamax, quiescence, TT) |
| `eval/`       | Static position evaluation                                            |
| `interface.rs`| UCI protocol implementation                                           |
| `benchmarks/` | Perft and strength-test harnesses                                    |

## License

See [LICENSE](LICENSE).
