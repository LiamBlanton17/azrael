use crate::search::magics::rook::get_rook_moves;
use crate::search::move_generation::push_move;
use crate::types::bidboard::BitBoard;
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move::{self, MOVE_FLAG_NONE};
use crate::types::square::Square;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_rook_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let rooks = self.get_friendly_piece(Piece::Rook);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();
        let occupancy = friendly | enemy;

        for rook in rooks {
            let moves = get_rook_moves(rook, occupancy) & !friendly;
            let targets = match level {
                MoveGenLevel::All => moves,
                MoveGenLevel::Captures => moves & enemy,
                MoveGenLevel::Quiets => moves & !enemy,
            };

            for to in targets {
                push_move(move_stack, to, rook, MOVE_FLAG_NONE, Piece::Knight);
            }
        }
    }

    // Returns true if rook for the color given can capture the square give
    pub fn is_square_underattack_by_rook(&self, sq: Square, c: Color) -> bool {
        let rooks = self.get_piece(Piece::Rook, c);
        let occupancy = self.get_all_pieces();

        for rook in rooks {
            if get_rook_moves(rook, occupancy) & sq.to_bitboard() != BitBoard(0) {
                return true;
            }
        }

        false
    }

}
