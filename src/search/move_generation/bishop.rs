use crate::search::move_generation::{generate_ray_moves, ray_can_attack_sq};
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use crate::types::square::Square;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_bishop_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let bishops = self.get_friendly_piece(Piece::Bishop);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();

        for bishop in bishops {
            generate_ray_moves(9, bishop, friendly, enemy, level, move_stack); // Up right
            generate_ray_moves(-7, bishop, friendly, enemy, level, move_stack); // Down Right
            generate_ray_moves(-9, bishop, friendly, enemy, level, move_stack); // Down Left
            generate_ray_moves(7, bishop, friendly, enemy, level, move_stack); // Up Left
        }  
    }

    // Returns true if bishop for the color given can capture the square give
    pub fn is_square_underattack_by_bishop(&self, sq: Square, c: Color) -> bool {
        let bishops = self.get_piece(Piece::Bishop, c);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();
        
        for bishop in bishops {
            if ray_can_attack_sq(9, bishop, sq, friendly, enemy) { return true; } // Up right
            if ray_can_attack_sq(-7, bishop, sq, friendly, enemy) { return true; } // Down Right
            if ray_can_attack_sq(-9, bishop, sq, friendly, enemy) { return true; } // Down Left
            if ray_can_attack_sq(7, bishop, sq, friendly, enemy) { return true; } // Up Left
        }  

        false
    }

}
