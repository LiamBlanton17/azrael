mod types;
mod eval;
mod interface;
mod state;
mod search;

use types::position::Position;
use types::chess_move::Move;
use search::move_generation::MoveGenLevel;

use crate::state::zobrist::init_zobrist;
use crate::types::position::ZobristHash;

use std::time::Instant;

fn main() {
    perf_test();
}

// Used by the CLI to run a perf test -- this verifies the move generation is working correctly
// It also provides a best test of raw nodes per second the engine can do, without any other overhead than raw search
pub fn perf_test() {

    // must init the zobrist tables
    unsafe { init_zobrist(); }

    // recursive search for counting visited notes
    fn count_nodes_visited(p: &mut Position, move_stack: &mut Vec<Vec<Move>>, history: &mut Vec<ZobristHash>, depth: usize) -> u64 {
        // if at leaf, count this node
        if depth == 0 {
            return 1;
        }

        // generate the new moves for this position
        // using a 2d stack for moves to avoid any allocation on hot path
        p.generate_moves(&mut move_stack[depth], MoveGenLevel::All);
        let num_moves = move_stack[depth].len();

        // add zobrist of the position to history (to avoid 3-folds)
        // maybe could use a map instead of a list, but list is never very long anyway
        history.push(p.zobrist);

        // recursive count of nodes
        let mut nodes = 0;
        for i in 0..num_moves {
            let m = move_stack[depth][i];
            let um = p.make_move(m);
            if !p.can_kill_king() && !p.is_fifty_move_rule() && !p.is_three_fold(history) {
                nodes += count_nodes_visited(p, move_stack, history, depth - 1);
            }
            p.unmake_move(um);
        }
        history.pop();

        nodes
    }

    // Define tuples for perf_test (name, depth, position, target)
    let tests = [
        ("Starting Position", 6, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 119_060_324),
        ("Kiwipete", 5, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 193_690_690),
        ("Rook Pawn Endgame", 7, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 178_633_661),
        ("Chaos", 6, "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 706_045_033),
        ("Engine Killer", 5, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 89_941_194),
        ("Standard Vibes", 6, "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 6_923_051_137),
    ];

    let mut passed = true;
    println!();  // formatting println
    for (name, depth, fen, target) in tests {
        let mut p = Position::from_fen(fen).expect("Starting position FEN should not fail to parse");
        let mut move_stack: Vec<Vec<Move>> = (0..=(depth + 1)).map(|_| Position::new_move_stack()).collect();
        let mut history: Vec<ZobristHash> = Vec::with_capacity(depth + 1);

        let start = Instant::now();
        let nodes = count_nodes_visited(&mut p, &mut move_stack, &mut history, depth);
        let duration = start.elapsed();
        let mnps = ((nodes as f64) / duration.as_secs_f64()) / 1_000_000.0;

        println!("Position: {}", name);
        if nodes != target {
            passed = false;
            println!("[FAILED] Got Nodes: {}; Expected {}", nodes, target);
        } else {
            println!("Nodes: {}", nodes);
        }
        println!("Time elapsed: {:?}", duration);
        println!("MN/S: {:.2}\n", mnps);
    }

    if passed {
        println!("\nPassed Perft Test.\n")
    } else {
        println!("\nFAILED PERFT TEST!\n")
    }

}

