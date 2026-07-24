use std::io::{self, Write};
use std::time::Duration;

use crate::search::{RootSearchType, init_engine, tt::TranspositionTable};
use crate::types::position::{Position, ZobristHash};

// https://en.wikipedia.org/wiki/Universal_Chess_Interface
pub fn uci_cmd() {

    init_engine();
    let mut tt = TranspositionTable::new(256);
    let mut position = None;
    let mut game_history: Vec<ZobristHash> = Vec::new();
    let mut current_eval = 0;

    'outerloop: while let Ok(args) = read_line_from_stdin() {

        // Must have at least one argument
        let args_len = args.len();
        if args_len == 0 {
            continue 'outerloop;
        }

        // First argument is the command type
        let cmd_type = &args[0];
        match cmd_type.as_str() {
            "uci" => {
                println!("id name Azrael 0.1");
                println!("id author LiamBlanton");
                println!("uciok");
                io::stdout().flush().unwrap();
            },
            "isready" => {
                println!("readyok");
                io::stdout().flush().unwrap();
            },
            "ucinewgame" => {
                position = None;
                game_history.clear();
                tt.clear();
            },
            "position" => {
                // must have at least 3 arguments
                if args_len < 3 {
                    continue 'outerloop;
                }

                // must have 8 arguments for fen positions
                if args[1] == "fen" && args_len < 8 {
                    continue 'outerloop;
                }

                // Read in fen string or start position
                let mut start_of_moves = 3;
                let fen: String = if args[1] == "startpos" {
                    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string()
                } else {
                    start_of_moves = 8;
                    args[2..8].join(" ")
                };
                match Position::from_fen(fen.as_str()) {
                    Ok(p) => position = Some(p),
                    Err(_) => continue,
                }
                
                // Play all moves given over the start position so the search can see repetitions
                let mut p = position.expect("Position must be set");
                let mut new_history: Vec<ZobristHash> = Vec::with_capacity(args.len().saturating_sub(start_of_moves));
                for long_alg in &args[start_of_moves..] {
                    match p.move_from_la(long_alg) {
                        Ok(m) => {
                            new_history.push(p.zobrist);
                            p.make_move(m);
                        },
                        Err(_) => {
                            position = None;
                            game_history.clear();
                            continue 'outerloop;
                        }
                    }
                }

                // Set the position to this new position
                position = Some(p);
                game_history = new_history;
            },
            "go" => {
                let p = position.get_or_insert_with(|| {
                    Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
                });
                let (e, m, _, _, _) = p.root_search(
        RootSearchType::TimeLimited(Duration::from_millis(500)),
                    &game_history,
                    &mut tt
                );
                current_eval = e;
                println!("bestmove {}", p.move_to_la(m));
                io::stdout().flush().unwrap();
            },
            "stop" => continue 'outerloop,
            "ponderhit" => continue 'outerloop,
            "quit" => break 'outerloop,
            "print" => {
                // this is NOT a standard uci command
                // custom command for printing the board to stdout
                if let Some(p) = &position {
                    p.print();
                    io::stdout().flush().unwrap();
                }
            }
            "eval" => {
                // this is NOT a standard uci command
                // custom command for printing the eval to stdout
                if let Some(_p) = &position {
                    println!("Eval: {:.2}", current_eval as f32 / 100f32);
                    io::stdout().flush().unwrap();
                }
            }
            _ => continue 'outerloop,
        }
    }

}

fn read_line_from_stdin() -> io::Result<Vec<String>> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    // Trim whitespace and split by spaces
    let args: Vec<String> = buffer
        .split_whitespace()
        .map(|s| s.to_string())  
        .collect(); 

    Ok(args)
}
