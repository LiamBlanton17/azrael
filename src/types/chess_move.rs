
// Move type aliasing to 16 packed bits
pub type Move = u16;

// Constants to unpack the move type
pub const MOVE_DESTINATION: Move = 0b0000_0000_0011_1111;
pub const MOVE_ORIGIN: Move = 0b0000_1111_1100_0000;
pub const MOVE_PROMO: Move = 0b0011_0000_0000_0000;
pub const MOVE_FLAG: Move = 0b1100_0000_0000_0000;

// Flags
pub const MOVE_FLAG_NONE: Move = 0b0000_0000_0000_0000;
pub const MOVE_FLAG_CAPTURE: Move = 0b0100_0000_0000_0000;
