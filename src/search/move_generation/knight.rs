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
        match level {
            MoveGenLevel::Captures => generate_knight_captures(self, move_stack),
            MoveGenLevel::All => {
                generate_knight_captures(self, move_stack);
                generate_knight_quiets(self, move_stack);
            },
            MoveGenLevel::Quiets => generate_knight_quiets(self, move_stack),
        }
    }


    // Returns true if knight for the color given can capture the square give
    pub fn is_square_underattack_by_knight(&self, sq: Square, c: Color) -> bool {
        // Get the bitboards for the knights
        let knights: BitBoard = self.get_piece(Piece::Knight, c);

        // Generate the bitboard for the attacks
        unsafe {
            let moves: BitBoard = KNIGHT_MOVES[sq.idx()];
            moves & knights != BitBoard(0)
        }

    }

}

fn generate_knight_captures(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let knights = p.get_friendly_piece(Piece::Knight);
    let enemy = p.get_enemy_pieces();

    for knight in knights {
        unsafe {
            let attacks = enemy & KNIGHT_MOVES[knight.idx()];
            for to in attacks {
                push_move(move_stack, to, knight, chess_move::MOVE_FLAG_NONE, Piece::Knight);
            }
        }
    }
}

fn generate_knight_quiets(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let knights = p.get_friendly_piece(Piece::Knight);
    let pieces = p.get_all_pieces();

    for knight in knights {
        unsafe {
            let attacks = !pieces & KNIGHT_MOVES[knight.idx()];

            for to in attacks {
                push_move(move_stack, to, knight, chess_move::MOVE_FLAG_NONE, Piece::Knight);
            }
        }
    }
}

