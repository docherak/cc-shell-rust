#[allow(unused_imports)]
use std::io::{self, Write};

use std::env;
use std::iter::Iterator;
use std::path::Path;
use std::process;
use std::process::Command;

mod arg_parser;
use arg_parser::SplitArgs;

fn get_path_dirs() -> Option<Vec<String>> {
    env::var("PATH")
        .ok()
        .map(|path| path.split(':').map(|s| s.trim().to_string()).collect())
}

fn command_in_path(command: &str, path: Option<Vec<String>>) -> Option<String> {
    path?.into_iter().find_map(|dir| {
        let full_path = format!("{}/{}", dir, command);
        if Path::new(&full_path).exists() {
            Some(full_path)
        } else {
            None
        }
    })
}

enum ShellCommand<'a> {
    Exit(Option<i32>),
    Echo(SplitArgs<'a>),
    Type(SplitArgs<'a>),
    Pwd,
    Cd(SplitArgs<'a>),
    External(String, SplitArgs<'a>),
}

impl<'a> ShellCommand<'a> {
    fn execute(self) {
        match self {
            ShellCommand::Exit(exit_code) => {
                if let Some(n) = exit_code {
                    process::exit(n)
                }
                process::exit(0)
            }
            ShellCommand::Pwd => {
                if let Ok(pwd) = env::current_dir() {
                    println!("{}", pwd.display())
                }
            }
            ShellCommand::Type(mut parts) => {
                while let Some(command) = parts.next() {
                    match CommandParser::parse(command.as_str()) {
                        Some(ShellCommand::External(..)) => {
                            if let Some(path) = command_in_path(&command, get_path_dirs()) {
                                println!("{} is {}", command, path);
                            } else {
                                println!("{}: not found", command);
                            }
                        }
                        _ => println!("{} is a shell builtin", command),
                    }
                }
            }
            ShellCommand::Cd(mut path) => {
                if let Some(first_arg) = path.next() {
                    let path = if first_arg == "~" {
                        env::var("HOME").unwrap_or_else(|_| ".".to_string())
                    } else {
                        format!("{}{}", first_arg, path.collect::<Vec<_>>().join(" "))
                    };

                    if env::set_current_dir(Path::new(&path)).is_err() {
                        println!("cd: {}: No such file or directory", path);
                    }
                } else {
                    let home_path = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    env::set_current_dir(Path::new(&home_path)).unwrap();
                }
            }
            ShellCommand::Echo(message) => println!("{}", message.collect::<Vec<_>>().join(" ")),
            ShellCommand::External(cmd, args) => {
                if let Some(_full_path) = command_in_path(&cmd, get_path_dirs()) {
                    let args: Vec<String> = args.collect();
                    let output = Command::new(cmd)
                        .args(args)
                        .output()
                        .expect("failed to execute process");
                    io::stdout().write_all(&output.stdout).unwrap();
                    io::stderr().write_all(&output.stderr).unwrap();
                } else {
                    println!("{}: command not found", cmd)
                }
            }
        }
    }
}

struct CommandParser;

impl CommandParser {
    fn parse(input: &str) -> Option<ShellCommand> {
        let mut parts = SplitArgs::new(input.trim());
        match parts.next()?.as_str() {
            "exit" => Some(ShellCommand::Exit(
                parts.next().and_then(|s| s.parse::<i32>().ok()),
            )),
            "pwd" => Some(ShellCommand::Pwd),
            "type" => Some(ShellCommand::Type(parts)),
            "cd" => Some(ShellCommand::Cd(parts)),
            "echo" => Some(ShellCommand::Echo(parts)),
            command => Some(ShellCommand::External(command.to_string(), parts)),
        }
    }
}

fn main() {
    let stdin = io::stdin();
    let mut input = String::new();

    loop {
        input.clear();
        print!("$ ");
        io::stdout().flush().unwrap();
        stdin.read_line(&mut input).unwrap();

        if let Some(cmd) = CommandParser::parse(&input) {
            cmd.execute();
        }
    }
}
