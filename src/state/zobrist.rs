use crate::types::chess_move::Move;
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::{Position, ZobristHash};
use crate::types::square::Square;
use std::sync::OnceLock;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

// Setup globals used for Zobrist hashing
const MASTER_ZOBRIST_KEY: u64 = 20260829;  // Used to create deterministic Zobrists
static mut PST_ZOBRIST: [[[u64; 64]; 6]; 2] = [[[0; 64]; 6]; 2]; // magic numbers are 64 squares, 6 different piece types, 2 colors
static mut TURN_ZOBRIST: u64 = 0;
static mut CASTLING_ZOBRIST: [u64; 16] = [0; 16]; // 1 for each castling combination
static mut ENPASSANT_ZOBRIST: [u64; 8] = [0; 8]; // 1 for each possible column

// Initialize at startup
pub unsafe fn init_zobrist() {
    
    let mut rng = StdRng::seed_from_u64(MASTER_ZOBRIST_KEY);

    let mut pst_zobrist: [[[u64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];
    for color in [Color::White, Color::Black] {
        for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King] {
            for sq in 0..64usize {
                pst_zobrist[color.idx()][piece.idx()][sq] = rng.random();
            }
        }
    }
    PST_ZOBRIST = pst_zobrist;

    TURN_ZOBRIST = rng.random();

    let mut castling_zobrist: [u64; 16] = [0; 16];
    for i in 0..16usize {
        castling_zobrist[i] = rng.random();
    }
    CASTLING_ZOBRIST = castling_zobrist;

    let mut enpassant_zobrist: [u64; 8] = [0; 8];
    for i in 0..8usize {
        enpassant_zobrist[i] = rng.random();
    }
    ENPASSANT_ZOBRIST = enpassant_zobrist; 

}


// https://www.chessprogramming.org/Zobrist_Hashing
impl Position {

    pub unsafe fn set_zobrist(&mut self) {
        self.zobrist = self.to_zobrist();
    }

    pub unsafe fn to_zobrist(&self) -> ZobristHash {
        let mut zobrist: ZobristHash = 0;

        // For each square, piece, color combo, add to zobrist
        for (sq, piece, color) in self.iter() {
            let (Some(p), Some(c)) = (piece, color) else { continue };
            zobrist ^= PST_ZOBRIST[c.idx()][p.idx()][sq.idx()];
        }

        // Turn zobrist is hashed in when white turn
        if self.turn == Color::White {
            zobrist ^= TURN_ZOBRIST;
        }

        // Add in castling zobrist
        zobrist ^= CASTLING_ZOBRIST[self.castling_rights as usize];

        // Add in en passant zobrist
        if let Some(sq) = self.en_passant {
            zobrist ^= ENPASSANT_ZOBRIST[sq.to_col() as usize];
        }

        zobrist

    }

    #[inline]
    pub unsafe fn zobrist_spc(&mut self, sq: Square, p: Piece, c: Color) {
        self.zobrist ^= PST_ZOBRIST[c.idx()][p.idx()][sq.idx()];
    }

    #[inline]
    pub unsafe fn zobrist_turn(&mut self) {
        self.zobrist ^= TURN_ZOBRIST;
    }

    #[inline]
    pub unsafe fn zobrist_castling(&mut self) {
        self.zobrist ^= CASTLING_ZOBRIST[self.castling_rights as usize];
    }

    #[inline]
    pub unsafe fn zobrist_enpassant(&mut self, sq: Square) {
        self.zobrist ^= ENPASSANT_ZOBRIST[sq.idx()];
    }

}
