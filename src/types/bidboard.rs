use crate::types::square::Square;

use std::ops::{BitOrAssign, BitAndAssign, BitAnd, Shr, Shl, ShlAssign, ShrAssign, Not, BitOr};

// Type Bit board
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct BitBoard(pub u64);

impl BitBoard {
    #[inline]
    pub fn pop_lsb_as_square(&mut self) -> Square {
        let sq = Square(self.0.trailing_zeros() as u8);
        self.0 &= self.0 - 1;
        sq
    }

    #[inline]
    pub fn lsb_as_square(&self) -> Square {
        Square(self.0.trailing_zeros() as u8)
    }
}

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAndAssign for BitBoard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) {
        self.0 &= rhs.0;
    }
}

impl BitOr for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn bitor(self, rhs: Self) -> BitBoard {
        BitBoard(self.0 | rhs.0)
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;
    
    #[inline]
    fn bitand(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0 & rhs.0)
    }
}

impl Shr<u32> for BitBoard {
    type Output = BitBoard;
    
    #[inline]
    fn shr(self, rhs: u32) -> BitBoard {
        BitBoard(self.0 >> rhs)
    }
}


impl Shl<u32> for BitBoard {
    type Output = BitBoard;
    
    #[inline]
    fn shl(self, rhs: u32) -> BitBoard {
        BitBoard(self.0 << rhs)
    }
}

impl Not for BitBoard {
    type Output = BitBoard;

    #[inline]
    fn not(self) -> BitBoard {
        BitBoard(!self.0)
    }
}

impl ShlAssign<u32> for BitBoard {
    #[inline]
    fn shl_assign(&mut self, rhs: u32) {
        self.0 <<= rhs;
    }
}

impl ShrAssign<u32> for BitBoard {
    #[inline]
    fn shr_assign(&mut self, rhs: u32) {
        self.0 >>= rhs;
    }
}

impl Iterator for BitBoard {
    type Item = Square;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        if self.0 == 0 {
            return None;
        }
        Some(self.pop_lsb_as_square())
    }
}
