use crate::search::move_generation::{ensure_move_stack_len, MoveGenLevel};
use crate::search::quiescence::quiescence;
use crate::types::chess_move::Move;
use crate::types::eval::{self, Eval, MATE};
use crate::types::position::{Position, ZobristHash};

// Return eval, move, negamax nodes, quiescence nodes
pub fn negamax(
    p: &mut Position, 
    depth: usize, 
    ply: usize, 
    mut alpha: Eval, 
    beta: Eval, 
    move_stack: &mut Vec<Vec<Move>>, 
    history: &mut Vec<ZobristHash>,
    killers: &mut Vec<(Move, Move)>,
) -> (Eval, Move, u64, u64) {
    if depth == 0 {
        let (e, m, q_nodes) = quiescence(p, ply, alpha, beta, move_stack, history);
        return (e, m, 0, q_nodes);
    }

    let move_stack_idx = ply as usize;
    ensure_move_stack_len(move_stack, move_stack_idx);
    p.generate_moves(&mut move_stack[move_stack_idx], MoveGenLevel::All, true, killers[ply]);
    let num_moves = move_stack[move_stack_idx].len();

    history.push(p.zobrist);

    // recursive negamax search
    let mut q_nodes = 0;
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
            let (e, n, qn) = if p.is_fifty_move_rule() || p.is_three_fold(history) {
                (0, 0, 0)
            } else {
                let (e, _, n, qn) = negamax(p, depth - 1, ply + 1, -beta, -alpha, move_stack, history, killers);
                (-e, n, qn)
            };
            nodes += n;
            q_nodes += qn;
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

            // Add to killers if not already in them and is not a capture
            if m != killers[ply].0 && !p.is_move_capture(m) {
                killers[ply].1 = killers[ply].0;
                killers[ply].0 = m;
            }
            break;
        }
    }
    history.pop();

    if !found_legal_move {
        p.turn = p.turn.flip();
        let in_check = p.can_kill_king();
        p.turn = p.turn.flip();
        if in_check {
            (-MATE + ply as i16, 0, nodes, q_nodes)
        } else {
            (0, 0, nodes, q_nodes)
        }
    } else {
        (best_eval, best_move, nodes, q_nodes)
    }

}
