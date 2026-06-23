use crate::types::position::Position;
use crate::types::chess_move;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_knight_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        match level {
            MoveGenLevel::Captures => generate_knight_captures(self, move_stack),
            MoveGenLevel::All => {
                generate_knight_captures(self, move_stack);
                generate_knight_quiets(self, move_stack);
            },
            MoveGenLevel::Quiets => generate_knight_quiets(self, move_stack),
        }
    }

}

pub fn generate_knight_captures(p: &Position, move_stack: &mut Vec<chess_move::Move>) {

}

pub fn generate_knight_quiets(p: &Position, move_stack: &mut Vec<chess_move::Move>) {

}
