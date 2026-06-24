use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_king_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        match level {
            MoveGenLevel::Captures => generate_king_captures(self, move_stack),
            MoveGenLevel::All => {
                generate_king_captures(self, move_stack);
                generate_king_quiets(self, move_stack);
            },
            MoveGenLevel::Quiets => generate_king_quiets(self, move_stack),
        }
    }

}

pub fn generate_king_captures(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let king = p.get_friendly_piece(Piece::King);
    let enemy = p.get_enemy_pieces();
}

pub fn generate_king_quiets(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let king = p.get_friendly_piece(Piece::King);
    let pieces = p.get_all_pieces();
}
