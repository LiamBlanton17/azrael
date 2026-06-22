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
    pub fn to_row_col(self) -> (u8, u8) {
        (self.0 / 8, self.0 % 8)
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

    pub fn is_valid_enpassant_square(self) -> bool {
        let (row, _) = self.to_row_col();
        row == 1 || row == 6
    }

    #[inline]
    pub fn to_bitboard(&self) -> BitBoard {
        BitBoard(1u64 << self.0)
    }
}
