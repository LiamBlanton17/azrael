use crate::types::color::Color;
use crate::types::eval::Eval;
use crate::types::piece::Piece;
use crate::types::position::Position;

// Define material evaluations
pub const PAWN: Eval = 100;
pub const KNIGHT: Eval = 300;
pub const BISHOP: Eval = 300;
pub const ROOK: Eval = 500;
pub const QUEEN: Eval = 900;

impl Position {

    pub fn eval(&self) -> Eval {
        let mut eval = 0;
        for (p, e) in [
            (Piece::Pawn, PAWN),
            (Piece::Bishop, BISHOP),
            (Piece::Knight, KNIGHT),
            (Piece::Rook, ROOK),
            (Piece::Queen, QUEEN),
        ] {
            eval += self.get_piece(p, Color::White).0.count_ones() as i16 * e;
            eval -= self.get_piece(p, Color::Black).0.count_ones() as i16 * e;
        }
        eval
    }

    pub fn eval_relative(&self) -> Eval {
        match self.turn  {
            Color::White => self.eval(),
            Color::Black => -self.eval(),
        }
    }

}

