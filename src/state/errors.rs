use std::fmt;

#[derive(Debug, Clone)]
pub enum FENErrors {
    NotEnoughParts,
    InvalidPieces,
    InvalidColor,
    InvalidCastlingRights,
    InvalidEnPassant,
    InvalidHalfMoves,
}

impl fmt::Display for FENErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FENErrors::NotEnoughParts => write!(f, "Not enough parts"),
            FENErrors::InvalidPieces => write!(f, "Invalid pieces"),
            FENErrors::InvalidCastlingRights => write!(f, "Invalid castling rights"),
            FENErrors::InvalidColor => write!(f, "Invalid color"),
            FENErrors::InvalidEnPassant => write!(f, "Invalid en passant"),
            FENErrors::InvalidHalfMoves => write!(f, "Invalid half moves"),
        }
    }
}
