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

use crate::interface::read_line_from_stdin;
use crate::search::tt::TranspositionTable;


fn main() {

    // TODO - use cli args to read from a TOML file
    let _cli_args: Vec<String> = env::args().collect();

    // enter the engine loop
    loop {
        match read_line_from_stdin() {
            Ok(input) => {
                if input.len() == 0 {
                    print_help();
                    continue;
                }

                // determine what to run matching on the first input arg
                let input_cmd = &input[0];
                match input_cmd.as_str() {
                    "uci" => {
                        interface::uci::run();
                        break;
                    },
                    "perft" => benchmarks::perft::test(),
                    "strength" => benchmarks::strength::test(),
                    "help" => print_help(),
                    "quit" => break,
                    _ => {
                        println!("Invalid cmd: {}", input_cmd);
                        print_help();
                    },
                }
            },
            Err(e) => {
                println!("{}", e);
                break;
            },
        }
    }

}

fn print_help() {
    println!("
    Command line options:
        'uci' -> switch the engine into UCI mode
        'help' -> print this help message
        'perft' -> run the perft test for verifying move generation and test raw speed
        'strength' -> run the battery of tests in the 'test_files' directory
        'quit' -> exit the program
    ");
}
