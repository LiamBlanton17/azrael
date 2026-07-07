mod types;
mod eval;
mod interface;
mod state;
mod search;

use types::position::Position;
use types::chess_move::Move;
use search::move_generation::MoveGenLevel;

use crate::types::position::ZobristHash;

fn main() {
    perf_test();
}

const MOVE_STACK_DEPTH: usize = 20;

// Used by the CLI to run a perf test -- this verifies the move generation is working correctly
// It also provides a best test of raw nodes per second the engine can do, without overhead of move ordering or eval
pub fn perf_test() {

    fn count_nodes_visited(p: &mut Position, move_stack: &mut Vec<Vec<Move>>, history: &mut Vec<ZobristHash>, depth: usize) -> u64 {
        if depth == 0 {
            return 1;
        }

        move_stack[depth].clear();
        p.generate_moves(&mut move_stack[depth], MoveGenLevel::All);

        let moves: Vec<Move> = move_stack[depth].clone();
        let mut nodes = 0;
        for m in moves {
            let um = p.make_move(m);
            history.push(p.zobrist);
            if !p.can_kill_king() && !p.is_three_fold(history) && !p.is_fifty_move_rule() {
                nodes += count_nodes_visited(p, move_stack, history, depth - 1);
            }
            p.unmake_move(um);
        }
        history.pop();

        nodes
    }

    let depth = 5;
    let starting_position_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let mut p = Position::from_fen(starting_position_fen).expect("Starting position FEN should not fail to parse");
    let mut move_stack: Vec<Vec<Move>> = (0..MOVE_STACK_DEPTH).map(|_| Position::new_move_stack()).collect();
    let mut history: Vec<ZobristHash> = Vec::with_capacity(MOVE_STACK_DEPTH);
    let nodes = count_nodes_visited(&mut p, &mut move_stack, &mut history, depth);
    println!("Nodes: {}", nodes);
}

