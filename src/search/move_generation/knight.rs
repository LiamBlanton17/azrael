use crate::search::move_generation::push_move;
use crate::types::bidboard::BitBoard;
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use crate::types::square::Square;
use super::MoveGenLevel;
use super::KNIGHT_MOVES;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_knight_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let knights = self.get_friendly_piece(Piece::Knight);
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();

        for knight in knights {
            let moves = unsafe { KNIGHT_MOVES[knight.idx()] } & !friendly;
            let targets = match level {
                MoveGenLevel::All => moves,
                MoveGenLevel::Captures => moves & enemy,
                MoveGenLevel::Quiets => moves & !enemy,
            };

            for to in targets {
                push_move(move_stack, to, knight, chess_move::MOVE_FLAG_NONE, Piece::Knight);
            }
        }
    }

    // Returns true if knight for the color given can capture the square give
    pub fn is_square_underattack_by_knight(&self, sq: Square, c: Color) -> bool {
        let knights: BitBoard = self.get_piece(Piece::Knight, c);
        unsafe { KNIGHT_MOVES[sq.idx()] & knights != BitBoard(0) }
    }

}

