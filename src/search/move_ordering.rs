use crate::search::move_generation::MAX_MOVES_IN_POSITION;
use crate::types::chess_move::{Move, split_move};
use crate::types::piece::Piece;
use crate::types::position::Position;

impl Position {

    pub fn order_moves(&self, move_stack: &mut Vec<Move>) {
        let len = move_stack.len();
        let mut scored: [(Move, u16); 256] = [(Move::default(), 0); 256];

        for i in 0..len {
            scored[i] = (move_stack[i], score_move_order(self, move_stack[i]));
        }

        scored[..len].sort_unstable_by_key(|&(_, score)| std::cmp::Reverse(score));

        for i in 0..len {
            move_stack[i] = scored[i].0;
        }
    }

}

// https://www.chessprogramming.org/Move_Ordering
const CAPTURE_BASE_SCORE: u16 = 1000;
fn score_move_order(p: &Position, m: Move) -> u16 {
    let (dest, orig, promo, flag) = split_move(m);
    let attacker = p.mailbox[orig.idx()];
    let victim = p.mailbox[dest.idx()];

    let mut score = 0;

    if victim != Piece::Empty {
        score += (victim.to_value() - attacker.to_value()) as u16 + CAPTURE_BASE_SCORE;
    }

    score
}
