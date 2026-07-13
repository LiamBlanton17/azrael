mod pst;

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
        pst_eval(self)
    }

    pub fn eval_relative(&self) -> Eval {
        match self.turn  {
            Color::White => self.eval(),
            Color::Black => -self.eval(),
        }
    }

}

// Get a PST evaluation of the position, based on phase of game
fn pst_eval(p: &Position) -> Eval {
    let mut opening: i32 = 0;
    let mut endgame: i32 = 0;

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
            opening += PST[OPENING][pi][sq.idx()] as i32;
            endgame += PST[ENDGAME][pi][sq.idx()] as i32;
        }

        // Black mirrors the square vertically (sq ^ 56) and subtracts.
        for sq in p.get_piece(piece, Color::Black) {
            opening -= PST[OPENING][pi][sq.idx() ^ 56] as i32;
            endgame -= PST[ENDGAME][pi][sq.idx() ^ 56] as i32;
        }
    }

    let phase = get_phase_score(p);
    interpolate_phase(phase, opening as Eval, endgame as Eval)
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
