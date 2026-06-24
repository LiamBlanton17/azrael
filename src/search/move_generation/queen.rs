use crate::search::move_generation::generate_ray_moves;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_queen_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let queens = self.get_friendly_piece(Piece::Queen);
        let friendly = self.get_friendly_pieces() & !queens;
        let enemy = self.get_enemy_pieces();

        for queen in queens {
            generate_ray_moves(9, queen, friendly, enemy, level, move_stack); // Up right
            generate_ray_moves(-7, queen, friendly, enemy, level, move_stack); // Down Right
            generate_ray_moves(-9, queen, friendly, enemy, level, move_stack); // Down Left
            generate_ray_moves(7, queen, friendly, enemy, level, move_stack); // Up Left
            generate_ray_moves(8, queen, friendly, enemy, level, move_stack); // Up
            generate_ray_moves(-8, queen, friendly, enemy, level, move_stack); // Down
            generate_ray_moves(-1, queen, friendly, enemy, level, move_stack); // Left
            generate_ray_moves(1, queen, friendly, enemy, level, move_stack); // Right
        } 
    }

}
