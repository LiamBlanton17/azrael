use super::MoveGenLevel;
use super::KING_MOVES;

use crate::search::move_generation::push_move;
use crate::types::bidboard::BitBoard;
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use crate::types::square;
use crate::types::square::Square;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_king_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        let king = self.get_friendly_piece(Piece::King);
        let king_sq = king.lsb_as_square();
        let friendly = self.get_friendly_pieces();
        let enemy = self.get_enemy_pieces();

        let moves = unsafe { KING_MOVES[king_sq.idx()] } & !friendly;
        let (targets, castling) = match level {
            MoveGenLevel::All => (moves, true),
            MoveGenLevel::Captures => (moves & enemy, false),
            MoveGenLevel::Quiets => (moves & !enemy, true),
        };

        for to in targets {
            push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_NONE, Piece::Knight);
        }

        // Castling is quiet-only
        if castling {
            if self.can_castle_kingside() {
                let to = if self.turn == Color::White { square::G1 } else { square::G8 };
                push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_CASTLE, Piece::Knight);
            }
            if self.can_castle_queenside() {
                let to = if self.turn == Color::White { square::C1 } else { square::C8 };
                push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_CASTLE, Piece::Knight);
            }
        }
    }

    // Returns true if king for the color given can capture the square give
    pub fn is_square_underattack_by_king(&self, sq: Square, c: Color) -> bool {
        let king: BitBoard = self.get_piece(Piece::King, c);
        unsafe { KING_MOVES[sq.idx()] & king != BitBoard(0) }
    }

}
