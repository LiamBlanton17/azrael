pub mod move_generation;
pub mod magics;
pub mod negamax;

use crate::search::negamax::negamax;
use crate::types::color::Color;
use crate::types::{chess_move::Move, position::ZobristHash};
use crate::types::eval::{Eval, min_eval_for_color};
use crate::types::position::Position;

use std::time::{Duration, Instant};

pub enum RootSearchType {
    TimeLimited(Duration),
    DepthLimited(u8),
}

impl Position {

    // https://www.chessprogramming.org/Iterative_Search
    pub fn root_search(&self, search_type: RootSearchType) -> (Eval, Move) {
        match search_type {
            RootSearchType::TimeLimited(budget) => time_search(self, budget),
            RootSearchType::DepthLimited(depth) => depth_search(self, depth),
        }
    }

}

// Main branch from root_search
fn time_search(p: &Position, budget: Duration) -> (Eval, Move) {
    unimplemented!("Time search not implemented yet")
}

// Main branch from root_search
fn depth_search(p: &Position, depth: u8) -> (Eval, Move) {
    let mut move_stack: Vec<Vec<Move>> = (0..=(depth + 1)).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(depth as usize + 1);

    let mut bestEval = min_eval_for_color(p.turn);
    let mut bestMove = 0;
    for d in 1..=depth {

        // reset the history (move gen resets the move stack)
        history.clear();

        // negamax search to this depth
        let (e, m) = negamax(p, d, &move_stack, &history);
        
        // update best eval/move if needed
        match p.turn {
            Color::White => {
                if e > bestEval {
                    bestEval = e;
                    bestMove = m;
                }
            },
            Color::Black => {
                if e < bestEval {
                    bestEval = e;
                    bestMove = m;
                }
            },
        }
        
    }

    (bestEval, bestMove)
}
