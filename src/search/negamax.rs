use std::cmp::{max, min};

use crate::search::move_generation::{ensure_move_stack_len, MoveGenLevel};
use crate::search::move_ordering::KILLER_SCORE;
use crate::search::quiescence::quiescence;
use crate::search::tt::{Bound, TranspositionTable};
use crate::types::chess_move::{Move, split_move};
use crate::types::color::Color;
use crate::types::eval::{self, Eval, MATE, score_from_tt, score_to_tt};
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
    history_heuristic: &mut [[[i16; 64]; 64]; 2],
    tt: &mut TranspositionTable,
    is_nmp_search: bool,
) -> (Eval, Move, u64, u64) {

    // Draw detection at node entry
    if ply > 0 && (p.is_fifty_move_rule() || p.is_repetition(history)) {
        return (0, 0, 1, 0);
    }

    // Determine if this node is a PV node
    // https://www.chessprogramming.org/Node_Types
    let is_pv = (beta as i32) - (alpha as i32) > 1;

    // Probe the transposition table
    let alpha_orig = alpha;
    let mut tt_move = 0;
    if let Some(e) = tt.probe(p.zobrist) {
        tt_move = e.best_move;
        // Only take a TT cutoff at non-PV nodes
        if ply > 0 && !is_pv && e.depth as usize >= depth {
            let tt_score = score_from_tt(e.score, ply);
            let cutoff = match e.bound {
                Bound::Exact => true,
                Bound::Lower => tt_score >= beta,
                Bound::Upper => tt_score <= alpha,
            };
            if cutoff {
                return (tt_score, tt_move, 1, 0);
            }
        }
    }

    // if at depth, run the quiescence search
    if depth == 0 {
        let (e, m, q_nodes) = quiescence(p, ply, alpha, beta, move_stack, history, history_heuristic);
        return (e, m, 0, q_nodes);
    }

    // Check if the king is in check in this position
    p.turn = p.turn.flip();
    let in_check = p.can_kill_king();
    p.turn = p.turn.flip();

    // Razoring - at low depths drop into a quiescence if decent margin from alpha
    let stand_pat;
    const RAZOR_MARGIN: Eval = 185;
    const RAZOR_DEPTH: usize = 2;
    let mut razor_q_nodes = 0;
    if depth <= RAZOR_DEPTH && !in_check {
        stand_pat = p.eval_relative();
        if stand_pat + RAZOR_MARGIN * (depth as Eval) < alpha {
            let (e, m, q_nodes) = quiescence(p, ply, alpha, beta, move_stack, history, history_heuristic);
            if e < alpha {
                return (e, m, 0, q_nodes);
            }
            razor_q_nodes = q_nodes;
        }
    }

    // Null Move Pruning
    // https://www.chessprogramming.org/Null_Move_Pruning
    if ply > 2 && depth >= 3 && !is_nmp_search {
        let count_material = (p.color[Color::White.idx()] | p.color[Color::Black.idx()]).0.count_ones(); // Must have at least 12 non-king pieces
        if !in_check && count_material > 14 {
            let r = 2 + (depth / 5); // null-move reduction
            let um = p.make_null_move();
            let (e, _, n, qn) = negamax(p, depth - 1 - r, ply + 1, -beta, -(beta - 1), move_stack, history, killers, history_heuristic, tt, true);
            p.undo_null_move(um);
            let e = -e;
            if e >= beta {
                return (beta, 0, n + 1, qn);
            }
        }
    }

    ensure_move_stack_len(move_stack, ply);
    if killers.len() <= ply {
        killers.resize(ply + 1, (0, 0));
    }
    p.generate_moves(&mut move_stack[ply], MoveGenLevel::All, true, tt_move, killers[ply], history_heuristic);
    let num_moves = move_stack[ply].len();

    history.push(p.zobrist);

    // recursive negamax search
    let mut q_nodes = razor_q_nodes;
    let mut nodes = 1;
    let mut best_eval = eval::MIN_EVAL;
    let mut best_move = 0;
    let mut found_legal_move = false;
    for i in 0..num_moves {
        let m = move_stack[ply][i];

        // play and make sure this is a legal move
        let is_capture_move = p.is_move_capture(m);
        let um = p.make_move(m);
        let is_legal_move = !p.can_kill_king();
        if is_legal_move {
            found_legal_move = true;
            // Apply LMR or depth extensions - https://www.chessprogramming.org/Late_Move_Reductions
            let relative_depth = get_relative_depth(depth, i, in_check);
            let (new_a, new_b) = if i > 0 { (-alpha-1, -alpha) } else { (-beta, -alpha) };
            let (e, _, n, qn1) = negamax(p, relative_depth, ply + 1, new_a, new_b, move_stack, history, killers, history_heuristic, tt, false);
            let (e, n, qn) = if i > 0 && -e > alpha && -e < beta {
                // Re-search at full depth/window
                let (e, _, n2, qn2) = negamax(p, depth - 1, ply + 1, -beta, -alpha, move_stack, history, killers, history_heuristic, tt, false);
                (-e, n + n2, qn1 + qn2)
            } else {
                (-e, n, qn1)
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

        // Only legal moves may drive cutoffs, killers and history
        if is_legal_move {
            // alpha-beta cutoff
            if alpha >= beta {

                // If is not a capture, add to history_heuristic (indexed by the side to move so the
                // two colors don't share one table and overwrite each other's ordering scores).
                // If not a killer, add to killers
                if !is_capture_move {
                    let (dest, origin, _, _) = split_move(m);
                    let c = p.turn.idx();
                    history_heuristic[c][origin.idx()][dest.idx()] = min(
                        history_heuristic[c][origin.idx()][dest.idx()] + (depth * depth) as i16,
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
                if !is_capture_move {
                    let (dest, origin, _, _) = split_move(m);
                    let c = p.turn.idx();
                    history_heuristic[c][origin.idx()][dest.idx()] = max(
                        history_heuristic[c][origin.idx()][dest.idx()] - (depth * depth) as i16,
                        0
                    );
                }
            }
        }
    }
    history.pop();

    if !found_legal_move {
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
        tt.store(p.zobrist, best_move, score_to_tt(best_eval, ply), depth as u8, bound);
        (best_eval, best_move, nodes, q_nodes)
    }

}

fn get_relative_depth(depth: usize, move_index: usize, in_check: bool) -> usize {
    // if in check and at a low depth, extend the search by 1 ply 
    if in_check && depth < 3 {
        return depth; 
    }

    // low move index or low depth - just search depth - 1 as normal
    if depth < 3 || move_index < 3 {
        return depth - 1;
    }

    // LMR
    // https://www.chessprogramming.org/Late_Move_Reductions
    // Obsidian reduces by 0.99 + ln(depth) * ln(moves) / 3.14
    let d = (depth as f64).ln();
    let i = (move_index as f64).ln();
    let r = 0.99 + d * i / 3.14;
    (r.round() as usize).min(depth - 1)
}
