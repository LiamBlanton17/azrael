use crate::types::chess_move::Move;
use crate::types::eval::Eval;
use crate::types::position::{Position, ZobristHash};

pub fn negamax(p: &Position, depth: u8, move_stack: &Vec<Vec<Move>>, history: &Vec<ZobristHash>) -> (Eval, Move) {
    if depth == 0 {
        return (p.eval(), 0);
    }

    (p.eval(), 0)
}
