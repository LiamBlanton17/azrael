use super::MoveGenLevel;

use crate::search::move_generation::push_pawn_move;
use crate::types::bidboard::BitBoard;
use crate::types::chess_move;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::color::{self, Color};
use crate::types::square::Square;

impl Position {

    // Returns pseudo-legal moves
    pub fn generate_pawn_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        match level {
            MoveGenLevel::All => {
                generate_pawn_captures(self, move_stack);
                generate_pawn_quiets(self, move_stack);
            },
            MoveGenLevel::Captures => generate_pawn_captures(self, move_stack),
            MoveGenLevel::Quiets => generate_pawn_quiets(self, move_stack),
        }
    }

    // Returns true if pawn for the color given can capture the square give
    pub fn is_square_underattack_by_pawn(&self, sq: Square, c: Color) -> bool {
        // Get the bitboards for the pawns
        let pawns: BitBoard = self.get_piece(Piece::Pawn, c);

        // Get the attacks depending on color
        let attacks= if c == color::Color::White {
            ((pawns & NOT_H_FILE) << 9u32) | ((pawns & NOT_A_FILE) << 7u32)
        } else {
            ((pawns & NOT_A_FILE) >> 9u32) | ((pawns & NOT_H_FILE) >> 7u32)
        };

        // Check if any attack can see the square
        attacks & sq.to_bitboard() != BitBoard(0)
    }

    // Return a bitboard for squares control by a pawn
    pub fn pawn_control_bitboard(&self, c: Color) -> BitBoard {
        // Get the bitboards for the players pawns and all enemy pieces (plus en passant square if exists)
        let pawns = self.get_piece(Piece::Pawn, c);

        // Get the possible attacks depending on color
        let (possible_left_attacks, possible_right_attacks) = if c == color::Color::White {
            ((pawns & NOT_H_FILE) << 9u32, (pawns & NOT_A_FILE) << 7u32)
        } else {
            ((pawns & NOT_A_FILE) >> 9u32, (pawns & NOT_H_FILE) >> 7u32)
        };

        // Return the combined bitboard
        possible_right_attacks | possible_left_attacks
    }

}

const NOT_A_FILE: BitBoard = BitBoard(0xFEFEFEFEFEFEFEFE);
const NOT_H_FILE: BitBoard = BitBoard(0x7F7F7F7F7F7F7F7F);

const RANK_2: BitBoard = BitBoard(0x000000000000FF00);
const RANK_7: BitBoard = BitBoard(0x00FF000000000000); 

fn generate_pawn_captures(p: &Position, move_stack: &mut Vec<chess_move::Move>) {

    // Get the bitboards for the players pawns and all enemy pieces (plus en passant square if exists)
    let pawns = p.get_friendly_piece(Piece::Pawn);
    let enemy = p.get_enemy_pieces() | p.en_passant.map_or(BitBoard(0), |sq| sq.to_bitboard());

    // Get the possible attacks depending on color
    let (possible_left_attacks, possible_right_attacks) = if p.turn == color::Color::White {
        ((pawns & NOT_H_FILE) << 9u32, (pawns & NOT_A_FILE) << 7u32)
    } else {
        ((pawns & NOT_A_FILE) >> 9u32, (pawns & NOT_H_FILE) >> 7u32)
    };

    // Get the actual attacks
    let right_attacks = possible_right_attacks & enemy;
    let left_attacks = possible_left_attacks & enemy;

    // For each right attack, push it to the move stack
    for to in right_attacks {
        let flag = if Some(to) == p.en_passant { chess_move::MOVE_FLAG_ENPASSANT} else { chess_move::MOVE_FLAG_NONE };
        push_pawn_move(move_stack, to, flag, 7, p.turn); // offset 7 means shift bitboard 7
    }

    // For each left attack, push it to the move stack
    for to in left_attacks {
        let flag = if Some(to) == p.en_passant { chess_move::MOVE_FLAG_ENPASSANT} else { chess_move::MOVE_FLAG_NONE };
        push_pawn_move(move_stack, to, flag, 9, p.turn); // offset 9 means shift bitboard 9
    }

}

fn generate_pawn_quiets(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    // Get the bitboards for the players pawns and all pieces
    let pawns = p.get_friendly_piece(Piece::Pawn);
    let pieces = p.get_all_pieces();

    // Generate the single and double rank pushes
    // Single push if no piece in front
    // Double push if no piece in front and no piece two squares in front and on first rank for that color
    let (single_pushes, double_pushes) = if p.turn == color::Color::White {
        let single = (pawns << 8u32) & !pieces;
        let double = ((pawns & RANK_2) << 16u32) & !pieces & !(pieces << 8u32);
        (single, double)
    } else {
        let single = (pawns >> 8u32) & !pieces;
        let double = ((pawns & RANK_7) >> 16u32) & !pieces & !(pieces >> 8u32);
        (single, double)
    };

    // For each single push add a move
    for to in single_pushes {
        push_pawn_move(move_stack, to, chess_move::MOVE_FLAG_NONE, 8, p.turn);
    }

    // For each double push add a move
    for to in double_pushes {
        push_pawn_move(move_stack, to, chess_move::MOVE_FLAG_NONE, 16, p.turn);
    }

}
