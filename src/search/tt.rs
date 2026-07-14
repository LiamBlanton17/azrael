use crate::types::chess_move::Move;
use crate::types::eval::Eval;
use crate::types::position::ZobristHash;

#[derive(Clone, Copy, PartialEq)]
pub enum Bound {
    Exact,
    Lower, // fail-high / beta cutoff
    Upper, // fail-low / alpha
}

#[derive(Clone, Copy)]
pub struct TTEntry {
    pub key: ZobristHash, 
    pub best_move: Move,
    pub score: Eval,
    pub depth: u8,
    pub bound: Bound,
}

pub struct TranspositionTable {
    entries: Vec<Option<TTEntry>>,
    mask: usize, // size - 1, size is power of 2
}

impl TranspositionTable {
    pub fn new(size_mb: usize) -> Self {
        let entry_size = std::mem::size_of::<Option<TTEntry>>();
        let num_entries = (size_mb * 1024 * 1024 / entry_size).next_power_of_two() / 2;
        Self {
            entries: vec![None; num_entries],
            mask: num_entries - 1,
        }
    }

    #[inline]
    fn index(&self, key: ZobristHash) -> usize {
        (key as usize) & self.mask
    }

    pub fn probe(&self, key: ZobristHash) -> Option<TTEntry> {
        let idx = self.index(key);
        match self.entries[idx] {
            Some(e) if e.key == key => Some(e),
            _ => None,
        }
    }

    pub fn store(&mut self, key: u64, best_move: Move, score: i16, depth: u8, bound: Bound) {
        let idx = self.index(key);
        let replace = match &self.entries[idx] {
            None => true,
            Some(existing) => depth >= existing.depth // always-replace-if-deeper scheme
        };
        if replace {
            self.entries[idx] = Some(TTEntry { key, best_move, score, depth, bound });
        }
    }

    pub fn clear(&mut self) {
        for e in self.entries.iter_mut() {
            *e = None;
        }
    }
}
