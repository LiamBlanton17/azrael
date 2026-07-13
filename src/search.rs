pub mod move_generation;
pub mod magics;
pub mod negamax;
pub mod quiescence;

use crate::search::magics::bishop::init_bishop_magic;
use crate::search::magics::rook::init_rook_magic;
use crate::search::move_generation::{init_king_moves, init_knight_moves};
use crate::search::negamax::negamax;
use crate::state::zobrist::init_zobrist;
use crate::types::{chess_move::Move, position::ZobristHash};
use crate::types::eval::{self, Eval};
use crate::types::position::Position;

use std::time::{Duration, Instant};

pub enum RootSearchType {
    TimeLimited(Duration),
    DepthLimited(usize),
}

impl Position {

    // https://www.chessprogramming.org/Iterative_Search
    pub fn root_search(&mut self, search_type: RootSearchType) -> (Eval, Move, u64, usize) {
        match search_type {
            RootSearchType::TimeLimited(budget) => time_search(self, budget),
            RootSearchType::DepthLimited(depth) => depth_search(self, depth),
        }
    }

}

// Main branch from root_search
const TIME_SEARCH_EXPECTED_MAX_DEPTH: usize = 32;  // Prepare allocation for up to a depth of 32, will allocate more if needed (unlikely)
fn time_search(p: &mut Position, budget: Duration) -> (Eval, Move, u64, usize) {
    let mut move_stack: Vec<Vec<Move>> = (0..=TIME_SEARCH_EXPECTED_MAX_DEPTH).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(TIME_SEARCH_EXPECTED_MAX_DEPTH);

    let mut best_eval = eval::MIN_EVAL;
    let mut best_move = 0;
    let mut total_nodes = 0;

    // search as long as we have not used 50% of the budget
    // todo: improve this time budget heuristic to check the estimated branching factor and last depth search time
    let start = Instant::now();
    let mut depth = 1;
    while start.elapsed() * 2 < budget {
        // reset the history (move gen resets the move stack)
        history.clear();

        // negamax search to this depth
        let (e, m, n) = negamax(p, depth, 0, eval::MIN_EVAL, eval::MATE, &mut move_stack, &mut history);
        best_eval = e;
        best_move = m;
        total_nodes += n;
        depth += 1;
    }

    (best_eval, best_move, total_nodes, depth)
}

// Main branch from root_search
// Allocating for depth + 16 (for quiesence search after depth reached, want to prevent reallocs)
fn depth_search(p: &mut Position, depth: usize) -> (Eval, Move, u64, usize) {
    let mut move_stack: Vec<Vec<Move>> = (0..=(depth + 16)).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(depth + 16);

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

    (best_eval, best_move, total_nodes, depth)
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
