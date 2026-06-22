use super::position;
use super::bidboard::BitBoard;

// Square type
#[derive(Default, Clone, Copy)]
pub struct Square(pub u8);

impl Square {
    #[inline]
    pub fn from_row_col(row: u8, col: u8) -> Self {
        Square(row * 8 + col)
    }

    #[inline]
    pub fn to_bitboard(&self) -> BitBoard {
        BitBoard(1u64 << self.0)
    }
}
