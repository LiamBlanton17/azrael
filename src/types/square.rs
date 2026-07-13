use super::bidboard::BitBoard;

// Square type
#[derive(Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Square(pub u8);

// Square constants
pub const A1: Square = Square(0);
pub const B1: Square = Square(1);
pub const C1: Square = Square(2);
pub const D1: Square = Square(3);
pub const E1: Square = Square(4);
pub const F1: Square = Square(5);
pub const G1: Square = Square(6);
pub const H1: Square = Square(7);
pub const A8: Square = Square(56);
pub const B8: Square = Square(57);
pub const C8: Square = Square(58);
pub const D8: Square = Square(59);
pub const E8: Square = Square(60);
pub const F8: Square = Square(61);
pub const G8: Square = Square(62);
pub const H8: Square = Square(63);

impl Square {
    #[inline]
    pub fn from_row_col(row: u8, col: u8) -> Self {
        Square(row * 8 + col)
    }

    #[inline]
    pub fn to_row_col(self) -> (u8, u8) {
        (self.0 / 8, self.0 % 8)
    }

    #[inline]
    pub fn to_row(self) -> u8 {
        self.0 / 8
    }

    #[inline]
    pub fn to_col(self) -> u8 {
        self.0 % 8
    }

    pub fn from_str(s: &str) -> Option<Self> {
        // must be only 2 characters long
        let s_bytes = s.as_bytes();
        if s_bytes.len() != 2 {
            return None;
        }

        // Match for the column (a-h)
        let col = match s_bytes[0] {
            b'a'..=b'h' => s_bytes[0] - b'a',
            b'A'..=b'H' => s_bytes[0] - b'A',
            _ => return None,
        };

        // Match for the row (1-8)
        let row = match s_bytes[1] {
            b'1'..=b'8' => s_bytes[1] - b'1',
            _ => return None,
        };

        Some(Square::from_row_col(row, col))
    }  

    #[inline]
    pub fn is_valid_enpassant_square(self) -> bool {
        let (row, _) = self.to_row_col();
        row == 1 || row == 6
    }

    #[inline]
    pub fn to_bitboard(&self) -> BitBoard {
        BitBoard(1u64 << self.0)
    }

    #[inline]
    pub fn idx(&self) -> usize {
        self.0 as usize
    }

}

impl std::ops::Sub<u8> for Square {
    type Output = Square;

    #[inline]
    fn sub(self, rhs: u8) -> Square {
        Square(self.0 - rhs)
    }
}

impl std::ops::Add<u8> for Square {
    type Output = Square;

    #[inline]
    fn add(self, rhs: u8) -> Square {
        Square(self.0 + rhs)
    }
}

impl std::ops::AddAssign<u8> for Square {
    
    #[inline]
    fn add_assign(&mut self, rhs: u8) {
        self.0 += rhs
    }
}

impl std::fmt::Display for Square {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (row, col) = self.to_row_col();
        write!(f, "{}{}", (b'a' + col) as char, (b'1' + row) as char)
    }
}
