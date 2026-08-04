use std::fs::{self, File};
use std::io::{self, BufRead, BufReader};
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::time::{Duration, Instant};

use crate::search::{RootSearchType, init_engine};
use crate::search::tt::TranspositionTable;
use crate::types::chess_move::{Move, split_move};
use crate::types::position::Position;

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
                let _ = position.print(&mut io::stdout());
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
pub fn test() {
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
    const DEPTH_LIMITS: [usize; 6] = [3, 5, 7, 9, 13, 15];
    const NUM_LIMITS: usize = DEPTH_LIMITS.len();

    // one accumulator slot per time limit
    let mut total_score = [0u32; NUM_LIMITS];
    let mut total_nodes = [0u64; NUM_LIMITS];
    let mut total_q_nodes = [0u64; NUM_LIMITS];
    let mut best_moves_found = [0u32; NUM_LIMITS];
    let mut total_search_duration = [Duration::new(0, 0); NUM_LIMITS];
    let mut total_abf = [0.0f64; NUM_LIMITS];
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

        let mut tt = TranspositionTable::new(256);

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
            for (i, &depth) in DEPTH_LIMITS.iter().enumerate() {
                tt.clear();  // make sure tt is cleared before each search, but don't include in search time


                let start = Instant::now();
                let (_eval, best_move, nodes, q_nodes, actual_depth) = test.position.root_search(RootSearchType::DepthLimited(depth), &[], &mut tt, Arc::new(AtomicBool::new(false)));
                total_search_duration[i] += start.elapsed();

                let scored = test
                    .candidates
                    .iter()
                    .find(|c| c.mv == best_move)
                    .map(|c| c.points)
                    .unwrap_or(0);

                total_score[i] += scored;
                total_nodes[i] += nodes + q_nodes;
                total_q_nodes[i] += q_nodes;
                if scored == best_available {
                    best_moves_found[i] += 1;
                }

                let (dest, orig, _, _) = split_move(best_move);
                let abf = abf_estimate(nodes, actual_depth);
                total_depth[i] += actual_depth;
                total_abf[i] += abf;
                println!("{:<15} engine played {}{} -> {}/{} -- {} nodes at depth {} (ABF {:.2})", test.id, orig, dest, scored, best_available, nodes, depth, abf);
            }
            println!();
        }
    }

    // print the results, broken out by each time limit
    println!("\nStrength Test Complete");
    for (i, depth) in DEPTH_LIMITS.iter().enumerate() {
        let mnps = ((total_nodes[i] as f64) / total_search_duration[i].as_secs_f64()) / 1_000_000.0;
        println!("\n--- Depth Limit: {} ---", depth);
        println!("Score: {}/{} ({:.2})", total_score[i], max_score, total_score[i] as f64 / max_score as f64);
        println!("Best moves found: {}/{} ({:.2})", best_moves_found[i], tests_run, best_moves_found[i] as f64 / tests_run as f64);
        println!("Search time: {:?}", total_search_duration[i]);
        println!("Total nodes: {}", total_nodes[i]);
        let negamax_nodes = total_nodes[i] - total_q_nodes[i];
        let q_pct = if total_nodes[i] > 0 {
            100.0 * total_q_nodes[i] as f64 / total_nodes[i] as f64
        } else {
            0.0
        };
        println!("  negamax nodes: {}", negamax_nodes);
        println!("  quiescence nodes: {} ({:.1}%)", total_q_nodes[i], q_pct);
        println!("Avg. Depth: {:.2}", (total_depth[i] as f64 / tests_run as f64));
        println!("MN/S: {:.2}", mnps);
        println!("ABF: {:.2}", (total_abf[i] / tests_run as f64));
    }

}

fn abf_estimate(nodes: u64, depth: usize) -> f64 {
    (nodes as f64).powf(1.0 / depth as f64)
}
