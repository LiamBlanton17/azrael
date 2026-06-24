use crate::search::move_generation::generate_ray_moves;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_rook_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let rooks = self.get_friendly_piece(Piece::Rook);
        let friendly = self.get_friendly_pieces() & !rooks;
        let enemy = self.get_enemy_pieces();

        for rook in rooks {
            generate_ray_moves(8, rook, friendly, enemy, level, move_stack); // Up
            generate_ray_moves(-8, rook, friendly, enemy, level, move_stack); // Down
            generate_ray_moves(-1, rook, friendly, enemy, level, move_stack); // Left
            generate_ray_moves(1, rook, friendly, enemy, level, move_stack); // Right
        }  
    }

}
