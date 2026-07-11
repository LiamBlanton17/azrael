pub mod bishop;
pub mod rook;

use crate::types::bidboard::BitBoard;

use rand::Rng;
use rand::rngs::StdRng;

const MASTER_MAGIC_KEY: u64 = 0x1340_9F00_7600_0000;

// Draw a candidate magic
// AND-ing three draws yields a sparse u64, which is far more likely to be a working magic than a dense one
#[inline]
fn random_magic(rng: &mut StdRng, mask: BitBoard) -> u64 {
    let a: u64 = rng.random();
    let b: u64 = rng.random();
    let c: u64 = rng.random();
    let magic = a & b & c;

    // Skip recalculate the magic if it doesn't scatter the mask's high bits enough
    if (mask.0.wrapping_mul(magic) & 0xFF00_0000_0000_0000).count_ones() < 6 {
        return random_magic(rng, mask);
    }

    magic
}

// Brute-force a magic multiplier for a single square's blocker/attack tables.
fn find_magic(rng: &mut StdRng, mask: BitBoard, bits: u32, blockers: &[BitBoard], attacks: &[BitBoard]) -> (u64, Vec<BitBoard>) {
    let size = 1usize << bits;
    let shift = 64 - bits;
    let mut used: Vec<BitBoard> = vec![BitBoard(0); size];
    let mut seen: Vec<bool> = vec![false; size];

    // loop until good magic found
    'find_magic: loop {
        let magic = random_magic(rng, mask);

        // reset seen to entirely false
        seen.iter_mut().for_each(|s| *s = false);

        // test the magic by filling seen and used
        for (blocker, attack) in blockers.iter().zip(attacks.iter()) {
            let index = (blocker.0.wrapping_mul(magic) >> shift) as usize;
            if !seen[index] {
                seen[index] = true;
                used[index] = *attack;
            } else if used[index] != *attack {
                // bad magic, retry
                continue 'find_magic;
            }
        }

        // magic passed tests, return it
        return (magic, used);
    }
}
