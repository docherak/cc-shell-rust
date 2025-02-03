#[allow(unused_imports)]
use std::io::{self, Write};

use std::process;
use std::iter::Iterator;
use std::env;
use std::path::Path;

fn get_path_dirs() -> Option<Vec<String>> {
    env::var("PATH").ok().map(|path| {
        path.split(':')
            .map(|s| s.trim().to_string())
            .collect()
    })
}

fn command_in_path(command: &str, path: Option<Vec<String>>) -> Option<String> {
    path?.into_iter()
        .find_map(|dir| {
            let full_path = format!("{}/{}", dir, command);
            if Path::new(&full_path).exists() {
                Some(full_path)
            } else {
                None
            }
        })
}

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
            Some("type") => {
                while let Some(command) = parts.next() {
                    match command {
                        "exit" | "echo" | "type" => println!("{} is a shell builtin", command),
                        _ => {
                            if let Some(path) = command_in_path(command, get_path_dirs()) {
                                println!("{} is {}", command, path);
                            } else {
                                println!("{}: not found", command);
                            }
                        }
                    }
                }
            }
            _ => println!("{}: command not found", input.trim())
        }


    }
}
