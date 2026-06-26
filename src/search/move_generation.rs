
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
use crate::types::chess_move::{self, Move};
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

#[inline]
pub fn push_move(move_stack: &mut Vec<chess_move::Move>, to: Square, from: Square, flag: chess_move::Move, promotion: Piece) {
    move_stack.push(
        (to.0 as u16) |
        ((from.0 as u16) << chess_move::ORIGIN_SHIFT) |
        ((promotion as u16) << chess_move::PROMOT_SHIFT) |
        (flag << chess_move::FLAG_SHIFT)
    );
}

// Define helper function to push pawn moves
#[inline]
fn push_pawn_move(move_stack: &mut Vec<chess_move::Move>, to: Square, flag: Move, offset: u8, turn: color::Color) {
    let (from, is_promotion) =  if turn == color::Color::White { 
        ((to - offset), to > Square(56)) // on black's final rank
    } else { 
        ((to + offset), to < Square(8)) // on white's final rank
    };
    
    if !is_promotion {
        push_move(move_stack, to, from, flag, Piece::Empty);
    } else {
        push_move(move_stack, to, from, flag, piece::Piece::Knight);
        push_move(move_stack, to, from, flag, piece::Piece::Bishop);
        push_move(move_stack, to, from, flag, piece::Piece::Rook);
        push_move(move_stack, to, from, flag, piece::Piece::Queen);
    }
}

// Define helper function to generate ray moves
// TODO: refactor to use magic bitboards instead
fn generate_ray_moves(shift: i8, sq: Square, friendly: BitBoard, enemy: BitBoard, level: MoveGenLevel, move_stack: &mut Vec<chess_move::Move>) {
    let apply_shift = |bb| if shift >= 0 { bb << (shift as u32) } else { bb >> ((-shift) as u32) };

    let mut current_bb = apply_shift(sq.to_bitboard());
    while current_bb != BitBoard(0) && current_bb & friendly == BitBoard(0) {
        if current_bb & enemy != BitBoard(0) {
            if level == MoveGenLevel::Captures || level == MoveGenLevel::All {
                push_move(move_stack, current_bb.lsb_as_square(), sq, chess_move::MOVE_FLAG_CAPTURE, Piece::Empty);
            }
            break;
        }
        if level == MoveGenLevel::Quiets || level == MoveGenLevel::All {
            push_move(move_stack, current_bb.lsb_as_square(), sq, chess_move::MOVE_FLAG_NONE, Piece::Empty);
        }
        current_bb = apply_shift(current_bb);
    }
}

// Define helper function to see if a ray moving piece can attack a square
// TODO: refactor to use magic bitboards instead
fn ray_can_attack_sq(shift: i8, start: Square, target: Square, friendly: BitBoard, enemy: BitBoard) -> bool {
    let apply_shift = |bb| if shift >= 0 { bb << (shift as u32) } else { bb >> ((-shift) as u32) };

    let target_bb = target.to_bitboard();
    let mut current_bb = apply_shift(start.to_bitboard());
    while current_bb != BitBoard(0) && current_bb & friendly == BitBoard(0) {
        if current_bb & target_bb != BitBoard(0) {
            return true;
        }
        if current_bb & enemy != BitBoard(0) {
            return false;
        }
        current_bb = apply_shift(current_bb);
    }

    false
}
