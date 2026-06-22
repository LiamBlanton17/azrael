
use crate::types::position::Position;
use crate::types::chess_move;

// Max is 218, in a very weird, impossible position
pub const MAX_MOVES_IN_POSITION: usize = 256;

impl Position {
    pub fn new_move_stack() -> Vec<chess_move::Move> {
        Vec::with_capacity(MAX_MOVES_IN_POSITION)
    }

    pub fn generate_moves(move_stack: &mut Vec<chess_move::Move>) {
        move_stack.clear();
    }
}
