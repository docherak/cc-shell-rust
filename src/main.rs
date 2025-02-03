#[allow(unused_imports)]
use std::io::{self, Write};

use std::process;
use std::iter::Iterator;

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();

        let mut parts = input.trim().split_whitespace();
        match parts.next() {
            Some("exit") => {
                if let Some(n) = parts.next() {
                    if let Ok(err_code) = n.parse::<i32>() {
                        process::exit(err_code)
                    }
                }
                process::exit(0)
            },
            Some("echo") => {
                let message: String = parts.fold(String::new(), |mut acc, word| {
                    if !acc.is_empty() {
                        acc.push(' ');
                    }
                    acc.push_str(word);
                    acc
                });
                println!("{}", message);
            },
            _ => println!("{}: command not found", input.trim())
        }


    }
}
