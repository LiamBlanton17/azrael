pub mod move_generation;
pub mod magics;
pub mod negamax;

use crate::search::magics::bishop::init_bishop_magic;
use crate::search::magics::rook::init_rook_magic;
use crate::search::move_generation::{init_king_moves, init_knight_moves};
use crate::search::negamax::negamax;
use crate::state::zobrist::init_zobrist;
use crate::types::{chess_move::Move, position::ZobristHash};
use crate::types::eval::{self, Eval};
use crate::types::position::Position;

use std::time::Duration;

pub enum RootSearchType {
    TimeLimited(Duration),
    DepthLimited(usize),
}

impl Position {

    // https://www.chessprogramming.org/Iterative_Search
    pub fn root_search(&mut self, search_type: RootSearchType) -> (Eval, Move, u64) {
        match search_type {
            RootSearchType::TimeLimited(budget) => time_search(self, budget),
            RootSearchType::DepthLimited(depth) => depth_search(self, depth),
        }
    }

}

// Main branch from root_search
fn time_search(p: &mut Position, budget: Duration) -> (Eval, Move, u64) {
    unimplemented!("Time search not implemented yet")
}

// Main branch from root_search
fn depth_search(p: &mut Position, depth: usize) -> (Eval, Move, u64) {
    let mut move_stack: Vec<Vec<Move>> = (0..=(depth + 1)).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(depth + 1);

    let mut best_eval = eval::MIN_EVAL;
    let mut best_move = 0;
    let mut total_nodes = 0;
    for d in 1..=depth {
        // reset the history (move gen resets the move stack)
        history.clear();

        // negamax search to this depth
        let (e, m, n) = negamax(p, d, 0, eval::MIN_EVAL, eval::MATE, &mut move_stack, &mut history);
        best_eval = e;
        best_move = m;
        total_nodes += n;
    }

    (best_eval, best_move, total_nodes)
}

// Must call this function for the engine to work
pub fn init_engine() {
    init_zobrist();
    init_rook_magic();
    init_bishop_magic();
    init_rook_magic();
    init_king_moves();
    init_knight_moves();
}
