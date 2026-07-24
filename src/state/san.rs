use crate::search::move_generation::MoveGenLevel;
use crate::types::chess_move::{MOVE_FLAG_CASTLE, MOVE_FLAG_ENPASSANT, MOVE_FLAG_PROMO, Move, split_move};
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
    pub fn san_to_move(&mut self, san: &str) -> Result<Move, SanError> {
        let query = parse_san(san, self.turn)?;

        // Generate pseudo-legal moves, then filter to legal ones as we scan for a match.
        let mut moves = Position::new_move_stack();
        self.generate_moves(&mut moves, MoveGenLevel::All, false, 0, (0, 0), &[[0; 64]; 64]);

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

    // Render an engine move as its SAN string relative to the current position
    #[allow(dead_code)]
    pub fn move_to_san(&mut self, m: Move) -> String {
        let (dest, orig, promo_piece, flag) = split_move(m);

        // Castling is spelled out by the king's destination file, not its from/to squares
        if flag == MOVE_FLAG_CASTLE {
            let mut san = if dest.to_col() == square::G1.to_col() { "O-O".to_string() } else { "O-O-O".to_string() };
            san.push_str(self.check_suffix(m));
            return san;
        }

        let piece = self.mailbox[orig.idx()];

        // A capture is either an en passant, or a move onto an occupied square
        let is_capture = flag == MOVE_FLAG_ENPASSANT || self.mailbox[dest.idx()] != Piece::Empty;

        let mut san = String::new();

        if piece == Piece::Pawn {
            // Pawn captures name the origin file ("exd5"); quiet pushes are just the target ("e4")
            if is_capture {
                san.push((b'a' + orig.to_col()) as char);
                san.push('x');
            }
            san.push_str(&dest.to_string());
            if flag == MOVE_FLAG_PROMO {
                san.push('=');
                san.push(san_piece_letter(promo_piece));
            }
        } else {
            san.push(san_piece_letter(piece));
            san.push_str(&self.disambiguation(m, piece, orig, dest));
            if is_capture {
                san.push('x');
            }
            san.push_str(&dest.to_string());
        }

        san.push_str(self.check_suffix(m));
        san
    }

    // The minimal origin qualifier ("", "b", "1", or "b1")
    #[allow(dead_code)]
    fn disambiguation(&mut self, m: Move, piece: Piece, orig: Square, dest: Square) -> String {
        let mut moves = Position::new_move_stack();
        self.generate_moves(&mut moves, MoveGenLevel::All, false, 0, (0, 0), &[[0; 64]; 64]);

        let mut ambiguous = false;
        let mut same_file = false;
        let mut same_rank = false;

        for &other in &moves {
            if other == m {
                continue;
            }

            let (o_dest, o_orig, _, _) = split_move(other);
            if o_dest != dest || self.mailbox[o_orig.idx()] != piece {
                continue;
            }

            // Only legal rivals force disambiguation (a pinned piece can't actually go there)
            let um = self.make_move(other);
            let legal = !self.can_kill_king();
            self.unmake_move(um);
            if !legal {
                continue;
            }

            ambiguous = true;
            same_file |= o_orig.to_col() == orig.to_col();
            same_rank |= o_orig.to_row() == orig.to_row();
        }

        if !ambiguous {
            return String::new();
        }
        if !same_file {
            return ((b'a' + orig.to_col()) as char).to_string();
        }
        if !same_rank {
            return ((b'1' + orig.to_row()) as char).to_string();
        }
        format!("{}{}", (b'a' + orig.to_col()) as char, (b'1' + orig.to_row()) as char)
    }

    // "+" if the move gives check, "#" if it also mates, "" otherwise
    #[allow(dead_code)]
    fn check_suffix(&mut self, m: Move) -> &'static str {
        let um = self.make_move(m);

        // After make_move the side to move is the opponent, so `is_square_underattack` on their king
        // asks whether we (the mover) attack it
        let opp_king = self.get_friendly_piece(Piece::King).lsb_as_square();
        let suffix = if self.is_square_underattack(opp_king) {
            if self.has_legal_move() { "+" } else { "#" }
        } else {
            ""
        };

        self.unmake_move(um);
        suffix
    }

    // Does the side to move have at least one legal reply
    #[allow(dead_code)]
    fn has_legal_move(&mut self) -> bool {
        let mut moves = Position::new_move_stack();
        self.generate_moves(&mut moves, MoveGenLevel::All, false, 0, (0, 0), &[[0; 64]; 64]);
        for &m in &moves {
            let um = self.make_move(m);
            let legal = !self.can_kill_king();
            self.unmake_move(um);
            if legal {
                return true;
            }
        }
        false
    }
}

// The uppercase SAN letter for a piece
#[allow(dead_code)]
fn san_piece_letter(p: Piece) -> char {
    match p {
        Piece::Knight => 'N',
        Piece::Bishop => 'B',
        Piece::Rook => 'R',
        Piece::Queen => 'Q',
        Piece::King => 'K',
        _ => '?',
    }
}

// Break a SAN string into the constraints needed to identify a move.
fn parse_san(san: &str, turn: Color) -> Result<SanQuery, SanError> {
    // Drop the trailing check/checkmate marker
    let s = san.trim().trim_end_matches(['+', '#']);

    // Castling is spelled out
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

    // Capture markers carry no location info
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
