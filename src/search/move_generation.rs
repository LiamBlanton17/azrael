
mod pawn;
mod knight;
mod bishop;
mod rook;
mod queen;
mod king;

use crate::types::bidboard::BitBoard;
use crate::types::{color, piece};
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move::{self, MOVE_FLAG_PROMO, Move};
use crate::types::square::Square;

// Max is 218, in a very weird, impossible position
pub const MAX_MOVES_IN_POSITION: usize = 256;

// Enum to define level of move generation
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum MoveGenLevel {
    All,
    Captures,
    Quiets,
}

impl Position {
    // Public function to call to allocate a new move stack
    #[inline]
    pub fn new_move_stack() -> Vec<chess_move::Move> {
        Vec::with_capacity(MAX_MOVES_IN_POSITION)
    }

    // Public function to call to generate moves for a position
    pub fn generate_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel) {
        move_stack.clear();
        self.generate_pawn_moves(move_stack, level);
        self.generate_knight_moves(move_stack, level);
        self.generate_bishop_moves(move_stack, level);
        self.generate_rook_moves(move_stack, level);
        self.generate_queen_moves(move_stack, level);
        self.generate_king_moves(move_stack, level);
    }

}


// NOTE: when flag is not a promotion, ALWAYS PASS Piece::Knight as the promotion piece to avoid move packing errors!
#[inline]
pub fn push_move(move_stack: &mut Vec<chess_move::Move>, to: Square, from: Square, flag: chess_move::Move, promotion: Piece) {
    move_stack.push(
        (to.0 as u16) |
        ((from.0 as u16) << chess_move::ORIGIN_SHIFT) |
        ((promotion as u16) << chess_move::PROMO_SHIFT) |
        (flag << chess_move::FLAG_SHIFT)
    );
}

// Define helper function to push pawn moves
#[inline]
fn push_pawn_move(move_stack: &mut Vec<chess_move::Move>, to: Square, flag: Move, offset: u8, turn: color::Color) {
    let (from, is_promotion) =  if turn == color::Color::White {
        ((to - offset), to >= Square(56)) // reached black's final rank (squares 56..=63)
    } else {
        ((to + offset), to < Square(8)) // reached white's final rank (squares 0..=7)
    };
    
    if !is_promotion {
        push_move(move_stack, to, from, flag, Piece::Knight);
    } else {
        push_move(move_stack, to, from, MOVE_FLAG_PROMO, piece::Piece::Knight);
        push_move(move_stack, to, from, MOVE_FLAG_PROMO, piece::Piece::Bishop);
        push_move(move_stack, to, from, MOVE_FLAG_PROMO, piece::Piece::Rook);
        push_move(move_stack, to, from, MOVE_FLAG_PROMO, piece::Piece::Queen);
    }
}

// File masks used to discard ray bits that wrap around the board edge.
const NOT_A_FILE: BitBoard = BitBoard(0xFEFEFEFEFEFEFEFE);
const NOT_H_FILE: BitBoard = BitBoard(0x7F7F7F7F7F7F7F7F);

// Mask applied after each ray step, preventing pieces from wrapping around the board
#[inline]
fn wrap_mask(shift: i8) -> BitBoard {
    match shift {
        1 | 9 | -7 => NOT_A_FILE,
        -1 | -9 | 7 => NOT_H_FILE,
        _ => BitBoard(0xFFFFFFFFFFFFFFFF),
    }
}

// Define helper function to generate ray moves
// TODO: refactor to use magic bitboards instead
pub fn generate_ray_moves(shift: i8, sq: Square, friendly: BitBoard, enemy: BitBoard, level: MoveGenLevel, move_stack: &mut Vec<chess_move::Move>) {
    let mask = wrap_mask(shift);
    let apply_shift = |bb: BitBoard| {
        let shifted = if shift >= 0 { bb << (shift as u32) } else { bb >> ((-shift) as u32) };
        shifted & mask
    };

    let mut current_bb = apply_shift(sq.to_bitboard());
    while current_bb != BitBoard(0) && current_bb & friendly == BitBoard(0) {
        if current_bb & enemy != BitBoard(0) {
            if level == MoveGenLevel::Captures || level == MoveGenLevel::All {
                push_move(move_stack, current_bb.lsb_as_square(), sq, chess_move::MOVE_FLAG_NONE, Piece::Knight);
            }
            break;
        }
        if level == MoveGenLevel::Quiets || level == MoveGenLevel::All {
            push_move(move_stack, current_bb.lsb_as_square(), sq, chess_move::MOVE_FLAG_NONE, Piece::Knight);
        }
        current_bb = apply_shift(current_bb);
    }
}

// Define helper function to see if a ray moving piece can attack a square
// TODO: refactor to use magic bitboards instead
fn ray_can_attack_sq(shift: i8, start: Square, target: Square, friendly: BitBoard, enemy: BitBoard) -> bool {
    let mask = wrap_mask(shift);
    let apply_shift = |bb: BitBoard| {
        let shifted = if shift >= 0 { bb << (shift as u32) } else { bb >> ((-shift) as u32) };
        shifted & mask
    };

    let target_bb = target.to_bitboard();
    let blockers = friendly | enemy;
    let mut current_bb = apply_shift(start.to_bitboard());
    while current_bb != BitBoard(0) {
        if current_bb & target_bb != BitBoard(0) {
            return true;
        }
        if current_bb & blockers != BitBoard(0) {
            return false;
        }
        current_bb = apply_shift(current_bb);
    }

    false
}
