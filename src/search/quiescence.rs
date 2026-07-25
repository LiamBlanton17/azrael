use crate::eval::PAWN;
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

    // generate captures only, or all moves if king in check (evasion moves)
    ensure_move_stack_len(move_stack, ply);
    p.generate_moves(&mut move_stack[ply], if in_check {MoveGenLevel::All} else {MoveGenLevel::Captures}, true, 0, (0, 0), history_heuristic);
    let num_moves = move_stack[ply].len();

    let mut nodes = 1;
    let mut best_move = 0;
    let mut has_legal_move = false;

    for i in 0..num_moves {
        let m = move_stack[ply][i];

        // delta pruning
        // https://www.chessprogramming.org/Delta_Pruning
        if !in_check {
            const DELTA_MARGIN: Eval = PAWN * 2;
            let (dest, _, _, flag) = split_move(m);
            // never prune promotion captures
            if flag != MOVE_FLAG_PROMO {
                // captured piece is on destination, unless en passant then just a pawn
                let value_of_captured_piece = if flag == MOVE_FLAG_ENPASSANT {
                    PAWN
                } else {
                    p.mailbox[dest.idx()].to_value()
                };
                if stand_pat + value_of_captured_piece + DELTA_MARGIN < alpha {
                    continue;
                }
            }
        }

        let um = p.make_move(m);
        let is_legal_move = !p.can_kill_king();
        if is_legal_move {
            has_legal_move = true;

            let (e, _, n) = quiescence(p, ply + 1, -beta, -alpha, move_stack, history, history_heuristic);
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
        return (-MATE + ply as Eval, 0, nodes); 
    }

    (best_eval, best_move, nodes)
}
