use crate::eval::PAWN;
use crate::search::tt::{Bound, TranspositionTable};
use crate::types::eval::{score_from_tt, score_to_tt};
use crate::{search::move_generation::{MoveGenLevel, ensure_move_stack_len}, types::{chess_move::{MOVE_FLAG_ENPASSANT, MOVE_FLAG_PROMO, Move, split_move}, eval::{Eval, MATE}, position::{Position, ZobristHash}}};

// quiescence search is basically just negamax but only looking at captures
// also includes a standpat, which prevents bad captures from being forced
pub fn quiescence(
    p: &mut Position,
    ply: usize,
    mut alpha: Eval,
    beta: Eval,
    move_stack: &mut Vec<Vec<Move>>,
    history: &mut Vec<ZobristHash>,
    history_heuristic: &[[[i16; 64]; 64]; 2],
    tt: &mut TranspositionTable,
) -> (Eval, Move, u64) {

    // check if king is in check
    p.turn = p.turn.flip();
    let in_check = p.can_kill_king();
    p.turn = p.turn.flip();

    // do stand pat if king is not in check
    let mut best_eval: Eval;
    // static eval used as the base for delta pruning below; only meaningful when not in check
    let mut stand_pat = -MATE;
    if !in_check {
        stand_pat = p.eval_relative_lazy(alpha, beta);
        if stand_pat >= beta {
            return (stand_pat, 0, 1);
        }
        if stand_pat > alpha {
            alpha = stand_pat;
        }
        best_eval = stand_pat;
    } else {
        best_eval = -MATE;
    }

    // Probe the transposition table
    let mut tt_move = 0;
    if let Some(e) = tt.probe(p.zobrist) {
        tt_move = e.best_move;
        let tt_score = score_from_tt(e.score, ply);
        let cutoff = match e.bound {
            Bound::Exact => true,
            Bound::Lower => tt_score >= beta,
            Bound::Upper => tt_score <= alpha,
        };
        if cutoff {
            return (tt_score, tt_move, 1);
        }
    }
    let alpha_orig = alpha;

    // generate captures only, or all moves if king in check (evasion moves)
    ensure_move_stack_len(move_stack, ply);
    p.generate_moves(&mut move_stack[ply], if in_check {MoveGenLevel::All} else {MoveGenLevel::Captures}, true, tt_move, (0, 0), history_heuristic);
    let num_moves = move_stack[ply].len();

    let mut nodes = 1;
    let mut best_move = 0;
    let mut has_legal_move = false;

    for i in 0..num_moves {
        let m = move_stack[ply][i];

        // When not in check, filter losing/hopeless captures before searching them.
        // Promotions are always kept: they usually swing material by far more than
        // the immediate exchange, so neither prune applies to them.
        if !in_check {
            let (dest, orig, _, flag) = split_move(m);
            if flag != MOVE_FLAG_PROMO {
                let value_of_captured_piece = if flag == MOVE_FLAG_ENPASSANT {
                    PAWN
                } else {
                    p.mailbox[dest.idx()].to_value()
                };

                // Delta pruning - https://www.chessprogramming.org/Delta_Pruning
                // If even winning the piece on `dest` can't climb near alpha, skip it.
                const DELTA_MARGIN: Eval = PAWN * 2;
                if stand_pat + value_of_captured_piece + DELTA_MARGIN < alpha {
                    continue;
                }

                // SEE pruning: skip captures that lose material on the exchange
                // square (e.g. grabbing a defended pawn with a rook). Only captures
                // of a less valuable piece can lose material, so the SEE probe is
                // skipped entirely for equal-or-up captures.
                let attacker_value = p.mailbox[orig.idx()].to_value();
                if value_of_captured_piece < attacker_value && !p.see_ge(m, 0) {
                    continue;
                }
            }
        }

        let um = p.make_move(m);
        let is_legal_move = !p.can_kill_king();
        if is_legal_move {
            has_legal_move = true;

            let (e, _, n) = quiescence(p, ply + 1, -beta, -alpha, move_stack, history, history_heuristic, tt);
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

    // In check with no legal moves = checkmate
    if in_check && !has_legal_move {
        let mate = -MATE + ply as Eval;
        tt.store(p.zobrist, 0, score_to_tt(mate, ply), 0, Bound::Exact);
        return (mate, 0, nodes);
    }

    // Store the result in the TT - Quiescence is a depth-0 node, so entries don't remove Negamax entries
    let bound = if best_eval <= alpha_orig {
        Bound::Upper
    } else if best_eval >= beta {
        Bound::Lower
    } else {
        Bound::Exact
    };
    tt.store(p.zobrist, best_move, score_to_tt(best_eval, ply), 0, bound);

    (best_eval, best_move, nodes)
}
