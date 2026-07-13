use crate::types::color::Color;

// Centipawns
pub type Eval = i16;

// Define infinity scores
pub const MIN_EVAL: Eval = -32_768;
pub const MAX_EVAL: Eval = 32_767;

// Defineing a match threshold
pub const MATE: Eval = 32000;

pub fn white_mate_in(n: u8) -> i16 { 
    MATE - n as i16 
}

pub fn black_mate_in(n: u8) -> i16 { 
    -MATE + n as i16 
}

pub fn min_eval_for_color(c: Color) -> Eval {
    match c {
        Color::White => MIN_EVAL,
        Color::Black => MAX_EVAL,
    }
}

