mod pawn;
mod knight;
mod bishop;
mod rook;
mod queen;
mod king;

use crate::types::bidboard::BitBoard;
use crate::types::color;
use crate::types::piece::{self, Piece};
use crate::types::position::Position;
use crate::types::chess_move::{self, MOVE_FLAG_PROMO, Move};
use crate::types::square::Square;

// Max is 218, in a very weird, impossible position
pub const MAX_MOVES_IN_POSITION: usize = 256;

// Enum to define level of move generation
#[allow(dead_code)]
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
}

#[inline]
pub fn ensure_move_stack_len(move_stack: &mut Vec<Vec<Move>>, ply: usize) {
    while move_stack.len() <= ply {
        move_stack.push(Position::new_move_stack());
    }
}

impl Position {

    // Public function to call to generate moves for a position
    pub fn generate_moves(&self, move_stack: &mut Vec<chess_move::Move>, level: MoveGenLevel, apply_ordering: bool, tt_move: Move, killers: (Move, Move), history_heuristic: &[[[i16; 64]; 64]; 2]) {
        move_stack.clear();
        self.generate_pawn_moves(move_stack, level);
        self.generate_knight_moves(move_stack, level);
        self.generate_bishop_moves(move_stack, level);
        self.generate_rook_moves(move_stack, level);
        self.generate_queen_moves(move_stack, level);
        self.generate_king_moves(move_stack, level);

        if apply_ordering {
            self.order_moves(move_stack, tt_move, killers, &history_heuristic[self.turn.idx()]);
        }
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

// initialize the knight/king move lookup tables
const NOT_A_FILE: BitBoard = BitBoard(0xFEFEFEFEFEFEFEFE);
const NOT_B_FILE: BitBoard = BitBoard(0xFDFDFDFDFDFDFDFD);
const NOT_G_FILE: BitBoard = BitBoard(0xBFBFBFBFBFBFBFBF);
const NOT_H_FILE: BitBoard = BitBoard(0x7F7F7F7F7F7F7F7F);

pub const KNIGHT_MOVES: [BitBoard; 64] = init_knight_moves();
pub const KING_MOVES: [BitBoard; 64] = init_king_moves();

// BitBoard's operator traits aren't const, so compute the tables on raw u64s.
const fn init_knight_moves() -> [BitBoard; 64] {
    let mut table = [BitBoard(0); 64];
    let mut i = 0;
    while i < 64 {
        let bb: u64 = 1u64 << i;
        table[i] = BitBoard(
            ((bb << 17) & NOT_A_FILE.0) |                    // 2 UP, 1 RIGHT
            ((bb << 10) & NOT_A_FILE.0 & NOT_B_FILE.0) |     // 1 UP, 2 RIGHT
            ((bb >> 6) & NOT_A_FILE.0 & NOT_B_FILE.0) |      // 1 DOWN, 2 RIGHT
            ((bb >> 15) & NOT_A_FILE.0) |                    // 2 DOWN, 1 RIGHT
            ((bb << 15) & NOT_H_FILE.0) |                    // 2 UP, 1 LEFT
            ((bb << 6) & NOT_H_FILE.0 & NOT_G_FILE.0) |      // 1 UP, 2 LEFT
            ((bb >> 10) & NOT_H_FILE.0 & NOT_G_FILE.0) |     // 1 DOWN, 2 LEFT
            ((bb >> 17) & NOT_H_FILE.0)                      // 2 DOWN, 1 LEFT
        );
        i += 1;
    }
    table
}

const fn init_king_moves() -> [BitBoard; 64] {
    let mut table = [BitBoard(0); 64];
    let mut i = 0;
    while i < 64 {
        let bb: u64 = 1u64 << i;
        table[i] = BitBoard(
            (bb << 8) |                     // north
            (bb >> 8) |                     // south
            ((bb << 1) & NOT_A_FILE.0) |    // east
            ((bb >> 1) & NOT_H_FILE.0) |    // west
            ((bb << 9) & NOT_A_FILE.0) |    // north-east
            ((bb << 7) & NOT_H_FILE.0) |    // north-west
            ((bb >> 7) & NOT_A_FILE.0) |    // south-east
            ((bb >> 9) & NOT_H_FILE.0)      // south-west
        );
        i += 1;
    }
    table
}
