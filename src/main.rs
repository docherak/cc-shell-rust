#[allow(unused_imports)]
use std::io::{self, Write};

use std::process;

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();

        let mut parts = input.trim().split_whitespace();
        match (parts.next(), parts.next()) {
            (Some("exit"), Some(n)) if n.parse::<i32>().is_ok() => process::exit(n.parse::<i32>().unwrap()),
            _ => println!("{}: command not found", input.trim())
        }


    }
}
