use crate::search::magics::bishop::get_bishop_moves;
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
    pub fn generate_bishop_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let bishops = self.get_friendly_piece(Piece::Bishop);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();
        let occupancy = friendly | enemy;

        for bishop in bishops {
            let moves: BitBoard = get_bishop_moves(bishop, occupancy) & !friendly;
            let targets = match level {
                MoveGenLevel::All => moves,
                MoveGenLevel::Captures => moves & enemy,
                MoveGenLevel::Quiets => moves & !enemy,
            };

            for to in targets {
                push_move(move_stack, to, bishop, MOVE_FLAG_NONE, Piece::Knight);
            }
        }
    }

    // Returns true if bishop for the color given can capture the square give
    pub fn is_square_underattack_by_bishop(&self, sq: Square, c: Color) -> bool {
        let bishops = self.get_piece(Piece::Bishop, c);
        let occupancy = self.get_all_pieces();

        for bishop in bishops {
            if get_bishop_moves(bishop, occupancy) & sq.to_bitboard() != BitBoard(0) {
                return true;
            }
        }

        false
    }

}
