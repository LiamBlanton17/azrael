
use crate::types::piece::Piece;

use super::bidboard::BitBoard;
use super::color::Color;
use super::square;

// Type aliasing u64 to Zobrist hash
pub type ZobristHash = u64;

// Castling right constants
pub const CASTLE_WK: u8 = 0b0001; // 1: White Kingside
pub const CASTLE_WQ: u8 = 0b0010; // 2: White Queenside
pub const CASTLE_BK: u8 = 0b0100; // 4: Black Kingside
pub const CASTLE_BQ: u8 = 0b1000; // 8: Black Queenside

// Main structure for the board state
#[derive(PartialEq, Debug)]
pub struct Position {

    // Piece bitboards
    pub pieces: [BitBoard; 6],

    // Color bitboards
    pub color: [BitBoard; 2],

    // Mailbox
    pub mailbox: [Piece; 64],

    // Zobrist hash of the position
    pub zobrist: ZobristHash,

    // Half move counter (won't exceed 100)
    pub half_moves: u8,

    // Castling rights 
    pub castling_rights: u8,

    // En passant square
    pub en_passant: Option<square::Square>,

    // Turn tracker
    pub turn: Color,

}

impl Default for Position {
    fn default() -> Self {
        Self {
            pieces: Default::default(),
            color: Default::default(),
            mailbox: [Piece::default(); 64],
            zobrist: Default::default(),
            half_moves: 0,
            castling_rights: 0,
            en_passant: None,
            turn: Default::default(),
        }
    }
}
