use crate::{search::move_generation::{ensure_move_stack_len, MoveGenLevel}, types::{chess_move::Move, eval::Eval, position::{Position, ZobristHash}}};

// quiescence search is basically just negamax but only looking at captures
// also includes a standpat, which prevents bad captures from being forced
pub fn quiescence(
    p: &mut Position,
    ply: i16,
    mut alpha: Eval,
    beta: Eval,
    move_stack: &mut Vec<Vec<Move>>,
    history: &mut Vec<ZobristHash>,
) -> (Eval, Move, u64) {

    // stand-pat: assume the side to move can "do nothing" and take the current eval
    // this prevents forcing a bad capture when just standing still is better
    let stand_pat = p.eval_relative();
    if stand_pat >= beta {
        return (stand_pat, 0, 1);
    }
    if stand_pat > alpha {
        alpha = stand_pat;
    }

    // only generate and check captures
    let move_stack_idx = ply as usize;
    ensure_move_stack_len(move_stack, move_stack_idx);
    p.generate_moves(&mut move_stack[move_stack_idx], MoveGenLevel::Captures, true);
    let num_moves = move_stack[move_stack_idx].len();

    let mut nodes = 1;
    let mut best_eval = stand_pat;
    let mut best_move = 0;
    for i in 0..num_moves {
        let m = move_stack[move_stack_idx][i];

        let um = p.make_move(m);
        let is_legal_move = !p.can_kill_king();
        if is_legal_move {
            let (e, _, n) = quiescence(p, ply + 1, -beta, -alpha, move_stack, history);
            let e = -e;
            nodes += n;

            if e > best_eval {
                best_eval = e;
                best_move = m;
            }
            if best_eval > alpha {
                alpha = best_eval;
            }
        }
        p.unmake_move(um);

        if alpha >= beta {
            break;
        }
    }

    (best_eval, best_move, nodes)
}
