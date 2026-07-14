use std::cmp::{max, min};

use crate::search::move_generation::{ensure_move_stack_len, MoveGenLevel};
use crate::search::move_ordering::KILLER_SCORE;
use crate::search::quiescence::quiescence;
use crate::search::tt::{Bound, TranspositionTable};
use crate::types::chess_move::{Move, split_move};
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
    history_heuristic: &mut [[i16; 64]; 64],
    tt: &mut TranspositionTable,
) -> (Eval, Move, u64, u64) {
    if depth == 0 {
        let (e, m, q_nodes) = quiescence(p, ply, alpha, beta, move_stack, history, history_heuristic);
        return (e, m, 0, q_nodes);
    }

    // Probe the transposition table 
    let alpha_orig = alpha;
    let mut tt_move = 0;
    if let Some(e) = tt.probe(p.zobrist) {
        tt_move = e.best_move;
        if ply > 0 && e.depth as usize >= depth {
            let cutoff = match e.bound {
                Bound::Exact => true,
                Bound::Lower => e.score >= beta,
                Bound::Upper => e.score <= alpha,
            };
            if cutoff {
                return (e.score, tt_move, 1, 0);
            }
        }
    }

    ensure_move_stack_len(move_stack, ply);
    p.generate_moves(&mut move_stack[ply], MoveGenLevel::All, true, tt_move, killers[ply], history_heuristic);
    let num_moves = move_stack[ply].len();

    history.push(p.zobrist);

    // recursive negamax search
    let mut q_nodes = 0;
    let mut nodes = 1;
    let mut best_eval = eval::MIN_EVAL;
    let mut best_move = 0;
    let mut found_legal_move = false;
    for i in 0..num_moves {
        let m = move_stack[ply][i];

        // play and make sure this is a legal move
        let um = p.make_move(m);
        let is_legal_move = !p.can_kill_king();
        if is_legal_move {
            found_legal_move = true;
            let (e, n, qn) = if p.is_fifty_move_rule() || p.is_three_fold(history) {
                (0, 0, 0)
            } else {
                let (e, _, n, qn) = negamax(p, depth - 1, ply + 1, -beta, -alpha, move_stack, history, killers, history_heuristic, tt);
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

            // If is not a capture, add to history_heuristic
            // If not a killer, add to killers
            if !p.is_move_capture(m) {
                let (dest, origin, _, _) = split_move(m);
                history_heuristic[origin.idx()][dest.idx()] = min(
                    history_heuristic[origin.idx()][dest.idx()] + (depth * depth) as i16,
                    KILLER_SCORE
                );
                if m != killers[ply].0 {
                    killers[ply].1 = killers[ply].0;
                    killers[ply].0 = m;
                }
            }
            break;
        } else {
            // decrement history if it didn't cause a cutoff
            if !p.is_move_capture(m) {
                let (dest, origin, _, _) = split_move(m);
                history_heuristic[origin.idx()][dest.idx()] = max(
                    history_heuristic[origin.idx()][dest.idx()] - (depth * depth) as i16,
                    0
                );
            }
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
        // Store the result in the TT as well as return it
        let bound = if best_eval <= alpha_orig {
            Bound::Upper
        } else if best_eval >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };
        tt.store(p.zobrist, best_move, best_eval, depth as u8, bound);
        (best_eval, best_move, nodes, q_nodes)
    }

}
