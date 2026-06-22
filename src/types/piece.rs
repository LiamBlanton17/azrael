
// Piece type enum
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Piece {
    Knight = 0,
    Bishop = 1,
    Rook = 2,
    Queen = 3,
    King = 4,
    Pawn = 5,
    Empty = 6,
}

impl Piece {
    #[inline]
    pub fn idx(&self) -> usize {
        *self as usize
    }
}
