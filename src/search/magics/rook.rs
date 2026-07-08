use crate::{search::move_generation::{MoveGenLevel, generate_ray_moves}, types::{bidboard::BitBoard, position::Position, square::Square}};


static mut ROOK_MASK: [BitBoard; 64] = [BitBoard(0); 64];
static mut ROOK_MAGIC: [u64; 64] = [0; 64];
static mut ROOK_BITS: [u32; 64] = [0; 64];
static mut ROOK_ATTACK_TABLE: Vec<Vec<BitBoard>> = Vec::new();

pub fn init_rook_magic() {

    unsafe {
        for sq in 0..64 {
            ROOK_MASK[sq] = rook_mask_for_square(Square(sq as u8));
        }
    }

}

fn rook_attacks_slow(sq: Square, blockers: BitBoard) -> BitBoard {
    let (sq_row, sq_col) = sq.to_row_col();
    let mut attacks = BitBoard(0);

    // North
    for row in (sq_row + 1)..8 {
        let target = Square::from_row_col(row, sq_col).to_bitboard();
        attacks |= target;
        if blockers & target != BitBoard(0) { break; }
    }

    // South
    for row in (0..sq_row).rev() {
        let target = Square::from_row_col(row, sq_col).to_bitboard();
        attacks |= target;
        if blockers & target != BitBoard(0) { break; }
    }

    // East
    for col in (sq_col + 1)..8 {
        let target = Square::from_row_col(sq_row, col).to_bitboard();
        attacks |= target;
        if blockers & target != BitBoard(0) { break; }
    }

    // West
    for col in (0..sq_col).rev() {
        let target = Square::from_row_col(sq_row, col).to_bitboard();
        attacks |= target;
        if blockers & target != BitBoard(0) { break; }
    }

    attacks
}

fn rook_mask_for_square(sq: Square) -> BitBoard {
    let (sq_row, sq_col) = sq.to_row_col();
    let mut mask = BitBoard(0);

    // North
    for row in (sq_row + 1)..7 {
        mask |= Square::from_row_col(row, sq_col).to_bitboard();
    }

    // South
    for row in 1..sq_row {
        mask |= Square::from_row_col(row, sq_col).to_bitboard();
    }

    // East
    for col in (sq_col + 1)..7 {
        mask |= Square::from_row_col(sq_row, col).to_bitboard();
    }

    // West
    for col in 1..sq_col {
        mask |= Square::from_row_col(sq_row, col).to_bitboard();
    }

    mask
}
