use crate::{search::move_generation::{MoveGenLevel, generate_ray_moves}, types::{bidboard::BitBoard, position::Position, square::Square}};


static mut ROOK_MASK: [BitBoard; 64] = [BitBoard(0); 64];

pub fn init_rook_mask() {

    unsafe {
        for sq in 0..64 {
            ROOK_MASK[sq] = rook_mask_for_square(Square(sq as u8));
        }
    }

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
