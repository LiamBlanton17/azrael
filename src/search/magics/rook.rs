use crate::search::magics::{MASTER_MAGIC_KEY, find_magic};
use crate::types::{bidboard::BitBoard, square::Square};

use rand::rngs::StdRng;
use rand::SeedableRng;

static mut ROOK_MASK: [BitBoard; 64] = [BitBoard(0); 64];
static mut ROOK_MAGIC: [u64; 64] = [0; 64];
static mut ROOK_SHIFT: [u32; 64] = [0; 64];
static mut ROOK_ATTACK_TABLE: Vec<Vec<BitBoard>> = Vec::new();

// https://www.chessprogramming.org/Magic_Bitboards
pub fn init_rook_magic() {

    let mut rng = StdRng::seed_from_u64(MASTER_MAGIC_KEY);

    let mut masks: [BitBoard; 64] = [BitBoard(0); 64];
    let mut magics: [u64; 64] = [0; 64];
    let mut shifts: [u32; 64] = [0; 64];
    let mut table: Vec<Vec<BitBoard>> = Vec::with_capacity(64);

    for sq in 0..64 {
        let square = Square(sq as u8);
        let mask = rook_mask_for_square(square);
        let n = mask.0.count_ones();
        masks[sq] = mask;
        shifts[sq] = 64 - n;

        // Enumerate every blocker subset of the mask and its true move set
        let size = 1usize << n;
        let mut blockers: Vec<BitBoard> = Vec::with_capacity(size);
        let mut attacks: Vec<BitBoard> = Vec::with_capacity(size);
        let mut subset = 0u64;
        loop {
            blockers.push(BitBoard(subset));
            attacks.push(rook_moves_slow(square, BitBoard(subset)));
            subset = subset.wrapping_sub(mask.0) & mask.0;
            if subset == 0 { break; }
        }

        // Search for a magic that maps every subset to a table slot without
        // destructive collisions (subsets sharing a slot must share an move set).
        let (magic, sq_table) = find_magic(&mut rng, mask, n, &blockers, &attacks);
        magics[sq] = magic;
        table.push(sq_table);
    }

    unsafe {
        ROOK_MASK = masks;
        ROOK_MAGIC = magics;
        ROOK_SHIFT = shifts;
        ROOK_ATTACK_TABLE = table;
    }

}

// Look up the squares a rook on `sq` moves given the full board occupancy
#[inline]
pub fn get_rook_moves(sq: Square, occupancy: BitBoard) -> BitBoard {
    let sq_idx = sq.idx();
    unsafe {
        let blockers = occupancy & ROOK_MASK[sq_idx];
        let index = (blockers.0.wrapping_mul(ROOK_MAGIC[sq_idx]) >> ROOK_SHIFT[sq_idx]) as usize;
        ROOK_ATTACK_TABLE[sq_idx][index]
    }
}

fn rook_moves_slow(sq: Square, blockers: BitBoard) -> BitBoard {
    let (sq_row, sq_col) = sq.to_row_col();
    let (sq_row, sq_col) = (sq_row as i8, sq_col as i8);
    let mut moves = BitBoard(0);

    // For each of the 4 direction, build the moves bitboard
    for &(dr, dc) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {

        // make the initial update for this direction
        let (mut row, mut col) = (sq_row + dr, sq_col + dc);

        // while still on the board, add the square to the moves bitboard until a blocker is hit
        while (0..8).contains(&row) && (0..8).contains(&col) {
            let target = Square::from_row_col(row as u8, col as u8).to_bitboard();
            moves |= target;
            if blockers & target != BitBoard(0) { 
                break; 
            }
            row += dr;
            col += dc;
        }
    }

    moves
}

fn rook_mask_for_square(sq: Square) -> BitBoard {
    let (sq_row, sq_col) = sq.to_row_col();
    let (sq_row, sq_col) = (sq_row as i8, sq_col as i8);
    let mut mask = BitBoard(0);

    // For each of the 4 directions, build the mask bitboard
    for &(dr, dc) in &[(1, 0), (-1, 0), (0, 1), (0, -1)] {

        // make the initial update for this direction
        let (mut row, mut col) = (sq_row + dr, sq_col + dc);

        // while still on the board (for the mask we also don't care about the edges, so only 1-6)
        // add square to the mask
        while (dr == 0 || (1..=6).contains(&row)) && (dc == 0 || (1..=6).contains(&col)) {
            mask |= Square::from_row_col(row as u8, col as u8).to_bitboard();
            row += dr;
            col += dc;
        }
    }

    mask
}

#[cfg(test)]
mod tests {
    use super::*;

    // Check every table entry against the slow reference
    // For each square, iterate all blocker subsets of its mask
    // Confirm the magic lookup returns the same attack set the slow ray walk does
    #[test]
    fn magic_lookup_matches_slow() {
        init_rook_magic();

        for sq in 0..64u8 {
            let square = Square(sq);
            let mask = unsafe { ROOK_MASK[sq as usize] };

            let mut subset = 0u64;
            while subset != 0 {
                let occ = BitBoard(subset);
                assert_eq!(
                    get_rook_moves(square, occ),
                    rook_moves_slow(square, occ),
                    "mismatch at square {sq} with occupancy {subset:#018x}"
                );
                subset = subset.wrapping_sub(mask.0) & mask.0;
            }
        }
    }
}
