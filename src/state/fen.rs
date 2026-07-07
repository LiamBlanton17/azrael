
use crate::types::position::{self, Position};
use crate::types::color::Color;
use crate::types::square::Square;
use crate::types::piece::Piece;
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
        let mut p = Position::default();

        // Split out the 5 parts we care about
        let pieces = parts[0].trim();
        let color = parts[1].trim();
        let castling_rights = parts[2].trim();
        let en_passant = parts[3].trim();
        let half_moves = parts[4].trim();

        // Try to parse the pieces
        if !parse_pieces_from(&mut p, pieces) {
            return Err(FENErrors::InvalidPieces)
        }

        // Try to parse the color
        if !parse_color_from(&mut p, color) {
            return Err(FENErrors::InvalidColor);
        }

        // Try to parse the castling rights
        if !parse_castling_rights(&mut p, castling_rights) {
            return Err(FENErrors::InvalidCastlingRights);
        }

        // Try to parse the en passant square
        if !parse_en_passant(&mut p, en_passant) {
            return Err(FENErrors::InvalidEnPassant);
        }

        // Try to parse the half moves
        if !parse_half_moves(&mut p, half_moves) {
            return Err(FENErrors::InvalidHalfMoves);
        }

        // Set zobrist hash
        unsafe { p.set_zobrist() };

        Ok(p)
    }

}

// Parse out the pieces from the FEN string part
// The string starts with the back rank and works forward
fn parse_pieces_from(p: &mut Position, pieces: &str) -> bool {
    let ranks: Vec<&str> = pieces.split("/").collect();

    // If not 8 ranks, then invalid
    if ranks.len() != 8 {
        return false;
    }

    // Loop over the ranks updating position and checking that it is valid
    for (row, rank) in ranks.iter().enumerate() {
        let mut col: u8 = 0;

        for c in rank.chars() {
            // if we ever exceed 7 columns before end of row, then illegal FEN
            if col > 7 {
                return false;
            }

            // match the current character
            let (piece, color) = match c {
                'p' => (Piece::Pawn, Color::Black),
                'P' => (Piece::Pawn, Color::White),
                'n' => (Piece::Knight, Color::Black),
                'N' => (Piece::Knight, Color::White),
                'b' => (Piece::Bishop, Color::Black),
                'B' => (Piece::Bishop, Color::White),
                'r' => (Piece::Rook, Color::Black),
                'R' => (Piece::Rook, Color::White),
                'q' => (Piece::Queen, Color::Black),
                'Q' => (Piece::Queen, Color::White),
                'k' => (Piece::King, Color::Black),
                'K' => (Piece::King, Color::White),
                '1'..='8' => {
                    col += c as u8 - b'0';
                    (Piece::Empty, Color::White)
                },
                _ => return false,
            };

            // if piece is found, update the board
            if piece != Piece::Empty {
                let sq: Square = Square::from_row_col((7 - row) as u8, col);
                let bit_board = sq.to_bitboard();
                p.color[color.idx()] |= bit_board;
                p.pieces[piece.idx()] |= bit_board;
                p.mailbox[sq.idx()] = piece;
                col += 1;
            }
        }

        // if not equal to 8 columns at end of row, then illegal FEN
        if col != 8 {
            return false;
        }
    }

    true
}

// Parse out the color from the FEN string part
fn parse_color_from(p: &mut Position, color: &str) -> bool {
    let c_bytes = color.as_bytes();
    if c_bytes.len() != 1 {
        return false;
    }

    match c_bytes[0] {
        b'w' => p.turn = Color::White,   
        b'b' => p.turn = Color::Black,   
        _ => return false,
    }

    true
}

// Parse out the castling rights from the FEN string part
fn parse_castling_rights(p: &mut Position, castling_rights: &str) -> bool {
    let mut highest_seen: u8 = 0; // 1 = K, 2 = Q, 3 = k, 4 = q
    for c in castling_rights.as_bytes() {
        match c {
            b'K' => {
                if highest_seen > 0 {
                    return false;
                }
                highest_seen = 1;
                p.castling_rights |= position::CASTLE_WK;
            },
            b'Q' => {
                if highest_seen > 1 {
                    return false;
                }
                highest_seen = 2;
                p.castling_rights |= position::CASTLE_WQ;
            },
            b'k' => {
                if highest_seen > 2 {
                    return false;
                }
                highest_seen = 3;
                p.castling_rights |= position::CASTLE_BK;
            },
            b'q' => {
                if highest_seen > 3 {
                    return false;
                }
                highest_seen = 4;
                p.castling_rights |= position::CASTLE_BQ;
            },
            b'-' => {
                if highest_seen != 0 {
                    return false;
                }
                highest_seen = 5;
            }
            _ => return false,
        }
    }

    // requiring string to not be empty
    highest_seen > 0 
}

// Parse out the en passant square from the FEN string part
fn parse_en_passant(position: &mut Position, en_passant: &str) -> bool {
    // if "-", then no en passant square
    if en_passant == "-" {
        position.en_passant = None;
        return true;
    }

    // try to parse square string and make sure it is valid for en passant
    match Square::from_str(en_passant) {
        Some(sq) => {
            if !sq.is_valid_enpassant_square() {
                return false;
            }

            position.en_passant = Some(sq);
        },
        None => return false,
    }

    true
}

// Parse out the half moves from the FEN string part
fn parse_half_moves(position: &mut Position, half_moves: &str) -> bool {
    match half_moves.parse::<i32>() {
        Ok(moves) => {
            // Half moves must be within the double 75-rule and positive
            if moves > 150 || moves < 0 {
                return false;
            } 
            position.half_moves = moves as u8;
        },
        Err(_) => return false,
    }

    true
}

#[cfg(test)]
mod tests {

    use crate::types::bidboard::BitBoard;

use super::*;

    #[test]
    fn test_from_fen() {

        struct Test {
            fen: &'static str,
            want: Result<Position, FENErrors>,
            description: &'static str,
        }

        let tests = vec![
            Test {
                fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                want: Ok(Position {
                    pieces: [
                        BitBoard(0x4200000000000042), // Knights
                        BitBoard(0x2400000000000024), // Bishops
                        BitBoard(0x8100000000000081), // Rooks
                        BitBoard(0x0800000000000008), // Queens
                        BitBoard(0x1000000000000010), // Kings
                        BitBoard(0x00FF00000000FF00), // Pawns
                    ],
                    color: [
                        BitBoard(0x000000000000FFFF), // White
                        BitBoard(0xFFFF000000000000), // Black
                    ],
                    mailbox: [
                        // Rank 1 (a1-h1) — White back rank
                        Piece::Rook, Piece::Knight, Piece::Bishop, Piece::Queen, Piece::King, Piece::Bishop, Piece::Knight, Piece::Rook,
                        // Rank 2 (a2-h2) — White pawns
                        Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn,
                        // Rank 3
                        Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                        // Rank 4
                        Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                        // Rank 5
                        Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                        // Rank 6
                        Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty, Piece::Empty,
                        // Rank 7 (a7-h7) — Black pawns
                        Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn, Piece::Pawn,
                        // Rank 8 (a8-h8) — Black back rank
                        Piece::Rook, Piece::Knight, Piece::Bishop, Piece::Queen, Piece::King, Piece::Bishop, Piece::Knight, Piece::Rook,
                    ],
                    zobrist: 0,
                    half_moves: 0,
                    castling_rights: 0b1111,
                    en_passant: None,
                    turn: Color::White,
                }),
                description: "starting position",
            },
            // TODO: add more tests
        ];

        for test in tests {
            assert_eq!(Position::from_fen(test.fen), test.want, "Failed for test: {}", test.description);
        }

    }

}