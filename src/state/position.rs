use crate::types::bidboard::BitBoard;
use crate::types::chess_move::{self, Move, UnMove, split_move};
use crate::types::color::Color;
use crate::types::piece::Piece;
use crate::types::position::{self, Position, ZobristHash};
use crate::types::square::{self, Square};
use crate::search::magics::bishop::get_bishop_moves;
use crate::search::magics::rook::get_rook_moves;

use std::io::{self, Write};

impl Position {

    #[inline]
    pub fn get_enemy_pieces(&self) -> BitBoard {
        self.color[(!self.turn).idx()]
    }

    #[inline]
    pub fn get_friendly_pieces(&self) -> BitBoard {
        self.color[self.turn.idx()]
    }

    #[inline]
    pub fn get_all_pieces(&self) -> BitBoard {
        self.color[Color::White.idx()] | self.color[Color::Black.idx()]
    }

    #[inline]
    pub fn get_all_pieces_of_color(&self, c: Color) -> BitBoard {
        self.color[c.idx()]
    }

    #[inline]
    pub fn get_piece(&self, p: Piece, c: Color) -> BitBoard {
        self.pieces[p.idx()] & self.color[c.idx()]
    }

    #[inline]
    pub fn get_friendly_piece(&self, p: Piece) -> BitBoard {
        self.pieces[p.idx()] & self.color[self.turn.idx()]
    }

    #[inline]
    pub fn is_square_empty(&self, sq: Square) -> bool {
        sq.to_bitboard() & self.get_all_pieces() == BitBoard(0)
    }

    #[inline]
    pub fn is_move_capture(&self, m: Move) -> bool {
        let (d, _, _, _) = split_move(m);
        self.mailbox[d.idx()] != Piece::Empty
    }

    #[inline]
    pub fn is_square_underattack(&self, sq: Square) -> bool {
        let c = !self.turn;

        // check non-sliding pieces
        if self.is_square_underattack_by_pawn(sq, c) { return true; }
        if self.is_square_underattack_by_knight(sq, c) { return true; }
        if self.is_square_underattack_by_king(sq, c) { return true; }

        // now check sliding pieces
        let occ = self.get_all_pieces();
        let queens = self.get_piece(Piece::Queen, c);
        if get_bishop_moves(sq, occ) & (self.get_piece(Piece::Bishop, c) | queens) != BitBoard(0) { return true; }
        if get_rook_moves(sq, occ)   & (self.get_piece(Piece::Rook, c)   | queens) != BitBoard(0) { return true; }
        false
    }

    #[inline]
    pub fn can_castle_kingside(&self) -> bool {
        // The search loop will make sure king is not in check at G1/G8, so no need for under attack check
        (
            self.turn == Color::White 
            && self.castling_rights & position::CASTLE_WK != 0
            && !self.is_square_underattack(square::E1)
            && !self.is_square_underattack(square::F1)
            && self.is_square_empty(square::F1)
            && self.is_square_empty(square::G1)
        ) || (   
            self.turn == Color::Black 
            && self.castling_rights & position::CASTLE_BK != 0
            && !self.is_square_underattack(square::E8)
            && !self.is_square_underattack(square::F8)
            && self.is_square_empty(square::F8)
            && self.is_square_empty(square::G8)
        )
    }

    #[inline]
    pub fn can_castle_queenside(&self) -> bool {
        // The search loop will make sure king is not in check at C1/C8, so no need for under attack check
        (
            self.turn == Color::White 
            && self.castling_rights & position::CASTLE_WQ != 0
            && !self.is_square_underattack(square::E1)
            && !self.is_square_underattack(square::D1)
            && self.is_square_empty(square::D1)
            && self.is_square_empty(square::C1)
            && self.is_square_empty(square::B1)
        ) || (   
            self.turn == Color::Black 
            && self.castling_rights & position::CASTLE_BQ != 0
            && !self.is_square_underattack(square::E8)
            && !self.is_square_underattack(square::D8)
            && self.is_square_empty(square::D8)
            && self.is_square_empty(square::C8)
            && self.is_square_empty(square::B8)
        )
    }

    pub fn make_move(&mut self, m: Move) -> UnMove {
        // Parse out move and set initial variables
        let (dest_sq, orig_sq, promo_piece, flag) = split_move(m);
        let dest_sq_bb = dest_sq.to_bitboard();
        let orig_sq_bb = orig_sq.to_bitboard();
        let this_piece = self.mailbox[orig_sq.idx()];
        let captured_piece = self.mailbox[dest_sq.idx()];
        let this_turn = self.turn;
        let this_turn_idx = self.turn.idx();
        let enemy_turn = self.turn.flip();
        let enemy_turn_idx = self.turn.flip().idx();
        let um = UnMove {
            en_passant: self.en_passant,
            captured_piece: captured_piece,
            origin: orig_sq,
            destination: dest_sq,
            flag: flag,
            castling_rights: self.castling_rights,
            half_moves: self.half_moves,
        };

        // Move from its origin
        self.color[this_turn_idx] &= !orig_sq_bb;
        self.pieces[this_piece.idx()] &= !orig_sq_bb;
        self.mailbox[orig_sq.idx()] = Piece::Empty;
        self.zobrist_spc(orig_sq, this_piece, this_turn);
        self.pst_remove_piece(this_piece, this_turn, orig_sq);

        // If a piece sat on the destination square, this remove enemy piece from square
        if captured_piece != Piece::Empty {
            self.color[enemy_turn_idx] &= !dest_sq_bb;
            self.pieces[captured_piece.idx()] &= !dest_sq_bb;
            self.zobrist_spc(dest_sq, captured_piece, enemy_turn);
            self.pst_remove_piece(captured_piece, enemy_turn, dest_sq);
        }

        // Reset the half-move clock on pawn moves and captures, or increment
        if this_piece == Piece::Pawn || captured_piece != Piece::Empty {
            self.half_moves = 0;
        } else {
            self.half_moves += 1;
        }

        // Reset enpassant, removing any stale en passant file from the hash
        if let Some(ep) = self.en_passant {
            self.zobrist_enpassant(ep);
        }
        self.en_passant = None;

        match flag {
            chess_move::MOVE_FLAG_PROMO => {
                // Replace the pawn with the promotion piece
                self.color[this_turn_idx] |= dest_sq_bb;
                self.pieces[promo_piece.idx()] |= dest_sq_bb;
                self.mailbox[dest_sq.idx()] = promo_piece;
                self.zobrist_spc(dest_sq, promo_piece, this_turn);
                self.pst_add_piece(promo_piece, this_turn, dest_sq);
            },
            chess_move::MOVE_FLAG_CASTLE => {
                // Move the king to the destination square
                self.color[this_turn_idx] |= dest_sq_bb;
                self.pieces[this_piece.idx()] |= dest_sq_bb;
                self.mailbox[dest_sq.idx()] = this_piece;
                self.zobrist_spc(dest_sq, this_piece, this_turn);
                self.pst_add_piece(this_piece, this_turn, dest_sq);

                // Move the rook to the far side of the king
                let (rook_from, rook_to) = match dest_sq {
                    square::C1 => (square::A1, square::D1), // White queenside
                    square::G1 => (square::H1, square::F1), // White kingside
                    square::C8 => (square::A8, square::D8), // Black queenside
                    square::G8 => (square::H8, square::F8), // Black kingside
                    _ => panic!("Castling but to an impossible square?"),
                };

                self.color[this_turn_idx] &= !rook_from.to_bitboard();
                self.pieces[Piece::Rook.idx()] &= !rook_from.to_bitboard();
                self.mailbox[rook_from.idx()] = Piece::Empty;
                self.zobrist_spc(rook_from, Piece::Rook, this_turn);
                self.pst_remove_piece(Piece::Rook, this_turn, rook_from);

                self.color[this_turn_idx] |= rook_to.to_bitboard();
                self.pieces[Piece::Rook.idx()] |= rook_to.to_bitboard();
                self.mailbox[rook_to.idx()] = Piece::Rook;
                self.zobrist_spc(rook_to, Piece::Rook, this_turn);
                self.pst_add_piece(Piece::Rook, this_turn, rook_to);
            },
            chess_move::MOVE_FLAG_ENPASSANT => {
                // Move the pawn to the destination
                self.color[this_turn_idx] |= dest_sq_bb;
                self.pieces[this_piece.idx()] |= dest_sq_bb;
                self.mailbox[dest_sq.idx()] = this_piece;
                self.zobrist_spc(dest_sq, this_piece, this_turn);
                self.pst_add_piece(this_piece, this_turn, dest_sq);

                // Remove the captured pawn, which sits at the interesect of the orig row and the dest column
                let captured_sq = Square::from_row_col(orig_sq.to_row(), dest_sq.to_col());
                let captured_sq_bb = captured_sq.to_bitboard();
                self.color[enemy_turn_idx] &= !captured_sq_bb;
                self.pieces[Piece::Pawn.idx()] &= !captured_sq_bb;
                self.mailbox[captured_sq.idx()] = Piece::Empty;
                self.zobrist_spc(captured_sq, Piece::Pawn, enemy_turn);
                self.pst_remove_piece(Piece::Pawn, enemy_turn, captured_sq);
            },
            _ => {
                // Move piece to the destination
                self.color[this_turn_idx] |= dest_sq_bb;
                self.pieces[this_piece.idx()] |= dest_sq_bb;
                self.mailbox[dest_sq.idx()] = this_piece;
                self.zobrist_spc(dest_sq, this_piece, this_turn);
                self.pst_add_piece(this_piece, this_turn, dest_sq);

                // A double pawn push exposes an en passant target on the square it skipped over
                if this_piece == Piece::Pawn && orig_sq.0.abs_diff(dest_sq.0) == 16 {
                    let ep = Square((orig_sq.0 + dest_sq.0) / 2);
                    self.en_passant = Some(ep);
                    self.zobrist_enpassant(ep);
                }
            },
        }

        // Unset castling rights, in a piece lands or moves away from the rook or king home squares.
        // XOR the old castling contribution out of the hash, then the updated one back in.
        self.zobrist_castling();
        self.castling_rights &= !castling_rights_voided_by(orig_sq);
        self.castling_rights &= !castling_rights_voided_by(dest_sq);
        self.zobrist_castling();

        // Hand the turn to the opponent, toggling the side-to-move contribution
        self.turn = self.turn.flip();
        self.zobrist_turn();

        um
    }

    pub fn unmake_move(&mut self, um: UnMove) {

        /*
        UnMove {
            en_passant: self.en_passant,
            captured_piece: captured_piece,
            origin: orig_sq,
            destination: dest_sq,
            flag: flag,
            castling_rights: self.castling_rights,
            half_moves: self.half_moves,
        }; */

        // get enemy turn and flip turn
        let enemy_turn_idx = self.turn.idx();
        self.turn = self.turn.flip();
        let this_turn = self.turn;
        let enemy_turn = self.turn.flip();

        // Restore the side-to-move contribution to match the flipped turn
        self.zobrist_turn();

        // reset enpassant, swapping the post-move en passant file out for the restored one
        if let Some(ep) = self.en_passant {
            self.zobrist_enpassant(ep);
        }
        self.en_passant = um.en_passant;
        if let Some(ep) = self.en_passant {
            self.zobrist_enpassant(ep);
        }

        // reset castlign rights, swapping the post-move contribution out for the restored one
        self.zobrist_castling();
        self.castling_rights = um.castling_rights;
        self.zobrist_castling();

        // reset half move counter
        self.half_moves = um.half_moves;

        // find out which piece is at destination
        let this_piece = self.mailbox[um.destination.idx()];

        // parse out some variables
        let this_turn_idx = self.turn.idx();
        let orig_bb = um.origin.to_bitboard();
        let dest_bb = um.destination.to_bitboard();

        // move piece off of the destination
        self.color[this_turn_idx] &= !dest_bb;
        self.pieces[this_piece.idx()] &= !dest_bb;
        self.mailbox[um.destination.idx()] = Piece::Empty;
        self.zobrist_spc(um.destination, this_piece, this_turn);
        self.pst_remove_piece(this_piece, this_turn, um.destination);

        // restore any piece that was captured on the destination square
        if um.captured_piece != Piece::Empty {
            self.color[enemy_turn_idx] |= dest_bb;
            self.pieces[um.captured_piece.idx()] |= dest_bb;
            self.mailbox[um.destination.idx()] = um.captured_piece;
            self.zobrist_spc(um.destination, um.captured_piece, enemy_turn);
            self.pst_add_piece(um.captured_piece, enemy_turn, um.destination);
        }

        match um.flag {
            chess_move::MOVE_FLAG_PROMO => {
                // Replace the piece with pawn
                self.color[this_turn_idx] |= orig_bb;
                self.pieces[Piece::Pawn.idx()] |= orig_bb;
                self.mailbox[um.origin.idx()] = Piece::Pawn;
                self.zobrist_spc(um.origin, Piece::Pawn, this_turn);
                self.pst_add_piece(Piece::Pawn, this_turn, um.origin);
            },
            chess_move::MOVE_FLAG_CASTLE => {
                // Move the king back to the origin
                self.color[this_turn_idx] |= orig_bb;
                self.pieces[this_piece.idx()] |= orig_bb;
                self.mailbox[um.origin.idx()] = this_piece;
                self.zobrist_spc(um.origin, this_piece, this_turn);
                self.pst_add_piece(this_piece, this_turn, um.origin);

                // Move the rook to the far side of the king
                let (rook_from, rook_to) = match um.destination {
                    square::C1 => (square::A1, square::D1), // White queenside
                    square::G1 => (square::H1, square::F1), // White kingside
                    square::C8 => (square::A8, square::D8), // Black queenside
                    square::G8 => (square::H8, square::F8), // Black kingside
                    _ => panic!("Unmoving castling but to an impossible square?"),
                };

                // Return the rook to the "rook from"
                self.color[this_turn_idx] |= rook_from.to_bitboard();
                self.pieces[Piece::Rook.idx()] |= rook_from.to_bitboard();
                self.mailbox[rook_from.idx()] = Piece::Rook;
                self.zobrist_spc(rook_from, Piece::Rook, this_turn);
                self.pst_add_piece(Piece::Rook, this_turn, rook_from);

                // Remove the rook to the "rook to"
                self.color[this_turn_idx] &= !rook_to.to_bitboard();
                self.pieces[Piece::Rook.idx()] &= !rook_to.to_bitboard();
                self.mailbox[rook_to.idx()] = Piece::Empty;
                self.zobrist_spc(rook_to, Piece::Rook, this_turn);
                self.pst_remove_piece(Piece::Rook, this_turn, rook_to);
            },
            chess_move::MOVE_FLAG_ENPASSANT => {
                // Move piece back to the origin
                self.color[this_turn_idx] |= orig_bb;
                self.pieces[this_piece.idx()] |= orig_bb;
                self.mailbox[um.origin.idx()] = Piece::Pawn;
                self.zobrist_spc(um.origin, Piece::Pawn, this_turn);
                self.pst_add_piece(Piece::Pawn, this_turn, um.origin);

                // Put the captured pawn back
                let captured_sq = Square::from_row_col(um.origin.to_row(), um.destination.to_col());
                let captured_sq_bb = captured_sq.to_bitboard();
                self.color[enemy_turn_idx] |= captured_sq_bb;
                self.pieces[Piece::Pawn.idx()] |= captured_sq_bb;
                self.mailbox[captured_sq.idx()] = Piece::Pawn;
                self.zobrist_spc(captured_sq, Piece::Pawn, enemy_turn);
                self.pst_add_piece(Piece::Pawn, enemy_turn, captured_sq);
            },
            _ => {
                // Move piece back to the origin
                self.color[this_turn_idx] |= orig_bb;
                self.pieces[this_piece.idx()] |= orig_bb;
                self.mailbox[um.origin.idx()] = this_piece;
                self.zobrist_spc(um.origin, this_piece, this_turn);
                self.pst_add_piece(this_piece, this_turn, um.origin);
            },
        }
    }

    // A null move hands the turn to the opponent without moving a piece (used by null move
    // pruning). Like make_move it keeps the zobrist hash in sync and returns the state needed to
    // undo; only en_passant and half_moves change, so the movement fields of UnMove are unused.
    pub fn make_null_move(&mut self) -> UnMove {
        let um = UnMove {
            en_passant: self.en_passant,
            captured_piece: Piece::Empty,
            origin: Square(0),
            destination: Square(0),
            flag: chess_move::MOVE_FLAG_NONE,
            castling_rights: self.castling_rights,
            half_moves: self.half_moves,
        };

        // Passing the turn clears any en passant target, removing its stale file from the hash.
        if let Some(ep) = self.en_passant {
            self.zobrist_enpassant(ep);
        }
        self.en_passant = None;

        // A null move is neither a pawn move nor a capture.
        self.half_moves += 1;

        // Hand the turn to the opponent, toggling the side-to-move contribution.
        self.turn = self.turn.flip();
        self.zobrist_turn();

        um
    }

    pub fn undo_null_move(&mut self, um: UnMove) {
        // Restore the side-to-move contribution to match the flipped-back turn.
        self.turn = self.turn.flip();
        self.zobrist_turn();

        // Swap the post-move en passant file out for the restored one.
        if let Some(ep) = self.en_passant {
            self.zobrist_enpassant(ep);
        }
        self.en_passant = um.en_passant;
        if let Some(ep) = self.en_passant {
            self.zobrist_enpassant(ep);
        }

        // reset half move counter
        self.half_moves = um.half_moves;
    }

    pub fn can_kill_king(&mut self) -> bool {
        // Called after make_move, so flip turn to see if that player's king is in check
        // Probably rename the function, bit confusing
        self.turn = self.turn.flip();
        let king_sq = self.get_friendly_piece(Piece::King).lsb_as_square();
        let can_kill = self.is_square_underattack(king_sq);
        self.turn = self.turn.flip();

        can_kill
    }

    // Repetition detection when searching for best move/eval
    pub fn is_repetition(&self, history: &Vec<ZobristHash>) -> bool {
        for h in history.iter().rev().skip(1).step_by(2) {
            if *h == self.zobrist {
                return true;
            }
        }
        false
    }

    // Draw guard against three fold rep (used in perft tests)
    pub fn is_three_fold(&self, history: &Vec<ZobristHash>) -> bool {
        let mut count = 0;
        // Most likely to find matches in a reverse history, and only every other (same turn)
        for h in history.iter().rev().skip(1).step_by(2) {
            if *h == self.zobrist {
                count += 1;
                if count == 3 {
                    return true;
                }
            }
        }

        false
    }

    pub fn is_fifty_move_rule(&self) -> bool {
        self.half_moves > 99
    }

    pub fn print<W: Write>(&self, w: &mut W) -> io::Result<()> {
        for row in (0..8).rev() {
            write!(w, "{} ", row + 1)?;
            for col in 0..8 {
                let sq = Square::from_row_col(row, col);
                let piece = self.mailbox[sq.idx()];
                let color = if self.color[Color::White.idx()] & sq.to_bitboard() != BitBoard(0) {
                    Color::White
                } else {
                    Color::Black
                };
                write!(w, "{} ", piece.to_char(color))?;
            }
            writeln!(w)?;
        }
        writeln!(w, "  a b c d e f g h")?;
        writeln!(
            w,
            "turn: {:?}, castling: {:04b}, en passant: {:?}, half moves: {}",
            self.turn, self.castling_rights, self.en_passant, self.half_moves
        )
    }

    pub fn iter(&self) -> PositionIter<'_> {
        PositionIter { 
            position: self, 
            square: Square(0) 
        }
    }

}

// Castling rights guarded by a given square. A king/rook home square maps to the rights that
// depend on that piece staying put; any move that vacates or captures onto it must clear them.
fn castling_rights_voided_by(sq: Square) -> u8 {
    match sq {
        square::E1 => position::CASTLE_WK | position::CASTLE_WQ,
        square::A1 => position::CASTLE_WQ,
        square::H1 => position::CASTLE_WK,
        square::E8 => position::CASTLE_BK | position::CASTLE_BQ,
        square::A8 => position::CASTLE_BQ,
        square::H8 => position::CASTLE_BK,
        _ => 0,
    }
}

pub struct PositionIter<'a> {
    position: &'a Position,
    square: Square, // 0..64
}

impl<'a> Iterator for PositionIter<'a> {
    type Item = (Square, Option<Piece>, Option<Color>);

    fn next(&mut self) -> Option<Self::Item> {
        while self.square < Square(64) {
            let sq = self.square;
            let sq_bb = self.square.to_bitboard();
            self.square += 1;

            let color = if self.position.color[Color::White.idx()] & sq_bb != BitBoard(0) {
                Color::White
            } else if self.position.color[Color::Black.idx()] & sq_bb != BitBoard(0) {
                Color::Black
            } else {
                return Some((sq, None, None));
            };

            for p in [Piece::Pawn, Piece::Bishop, Piece::Knight, Piece::Rook, Piece::Queen, Piece::King] {
                if self.position.pieces[p.idx()] & sq_bb != BitBoard(0) {
                    return Some((sq, Some(p), Some(color)));
                }
            }
        }

        None
    }
}
