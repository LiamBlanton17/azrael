use crate::{eval, types::{color::Color, eval::Eval}};

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

    #[inline]
    pub fn to_char(&self, c: Color) -> char {
        let ch = match self {
            Piece::Pawn => 'p',
            Piece::Knight => 'n',
            Piece::Bishop => 'b',
            Piece::Rook => 'r',
            Piece::Queen => 'q',
            Piece::King => 'k',
            Piece::Empty => return '.',
        };
        match c {
            Color::White => ch.to_ascii_uppercase(),
            Color::Black => ch,
        }
    }

    #[inline]
    pub fn to_value(self) -> Eval {
        match self {
            Piece::Pawn => eval::PAWN,
            Piece::Knight => eval::KNIGHT,
            Piece::Bishop => eval::BISHOP,
            Piece::Rook => eval::ROOK,
            Piece::Queen => eval::QUEEN,
            _ => 0,
        }
    }

}

impl Default for Piece {
    fn default() -> Self {
        Self::Empty
    }
}
