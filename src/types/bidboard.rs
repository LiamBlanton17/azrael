use std::ops::BitOrAssign;

// Type Bit board
#[derive(Default, Clone, Copy)]
pub struct BitBoard(pub u64);

impl BitOrAssign for BitBoard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) {
        self.0 |= rhs.0;
    }
}
