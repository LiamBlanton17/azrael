mod types;
mod eval;
mod interface;
mod state;
mod search;
mod benchmarks;

use std::time::{Duration, Instant};
use std::env;

use search::{RootSearchType, init_engine};
use types::chess_move::split_move;
use types::position::Position;

use crate::search::tt::TranspositionTable;


fn main() {

    let args: Vec<String> = env::args().collect();
    let args_len = args.len();
    let cmd = if args_len < 2 { "uci" } else { &args[1] };

    match cmd {
        "uci" => interface::uci_cmd(),
        "perft" => benchmarks::perft::perf_test(),
        "strength" => benchmarks::strength::strength_test(),
        "search" => {
            if args_len < 5 {
                print_help();
                return;
            }
            let fen: &str = &args[2];
            let type_limit: usize = args[4].parse().expect("Must pass an integer for time limit (in ms) or max depth (in ply)");
            let search_type: RootSearchType = match args[3].as_str() {
                "depth" => RootSearchType::DepthLimited(type_limit),
                "time" => RootSearchType::TimeLimited(Duration::from_millis(type_limit as u64)),
                _ => {
                    print_help();
                    return;
                }
            };
            init_engine();
            let p = Position::from_fen(fen);
            match p {
                Ok(mut p) => {
                    let mut tt = TranspositionTable::new(128);
                    let start = Instant::now();
                    let (e, m, n, qn, d) = p.root_search(search_type, &[], &mut tt);
                    let total_nodes = n + qn;
                    let duration = start.elapsed();
                    let (dest, orig, _, _) = split_move(m);
                    let mnps = ((total_nodes as f64) / duration.as_secs_f64()) / 1_000_000.0;
                    println!("Move from {} to {}. Eval is: {}", orig, dest, e);
                    println!("Found this move in {} ms, reached a depth of {}, searching {} nodes ({} MN/s)", duration.as_millis(), d, format_with_commas(total_nodes), mnps);
                },
                Err(e) => {
                    println!("Failed to parse FEN: ");
                    println!("{}", e);
                }
            }
        },
        _ => print_help(),
    }
}

fn print_help() {
    println!("Must call with one of these arguments:");
    println!("'uci': run the engine with the uci protocol");
    println!("'perft': run the perft test - do this EVERYTIME movegen is touched");
    println!("'strength': run the strength test - a comprehensive battery of tests to determine if a change improved strength");
    println!("'search [fen] [search_type|'depth','time'] [time_ms|depth_ply]': return the best move and evaluation of the position");
}

pub fn format_with_commas(n: u64) -> String {
    let s = n.to_string();
    let mut result = String::new();
    for (i, c) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }
    result.chars().rev().collect()
}