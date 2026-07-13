mod types;
mod eval;
mod interface;
mod state;
mod search;
mod perft;

use perft::perf_test;

use std::time::{Duration, Instant};
use std::env;

use search::{RootSearchType, init_engine};
use types::chess_move::split_move;
use types::position::Position;


fn main() {

    let args: Vec<String> = env::args().collect();
    let args_len = args.len();
    if args_len < 2 {
        print_help();
        return;
    }

    let cmd: &str = &args[1];

    match cmd {
        "perft" => perf_test(),
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
                    let start = Instant::now();
                    let (e, m) = p.root_search(search_type);
                    let duration = start.elapsed();
                    let (dest, orig, _, _) = split_move(m);
                    println!("Move from {} to {}. Eval is: {}", dest, orig, e);
                    println!("Found this move in {} ms", duration.as_millis());
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
    println!("'perft': run the perft test");
    println!("'search [fen] [search_type|'depth','time'] [time_ms|depth_ply]': return the best move and evaluation of the position");
}
