
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

    #[inline]
    pub fn from_promo(bits: u8) -> Piece {
        match bits {
            0 => Piece::Knight,
            1 => Piece::Bishop,
            2 => Piece::Rook,
            _ => Piece::Queen,
        }
    }

}

impl Default for Piece {
    fn default() -> Self {
        Self::Empty
    }
}
