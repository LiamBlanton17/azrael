use crate::{search::move_generation, types::{bidboard::BitBoard, chess_move::Move, color::Color, piece::Piece, position::{self, Position, ZobristHash}, square::{self, Square}}};

impl Position {

    #[inline]
    pub fn get_enemy_pieces(&self) -> BitBoard {
        self.color[(!self.turn).idx()]
    }

    #[inline]
    pub fn get_friendly_pieces(&self) -> BitBoard {
        self.color[self.turn.idx()]
    }

    #[inline]
    pub fn get_all_pieces(&self) -> BitBoard {
        self.color[Color::White.idx()] | self.color[Color::Black.idx()]
    }

    #[inline]
    pub fn get_piece(&self, p: Piece, c: Color) -> BitBoard {
        self.pieces[p.idx()] & self.color[c.idx()]
    }

    #[inline]
    pub fn get_friendly_piece(&self, p: Piece) -> BitBoard {
        self.pieces[p.idx()] & self.color[self.turn.idx()]
    }

    #[inline]
    pub fn is_square_empty(&self, sq: Square) -> bool {
        sq.to_bitboard() & self.get_all_pieces() == BitBoard(0)
    }

    #[inline]
    pub fn is_square_underattack(&self, sq: Square) -> bool {
        let c = !self.turn;
        if self.is_square_underattack_by_pawn(sq, c) { return true; }
        if self.is_square_underattack_by_knight(sq, c) { return true; }
        if self.is_square_underattack_by_king(sq, c) { return true; }
        if self.is_square_underattack_by_bishop(sq, c) { return true; }
        if self.is_square_underattack_by_rook(sq, c) { return true; }
        if self.is_square_underattack_by_queen(sq, c) { return true; }
        false
    }

    #[inline]
    pub fn can_castle_kingside(&self) -> bool {
        // The search loop will make sure king is not in check at G1/G8, so no need for under attack check
        (
            self.turn == Color::White 
            && self.castling_rights & position::CASTLE_WK != 0
            && !self.is_square_underattack(square::E1)
            && !self.is_square_underattack(square::F1)
            && self.is_square_empty(square::F1)
            && self.is_square_empty(square::G1)
        ) || (   
            self.turn == Color::Black 
            && self.castling_rights & position::CASTLE_BK != 0
            && !self.is_square_underattack(square::E8)
            && !self.is_square_underattack(square::F8)
            && self.is_square_empty(square::F8)
            && self.is_square_empty(square::G8)
        )
    }

    #[inline]
    pub fn can_castle_queenside(&self) -> bool {
        // The search loop will make sure king is not in check at C1/C8, so no need for under attack check
        (
            self.turn == Color::White 
            && self.castling_rights & position::CASTLE_WQ != 0
            && !self.is_square_underattack(square::E1)
            && !self.is_square_underattack(square::D1)
            && self.is_square_empty(square::D1)
            && self.is_square_empty(square::C1)
            && self.is_square_empty(square::B1)
        ) || (   
            self.turn == Color::Black 
            && self.castling_rights & position::CASTLE_BQ != 0
            && !self.is_square_underattack(square::E8)
            && !self.is_square_underattack(square::D8)
            && self.is_square_empty(square::D8)
            && self.is_square_empty(square::C8)
            && self.is_square_empty(square::B8)
        )
    }

    pub fn make_move(&self, m: Move) {

    }

    pub fn unmake_move(&self, m: Move) {

    }

    pub fn can_kill_king(&self) -> bool {
        false
    }

    pub fn is_three_fold(&self, history: &Vec<ZobristHash>) -> bool {
        false
    }

    pub fn is_fifty_move_rule(&self) -> bool {
        self.half_moves > 99
    }

    pub fn iter(&self) -> PositionIter {
        PositionIter { 
            position: self, 
            square: Square(0) 
        }
    }

}

pub struct PositionIter<'a> {
    position: &'a Position,
    square: Square, // 0..64
}

impl<'a> Iterator for PositionIter<'a> {
    type Item = (Square, Option<Piece>, Option<Color>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.square < Square(64) {
            let sq = self.square;
            let sq_bb = self.square.to_bitboard();
            self.square += 1;


            // Find which color is on this square
            let color = if self.position.color[Color::White.idx()] & sq_bb != BitBoard(0) {
                Color::White
            } else if self.position.color[Color::Black.idx()] & sq_bb != BitBoard(0) {
                Color::Black
            } else {
                return Some((sq, None, None));
            };

            // Find which piece is on this square
            for p in [Piece::Pawn, Piece::Bishop, Piece::Knight, Piece::Rook, Piece::Queen, Piece::King] {
                if self.position.pieces[p.idx()] & sq_bb != BitBoard(0) {
                    return Some((sq, Some(p), Some(color)));
                }
            }
        }

        None
    }
}
