
use crate::types::position::Position;
use super::errors::FENErrors;

impl Position {

    // Create a new position from a FEN string
    // FEN strings are made up of 6 parts seperated by a space
    // Pieces, Active Color, Castling Rights, En passant, Half Moves, Full Moves
    // For a chess engine, we don't really care about full moves
    // TODO: improve error handling in future to return a Result with the reason the FEN was invalid
    pub fn from_fen(fen: &str) -> Result<Self, FENErrors> {
        let parts: Vec<&str> = fen.split_whitespace().collect();

        // Must have 6 parts or invalid
        if parts.len() != 6 {
            return Err(FENErrors::NotEnoughParts);
        }

        // Create the structure
        let mut position = Position::default();

        // Split out the 5 parts we care about
        let pieces = parts[0];
        let color = parts[1];
        let castling_rights = parts[2];
        let en_passant = parts[3];
        let half_moves = parts[4];

        // Try to parse the pieces
        if !parse_pieces_from(&mut position, pieces) {
            return Err(FENErrors::InvalidPieces)
        }

        // Try to parse the color
        if !parse_color_from(&mut position, color) {
            return Err(FENErrors::InvalidColor);
        }

        // Try to parse the castling rights
        if !parse_castling_rights(&mut position, castling_rights) {
            return Err(FENErrors::InvalidCastlingRights);
        }

        // Try to parse the en passant square
        if !parse_en_passant(&mut position, en_passant) {
            return Err(FENErrors::InvalidEnPassant);
        }

        // Try to parse the half moves
        if !parse_half_moves(&mut position, half_moves) {
            return Err(FENErrors::InvalidHalfMoves);
        }
        
        Ok(position)
    }

}

// Parse out the pieces from the FEN string part
fn parse_pieces_from(position: &mut Position, pieces: &str) -> bool {
    return false;
}

// Parse out the color from the FEN string part
fn parse_color_from(position: &mut Position, color: &str) -> bool {
    return false;
}

// Parse out the castling rights from the FEN string part
fn parse_castling_rights(position: &mut Position, castling_rights: &str) -> bool {
    return false;
}

// Parse out the en passant square from the FEN string part
fn parse_en_passant(position: &mut Position, en_passant: &str) -> bool {
    return false;
}

// Parse out the half moves from the FEN string part
fn parse_half_moves(position: &mut Position, half_moves: &str) -> bool {
    return false;
}
