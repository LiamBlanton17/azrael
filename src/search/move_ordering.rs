use crate::eval::PAWN;
use crate::types::chess_move::{MOVE_FLAG_CASTLE, MOVE_FLAG_ENPASSANT, MOVE_FLAG_PROMO, Move, split_move};
use crate::types::piece::Piece;
use crate::types::position::Position;

impl Position {

    pub fn order_moves(&self, move_stack: &mut Vec<Move>, tt_move: Move, killers: (Move, Move), history_heuristic: &[[i16; 64]; 64]) {
        let len = move_stack.len();
        let mut scored: [(Move, i16); 256] = [(Move::default(), 0); 256];

        for i in 0..len {
            scored[i] = (move_stack[i], score_move_order(self, move_stack[i], tt_move, killers, history_heuristic));
        }

        scored[..len].sort_unstable_by_key(|&(_, score)| std::cmp::Reverse(score));

        for i in 0..len {
            move_stack[i] = scored[i].0;
        }
    }

}

// https://www.chessprogramming.org/Move_Ordering
//
// Ordering bands, high to low:
//   TT move                                          (TT_MOVE_SCORE)
//   winning / equal captures  (SEE >= 0), MVV-LVA    (~CAPTURE_BASE_SCORE)
//   killer quiets                                    (KILLER_SCORE)
//   other quiets, by history heuristic               (0 .. KILLER_SCORE)
//   losing captures           (SEE < 0),  MVV-LVA    (~LOSING_CAPTURE_SCORE)
// Promotions keep their own scoring so under-promotions stay out of the way.
const CAPTURE_BASE_SCORE: i16 = 26_000;   // winning/equal captures, just above the killers
const LOSING_CAPTURE_SCORE: i16 = -26_000; // losing captures, below every quiet
const QUEEN_PROMO_SCORE: i16 = 2_500;
pub const KILLER_SCORE: i16 = 25_000;
const CASTLE_SCORE: i16 = 500;
const TT_MOVE_SCORE: i16 = 30_000;
fn score_move_order(p: &Position, m: Move, tt_move: Move, killers: (Move, Move), history_heuristic: &[[i16; 64]; 64]) -> i16 {
    if m == tt_move {
        return TT_MOVE_SCORE;
    }

    let (dest, orig, promo, flag) = split_move(m);
    let attacker = p.mailbox[orig.idx()];
    let victim = p.mailbox[dest.idx()];

    // Promotions keep their own ordering (queen promos high, under-promos low); a
    // promotion that is also a capture additionally gets the capture bonus.
    if flag == MOVE_FLAG_PROMO {
        let mut score = 0;
        if victim != Piece::Empty {
            score += (victim.to_value() - attacker.to_value()) + CAPTURE_BASE_SCORE;
        }
        if promo == Piece::Queen {
            score += QUEEN_PROMO_SCORE;
        } else {
            score -= QUEEN_PROMO_SCORE;
        }
        return score;
    }

    // Non-promotion captures (including en passant): rank by MVV-LVA, but let SEE
    // decide the band — a capture that loses material on the exchange square drops
    // below the quiet moves instead of being tried early.
    let is_capture = victim != Piece::Empty || flag == MOVE_FLAG_ENPASSANT;
    if is_capture {
        // En passant's victim is not on `dest`, so value it as a pawn explicitly.
        let victim_value = if flag == MOVE_FLAG_ENPASSANT { PAWN } else { victim.to_value() };
        let mvv_lva = victim_value - attacker.to_value();
        // Capturing an equal-or-more-valuable piece can never lose material on the
        // exchange square, so only pay for a SEE probe when capturing down.
        let winning = mvv_lva >= 0 || p.see_ge(m, 0);
        return if winning {
            CAPTURE_BASE_SCORE + mvv_lva
        } else {
            LOSING_CAPTURE_SCORE + mvv_lva
        };
    }

    // Quiet moves: killers first, a small nudge for castling, then history.
    let mut score = 0;
    if m == killers.0 || m == killers.1 {
        score += KILLER_SCORE;
    }
    if flag == MOVE_FLAG_CASTLE {
        score += CASTLE_SCORE;
    }

    score + history_heuristic[orig.idx()][dest.idx()]
}
