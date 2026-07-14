use crate::types::chess_move::{Move, split_move};
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
const CAPTURE_BASE_SCORE: i16 = 1_000;
const QUEEN_PROMO_SCORE: i16 = 1_000;
pub const KILLER_SCORE: i16 = 500;
const TT_MOVE_SCORE: i16 = 10_000;
fn score_move_order(p: &Position, m: Move, tt_move: Move, killers: (Move, Move), history_heuristic: &[[i16; 64]; 64]) -> i16 {
    if m == tt_move {
        return TT_MOVE_SCORE;
    }

    let (dest, orig, promo, flag) = split_move(m);
    let attacker = p.mailbox[orig.idx()];
    let victim = p.mailbox[dest.idx()];

    let mut score = 0;

    if m == killers.0 || m == killers.1 {
        score += KILLER_SCORE;
    }

    if victim != Piece::Empty {
        score += (victim.to_value() - attacker.to_value()) + CAPTURE_BASE_SCORE;
    }

    // If promotion, filter queens high and the rest low
    if promo != Piece::Empty {
        if promo == Piece::Queen {
            score += QUEEN_PROMO_SCORE;
        } else {
            score -= QUEEN_PROMO_SCORE;
        }
    }

    score + history_heuristic[orig.idx()][dest.idx()]
}
