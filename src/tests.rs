use std::{fs::{self, File}, io::{BufRead, BufReader}, time::{Duration, Instant}};

use crate::{search::{RootSearchType, init_engine, move_generation::MoveGenLevel, tt::TranspositionTable}, types::{chess_move::{Move, split_move}, position::{Position, ZobristHash}}};

// Used by the CLI to run a perf test -- this verifies the move generation is working correctly
// It also provides a best test of raw nodes per second the engine can do, without any other overhead than raw search
pub fn perf_test() {

    // must init the zobrist tables, the rook/bishop magic tables, and the other piece tables
    init_engine();

    // recursive search for counting visited notes
    fn count_nodes_visited(p: &mut Position, move_stack: &mut Vec<Vec<Move>>, history: &mut Vec<ZobristHash>, depth: usize) -> u64 {
        // if at leaf, count this node
        if depth == 0 {
            return 1;
        }

        // generate the new moves for this position
        // using a 2d stack for moves to avoid any allocation on hot path
        p.generate_moves(&mut move_stack[depth], MoveGenLevel::All, false, 0, (0, 0), &[[0; 64]; 64]);
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
    // https://www.chessprogramming.org/Perft_Results
    let tests = [
        ("Starting Position", 6, "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1", 119_060_324),
        ("Kiwipete", 5, "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1", 193_690_690),
        ("Rook Pawn Endgame", 7, "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1", 178_633_661),
        ("Chaos", 6, "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1", 706_045_033),
        ("Engine Killer", 5, "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8", 89_941_194),
        ("Standard Vibes", 5, "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10", 164_075_551),
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

// Bnech mark test structs
struct BenchmarkTestCandidate {
    mv: Move,
    points: u32,
}

struct BenchmarkTest {
    position: Position,
    id: String,
    candidates: Vec<BenchmarkTestCandidate>,
}

// Attempt to load a line from a file into a bennchmark test
fn load_test(line: &str) -> Option<BenchmarkTest> {
    let parts: Vec<&str> = line.split(';').collect();
    if parts.len() < 3 {
        return None;
    }

    // Rebuild a full FEN: keep board/turn/castling/en-passant, reset the move clocks to 0 0
    let fen_fields: Vec<&str> = parts[0].split_whitespace().take(4).collect();
    if fen_fields.len() < 4 {
        return None;
    }
    let fen = format!("{} 0 0", fen_fields.join(" "));
    let mut position = match Position::from_fen(&fen) {
        Ok(p) => p,
        Err(e) => {
            println!("Skipping test, could not parse FEN '{}': {}", fen, e);
            return None;
        }
    };

    // The `id` and candidate (`c0`) fields appear in either order, so must be handled together
    let mut id = String::new();
    let mut candidate_str = None;
    for field in &parts[1..] {
        let field = field.trim();
        let quoted = field.split('"').nth(1).unwrap_or("");
        if field.starts_with("id") {
            id = quoted.to_string();
        } else if field.starts_with("c0") {
            candidate_str = Some(quoted);
        }
    }
    let candidate_str = candidate_str?;

    // Each candidate is "<san>=<points>"; rsplit on '=' so promotions like "e8=Q=10" survive
    let mut candidates = Vec::new();
    for candidate in candidate_str.split(',') {
        let Some((move_san, points_str)) = candidate.rsplit_once('=') else {
            continue;
        };
        let Ok(points) = points_str.trim().parse::<u32>() else {
            continue;
        };

        let move_san = move_san.trim();
        match position.san_to_move(move_san) {
            Ok(mv) => candidates.push(BenchmarkTestCandidate { mv, points }),
            Err(e) => {
                position.print();
                panic!("Failed to resolve SAN '{}' in {}: {}", move_san, id, e);
            }
        }
    }

    Some(BenchmarkTest { position, id, candidates })
}

// Used by the CLI to run a battery of tests
// Bunch of different positions, for scores for the move determines for the position
// Better score equals better moves found, and also will display node counts and efficiency
// https://www.chessprogramming.org/Strategic_Test_Suite
pub fn strength_test() {
    // must init the zobrist tables, the rook/bishop magic tables, and the other piece tables
    init_engine();

    // read every .epd file in the test_files directory
    let entries = match fs::read_dir("test_files") {
        Ok(entries) => entries,
        Err(e) => {
            println!("Could not read the test_files directory: {}", e);
            return;
        }
    };

    // the time limits each position is searched at -- metrics are tracked separately per limit
    const TIME_LIMITS: [Duration; 5] = [
        Duration::from_millis(10),
        Duration::from_millis(50),
        Duration::from_millis(250),
        Duration::from_millis(1000),
        Duration::from_millis(5000),
    ];
    const NUM_LIMITS: usize = TIME_LIMITS.len();

    // one accumulator slot per time limit
    let mut total_score = [0u32; NUM_LIMITS];
    let mut total_nodes = [0u64; NUM_LIMITS];
    let mut best_moves_found = [0u32; NUM_LIMITS];
    let mut total_search_duration = [Duration::new(0, 0); NUM_LIMITS];
    let mut total_ebf = [0.0f64; NUM_LIMITS];
    let mut total_depth = [0usize; NUM_LIMITS];

    // these are per-position, shared across every time limit
    let mut max_score = 0u32;
    let mut tests_run = 0u32;

    for entry in entries {
        let path = match entry {
            Ok(entry) => entry.path(),
            Err(e) => {
                println!("{}", e);
                continue;
            }
        };

        // only test .epd files
        if path.extension().and_then(|ext| ext.to_str()) != Some("epd") {
            continue;
        }

        let file = match File::open(&path) {
            Ok(file) => file,
            Err(e) => {
                println!("Could not open {}: {}", path.display(), e);
                continue;
            }
        };
        println!("\n=== {} ===", path.display());

        let mut tt = TranspositionTable::new(128);

        for line in BufReader::new(file).lines() {
            let line = match line {
                Ok(line) => line,
                Err(e) => {
                    println!("{}", e);
                    break;
                }
            };
            if line.trim().is_empty() {
                continue;
            }

            let Some(mut test) = load_test(&line) else {
                println!("Failed to parse test!");
                continue;
            };

            // the best obtainable score for this position is the top-valued candidate
            let best_available = test.candidates.iter().map(|c| c.points).max().unwrap_or(0);
            if best_available == 0 {
                println!("Failed to parse test!");
                continue;
            }
            max_score += best_available;
            tests_run += 1;

            // search the position and score the move the engine chose
            for (i, &time_limit) in TIME_LIMITS.iter().enumerate() {
                let start = Instant::now();
                let (_eval, best_move, nodes, q_nodes, depth) = test.position.root_search(RootSearchType::TimeLimited(time_limit), &mut tt);
                total_search_duration[i] += start.elapsed();

                let scored = test
                    .candidates
                    .iter()
                    .find(|c| c.mv == best_move)
                    .map(|c| c.points)
                    .unwrap_or(0);

                total_score[i] += scored;
                total_nodes[i] += nodes + q_nodes;
                if scored == best_available {
                    best_moves_found[i] += 1;
                }

                let (dest, orig, _, _) = split_move(best_move);
                let ebf = ebf_estimate(nodes, depth);
                total_depth[i] += depth;
                total_ebf[i] += ebf;
                println!("{:<15} engine played {}{} -> {}/{} -- {} nodes at depth {} (EBF {:.2})", test.id, orig, dest, scored, best_available, nodes, depth, ebf);
            }
            println!();
        }
    }

    // print the results, broken out by each time limit
    println!("\nStrength Test Complete");
    for (i, time_limit) in TIME_LIMITS.iter().enumerate() {
        let mnps = ((total_nodes[i] as f64) / total_search_duration[i].as_secs_f64()) / 1_000_000.0;
        println!("\n--- Time Limit: {:?} ---", time_limit);
        println!("Score: {}/{} ({:.2})", total_score[i], max_score, total_score[i] as f64 / max_score as f64);
        println!("Best moves found: {}/{} ({:.2})", best_moves_found[i], tests_run, best_moves_found[i] as f64 / tests_run as f64);
        println!("Search time: {:?}", total_search_duration[i]);
        println!("Total nodes: {}", crate::format_with_commas(total_nodes[i]));
        println!("Avg. Depth: {:.2}", (total_depth[i] as f64 / tests_run as f64));
        println!("MN/S: {:.2}", mnps);
        println!("EBF: {:.2}", (total_ebf[i] / tests_run as f64));
    }
    println!();
}

fn ebf_estimate(nodes: u64, depth: usize) -> f64 {
    (nodes as f64).powf(1.0 / depth as f64)
}
