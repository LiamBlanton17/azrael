use crate::search::move_generation::MoveGenLevel;
use crate::types::chess_move::{MOVE_FLAG_PROMO, Move, split_move};
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::square::{self, Square};

#[derive(Debug, Clone, PartialEq)]
pub enum SanError {
    InvalidFormat,
    NoLegalMove,
}

impl std::fmt::Display for SanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            SanError::InvalidFormat => write!(f, "invalid SAN format"),
            SanError::NoLegalMove => write!(f, "no legal move matches the SAN"),
        }
    }
}

struct SanQuery {
    piece: Piece,               // moving piece type (Pawn if no leading N/B/R/Q/K)
    target: Square,             // destination square
    disambig_file: Option<u8>,  // origin file constraint (0..=7), e.g. the 'b' in "Nbd7"
    disambig_rank: Option<u8>,  // origin rank constraint (0..=7), e.g. the '1' in "R1e1"
    promo: Option<Piece>,       // promotion target, e.g. the Queen in "e8=Q"
}

impl Position {
    // Resolve a SAN move string (e.g. "Nf3", "exd5", "O-O", "e8=Q+") to the engine's packed move (i16)
    // SAN is relative to the actual position, and needs to be checked against moves in the position
    pub fn san_to_move(&mut self, san: &str) -> Result<Move, SanError> {
        let query = parse_san(san, self.turn)?;

        // Generate pseudo-legal moves, then filter to legal ones as we scan for a match.
        let mut moves = Position::new_move_stack();
        self.generate_moves(&mut moves, MoveGenLevel::All, false);

        for &m in &moves {
            
            // Skip moves that leave our own king in check
            let um = self.make_move(m);
            let legal = !self.can_kill_king();
            self.unmake_move(um);
            if !legal {
                continue;
            }

            let (dest, orig, promo_piece, flag) = split_move(m);

            // Destination and moving piece type must match
            if dest != query.target {
                continue;
            }
            if self.mailbox[orig.idx()] != query.piece {
                continue;
            }

            // Promotion: match the exact target piece, or require a non-promotion when SAN
            // didn't ask for one (so "e8" never matches a promotion move)
            match query.promo {
                Some(p) => {
                    if flag != MOVE_FLAG_PROMO || promo_piece != p {
                        continue;
                    }
                }
                None => {
                    if flag == MOVE_FLAG_PROMO {
                        continue;
                    }
                }
            }

            // Disambiguation by origin file and/or rank
            if let Some(file) = query.disambig_file {
                if orig.to_col() != file {
                    continue;
                }
            }
            if let Some(rank) = query.disambig_rank {
                if orig.to_row() != rank {
                    continue;
                }
            }

            return Ok(m);
        }

        Err(SanError::NoLegalMove)
    }
}

// Break a SAN string into the constraints needed to identify a move.
fn parse_san(san: &str, turn: Color) -> Result<SanQuery, SanError> {
    // Drop the trailing check/checkmate marker, it carries no information for us
    let s = san.trim().trim_end_matches(['+', '#']);

    // Castling is spelled out; translate it to the concrete king destination for the side to move
    if s == "O-O-O" || s == "0-0-0" {
        let target = if turn == Color::White { square::C1 } else { square::C8 };
        return Ok(SanQuery { piece: Piece::King, target, disambig_file: None, disambig_rank: None, promo: None });
    }
    if s == "O-O" || s == "0-0" {
        let target = if turn == Color::White { square::G1 } else { square::G8 };
        return Ok(SanQuery { piece: Piece::King, target, disambig_file: None, disambig_rank: None, promo: None });
    }

    // Peel off a promotion suffix like "=Q"
    let (main, promo) = match s.split_once('=') {
        Some((head, tail)) => {
            let piece = match tail.as_bytes().first() {
                Some(b'N') => Piece::Knight,
                Some(b'B') => Piece::Bishop,
                Some(b'R') => Piece::Rook,
                Some(b'Q') => Piece::Queen,
                _ => return Err(SanError::InvalidFormat),
            };
            (head, Some(piece))
        }
        None => (s, None),
    };

    // A leading N/B/R/Q/K names the moving piece; otherwise it's a pawn
    let (piece, rest) = match main.as_bytes().first() {
        Some(b'N') => (Piece::Knight, &main[1..]),
        Some(b'B') => (Piece::Bishop, &main[1..]),
        Some(b'R') => (Piece::Rook, &main[1..]),
        Some(b'Q') => (Piece::Queen, &main[1..]),
        Some(b'K') => (Piece::King, &main[1..]),
        _ => (Piece::Pawn, main),
    };

    // Capture markers carry no location info; the last two chars are always the target square
    let rest = rest.replace('x', "");
    if rest.len() < 2 {
        return Err(SanError::InvalidFormat);
    }
    let split_at = rest.len() - 2;
    let target = Square::from_str(&rest[split_at..]).ok_or(SanError::InvalidFormat)?;

    // Whatever precedes the target square is the disambiguation (a file, a rank, or both)
    let mut disambig_file = None;
    let mut disambig_rank = None;
    for &b in rest[..split_at].as_bytes() {
        match b {
            b'a'..=b'h' => disambig_file = Some(b - b'a'),
            b'1'..=b'8' => disambig_rank = Some(b - b'1'),
            _ => return Err(SanError::InvalidFormat),
        }
    }

    Ok(SanQuery { piece, target, disambig_file, disambig_rank, promo })
}
