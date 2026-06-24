use crate::types::{bidboard::BitBoard, color::Color, piece::Piece, position::Position};

impl Position {

    #[inline]
    pub fn get_enemy_pieces(&self) -> BitBoard {
        self.color[(!self.turn).idx()]
    }

    #[inline]
    pub fn get_all_pieces(&self) -> BitBoard {
        self.color[Color::White.idx()] | self.color[Color::Black.idx()]
    }

    #[inline]
    pub fn get_friendly_piece(&self, p: Piece) -> BitBoard {
        self.pieces[p.idx()] & self.color[self.turn.idx()]
    }

}
