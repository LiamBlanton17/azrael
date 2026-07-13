use crate::search::move_generation::{ensure_move_stack_len, MoveGenLevel};
use crate::search::quiescence::quiescence;
use crate::types::chess_move::Move;
use crate::types::eval::{self, Eval, MATE};
use crate::types::position::{Position, ZobristHash};

pub fn negamax(
    p: &mut Position, 
    depth: usize, 
    ply: i16, 
    mut alpha: Eval, 
    beta: Eval, 
    move_stack: &mut Vec<Vec<Move>>, 
    history: &mut Vec<ZobristHash>
) -> (Eval, Move, u64) {
    if depth == 0 {
        return quiescence(p, ply, alpha, beta, move_stack, history)
    }

    let move_stack_idx = ply as usize;
    ensure_move_stack_len(move_stack, move_stack_idx);
    p.generate_moves(&mut move_stack[move_stack_idx], MoveGenLevel::All);
    let num_moves = move_stack[move_stack_idx].len();

    history.push(p.zobrist);

    // recursive negamax search
    let mut nodes = 1;
    let mut best_eval = eval::MIN_EVAL;
    let mut best_move = 0;
    let mut found_legal_move = false;
    for i in 0..num_moves {
        let m = move_stack[move_stack_idx][i];

        // play and make sure this is a legal move
        let um = p.make_move(m);
        let is_legal_move = !p.can_kill_king();
        if is_legal_move {
            found_legal_move = true;
            let (e, n) = if p.is_fifty_move_rule() || p.is_three_fold(history) {
                (0, 0)
            } else {
                let (e, _, n) = negamax(p, depth - 1, ply + 1, -beta, -alpha, move_stack, history);
                (-e, n)
            };
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

        // alpha-beta cutoff
        if alpha >= beta {
            break;
        }
    }
    history.pop();

    if !found_legal_move {
        p.turn = p.turn.flip();
        let in_check = p.can_kill_king();
        p.turn = p.turn.flip();
        if in_check {
            (-MATE + ply, 0, nodes)
        } else {
            (0, 0, nodes)
        }
    } else {
        (best_eval, best_move, nodes)
    }

}
