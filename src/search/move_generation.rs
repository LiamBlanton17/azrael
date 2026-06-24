
mod pawn;
mod knight;
mod bishop;
mod rook;
mod queen;
mod king;

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
