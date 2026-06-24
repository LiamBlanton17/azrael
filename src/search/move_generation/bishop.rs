use crate::search::move_generation::generate_ray_moves;
use crate::types::bidboard::BitBoard;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use crate::types::square::Square;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_bishop_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let bishops = self.get_friendly_piece(Piece::Bishop);
        let friendly = self.get_friendly_pieces() & !bishops;
        let enemy = self.get_enemy_pieces();

        for bishop in bishops {
            generate_ray_moves(9, bishop, friendly, enemy, level, move_stack); // Up right
            generate_ray_moves(-7, bishop, friendly, enemy, level, move_stack); // Down Right
            generate_ray_moves(-9, bishop, friendly, enemy, level, move_stack); // Down Left
            generate_ray_moves(7, bishop, friendly, enemy, level, move_stack); // Up Left
        }  
    }

}
