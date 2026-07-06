use crate::types::{piece::Piece, square::Square};


// Unmove type
pub struct UnMove {
    pub en_passant: Option<Square>,
    pub captured_piece: Piece,
    pub origin: Square,
    pub destination: Square,
    pub flag: u16,
    pub castling_rights: u8,
    pub half_moves: u8,
}

// Move type aliasing to 16 packed bits
pub type Move = u16;

// Constants to shift for move packing
pub const DESTINATION_SHIFT: u8 = 0;
pub const ORIGIN_SHIFT: u8 = 6;
pub const PROMO_SHIFT: u8 = 12;
pub const FLAG_SHIFT: u8 = 14;

// Constants to unpack the move type
pub const MOVE_DESTINATION: Move = 0b0000_0000_0011_1111;
pub const MOVE_ORIGIN: Move = 0b0000_1111_1100_0000;
pub const MOVE_PROMO: Move = 0b0011_0000_0000_0000;
pub const MOVE_FLAG: Move = 0b1100_0000_0000_0000;

// Flags
pub const MOVE_FLAG_NONE: Move = 0b0000_0000_0000_0000;
pub const MOVE_FLAG_ENPASSANT: Move = 0b010_0000_0000_0000;
pub const MOVE_FLAG_CASTLE: Move = 0b0100_0000_0000_0000;
pub const MOVE_FLAG_PROMO: Move = 0b1000_0000_0000_0000;

pub fn split_move(m: Move) -> (Square, Square, Option<Piece>, Move) {
    let piece= if m & MOVE_FLAG_PROMO != 0 {
        Some(Piece::from_promo(((m & MOVE_PROMO) >> PROMO_SHIFT) as u8))
    } else {
        None
    };

    (
        Square(((m & MOVE_DESTINATION) >> DESTINATION_SHIFT) as u8),
        Square(((m & MOVE_ORIGIN) >> ORIGIN_SHIFT) as u8),
        piece,
        (m & MOVE_FLAG) >> FLAG_SHIFT,
    )
}