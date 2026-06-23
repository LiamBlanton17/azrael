use crate::types::bidboard::BitBoard;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::{chess_move, color};
use super::MoveGenLevel;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_pawn_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        match level {
            MoveGenLevel::Captures => generate_pawn_captures(self, move_stack),
            MoveGenLevel::All => {
                generate_pawn_captures(self, move_stack);
                generate_pawn_quiets(self, move_stack);
            },
            MoveGenLevel::Quiets => generate_pawn_quiets(self, move_stack),
        }
    }

}

const NOT_A_FILE: BitBoard = BitBoard(0x7F7F7F7F7F7F7F7F);
const NOT_H_FILE: BitBoard = BitBoard(0xFEFEFEFEFEFEFEFE);

pub fn generate_pawn_captures(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let pawns = p.pieces[Piece::Pawn.idx()] & p.color[p.turn.idx()];
    let enemy = p.color[(!p.turn).idx()];

    let (possible_left_attacks, possible_right_attacks) = if p.turn == color::Color::White {
        ((pawns & NOT_H_FILE) >> 9u32, (pawns & NOT_A_FILE) >> 7u32)
    } else {
        ((pawns & NOT_H_FILE) << 9u32, (pawns & NOT_A_FILE) << 7u32)
    };

    let left_attacks = possible_left_attacks & enemy;
    let right_attacks = possible_right_attacks & enemy;
}

pub fn generate_pawn_quiets(p: &Position, move_stack: &mut Vec<chess_move::Move>) {

}
