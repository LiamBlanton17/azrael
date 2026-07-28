use crate::search::magics::bishop::get_bishop_moves;
use crate::search::magics::rook::get_rook_moves;
use crate::search::move_generation::{KING_MOVES, KNIGHT_MOVES};
use crate::types::bidboard::BitBoard;
use crate::types::chess_move::{MOVE_FLAG_ENPASSANT, MOVE_FLAG_PROMO, Move, split_move};
use crate::types::color::Color;
use crate::types::eval::Eval;
use crate::types::piece::Piece;
use crate::types::position::Position;
use crate::types::square::Square;

// File masks .
const NOT_A_FILE: BitBoard = BitBoard(0xFEFEFEFEFEFEFEFE);
const NOT_H_FILE: BitBoard = BitBoard(0x7F7F7F7F7F7F7F7F);

// The king can never actually be captured, so its value is so large a capture would be a win
const KING_VALUE: i32 = 10_000;

// Attackers are spent cheapest-first
const LVA_ORDER: [Piece; 6] = [
    Piece::Pawn,
    Piece::Knight,
    Piece::Bishop,
    Piece::Rook,
    Piece::Queen,
    Piece::King,
];

impl Position {
    // Exact static exchange evaluation of m in centipawns
    // https://www.chessprogramming.org/Static_Exchange_Evaluation
    pub fn see(&self, m: Move) -> Eval {
        let (to, from, promo, flag) = split_move(m);

        let mut occ = self.get_all_pieces();
        let mover = self.mailbox[from.idx()];
        let side_to_move = self.turn;

        // gain[d] holds the running material balance if the exchange were to stop after the capture at ply d.
        // next_piece is the piece that ends up standing on `to`
        let mut gain = [0; 32];
        let mut next_piece;

        // must handle en passant/promo up front on the inital move, future moves won't need too
        if flag == MOVE_FLAG_ENPASSANT {
            // The captured pawn sits behind the destination square, not on it.
            // Remove it up front so any x-ray behind it is revealed correctly
            let captured_sq = if side_to_move == Color::White { to - 8 } else { to + 8 };
            occ &= !captured_sq.to_bitboard();
            gain[0] = see_piece_value(Piece::Pawn);
            next_piece = mover;
        } else if flag == MOVE_FLAG_PROMO {
            gain[0] = see_piece_value(self.mailbox[to.idx()]) + see_piece_value(promo) - see_piece_value(Piece::Pawn);
            next_piece = promo;
        } else {
            gain[0] = see_piece_value(self.mailbox[to.idx()]);
            next_piece = mover;
        }

        // The moving piece vacates its origin, which may uncover an x-ray attacker
        occ &= !from.to_bitboard();

        let mut attackers = self.see_attackers_to(to, occ);
        let mut side = side_to_move.flip();
        let mut d = 0;

        loop {
            d += 1;
            gain[d] = see_piece_value(next_piece) - gain[d - 1];

            // Who recaptures next - cheapest attacker of the side now to move
            let (lva_bb, lva_piece) = self.see_least_valuable_attacker(attackers, side);
            if lva_bb == BitBoard(0) || d >= gain.len() - 1 {
                break;
            }

            // That piece captures on to - pull it off the board and rebuild the attackers
            occ &= !lva_bb;
            attackers = self.see_attackers_to(to, occ);
            next_piece = lva_piece;
            side = !side;
        }

        // Negamax the swap list back down to the root
        // The side to move will only continue down the list as long as it gains from making the capture
        for i in (1..d).rev() {
            gain[i - 1] = -std::cmp::max(-gain[i - 1], gain[i]);
        }

        gain[0].clamp(i16::MIN as i32 + 1, i16::MAX as i32) as Eval
    }

    // Returns true if the SEE matches or exceeds the given threshold
    pub fn see_ge(&self, m: Move, threshold: Eval) -> bool {
        self.see(m) >= threshold
    }

    // All pieces of either colour that attack `sq` under occupancy `occ`.
    fn see_attackers_to(&self, sq: Square, occ: BitBoard) -> BitBoard {
        let sq_bb = sq.to_bitboard();
        let white_pawns = self.get_piece(Piece::Pawn, Color::White);
        let black_pawns = self.get_piece(Piece::Pawn, Color::Black);

        // Squares a pawn of each colour would have to stand on to attack `sq`
        let white_pawn_attackers =
            (((sq_bb & NOT_A_FILE) >> 9u32) | ((sq_bb & NOT_H_FILE) >> 7u32)) & white_pawns;
        let black_pawn_attackers =
            (((sq_bb & NOT_A_FILE) << 7u32) | ((sq_bb & NOT_H_FILE) << 9u32)) & black_pawns;

        let mut attackers = white_pawn_attackers | black_pawn_attackers;
        attackers |= KNIGHT_MOVES[sq.idx()] & self.pieces[Piece::Knight.idx()];
        attackers |= KING_MOVES[sq.idx()] & self.pieces[Piece::King.idx()];

        let bishops_queens = self.pieces[Piece::Bishop.idx()] | self.pieces[Piece::Queen.idx()];
        attackers |= get_bishop_moves(sq, occ) & bishops_queens;

        let rooks_queens = self.pieces[Piece::Rook.idx()] | self.pieces[Piece::Queen.idx()];
        attackers |= get_rook_moves(sq, occ) & rooks_queens;

        attackers & occ
    }

    // The cheapest attacker belonging to side within attackers
    // Returns (0, Empty) if side has no attacker left.
    fn see_least_valuable_attacker(&self, attackers: BitBoard, side: Color) -> (BitBoard, Piece) {
        let side_attackers = attackers & self.color[side.idx()];
        for &piece in LVA_ORDER.iter() {
            let subset = side_attackers & self.pieces[piece.idx()];
            if subset != BitBoard(0) {
                let lsb = BitBoard(subset.0 & subset.0.wrapping_neg());
                return (lsb, piece);
            }
        }
        (BitBoard(0), Piece::Empty)
    }
}

// Material values used inside the swap-off -- king just cannot be swapped for it is so large
fn see_piece_value(p: Piece) -> i32 {
    match p {
        Piece::King => KING_VALUE,
        Piece::Empty => 0,
        other => other.to_value() as i32,
    }
}

// Large AI written test set for SEE
#[cfg(test)]
mod tests {
    use crate::eval::{KNIGHT, PAWN, ROOK};
    use crate::types::position::Position;
    use std::sync::Once;

    static INIT: Once = Once::new();
    fn ensure_init() {
        INIT.call_once(|| crate::search::init_engine());
    }

    // Helper: build a position, resolve a long-algebraic move, and return its SEE.
    fn see_of(fen: &str, mv: &str) -> i16 {
        ensure_init();
        let mut p = Position::from_fen(fen).expect("valid fen");
        let m = p.move_from_la(mv).expect("legal move");
        p.see(m)
    }

    // Like `see_of`, but first plays `setup` (used to reach en-passant positions,
    // since the FEN parser can't accept a bare ep square). `setup` should be the
    // double pawn push that creates the ep target.
    fn see_after(fen: &str, setup: &str, mv: &str) -> i16 {
        ensure_init();
        let mut p = Position::from_fen(fen).expect("valid fen");
        let sm = p.move_from_la(setup).expect("legal setup move");
        p.make_move(sm);
        let m = p.move_from_la(mv).expect("legal move");
        p.see(m)
    }

    #[test]
    fn undefended_capture_wins_the_victim() {
        // Black pawn on d5 is defended by nothing; exd5 simply wins a pawn.
        assert_eq!(see_of("4k3/8/8/3p4/4P3/8/8/4K3 w - - 0 1", "e4d5"), PAWN);
    }

    #[test]
    fn equal_pawn_trade_is_zero() {
        // d5 is defended by the e6 pawn: pawn for pawn, net zero.
        assert_eq!(see_of("4k3/8/4p3/3p4/4P3/8/8/4K3 w - - 0 1", "e4d5"), 0);
    }

    #[test]
    fn rook_takes_pawn_defended_by_pawn_loses_the_exchange() {
        // Rxd5 grabs a pawn but the e6 pawn recaptures the rook: +100 - 515.
        assert_eq!(
            see_of("4k3/8/4p3/3p4/8/8/8/3RK3 w - - 0 1", "d1d5"),
            PAWN - ROOK
        );
    }

    #[test]
    fn recapture_battery_resolves_optimally() {
        // White: pawn e4, knight c3, both hitting d5. Black: pawn d5 (defended by
        // c6 pawn) and knight f6 also guarding d5.
        // White wins the d5 pawn (exd5) but if the exchange continues it goes
        // ...cxd5, Nxd5, Nxd5 and White ends down material — so optimal play stops
        // after Black equalises: the swap-off nets exactly zero.
        assert_eq!(
            see_of("4k3/8/2p2n2/3p4/4P3/2N5/8/4K3 w - - 0 1", "e4d5"),
            0
        );
    }

    #[test]
    fn quiet_move_into_attack_is_negative() {
        // Moving the knight to d5 where only a black pawn (c6) guards it: the
        // square is defended, so the swap-off value of landing there is -knight.
        let v = see_of("4k3/8/2p5/8/8/2N5/8/4K3 w - - 0 1", "c3d5");
        assert_eq!(v, -KNIGHT);
    }

    #[test]
    fn en_passant_undefended_wins_a_pawn() {
        // Play ...d7-d5 to open the ep target on d6, then White's e5 pawn takes en
        // passant. Nothing recaptures, so it nets a pawn.
        assert_eq!(
            see_after("4k3/3p4/8/4P3/8/8/8/4K3 b - - 0 1", "d7d5", "e5d6"),
            PAWN
        );
    }

    #[test]
    fn en_passant_defended_is_even() {
        // Same, but a black pawn on e7 recaptures on d6: pawn for pawn.
        assert_eq!(
            see_after("4k3/3pp3/8/4P3/8/8/8/4K3 b - - 0 1", "d7d5", "e5d6"),
            0
        );
    }

    #[test]
    fn xray_attacker_is_counted() {
        // White rooks doubled on the d-file (d1, d2) both bear on d5 through each
        // other; black pawn d5 defended by the c6 pawn.
        // Rxd5 +P, cxd5 -R, R(d1)xd5 +P  ->  the x-ray rook must be seen or the
        // result would be wrong. Net: +P -R +P.
        let v = see_of("4k3/8/2p5/3p4/8/8/3R4/3RK3 w - - 0 1", "d2d5");
        assert_eq!(v, PAWN - ROOK + PAWN);
    }

    #[test]
    fn threshold_helper_matches_value() {
        let fen = "4k3/8/4p3/3p4/8/8/8/3RK3 w - - 0 1";
        ensure_init();
        let mut p = Position::from_fen(fen).unwrap();
        let m = p.move_from_la("d1d5").unwrap();
        let v = p.see(m);
        assert!(p.see_ge(m, v));
        assert!(!p.see_ge(m, v + 1));
    }
}
