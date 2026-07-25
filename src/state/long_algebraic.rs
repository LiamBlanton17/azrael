use crate::search::move_generation::MoveGenLevel;
use crate::types::chess_move::{MOVE_FLAG_PROMO, Move, split_move};
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::square::Square;

#[derive(Debug, Clone, PartialEq)]
pub enum LanError {
    InvalidFormat,
    NoLegalMove,
}

impl std::fmt::Display for LanError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LanError::InvalidFormat => write!(f, "invalid long algebraic format"),
            LanError::NoLegalMove => write!(f, "no legal move matches the long algebraic notation"),
        }
    }
}

// The origin, destination, and any promotion target a long algebraic string names
struct LanQuery {
    from: Square,          // origin square
    to: Square,            // destination square
    promo: Option<Piece>,  // promotion target, e.g. the Queen in "a7a8q"
}

impl Position {
    // Resolve a long algebraic (UCI coordinate) move string to the engine's packed move
    pub fn move_from_la(&mut self, la: &str) -> Result<Move, LanError> {
        let query = parse_la(la)?;

        // Generate pseudo-legal moves, then filter to legal ones as we scan for a match.
        let mut moves = Position::new_move_stack();
        self.generate_moves(&mut moves, MoveGenLevel::All, false, 0, (0, 0), &[[[0; 64]; 64]; 2]);

        for &m in &moves {
            // Skip moves that leave our own king in check
            let um = self.make_move(m);
            let legal = !self.can_kill_king();
            self.unmake_move(um);
            if !legal {
                continue;
            }

            let (dest, orig, promo_piece, flag) = split_move(m);

            // Origin and destination pin the move down exactly
            if orig != query.from || dest != query.to {
                continue;
            }

            // Promotion: match the exact target piece
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

            return Ok(m);
        }

        Err(LanError::NoLegalMove)
    }

    // Render an engine move as its long algebraic (UCI coordinate) string
    pub fn move_to_la(&self, m: Move) -> String {
        let (dest, orig, promo_piece, flag) = split_move(m);

        let mut la = format!("{orig}{dest}");
        if flag == MOVE_FLAG_PROMO {
            la.push(promo_char(promo_piece));
        }
        la
    }
}

// The lowercase promotion letter for a piece; only the four promotable pieces ever reach here.
fn promo_char(p: Piece) -> char {
    match p {
        Piece::Knight => 'n',
        Piece::Bishop => 'b',
        Piece::Rook => 'r',
        Piece::Queen => 'q',
        _ => '?',
    }
}

// Break a long algebraic string into the origin, destination, and any promotion target
fn parse_la(la: &str) -> Result<LanQuery, LanError> {
    let s = la.trim();
    if s.len() != 4 && s.len() != 5 {
        return Err(LanError::InvalidFormat);
    }

    let from = Square::from_str(&s[0..2]).ok_or(LanError::InvalidFormat)?;
    let to = Square::from_str(&s[2..4]).ok_or(LanError::InvalidFormat)?;

    let promo = match s.as_bytes().get(4) {
        Some(b) => Some(match b.to_ascii_lowercase() {
            b'n' => Piece::Knight,
            b'b' => Piece::Bishop,
            b'r' => Piece::Rook,
            b'q' => Piece::Queen,
            _ => return Err(LanError::InvalidFormat),
        }),
        None => None,
    };

    Ok(LanQuery { from, to, promo })
}
