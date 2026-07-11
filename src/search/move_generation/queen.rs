use super::MoveGenLevel;

use crate::search::magics::bishop::get_bishop_moves;
use crate::search::magics::rook::get_rook_moves;
use crate::search::move_generation::push_move;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move::{self, MOVE_FLAG_NONE};

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_queen_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let queens = self.get_friendly_piece(Piece::Queen);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();
        let occupancy = friendly | enemy;

        for queen in queens {
            let moves = (get_rook_moves(queen, occupancy) | get_bishop_moves(queen, occupancy)) & !friendly;
            let targets = match level {
                MoveGenLevel::All => moves,
                MoveGenLevel::Captures => moves & enemy,
                MoveGenLevel::Quiets => moves & !enemy,
            };

            for to in targets {
                push_move(move_stack, to, queen, MOVE_FLAG_NONE, Piece::Knight);
            }
        }
    }

}
