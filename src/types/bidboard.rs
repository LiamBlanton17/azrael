use std::ops::{BitOrAssign, BitAnd, Shr, Shl};

// Type Bit board
#[derive(Default, Clone, Copy, PartialEq, Debug)]
pub struct BitBoard(pub u64);

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}

impl BitAnd for BitBoard {
    type Output = BitBoard;
    
    fn bitand(self, rhs: BitBoard) -> BitBoard {
        BitBoard(self.0 & rhs.0)
    }
}

impl Shr<u32> for BitBoard {
    type Output = BitBoard;
    
    fn shr(self, rhs: u32) -> BitBoard {
        BitBoard(self.0 >> rhs)
    }
}


impl Shl<u32> for BitBoard {
    type Output = BitBoard;
    
    fn shl(self, rhs: u32) -> BitBoard {
        BitBoard(self.0 << rhs)
    }
}
