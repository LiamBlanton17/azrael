use std::io::{self, Write};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use crate::interface::read_line_from_stdin;
use crate::search::{RootSearchType, init_engine, tt::TranspositionTable};
use crate::types::color::Color;
use crate::types::position::{Position, ZobristHash};

// https://en.wikipedia.org/wiki/Universal_Chess_Interface
struct UCIState {
    tt: Arc<Mutex<TranspositionTable>>,
    game_history: Vec<ZobristHash>,
    position: Option<Position>,
    stop: Arc<AtomicBool>,
}

pub fn run() {

    // initial greeting message
    println!("id name Azrael 0.1");
    println!("id author LiamBlanton");
    println!("uciok");
    io::stdout().flush().unwrap();

    init_engine();
    const TT_SIZE_MB: usize = 512;
    let mut state = UCIState {
        tt: Arc::new(Mutex::new(TranspositionTable::new(TT_SIZE_MB))),
        game_history: Vec::new(),
        position: None,
        stop: Arc::new(AtomicBool::new(false)),
    };

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
                state.position = None;
                state.game_history.clear();
                state.tt.lock().unwrap().clear();
            },
            "position" => {
                if args_len < 2 { 
                    continue 'outerloop; 
                }

                let moves_idx = args.iter().position(|a| a == "moves");
                let fen_end = moves_idx.unwrap_or(args_len);
                let start_of_moves = moves_idx.map_or(args_len, |i| i + 1);
                let fen = match args[1].as_str() {
                    "startpos" => "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
                    "fen" if fen_end > 2 => args[2..fen_end].join(" "),
                    _ => continue 'outerloop,
                };

                let mut p = match Position::from_fen(&fen) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("info string bad fen: {:?}", e);
                        continue 'outerloop;
                    }
                };

                let mut new_history = Vec::with_capacity(args_len - start_of_moves);
                for long_alg in &args[start_of_moves..] {
                    match p.move_from_la(long_alg) {
                        Ok(m) => { new_history.push(p.zobrist); p.make_move(m); }
                        Err(_) => {
                            eprintln!("info string bad move: {}", long_alg);
                            continue 'outerloop;
                        }
                    }
                }

                state.position = Some(p);
                state.game_history = new_history.clone();
            },
            "go" => {
                // stop any lasting search
                state.stop.store(false, Ordering::Relaxed);

                // create clone of reference to tt and clone of game history for the seach
                let tt = Arc::clone(&state.tt);
                let game_history = state.game_history.clone();
                let stop = Arc::clone(&state.stop);

                // if no position defined, just use the starting position
                let position = *state.position.get_or_insert_with(|| {
                    Position::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap()
                });

                // move ownership of these new variables into the search thread
                thread::spawn(move || {
                    run_go(args, position, game_history, tt, stop);
                });
            },
            "stop" => {
                state.stop.store(true, Ordering::Relaxed);
                continue 'outerloop;
            }
            "ponderhit" => continue 'outerloop,
            "quit" => break 'outerloop,
            _ => continue 'outerloop,
        }
    }

}

#[derive(Default)]
struct GoParams {
    wtime: Option<u64>,
    btime: Option<u64>,
    winc: u64,
    binc: u64,
    movetime: Option<u64>,
    is_infinite: bool,
}

fn run_go(args: Vec<String>, mut position: Position, game_history: Vec<ZobristHash>, tt: Arc<Mutex<TranspositionTable>>, stop: Arc<AtomicBool>) {
    // parse the cmd line args from the go cmd
    let params = parse_go(&args);

    // determine the budget for this round
    // either dictated from the GUI via movetime
    // or is some fraction of the remaining time
    let budget_ms = if let Some(mt) = params.movetime { mt } else {
        let (my_time, my_inc) = if position.turn == Color::White {
            (params.wtime, params.winc)
        } else {
            (params.btime, params.binc)
        };
        match my_time {
            // 7.14% of the remaining clock + half the increment,
            // capped below the clock, floored at 5ms.
            Some(t) => (t / 14 + my_inc / 2).min(t).max(5),
            None => 100, // if no clock info, play really fast (100ms)
        }
    };

    let search_type = if params.is_infinite {
        RootSearchType::Infinite
    } else {
        RootSearchType::StableTimeLimited(Duration::from_millis(budget_ms))
    };

    let (e, m, _, _, _) = position.root_search(
        search_type, 
        &game_history, 
        &mut tt.lock().unwrap(),
        stop,
    );
    println!("bestmove {}", position.move_to_la(m));
    eprintln!("Current Eval: {}", e);
    io::stdout().flush().unwrap();
}

fn parse_go(args: &[String]) -> GoParams {
    let mut g = GoParams::default();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "infinite" => {
                g.is_infinite = true;
                i += 1;
            }
            "wtime" => { 
                g.wtime = args.get(i + 1).and_then(|s| s.parse().ok());              
                i += 2; 
            },
            "btime" => { 
                g.btime = args.get(i + 1).and_then(|s| s.parse().ok());              
                i += 2; 
            },
            "winc" => { 
                g.winc = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0); 
                i += 2; 
            },
            "binc"     => { 
                g.binc = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(0); 
                i += 2; 
            },
            "movetime" => { 
                g.movetime = args.get(i + 1).and_then(|s| s.parse().ok());              
                i += 2; 
            },
            _ => {  // all other cmd line args are ignored for now
                i += 1; 
            },
        }
    }
    g
}

