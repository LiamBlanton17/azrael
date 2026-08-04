
pub mod uci;

use std::io;

pub fn read_line_from_stdin() -> io::Result<Vec<String>> {
    let mut buffer = String::new();
    io::stdin().read_line(&mut buffer)?;

    // Trim whitespace and split by spaces
    let args: Vec<String> = buffer
        .split_whitespace()
        .map(|s| s.to_string())  
        .collect(); 

    Ok(args)
}
