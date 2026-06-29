use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::{Position, ZobristHash};
use std::sync::OnceLock;
use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;

// Setup globals used for Zobrist hashing
const MASTER_ZOBRIST_KEY: u64 = 20260829;  // Used to create deterministic Zobrists
static PST_ZOBRIST: OnceLock<[[[u64; 64]; 6]; 2]> = OnceLock::new(); // magic numbers are 64 squares, 6 different piece types, 2 colors
static TURN_ZOBRIST: OnceLock<u64> = OnceLock::new();
static CASTLING_ZOBRIST: OnceLock<[u64; 16]> = OnceLock::new(); // 1 for each castling combination
static ENPASSANT_ZOBRIST: OnceLock<[u64; 8]> = OnceLock::new(); // 1 for each possible column

// Initialize at startup
pub fn init_zobrist() {
    let mut rng = StdRng::seed_from_u64(MASTER_ZOBRIST_KEY);

    PST_ZOBRIST.get_or_init(|| {
        let mut pst_zobrist: [[[u64; 64]; 6]; 2] = [[[0; 64]; 6]; 2];

        for color in [Color::White, Color::Black] {
            for piece in [Piece::Pawn, Piece::Knight, Piece::Bishop, Piece::Rook, Piece::Queen, Piece::King] {
                for sq in 0..64usize {
                    pst_zobrist[color.idx()][piece.idx()][sq] = rng.random();
                }
            }
        }

        pst_zobrist
    });

    TURN_ZOBRIST.get_or_init(|| {
        rng.random()
    });


    CASTLING_ZOBRIST.get_or_init(|| {
        let mut castling_zobrist: [u64; 16] = [0; 16];

        for i in 0..16usize {
            castling_zobrist[i] = rng.random();
        }

        castling_zobrist
    });

    ENPASSANT_ZOBRIST.get_or_init(|| {
        let mut enpassant_zobrist: [u64; 8] = [0; 8];

        for i in 0..8usize {
            enpassant_zobrist[i] = rng.random();
        }

        enpassant_zobrist
    });

}


// https://www.chessprogramming.org/Zobrist_Hashing
impl Position {

    pub fn set_zobrist(&mut self) {
        self.zobrist = self.to_zobrist();
    }

    pub fn to_zobrist(&self) -> ZobristHash {
        
    }

}
