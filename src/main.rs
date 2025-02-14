use std::io::{self, Write};

mod command_parser;
mod shell_command;
mod tokenizer;

use command_parser::CommandParser;

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();

        if let Some(cmd) = CommandParser::parse_command(&input.trim()) {
            cmd.execute();
        }
    }
}
