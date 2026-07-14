mod pst;

use crate::types::bidboard::BitBoard;
use crate::types::color::Color;
use crate::types::eval::Eval;
use crate::types::piece::Piece;
use crate::types::position::Position;

use pst::{ENDGAME, OPENING, PST};

// Define material evaluations
pub const PAWN: Eval = 100;
pub const KNIGHT: Eval = 300;
pub const BISHOP: Eval = 300;
pub const ROOK: Eval = 500;
pub const QUEEN: Eval = 900;

// Weights for calculating game phase (Non-Pawn Material is standard).
const PAWN_PHASE: i32 = 0;
const KNIGHT_PHASE: i32 = 1;
const BISHOP_PHASE: i32 = 1;
const ROOK_PHASE: i32 = 2;
const QUEEN_PHASE: i32 = 4;

// The maximum possible phase
const TOTAL_PHASE: i32 = PAWN_PHASE * 16 + KNIGHT_PHASE * 4 + BISHOP_PHASE * 4 + ROOK_PHASE * 4 + QUEEN_PHASE * 2;

impl Position {

    pub fn eval(&self) -> Eval {
        let phase = get_phase_score(self);
        let mut eval = pst_eval(self, phase);
        eval += pawn_structure_eval(self, phase);
        eval += bishop_pair_eval(self, phase);
        eval += tempo_eval(self, phase);

        eval
    }

    pub fn eval_relative(&self) -> Eval {
        match self.turn  {
            Color::White => self.eval(),
            Color::Black => -self.eval(),
        }
    }

}

// Give a slight bonus for tempo, slight better later in game
fn tempo_eval(p: &Position, phase: i32) -> Eval {
    if p.turn == Color::White { 
        interpolate_phase(phase, 9, 17)
    } else {
        -interpolate_phase(phase, 9, 17)
    }
}

// Get an evaluation of the minor piece match ups, based on the phase of game
fn bishop_pair_eval(p: &Position, phase: i32) -> Eval {
    let count_white_bishops = p.get_piece(Piece::Bishop, Color::White).0.count_ones();
    let count_black_bishops = p.get_piece(Piece::Bishop, Color::Black).0.count_ones();
    let bishop_pair_bonus = interpolate_phase(phase, 17, 29);

    let mut eval = if count_white_bishops > 1 { bishop_pair_bonus } else { 0 };
    eval -= if count_black_bishops > 1 { bishop_pair_bonus } else { 0 };
    eval
}

// Get a PST evaluation of the position, based on phase of game
fn pst_eval(p: &Position, phase: i32) -> Eval {
    let mut opening: Eval = 0;
    let mut endgame: Eval = 0;

    for piece in [
        Piece::Pawn,
        Piece::Knight,
        Piece::Bishop,
        Piece::Rook,
        Piece::Queen,
        Piece::King,
    ] {
        let pi = piece.idx();

        // White reads the table directly.
        for sq in p.get_piece(piece, Color::White) {
            opening += PST[OPENING][pi][sq.idx()];
            endgame += PST[ENDGAME][pi][sq.idx()];
        }

        // Black mirrors the square vertically (sq ^ 56) and subtracts.
        for sq in p.get_piece(piece, Color::Black) {
            opening -= PST[OPENING][pi][sq.idx() ^ 56];
            endgame -= PST[ENDGAME][pi][sq.idx() ^ 56];
        }
    }

    interpolate_phase(phase, opening, endgame)
}

// Game phase in [0, 256], where 0 is the opening and 256 is the endgame
fn get_phase_score(p: &Position) -> i32 {
    let mut phase = TOTAL_PHASE;

    for (piece, weight) in [
        (Piece::Pawn, PAWN_PHASE),
        (Piece::Knight, KNIGHT_PHASE),
        (Piece::Bishop, BISHOP_PHASE),
        (Piece::Rook, ROOK_PHASE),
        (Piece::Queen, QUEEN_PHASE),
    ] {
        let count = (p.get_piece(piece, Color::White).0.count_ones() + p.get_piece(piece, Color::Black).0.count_ones()) as i32;
        phase -= count * weight;
    }

    // Normalize to range [0, 256] (0 = opening, 256 = endgame)
    (phase * 256 + (TOTAL_PHASE / 2)) / TOTAL_PHASE
}

// Blend opening and endgame scores based on the phase of the game
fn interpolate_phase(phase_score: i32, opening: Eval, endgame: Eval) -> Eval {
    (((opening as i32) * (256 - phase_score) + (endgame as i32) * phase_score) / 256) as Eval
}

// Score doubled, isolated, and passed pawns from White's perspective (+ favours White).
fn pawn_structure_eval(p: &Position, phase: i32) -> Eval {
    let mut eval: Eval = 0;

    let white_pawns = p.get_piece(Piece::Pawn, Color::White);
    let black_pawns = p.get_piece(Piece::Pawn, Color::Black);

    // Doubled pawns - penalty that increases in the endgame
    let doubled_penalty = interpolate_phase(phase, 14, 36);
    eval -= doubled_pawns(white_pawns) as Eval * doubled_penalty;
    eval += doubled_pawns(black_pawns) as Eval * doubled_penalty;

    // Isolated pawns - penalty that increases in the endgame
    let isolated_penalty = interpolate_phase(phase, 17, 31);
    eval -= isolated_pawns(white_pawns).0.count_ones() as Eval * isolated_penalty;
    eval += isolated_pawns(black_pawns).0.count_ones() as Eval * isolated_penalty;

    // Passed pawns - bonus that increases in the endgame
    let passed_bonus = interpolate_phase(phase, 14, 39);
    eval += passed_pawns_white(white_pawns, black_pawns) as Eval * passed_bonus;
    eval -= passed_pawns_black(black_pawns, white_pawns) as Eval * passed_bonus;

    eval
}

// Count white pawns that no black pawn can stop from promoting.
fn passed_pawns_white(white_pawns: BitBoard, black_pawns: BitBoard) -> u32 {
    let mut count = 0;
    for sq in white_pawns {
        if WHITE_PASSED_MASK[sq.idx()] & black_pawns == BitBoard(0) {
            count += 1;
        }
    }
    count
}

// Count black pawns that no white pawn can stop from promoting.
fn passed_pawns_black(black_pawns: BitBoard, white_pawns: BitBoard) -> u32 {
    let mut count = 0;
    for sq in black_pawns {
        if BLACK_PASSED_MASK[sq.idx()] & white_pawns == BitBoard(0) {
            count += 1;
        }
    }
    count
}

// Count files holding two or more of these pawns (each such file counts once).
fn doubled_pawns(pawns: BitBoard) -> u32 {
    let mut doubled = 0;
    for file in 0..8 {
        let on_file = pawns & FILE_MASK[file];
        if on_file.0.count_ones() >= 2 {
            doubled += 1;
        }
    }
    doubled
}

// Mark pawns with no friendly pawn on either adjacent file.
fn isolated_pawns(pawns: BitBoard) -> BitBoard {
    let mut isolated = BitBoard(0);
    for file in 0..8 {
        let on_file = pawns & FILE_MASK[file];

        // Friendly pawns on the adjacent files.
        let mut neighbors = BitBoard(0);
        if file > 0 {
            neighbors |= pawns & FILE_MASK[file - 1];
        }
        if file < 7 {
            neighbors |= pawns & FILE_MASK[file + 1];
        }

        // No neighbours on either side means every pawn on this file is isolated.
        if neighbors == BitBoard(0) {
            isolated |= on_file;
        }
    }
    isolated
}

const FILE_MASK: [BitBoard; 8] = {
    let mut masks = [BitBoard(0); 8];
    let mut file = 0;
    while file < 8 {
        masks[file] = BitBoard(0x0101010101010101u64 << file);
        file += 1;
    }
    masks
};

// Passed pawn masks look at every sq in the file ahead for the pawn
const WHITE_PASSED_MASK: [BitBoard; 64] = build_passed_mask(true);
const BLACK_PASSED_MASK: [BitBoard; 64] = build_passed_mask(false);
const fn build_passed_mask(white: bool) -> [BitBoard; 64] {
    let mut masks = [BitBoard(0); 64];
    let mut sq = 0;
    while sq < 64 {
        let row = sq / 8;
        let col = sq % 8;
        let file_lo = if col == 0 { 0 } else { col - 1 };
        let file_hi = if col == 7 { 7 } else { col + 1 };

        let mut mask: u64 = 0;
        let mut r = 0;
        while r < 8 {
            // "Ahead" is toward rank 8 for White, toward rank 1 for Black.
            let ahead = if white { r > row } else { r < row };
            if ahead {
                let mut f = file_lo;
                while f <= file_hi {
                    mask |= 1u64 << (r * 8 + f);
                    f += 1;
                }
            }
            r += 1;
        }

        masks[sq] = BitBoard(mask);
        sq += 1;
    }
    masks
}
