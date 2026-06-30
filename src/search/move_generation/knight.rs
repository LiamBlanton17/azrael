use crate::search::move_generation::push_move;
use crate::types::bidboard::BitBoard;
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::chess_move;
use crate::types::square::Square;
use super::MoveGenLevel;

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
        let attacks = 
            ((knights << 17) & NOT_A_FILE) |                  // 2 UP, 1 RIGHT
            ((knights << 10) & NOT_A_FILE & NOT_B_FILE) |     // 1 UP, 2 RIGHT
            ((knights >> 6) & NOT_A_FILE & NOT_B_FILE) |      // 1 DOWN, 2 RIGHT
            ((knights >> 15) & NOT_A_FILE) |                  // 2 DOWN, 1 RIGHT
            ((knights << 15) & NOT_H_FILE) |                  // 2 UP, 1 LEFT
            ((knights << 6) & NOT_H_FILE & NOT_G_FILE) |      // 1 UP, 2 LEFT
            ((knights >> 10) & NOT_H_FILE & NOT_G_FILE) |     // 1 DOWN, 2 LEFT
            ((knights >> 17) & NOT_H_FILE);                   // 2 DOWN, 1 LEFT
            

        // Check if any attack can see the square
        attacks & sq.to_bitboard() != BitBoard(0)
    }

}

const NOT_A_FILE: BitBoard = BitBoard(0xFEFEFEFEFEFEFEFE);
const NOT_B_FILE: BitBoard = BitBoard(0xFDFDFDFDFDFDFDFD);
const NOT_G_FILE: BitBoard = BitBoard(0xBFBFBFBFBFBFBFBF);
const NOT_H_FILE: BitBoard = BitBoard(0x7F7F7F7F7F7F7F7F);

fn generate_knight_captures(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let knights = p.get_friendly_piece(Piece::Knight);
    let enemy = p.get_enemy_pieces();

    // TODO: compute a lookup table of knight moves at startup and the lookup
    for knight in knights {
        let knight_bb = knight.to_bitboard();
        let attacks = enemy & (
            ((knight_bb << 17) & NOT_A_FILE) |                  // 2 UP, 1 RIGHT
            ((knight_bb << 10) & NOT_A_FILE & NOT_B_FILE) |     // 1 UP, 2 RIGHT
            ((knight_bb >> 6) & NOT_A_FILE & NOT_B_FILE) |      // 1 DOWN, 2 RIGHT
            ((knight_bb >> 15) & NOT_A_FILE) |                  // 2 DOWN, 1 RIGHT
            ((knight_bb << 15) & NOT_H_FILE) |                  // 2 UP, 1 LEFT
            ((knight_bb << 6) & NOT_H_FILE & NOT_G_FILE) |      // 1 UP, 2 LEFT
            ((knight_bb >> 10) & NOT_H_FILE & NOT_G_FILE) |     // 1 DOWN, 2 LEFT
            ((knight_bb >> 17) & NOT_H_FILE)                    // 2 DOWN, 1 LEFT
        );

        for to in attacks {
            push_move(move_stack, to, knight, chess_move::MOVE_FLAG_NONE, Piece::Empty);
        }
    }

}

fn generate_knight_quiets(p: &Position, move_stack: &mut Vec<chess_move::Move>) {
    let knights = p.get_friendly_piece(Piece::Knight);
    let pieces = p.get_all_pieces();

    // TODO: compute a lookup table of knight moves at startup and the lookup
    for knight in knights {
        let knight_bb = knight.to_bitboard();
        let attacks = !pieces & (
            ((knight_bb << 17) & NOT_A_FILE) |                  // 2 UP, 1 RIGHT
            ((knight_bb << 10) & NOT_A_FILE & NOT_B_FILE) |     // 1 UP, 2 RIGHT
            ((knight_bb >> 6) & NOT_A_FILE & NOT_B_FILE) |      // 1 DOWN, 2 RIGHT
            ((knight_bb >> 15) & NOT_A_FILE) |                  // 2 DOWN, 1 RIGHT
            ((knight_bb << 15) & NOT_H_FILE) |                  // 2 UP, 1 LEFT
            ((knight_bb << 6) & NOT_H_FILE & NOT_G_FILE) |      // 1 UP, 2 LEFT
            ((knight_bb >> 10) & NOT_H_FILE & NOT_G_FILE) |     // 1 DOWN, 2 LEFT
            ((knight_bb >> 17) & NOT_H_FILE)                    // 2 DOWN, 1 LEFT
        );

        for to in attacks {
            push_move(move_stack, to, knight, chess_move::MOVE_FLAG_NONE, Piece::Empty);
        }
    }

}
