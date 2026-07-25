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

#[allow(dead_code)]
pub enum RootSearchType {
    StableTimeLimited(Duration),
    TimeLimited(Duration),
    StableDepthLimited(usize),
    DepthLimited(usize),
}

impl Position {

    // https://www.chessprogramming.org/Iterative_Search
    // `game_history` holds the zobrist hashes of every position that occurred BEFORE
    // the current (root) position in the actual game, oldest first. It seeds the search
    // history each iteration so repetitions against already-played moves are detected.
    // Pass an empty slice when no game history is available (e.g. one-off analysis).
    pub fn root_search(&mut self, search_type: RootSearchType, game_history: &[ZobristHash], tt: &mut TranspositionTable) -> (Eval, Move, u64, u64, usize) {
        match search_type {
            RootSearchType::StableTimeLimited(budget) => stable_time_search(self, budget, game_history, tt),
            RootSearchType::StableDepthLimited(depth) => stable_depth_search(self, depth, game_history, tt),
            RootSearchType::TimeLimited(budget) => time_search(self, budget, game_history, tt),
            RootSearchType::DepthLimited(depth) => depth_search(self, depth, game_history, tt),
        }
    }

}

// Decay the history heuristic between iterations when playing a game
const HISTORY_DECAY: i16 = 3;
fn age_history(history_heuristic: &mut [[[i16; 64]; 64]; 2]) {
    for color in history_heuristic.iter_mut() {
        for row in color.iter_mut() {
            for cell in row.iter_mut() {
                *cell /= HISTORY_DECAY;
            }
        }
    }
}

fn new_history_heuristic() -> [[[i16; 64]; 64]; 2] {
    [[[0; 64]; 64]; 2]
}

const TIME_SEARCH_EXPECTED_MAX_DEPTH: usize = 32;  // Prepare allocation for up to a depth of 32, will allocate more if needed (very unlikely)

// Main breanch from root_search - if best_move is stable for 4 iterations in a row, or a time budget is exceeded
// Stable is defined as the same best move and the best eval not moving more than 5 centipawns
const STABLE_ITERATION_THRESHOLD: u8 = 3;
const STABLE_ITERATION_START: usize = 10;
const STABLE_EVAL_THRESHOLD: Eval = 5;
const STABLE_ABF: u32 = 5;
const DEPTH_TO_START_ASPIRATION_WINDOWS: usize = 5;
fn stable_time_search(p: &mut Position, budget: Duration, game_history: &[ZobristHash], tt: &mut TranspositionTable) -> (Eval, Move, u64, u64, usize) {
    let mut move_stack: Vec<Vec<Move>> = (0..=TIME_SEARCH_EXPECTED_MAX_DEPTH).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(TIME_SEARCH_EXPECTED_MAX_DEPTH + game_history.len());
    
    let mut killers: Vec<(Move, Move)> = vec![(Move::default(), Move::default()); TIME_SEARCH_EXPECTED_MAX_DEPTH];
    let mut history_heuristic= new_history_heuristic();

    let mut best_eval = MIN_EVAL;
    let mut best_move = 0;
    let mut total_nodes = 0;
    let mut total_q_nodes = 0;

    // search as long as the estimated branching factor times the last search time is less than the remaining budget
    let mut last_search_time = Duration::new(0, 0);
    let mut depth = 0;
    let mut iterations_stable = 0;
    while last_search_time * STABLE_ABF < budget {
        // reset the history to the real game history (move gen resets the move stack)
        // so repetitions against already-played moves are detected during search
        history.clear();
        history.extend_from_slice(game_history);
        age_history(&mut history_heuristic);

        // negamax search to this depth
        let last_search_start = Instant::now();
        let (e, m, n, qn) = if depth <= DEPTH_TO_START_ASPIRATION_WINDOWS {
            negamax(p, depth, 0, -MATE, MATE, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt)
        } else {
            let mut window: i32 = 25;
            let mut asp_n = 0;
            let mut asp_qn = 0;
            loop {
                let alpha = (best_eval as i32 - window).max(-(MATE as i32)) as Eval;
                let beta  = (best_eval as i32 + window).min(  MATE as i32) as Eval;
                let (e, m, n, qn) = negamax(p, depth, 0, alpha, beta, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt);
                asp_n += n;
                asp_qn += qn;
                // re-search only if we failed AND still have room to widen
                if (e <= alpha && alpha > -(MATE)) || (e >= beta && beta < MATE) {
                    window *= 4;
                    continue;
                }
                break (e, m, asp_n, asp_qn);
            }
        };
        last_search_time = last_search_start.elapsed();

        // if stable from last iteration, increment and break if now 4 iterations stable (don't count depths 1-4)
        if m == best_move && (best_eval - e).abs() <= STABLE_EVAL_THRESHOLD && depth > STABLE_ITERATION_START {
            iterations_stable += 1;
            if iterations_stable == STABLE_ITERATION_THRESHOLD {
                best_eval = e;
                best_move = m;
                total_nodes += n;
                total_q_nodes += qn;
                break;
            }
        } else {
            iterations_stable = 0;
        }

        best_eval = e;
        best_move = m;
        total_nodes += n;
        total_q_nodes += qn;
        
        depth += 1;
    }

    (best_eval, best_move, total_nodes, total_q_nodes, depth)
}

// Main branch from root_search - do not exceed a time budget
fn time_search(p: &mut Position, budget: Duration, game_history: &[ZobristHash], tt: &mut TranspositionTable) -> (Eval, Move, u64, u64, usize) {
    let mut move_stack: Vec<Vec<Move>> = (0..=TIME_SEARCH_EXPECTED_MAX_DEPTH).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(TIME_SEARCH_EXPECTED_MAX_DEPTH + game_history.len());
    
    let mut killers: Vec<(Move, Move)> = vec![(Move::default(), Move::default()); TIME_SEARCH_EXPECTED_MAX_DEPTH];
    let mut history_heuristic= new_history_heuristic();

    let mut best_eval = MIN_EVAL;
    let mut best_move = 0;
    let mut total_nodes = 0;
    let mut total_q_nodes = 0;

    // search as long as the estimated branching factor times the last search time is less than the remaining budget
    let mut last_search_time = Duration::new(0, 0);
    let mut depth = 0;
    while last_search_time * STABLE_ABF < budget {

        // reset the history to the real game history (move gen resets the move stack)
        // so repetitions against already-played moves are detected during search
        history.clear();
        history.extend_from_slice(game_history);
        age_history(&mut history_heuristic);

        // negamax search to this depth
        let last_search_start = Instant::now();
        let (e, m, n, qn) = if depth <= DEPTH_TO_START_ASPIRATION_WINDOWS {
            negamax(p, depth, 0, -MATE, MATE, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt)
        } else {
            let mut window: i32 = 25;
            let mut asp_n = 0;
            let mut asp_qn = 0;
            loop {
                let alpha = (best_eval as i32 - window).max(-(MATE as i32)) as Eval;
                let beta  = (best_eval as i32 + window).min(  MATE as i32) as Eval;
                let (e, m, n, qn) = negamax(p, depth, 0, alpha, beta, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt);
                asp_n += n;
                asp_qn += qn;
                // re-search only if we failed AND still have room to widen
                if (e <= alpha && alpha > -(MATE)) || (e >= beta && beta < MATE) {
                    window *= 4;
                    continue;
                }
                break (e, m, asp_n, asp_qn);
            }
        };
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
fn stable_depth_search(p: &mut Position, depth: usize, game_history: &[ZobristHash], tt: &mut TranspositionTable) -> (Eval, Move, u64, u64, usize) {
    let mut move_stack: Vec<Vec<Move>> = (0..=(depth + 16)).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(depth + 16 + game_history.len());

    let mut killers: Vec<(Move, Move)> = vec![(Move::default(), Move::default()); TIME_SEARCH_EXPECTED_MAX_DEPTH];
    let mut history_heuristic= new_history_heuristic();

    let mut iterations_stable = 0;
    let mut best_eval = MIN_EVAL;
    let mut best_move = 0;
    let mut total_nodes = 0;
    let mut total_q_nodes = 0;
    let mut actual_depth_reached = 0;
    for d in 0..=depth {
        // reset the history to the real game history (move gen resets the move stack)
        // so repetitions against already-played moves are detected during search
        history.clear();
        history.extend_from_slice(game_history);
        age_history(&mut history_heuristic);
        actual_depth_reached = d;

        // negamax search to this depth
        let (e, m, n, qn) = if d <= DEPTH_TO_START_ASPIRATION_WINDOWS {
            negamax(p, d, 0, -MATE, MATE, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt)
        } else {
            let mut window: i32 = 25;
            let mut asp_n = 0;
            let mut asp_qn = 0;
            loop {
                let alpha = (best_eval as i32 - window).max(-(MATE as i32)) as Eval;
                let beta  = (best_eval as i32 + window).min(  MATE as i32) as Eval;
                let (e, m, n, qn) = negamax(p, depth, 0, alpha, beta, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt);
                asp_n += n;
                asp_qn += qn;
                // re-search only if we failed AND still have room to widen
                if (e <= alpha && alpha > -(MATE)) || (e >= beta && beta < MATE) {
                    window *= 4;
                    continue;
                }
                break (e, m, asp_n, asp_qn);
            }
        };

        
        // if stable from last iteration, increment and break if now 3 iterations stable (don't count depths 1-3)
        if m == best_move && (best_eval - e).abs() <= STABLE_EVAL_THRESHOLD && depth > STABLE_ITERATION_START {
            iterations_stable += 1;
            if iterations_stable == STABLE_ITERATION_THRESHOLD {
                best_eval = e;
                best_move = m;
                total_nodes += n;
                total_q_nodes += qn;
                break;
            }
        } else {
            iterations_stable = 0;
        }
        
        best_eval = e;
        best_move = m;
        total_nodes += n;
        total_q_nodes += qn;
    }

    (best_eval, best_move, total_nodes, total_q_nodes, actual_depth_reached)
}

// Main branch from root_search - go to a predefined depth
// Allocating for depth + 16 (for quiesence search after depth reached, want to prevent reallocs)
fn depth_search(p: &mut Position, depth: usize, game_history: &[ZobristHash], tt: &mut TranspositionTable) -> (Eval, Move, u64, u64, usize) {
    let mut move_stack: Vec<Vec<Move>> = (0..=(depth + 16)).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(depth + 16 + game_history.len());

    let mut killers: Vec<(Move, Move)> = vec![(Move::default(), Move::default()); TIME_SEARCH_EXPECTED_MAX_DEPTH];
    let mut history_heuristic = new_history_heuristic();

    let mut best_eval = MIN_EVAL;
    let mut best_move = 0;
    let mut total_nodes = 0;
    let mut total_q_nodes = 0;
    for d in 0..=depth {
        // reset the history to the real game history (move gen resets the move stack)
        // so repetitions against already-played moves are detected during search
        history.clear();
        history.extend_from_slice(game_history);
        age_history(&mut history_heuristic);

        // negamax search to this depth
        let (e, m, n, qn) = if d <= DEPTH_TO_START_ASPIRATION_WINDOWS {
            negamax(p, d, 0, -MATE, MATE, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt)
        } else {
            let mut window: i32 = 25;
            let mut asp_n = 0;
            let mut asp_qn = 0;
            loop {
                let alpha = (best_eval as i32 - window).max(-(MATE as i32)) as Eval;
                let beta  = (best_eval as i32 + window).min(  MATE as i32) as Eval;
                let (e, m, n, qn) = negamax(p, depth, 0, alpha, beta, &mut move_stack, &mut history, &mut killers, &mut history_heuristic, tt);
                asp_n += n;
                asp_qn += qn;
                // re-search only if we failed AND still have room to widen
                if (e <= alpha && alpha > -(MATE)) || (e >= beta && beta < MATE) {
                    window *= 4;
                    continue;
                }
                break (e, m, asp_n, asp_qn);
            }
        };

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
