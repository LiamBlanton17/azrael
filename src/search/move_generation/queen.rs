use crate::search::move_generation::{generate_ray_moves, ray_can_attack_sq};
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use crate::types::square::Square;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_queen_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let queens = self.get_friendly_piece(Piece::Queen);
        let friendly = self.get_friendly_pieces();
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

    // Returns true if queen for the color given can capture the square give
    pub fn is_square_underattack_by_queen(&self, sq: Square, c: Color) -> bool {
        let queens = self.get_friendly_piece(Piece::Queen);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();

        for queen in queens {
            if ray_can_attack_sq(9, queen, sq, friendly, enemy) { return true; } // Up right
            if ray_can_attack_sq(-7, queen, sq, friendly, enemy) { return true; } // Down Right
            if ray_can_attack_sq(-9, queen, sq, friendly, enemy) { return true; } // Down Left
            if ray_can_attack_sq(7, queen, sq, friendly, enemy) { return true; } // Up Left
            if ray_can_attack_sq(8, queen, sq, friendly, enemy) { return true; } // Up
            if ray_can_attack_sq(-8, queen, sq, friendly, enemy) { return true; } // Down
            if ray_can_attack_sq(-1, queen, sq, friendly, enemy) { return true; } // Left
            if ray_can_attack_sq(1, queen, sq, friendly, enemy) { return true; } // Right
        } 

        false
    }

}
