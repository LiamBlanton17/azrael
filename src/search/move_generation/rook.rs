use crate::search::move_generation::{generate_ray_moves, ray_can_attack_sq};
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use crate::types::square::Square;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_rook_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let rooks = self.get_friendly_piece(Piece::Rook);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();

        for rook in rooks {
            generate_ray_moves(8, rook, friendly, enemy, level, move_stack); // Up
            generate_ray_moves(-8, rook, friendly, enemy, level, move_stack); // Down
            generate_ray_moves(-1, rook, friendly, enemy, level, move_stack); // Left
            generate_ray_moves(1, rook, friendly, enemy, level, move_stack); // Right
        }  
    }

    // Returns true if rook for the color given can capture the square give
    pub fn is_square_underattack_by_rook(&self, sq: Square, c: Color) -> bool {
        let rooks = self.get_piece(Piece::Rook, c);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();
        
        for rook in rooks {
            if ray_can_attack_sq(8, rook, sq, friendly, enemy) { return true; } // Up 
            if ray_can_attack_sq(-8, rook, sq, friendly, enemy) { return true; } // Down
            if ray_can_attack_sq(-1, rook, sq, friendly, enemy) { return true; } // Left
            if ray_can_attack_sq(1, rook, sq, friendly, enemy) { return true; } // Right
        }  

        false
    }

}
