use std::cmp::{max, min};

use crate::search::move_generation::{ensure_move_stack_len, MoveGenLevel};
use crate::search::move_ordering::KILLER_SCORE;
use crate::search::quiescence::quiescence;
use crate::search::tt::{Bound, TranspositionTable};
use crate::types::chess_move::{Move, split_move};
use crate::types::color::Color;
use crate::types::eval::{self, Eval, MATE, score_from_tt, score_to_tt};
use crate::types::position::{Position, ZobristHash};

// Negamax searcher config
pub struct NegamaxSearcher<'a> {
    pub move_stack: &'a mut Vec<Vec<Move>>,
    pub history: &'a mut Vec<ZobristHash>,
    pub killers: &'a mut Vec<(Move, Move)>,
    pub history_heuristic: &'a mut [[[i16; 64]; 64]; 2],
    pub tt: &'a mut TranspositionTable,
}

#[derive(Clone, Copy)]
pub struct NegamaxSearchParams {
    pub depth: usize,
    pub ply: usize,
    pub alpha: Eval,
    pub beta: Eval,
    pub is_nmp_search: bool,
}

pub struct NegamaxSearchResult {
    pub eval: Eval,
    pub best_move: Move,
    pub nodes: u64,
    pub q_nodes: u64,
    pub is_three_fold: bool,
}

impl NegamaxSearcher<'_> {

    // Return eval, move, negamax nodes, quiescence nodes
    pub fn search(&mut self, p: &mut Position, params: NegamaxSearchParams) -> NegamaxSearchResult {

        // aliases for all the params
        let ply = params.ply;
        let beta = params.beta;
        let mut alpha = params.alpha;
        let depth = params.depth;
        let is_nmp_search = params.is_nmp_search;

        // Draw detection at node entry
        if ply > 0 && (p.is_fifty_move_rule() || p.is_repetition(&self.history)) {
            return NegamaxSearchResult {
                eval: 0,
                best_move: 0,
                nodes: 0,
                q_nodes: 0,
                is_three_fold: true,
            };
        }

        // Determine if this node is a PV node
        // https://www.chessprogramming.org/Node_Types
        let is_pv = beta - alpha > 1;

        // Probe the transposition table
        let alpha_orig = alpha;
        let mut tt_move = 0;
        if let Some(e) = self.tt.probe(p.zobrist) {
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
                    return NegamaxSearchResult {
                        eval: tt_score,
                        best_move: tt_move,
                        nodes: 1,
                        q_nodes: 0,
                        is_three_fold: false,
                    };
                }
            }
        }

        // if at depth, run the quiescence search
        if depth == 0 {
            let (e, m, q_nodes) = quiescence(p, ply, alpha, beta, &mut self.move_stack, &mut self.history, &mut self.history_heuristic, &mut self.tt);
            return NegamaxSearchResult {
                eval: e,
                best_move: m,
                nodes: 0,
                q_nodes: q_nodes,
                is_three_fold: false,
            };
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
                let (e, m, q_nodes) = quiescence(p, ply, alpha, beta, &mut self.move_stack, &mut self.history, &mut self.history_heuristic, &mut self.tt);
                if e < alpha {
                    return NegamaxSearchResult {
                        eval: e,
                        best_move: m,
                        nodes: 0,
                        q_nodes: q_nodes,
                        is_three_fold: false,
                    };
                }
                razor_q_nodes = q_nodes;
            }
        }

        // Null Move Pruning
        // https://www.chessprogramming.org/Null_Move_Pruning
        if ply > 2 && depth >= 3 && !is_nmp_search {
            // todo: improve this to also exclude pawns (maybe actually just use the p.phase score used for tapered eval)
            let count_material = (p.color[Color::White.idx()] | p.color[Color::Black.idx()]).0.count_ones(); // Must have at least 12 non-king pieces
            if !in_check && count_material > 14 {
                let r = 2 + (depth / 5); // null-move reduction
                let um: crate::types::chess_move::UnMove = p.make_null_move();
                let nmp_result = self.search(p, NegamaxSearchParams { 
                    depth: depth - 1 - r, 
                    ply: ply + 1,
                    alpha: - beta, 
                    beta: -(beta - 1), 
                    is_nmp_search: true,
                });
                p.undo_null_move(um);
                if -nmp_result.eval >= beta {
                    return NegamaxSearchResult {
                        eval: beta,
                        best_move: 0,
                        nodes: nmp_result.nodes + 1,
                        q_nodes: nmp_result.q_nodes,
                        is_three_fold: false,
                    };
                }
            }
        }

        ensure_move_stack_len(&mut self.move_stack, ply);
        if self.killers.len() <= ply {
            self.killers.resize(ply + 1, (0, 0));
        }
        p.generate_moves(&mut self.move_stack[ply], MoveGenLevel::All, true, tt_move, self.killers[ply], &mut self.history_heuristic);
        let num_moves = self.move_stack[ply].len();

        self.history.push(p.zobrist);

        // recursive negamax search
        let mut q_nodes = razor_q_nodes;
        let mut nodes = 1;
        let mut best_eval = eval::MIN_EVAL;
        let mut best_move = 0;
        let mut found_legal_move = false;
        for i in 0..num_moves {
            let m = self.move_stack[ply][i];

            // play and make sure this is a legal move
            let is_capture_move = p.is_move_capture(m);
            let um = p.make_move(m);
            let is_legal_move = !p.can_kill_king();
            if is_legal_move {
                found_legal_move = true;
                // Apply LMR or depth extensions - https://www.chessprogramming.org/Late_Move_Reductions
                let relative_depth = get_relative_depth(depth, i, in_check);

                // if not root -- attempt a reduced window search and attempt to fail over alpha
                let mut child;
                if i > 0 {
                    child = self.search(p, NegamaxSearchParams { 
                        depth: relative_depth, 
                        ply: ply + 1, 
                        alpha: -alpha - 1, 
                        beta: -alpha,
                        is_nmp_search: false,
                    });
                    
                    // full search if reduced window search if it failed over alpha
                    if -child.eval > alpha && -child.eval < beta {
                        let (n, qn) = (child.nodes, child.q_nodes);
                        child = self.search(p, NegamaxSearchParams { 
                            depth: relative_depth, 
                            ply: ply + 1, 
                            alpha: -beta, 
                            beta: -alpha,
                            is_nmp_search: false,
                        });
                        child.nodes += n;
                        child.q_nodes += qn;
                    }
                } else {
                    // full search if reduced window search didn't happen
                    child = self.search(p, NegamaxSearchParams { 
                        depth: relative_depth, 
                        ply: ply + 1, 
                        alpha: -beta, 
                        beta: -alpha,
                        is_nmp_search: false,
                    });
                }

                // updates based on child results
                nodes += child.nodes;
                q_nodes += child.q_nodes;
                if -child.eval > best_eval {
                    best_eval = -child.eval;
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
                        self.history_heuristic[c][origin.idx()][dest.idx()] = min(
                            self.history_heuristic[c][origin.idx()][dest.idx()] + (depth * depth) as i16,
                            KILLER_SCORE
                        );
                        if m != self.killers[ply].0 {
                            self.killers[ply].1 = self.killers[ply].0;
                            self.killers[ply].0 = m;
                        }
                    }
                    break;
                } else {
                    // decrement history if it didn't cause a cutoff
                    if !is_capture_move {
                        let (dest, origin, _, _) = split_move(m);
                        let c = p.turn.idx();
                        self.history_heuristic[c][origin.idx()][dest.idx()] = max(
                            self.history_heuristic[c][origin.idx()][dest.idx()] - (depth * depth) as i16,
                            0
                        );
                    }
                }
            }
        }
        self.history.pop();

        if !found_legal_move {
            if in_check {
                NegamaxSearchResult { eval: -MATE + ply as i16, best_move: 0, nodes, q_nodes, is_three_fold: false }
            } else {
                NegamaxSearchResult { eval: 0, best_move: 0, nodes, q_nodes, is_three_fold: false }
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
            self.tt.store(p.zobrist, best_move, score_to_tt(best_eval, ply), depth as u8, bound);
            NegamaxSearchResult { eval: best_eval, best_move: best_move, nodes, q_nodes, is_three_fold: false }
        }

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
    let reduction = r.round() as usize;
    (depth - 1).saturating_sub(reduction).max(1)
}
