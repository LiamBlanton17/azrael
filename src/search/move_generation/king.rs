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
        let attacks =
            (king << 8) |                   // north
            (king >> 8) |                   // south
            ((king << 1) & NOT_H_FILE) |    // east
            ((king >> 1) & NOT_A_FILE) |    // west
            ((king << 9) & NOT_H_FILE) |    // north-east
            ((king << 7) & NOT_A_FILE) |    // north-west
            ((king >> 7) & NOT_H_FILE) |    // south-east
            ((king >> 9) & NOT_A_FILE);     // south-west
            

        // Check if any attack can see the square
        attacks & sq.to_bitboard() != BitBoard(0)
    }

}

const NOT_A_FILE: BitBoard = BitBoard(0x7F7F7F7F7F7F7F7F);
const NOT_H_FILE: BitBoard = BitBoard(0xFEFEFEFEFEFEFEFE);

pub fn generate_king_captures(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let king = p.get_friendly_piece(Piece::King);
    let king_sq = king.lsb_as_square();
    let enemy = p.get_enemy_pieces();

    // TODO: precompute this and look it up for both captures and quiets
    let attacks = enemy & (
        (king << 8) |                   // north
        (king >> 8) |                   // south
        ((king << 1) & NOT_H_FILE) |    // east
        ((king >> 1) & NOT_A_FILE) |    // west
        ((king << 9) & NOT_H_FILE) |    // north-east
        ((king << 7) & NOT_A_FILE) |    // north-west
        ((king >> 7) & NOT_H_FILE) |    // south-east
        ((king >> 9) & NOT_A_FILE)      // south-west
    );

    // Add all the attacks to the move stack
    for to in attacks {
        push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_NONE, Piece::Empty);
    }

}

pub fn generate_king_quiets(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let king = p.get_friendly_piece(Piece::King);
    let king_sq = king.lsb_as_square();
    let pieces = p.get_all_pieces();

    // TODO: precompute this and look it up for both captures and quiets
    let moves = !pieces & (
        (king << 8) |                   // north
        (king >> 8) |                   // south
        ((king << 1) & NOT_H_FILE) |    // east
        ((king >> 1) & NOT_A_FILE) |    // west
        ((king << 9) & NOT_H_FILE) |    // north-east
        ((king << 7) & NOT_A_FILE) |    // north-west
        ((king >> 7) & NOT_H_FILE) |    // south-east
        ((king >> 9) & NOT_A_FILE)      // south-west
    );

    // Add all the moves to the move stack
    for to in moves {
        push_move(move_stack, to, king_sq, chess_move::MOVE_FLAG_NONE, Piece::Empty);
    }

    // Add castling moves if possible
    if p.can_castle_kingside() {
        push_move(move_stack, square::G1, king_sq, chess_move::MOVE_FLAG_CASTLE, Piece::Empty);
    }
    if p.can_castle_queenside() {
        push_move(move_stack, square::G1, king_sq, chess_move::MOVE_FLAG_CASTLE, Piece::Empty);
    }

}
