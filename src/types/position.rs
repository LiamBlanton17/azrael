
// Type aliasing u64 to bit boards
pub type BitBoard = u64;

// Type aliasing u64 to Zobrist hash
pub type ZobristHash = u64;

// Square type aliasing
pub type Square = u8;

// Enum for color
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum Color {
    #[default] White,
    Black,
}

// Castling right constants
pub const CASTLE_WK: u8 = 0b0001; // 1: White Kingside
pub const CASTLE_WQ: u8 = 0b0100; // 2: White Queenside
pub const CASTLE_BK: u8 = 0b0100; // 4: Black Kingside
pub const CASTLE_BQ: u8 = 0b1000; // 8: Black Queenside

// Main structure for the board state
#[derive(Default)]
pub struct Position {

    // Piece bitboards
    pub pieces: [BitBoard; 6],

    // Color bitboards
    pub color: [BitBoard; 2],

    // Zobrist hash of the position
    pub zobrist: ZobristHash,

    // Half move counter (won't exceed 100)
    pub half_moves: u8,

    // Castling rights 
    pub castling_rights: u8,

    // En passant square
    pub en_passant: u8,

    // Turn tracker
    pub turn: Color,

}
