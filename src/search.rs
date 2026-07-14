pub mod move_generation;
pub mod magics;
pub mod negamax;
pub mod quiescence;
pub mod move_ordering;
pub mod tt;

use crate::search::magics::bishop::init_bishop_magic;
use crate::search::magics::rook::init_rook_magic;
use crate::search::move_generation::{init_king_moves, init_knight_moves};
use crate::search::negamax::negamax;
use crate::search::tt::TranspositionTable;
use crate::state::zobrist::init_zobrist;
use crate::types::{chess_move::Move, position::ZobristHash};
use crate::types::eval::{Eval, MATE, MIN_EVAL};
use crate::types::position::Position;

use std::time::{Duration, Instant};

pub enum RootSearchType {
    TimeLimited(Duration),
    DepthLimited(usize),
}

impl Position {

    // https://www.chessprogramming.org/Iterative_Search
    pub fn root_search(&mut self, search_type: RootSearchType, tt: &mut TranspositionTable) -> (Eval, Move, u64, u64, usize) {
        match search_type {
            RootSearchType::TimeLimited(budget) => time_search(self, budget, tt),
            RootSearchType::DepthLimited(depth) => depth_search(self, depth, tt),
        }
    }

}

// Main breanch from root_search - if best_move is stable for 3 iterations in a row, or a time budget is exceeded
// Stable is defined as the same best move and the best eval not moving more than 10 centipawns
fn stable_search() {
    unimplemented!("todo in future")
}

// Main branch from root_search - do not exceed a time budget
const TIME_SEARCH_EXPECTED_MAX_DEPTH: usize = 32;  // Prepare allocation for up to a depth of 32, will allocate more if needed (unlikely)
fn time_search(p: &mut Position, budget: Duration, tt: &mut TranspositionTable) -> (Eval, Move, u64, u64, usize) {
    let mut move_stack: Vec<Vec<Move>> = (0..=TIME_SEARCH_EXPECTED_MAX_DEPTH).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(TIME_SEARCH_EXPECTED_MAX_DEPTH);
    
    let mut killers: Vec<(Move, Move)> = vec![(Move::default(), Move::default()); TIME_SEARCH_EXPECTED_MAX_DEPTH];
    let mut history_heuristic: [[i16; 64]; 64] = [[0; 64]; 64]; // 64 squares by 64 squares, allowing for a history_heuristic[from][to] lookup

    let mut best_eval = MIN_EVAL;
    let mut best_move = 0;
    let mut total_nodes = 0;
    let mut total_q_nodes = 0;

    // search as long as the estimated branching factor times the last search time is less than the remaining budget
    const EBF: u32 = 5; // Estimated branching factor is 5, better move ordering/TTs will decrease this
    let mut last_search_time = Duration::new(0, 0);
    let mut depth = 1;
    while last_search_time * EBF < budget {
        // reset the history (move gen resets the move stack)
        history.clear();

        // negamax search to this depth
        let last_search_start = Instant::now();
        let (e, m, n, qn) = negamax(p, depth, 0, MIN_EVAL, MATE, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt);
        last_search_time = last_search_start.elapsed();
        best_eval = e;
        best_move = m;
        total_nodes += n;
        total_q_nodes += qn;
        depth += 1;
    }

    (best_eval, best_move, total_nodes, total_q_nodes, depth)
}

// Main branch from root_search - go to a predefined depth
// Allocating for depth + 16 (for quiesence search after depth reached, want to prevent reallocs)
fn depth_search(p: &mut Position, depth: usize, tt: &mut TranspositionTable) -> (Eval, Move, u64, u64, usize) {
    let mut move_stack: Vec<Vec<Move>> = (0..=(depth + 16)).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(depth + 16);

    let mut killers: Vec<(Move, Move)> = vec![(Move::default(), Move::default()); TIME_SEARCH_EXPECTED_MAX_DEPTH];
    let mut history_heuristic: [[i16; 64]; 64] = [[0; 64]; 64]; // 64 squares by 64 squares, allowing for a history_heuristic[from][to] lookup

    let mut best_eval = MIN_EVAL;
    let mut best_move = 0;
    let mut total_nodes = 0;
    let mut total_q_nodes = 0;
    for d in 1..=depth {
        // reset the history (move gen resets the move stack)
        history.clear();

        // negamax search to this depth
        let (e, m, n, qn) = negamax(p, d, 0, MIN_EVAL, MATE, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt);
        best_eval = e;
        best_move = m;
        total_nodes += n;
        total_q_nodes += qn;
    }

    (best_eval, best_move, total_nodes, total_q_nodes, depth)
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
