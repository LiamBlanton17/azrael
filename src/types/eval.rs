// Centipawns
pub type Eval = i16;

// Defineing a match threshold
pub const MATE: Eval = 32000;

pub fn white_mate_in(n: u8) -> i16 { 
    MATE - n as i16 
}

pub fn black_mate_in(n: u8) -> i16 { 
    -MATE + n as i16 
}
