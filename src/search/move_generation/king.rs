use crate::search::move_generation::KING_MOVES;
use crate::search::move_generation::push_move;
use crate::types::bidboard::BitBoard;
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use crate::types::square;
use crate::types::square::Square;
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_king_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        match level {
            MoveGenLevel::Captures => generate_king_captures(self, move_stack),
            MoveGenLevel::All => {
                generate_king_captures(self, move_stack);
                generate_king_quiets(self, move_stack);
            },
            MoveGenLevel::Quiets => generate_king_quiets(self, move_stack),
        }
    }

    // Returns true if king for the color given can capture the square give
    pub fn is_square_underattack_by_king(&self, sq: Square, c: Color) -> bool {
        // Get the bitboards for the king
        let king: BitBoard = self.get_piece(Piece::King, c);

        // Generate the bitboard for the attacks
        unsafe {
            let moves = KING_MOVES[sq.idx()]; 
            moves & king != BitBoard(0)
        }
    }

}

pub fn generate_king_captures(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let king = p.get_friendly_piece(Piece::King);
    let king_sq = king.lsb_as_square();
    let enemy = p.get_enemy_pieces();

    unsafe {
        let attacks = enemy & KING_MOVES[king_sq.idx()];

        // Add all the attacks to the move stack
        for to in attacks {
            push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_NONE, Piece::Knight);
        }
    }
}

pub fn generate_king_quiets(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let king = p.get_friendly_piece(Piece::King);
    let king_sq = king.lsb_as_square();
    let pieces = p.get_all_pieces();

    unsafe {
        let moves = !pieces & KING_MOVES[king_sq.idx()];

        // Add all the moves to the move stack
        for to in moves {
            push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_NONE, Piece::Knight);
        }
    }

    // Add castling moves if possible
    if p.can_castle_kingside() {
        let to = if p.turn == Color::White { square::G1 } else { square::G8 };
        push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_CASTLE, Piece::Knight);
    }
    if p.can_castle_queenside() {
        let to = if p.turn == Color::White { square::C1 } else { square::C8 };
        push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_CASTLE, Piece::Knight);
    }

}
