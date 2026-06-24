use crate::types::{bidboard::BitBoard, color::Color, piece::Piece, position::{self, Position}};

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
    pub fn get_friendly_piece(&self, p: Piece) -> BitBoard {
        self.pieces[p.idx()] & self.color[self.turn.idx()]
    }

    #[inline]
    // VITAL TODO: add check for castling through check and if pieces are in the way!!!! this is once all other move generation is done
    pub fn can_castle_kingside(&self) -> bool {
        if self.turn == Color::White {
            self.castling_rights & position::CASTLE_WK != 0
        } else {
            self.castling_rights & position::CASTLE_BK != 0
        }
    }

    #[inline]
    // VITAL TODO: add check for castling through check and if pieces are in the way!!!! this is once all other move generation is done
    pub fn can_castle_queenside(&self) -> bool {
        if self.turn == Color::White {
            self.castling_rights & position::CASTLE_WQ != 0
        } else {
            self.castling_rights & position::CASTLE_BQ != 0
        }
    }

}
