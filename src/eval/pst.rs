// Piece-Square Tables (PSTs)
// Values are PeSTO's, tuned by Ronald Friederich with Texel's tuning method:
// https://www.chessprogramming.org/PeSTO%27s_Evaluation_Function

use crate::types::eval::Eval;

// Game-phase indices into the master table.
pub const OPENING: usize = 0;
pub const ENDGAME: usize = 1;
pub const NUM_PHASES: usize = 2;

// Fold a piece's base material value into its positional shifts, converting from the
// source tables' a8-first layout to Azrael's a1-first indexing.
const fn build(base: Eval, shift: &[Eval; 64]) -> [Eval; 64] {
    let mut pst = [0; 64];
    let mut sq = 0;
    while sq < 64 {
        pst[sq] = base + shift[sq ^ 56];
        sq += 1;
    }
    pst
}

// PAWNS
const PAWN_OPENING_BASE: Eval = 82;
const PAWN_ENDGAME_BASE: Eval = 94;

const PAWN_OPENING_SHIFT: [Eval; 64] = [
      0,   0,   0,   0,   0,   0,  0,   0,
     98, 134,  61,  95,  68, 126, 34, -11,
     -6,   7,  26,  31,  65,  56, 25, -20,
    -14,  13,   6,  21,  23,  12, 17, -23,
    -27,  -2,  -5,  12,  17,   6, 10, -25,
    -26,  -4,  -4, -10,   3,   3, 33, -12,
    -35,  -1, -20, -23, -15,  24, 38, -22,
      0,   0,   0,   0,   0,   0,  0,   0,
];

const PAWN_ENDGAME_SHIFT: [Eval; 64] = [
      0,   0,   0,   0,   0,   0,   0,   0,
    178, 173, 158, 134, 147, 132, 165, 187,
     94, 100,  85,  67,  56,  53,  82,  84,
     32,  24,  13,   5,  -2,   4,  17,  17,
     13,   9,  -3,  -7,  -7,  -8,   3,  -1,
      4,   7,  -6,   1,   0,  -5,  -1,  -8,
     13,   8,   8,  10,  13,   0,   2,  -7,
      0,   0,   0,   0,   0,   0,   0,   0,
];

const PAWN_OPENING: [Eval; 64] = build(PAWN_OPENING_BASE, &PAWN_OPENING_SHIFT);
const PAWN_ENDGAME: [Eval; 64] = build(PAWN_ENDGAME_BASE, &PAWN_ENDGAME_SHIFT);

// KNIGHTS
const KNIGHT_OPENING_BASE: Eval = 337;
const KNIGHT_ENDGAME_BASE: Eval = 281;

const KNIGHT_OPENING_SHIFT: [Eval; 64] = [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
];

const KNIGHT_ENDGAME_SHIFT: [Eval; 64] = [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
];

const KNIGHT_OPENING: [Eval; 64] = build(KNIGHT_OPENING_BASE, &KNIGHT_OPENING_SHIFT);
const KNIGHT_ENDGAME: [Eval; 64] = build(KNIGHT_ENDGAME_BASE, &KNIGHT_ENDGAME_SHIFT);

// BISHOPS
const BISHOP_OPENING_BASE: Eval = 365;
const BISHOP_ENDGAME_BASE: Eval = 297;

const BISHOP_OPENING_SHIFT: [Eval; 64] = [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
];

const BISHOP_ENDGAME_SHIFT: [Eval; 64] = [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
];

const BISHOP_OPENING: [Eval; 64] = build(BISHOP_OPENING_BASE, &BISHOP_OPENING_SHIFT);
const BISHOP_ENDGAME: [Eval; 64] = build(BISHOP_ENDGAME_BASE, &BISHOP_ENDGAME_SHIFT);

// ROOKS
const ROOK_OPENING_BASE: Eval = 477;
const ROOK_ENDGAME_BASE: Eval = 512;

const ROOK_OPENING_SHIFT: [Eval; 64] = [
     32,  42,  32,  51, 63,  9,  31,  43,
     27,  32,  58,  62, 80, 67,  26,  44,
     -5,  19,  26,  36, 17, 45,  61,  16,
    -24, -11,   7,  26, 24, 35,  -8, -20,
    -36, -26, -12,  -1,  9, -7,   6, -23,
    -45, -25, -16, -17,  3,  0,  -5, -33,
    -44, -16, -20,  -9, -1, 11,  -6, -71,
    -19, -13,   1,  17, 16,  7, -37, -26,
];

const ROOK_ENDGAME_SHIFT: [Eval; 64] = [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
];

const ROOK_OPENING: [Eval; 64] = build(ROOK_OPENING_BASE, &ROOK_OPENING_SHIFT);
const ROOK_ENDGAME: [Eval; 64] = build(ROOK_ENDGAME_BASE, &ROOK_ENDGAME_SHIFT);

// QUEENS
const QUEEN_OPENING_BASE: Eval = 1025;
const QUEEN_ENDGAME_BASE: Eval = 936;

const QUEEN_OPENING_SHIFT: [Eval; 64] = [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
];

const QUEEN_ENDGAME_SHIFT: [Eval; 64] = [
     -9,  22,  22,  27,  27,  19,  10,  20,
    -17,  20,  32,  41,  58,  25,  30,   0,
    -20,   6,   9,  49,  47,  35,  19,   9,
      3,  22,  24,  45,  57,  40,  57,  36,
    -18,  28,  19,  47,  31,  34,  39,  23,
    -16, -27,  15,   6,   9,  17,  10,   5,
    -22, -23, -30, -16, -16, -23, -36, -32,
    -33, -28, -22, -43,  -5, -32, -20, -41,
];

const QUEEN_OPENING: [Eval; 64] = build(QUEEN_OPENING_BASE, &QUEEN_OPENING_SHIFT);
const QUEEN_ENDGAME: [Eval; 64] = build(QUEEN_ENDGAME_BASE, &QUEEN_ENDGAME_SHIFT);

// KINGS
// Both sides always have exactly one king, so its base value cancels out of the score.
const KING_OPENING_BASE: Eval = 0;
const KING_ENDGAME_BASE: Eval = 0;

const KING_OPENING_SHIFT: [Eval; 64] = [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
];

const KING_ENDGAME_SHIFT: [Eval; 64] = [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43,
];

const KING_OPENING: [Eval; 64] = build(KING_OPENING_BASE, &KING_OPENING_SHIFT);
const KING_ENDGAME: [Eval; 64] = build(KING_ENDGAME_BASE, &KING_ENDGAME_SHIFT);

// Piece order matches `Piece::idx()`: Knight, Bishop, Rook, Queen, King, Pawn.
pub const PST: [[[Eval; 64]; 6]; NUM_PHASES] = [
    // OPENING
    [
        KNIGHT_OPENING,
        BISHOP_OPENING,
        ROOK_OPENING,
        QUEEN_OPENING,
        KING_OPENING,
        PAWN_OPENING,
    ],
    // ENDGAME
    [
        KNIGHT_ENDGAME,
        BISHOP_ENDGAME,
        ROOK_ENDGAME,
        QUEEN_ENDGAME,
        KING_ENDGAME,
        PAWN_ENDGAME,
    ],
];

// some AI generated tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::square;
    use crate::types::square::Square;

    // The source tables are a8-first; the built tables must be a1-first.
    #[test]
    fn test_build_flips_vertically() {
        // a8 is the first entry of the source table and index 56 once flipped.
        assert_eq!(PAWN_OPENING[square::A8.idx()], PAWN_OPENING_BASE + PAWN_OPENING_SHIFT[0]);
        assert_eq!(PAWN_OPENING[square::A1.idx()], PAWN_OPENING_BASE + PAWN_OPENING_SHIFT[56]);
        assert_eq!(KING_OPENING[square::H1.idx()], KING_OPENING_SHIFT[63]);
    }

    // Sanity-check the orientation against known PeSTO preferences rather than raw indices.
    #[test]
    fn test_tables_are_oriented_for_white() {
        // A white pawn on the 7th is worth far more than one on the 2nd.
        let d7 = Square::from_row_col(6, 3).idx();
        let d2 = Square::from_row_col(1, 3).idx();
        assert!(PAWN_ENDGAME[d7] > PAWN_ENDGAME[d2]);

        // In the opening the king belongs on g1, not in the centre on e4.
        let g1 = square::G1.idx();
        let e4 = Square::from_row_col(3, 4).idx();
        assert!(KING_OPENING[g1] > KING_OPENING[e4]);

        // In the endgame that reverses: the king wants the centre.
        assert!(KING_ENDGAME[e4] > KING_ENDGAME[g1]);

        // A knight on the rim is dim.
        let e5 = Square::from_row_col(4, 4).idx();
        let a1 = square::A1.idx();
        assert!(KNIGHT_OPENING[e5] > KNIGHT_OPENING[a1]);
    }

    // The base value must be the material floor of every table.
    #[test]
    fn test_bases_are_folded_in() {
        assert_eq!(PAWN_OPENING[square::E1.idx()], PAWN_OPENING_BASE);
        // d4 is source index 35, whose shift is -10.
        assert_eq!(QUEEN_OPENING[Square::from_row_col(3, 3).idx()], QUEEN_OPENING_BASE - 10);
    }
}
