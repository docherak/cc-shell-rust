#[allow(unused_imports)]
use std::io::{self, Write};

use std::env;
use std::fs::OpenOptions;
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

#[derive(Debug)]
struct ShellCommand<'a> {
    command: ShellCommandType<'a>,
    redirect_file: Option<String>,
    redirect_stderr: bool,
    append: bool,
}

impl<'a> ShellCommand<'a> {
    fn new(
        command: ShellCommandType<'a>,
        redirect_file: Option<String>,
        redirect_stderr: bool,
        append: bool,
    ) -> Self {
        ShellCommand {
            command,
            redirect_file,
            redirect_stderr,
            append,
        }
    }
}

#[derive(Debug)]
enum ShellCommandType<'a> {
    Exit(Option<i32>),
    Echo(SplitArgs<'a>),
    Type(SplitArgs<'a>),
    Pwd,
    Cd(SplitArgs<'a>),
    External(String, SplitArgs<'a>),
}

impl<'a> ShellCommand<'a> {
    fn handle_output(&self, stdout: Option<String>, stderr: Option<String>) {
        if let Some(file) = &self.redirect_file {
            let mut options = OpenOptions::new();
            options.create(true).write(true);
            if self.append {
                options.append(true);
            }

            if let Ok(mut file) = options.open(file) {
                if self.redirect_stderr {
                    if let Some(stderr) = &stderr {
                        write!(file, "{}", stderr).unwrap();
                    }
                    if let Some(stdout) = &stdout {
                        io::stdout().write_all(stdout.as_bytes()).unwrap();
                    }
                } else {
                    if let Some(stdout) = &stdout {
                        write!(file, "{}", stdout).unwrap();
                    }
                    if let Some(stderr) = &stderr {
                        io::stderr().write_all(stderr.as_bytes()).unwrap();
                    }
                }
            }
        } else {
            if let Some(stdout) = stdout {
                io::stdout().write_all(stdout.as_bytes()).unwrap();
            }
            if let Some(stderr) = stderr {
                io::stderr().write_all(stderr.as_bytes()).unwrap();
            }
        }
    }
    fn execute(mut self) {
        match self.command {
            ShellCommandType::Exit(exit_code) => {
                if let Some(n) = exit_code {
                    process::exit(n)
                }
                process::exit(0)
            }
            ShellCommandType::Pwd => match env::current_dir() {
                Ok(pwd) => {
                    let mut pwd = pwd.display().to_string();
                    pwd.push('\n');
                    self.handle_output(Some(pwd), None)
                }
                Err(err) => self.handle_output(None, Some(err.to_string())),
            },

            ShellCommandType::Type(ref mut parts) => {
                let mut result = String::new();
                while let Some(command) = parts.next() {
                    match command.as_str() {
                        ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => break,
                        _ => match CommandParser::parse_type(command.as_str()) {
                            Some(ShellCommandType::External(..)) => {
                                if let Some(path) = command_in_path(&command, get_path_dirs()) {
                                    result.push_str(&format!("{} is {}\n", command, path));
                                } else {
                                    result.push_str(&format!("{}: not found\n", command));
                                }
                            }
                            _ => result.push_str(&format!("{} is a shell builtin\n", command)),
                        },
                    }
                }
                self.handle_output(Some(result.to_string()), None);
            }
            ShellCommandType::Cd(ref mut path) => {
                if let Some(first_arg) = path.next() {
                    let path = if first_arg == "~" {
                        env::var("HOME").unwrap_or_else(|_| ".".to_string())
                    } else {
                        format!("{}{}", first_arg, path.collect::<Vec<_>>().join(" "))
                    };

                    if env::set_current_dir(Path::new(&path)).is_err() {
                        let mut no_dir = format!("cd: {}: No such file or directory", path);
                        no_dir.push('\n');
                        self.handle_output(None, Some(no_dir));
                    }
                } else {
                    let home_path = env::var("HOME").unwrap_or_else(|_| ".".to_string());
                    env::set_current_dir(Path::new(&home_path)).unwrap();
                }
            }
            ShellCommandType::Echo(ref mut message) => {
                let mut result = String::new();
                let mut first = true;
                while let Some(command) = message.next() {
                    match command.as_str() {
                        ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => break,
                        _ => {
                            if !first {
                                result.push(' ');
                            }
                            first = false;
                            result.push_str(&command);
                        }
                    }
                }
                result.push('\n');
                self.handle_output(Some(result), None);
            }
            ShellCommandType::External(ref cmd, ref mut args) => {
                let mut command = Command::new(&cmd);
                let mut processed_args = Vec::new();

                while let Some(arg) = args.next() {
                    match arg.as_str() {
                        ">" | "1>" | "2>" | ">>" | "1>>" | "2>>" => break,
                        _ => processed_args.push(arg),
                    }
                }

                command.args(&processed_args);

                match command.output() {
                    Ok(output) => {
                        let trimmed_stdout = String::from_utf8_lossy(&output.stdout).to_string();
                        let trimmed_stderr = String::from_utf8_lossy(&output.stderr).to_string();

                        self.handle_output(Some(trimmed_stdout), Some(trimmed_stderr));
                    }
                    Err(_) => {
                        let mut not_found = format!("{}: command not found", cmd);
                        not_found.push('\n');
                        self.handle_output(None, Some(not_found));
                    }
                }
            }
        }
    }
}

struct CommandParser;

impl CommandParser {
    fn parse_type(input: &str) -> Option<ShellCommandType> {
        let mut parts = SplitArgs::new(input.trim());
        match parts.next()?.as_str() {
            "exit" => Some(ShellCommandType::Exit(
                parts.next().and_then(|s| s.parse::<i32>().ok()),
            )),
            "pwd" => Some(ShellCommandType::Pwd),
            "type" => Some(ShellCommandType::Type(parts)),
            "cd" => Some(ShellCommandType::Cd(parts)),
            "echo" => Some(ShellCommandType::Echo(parts)),
            command => Some(ShellCommandType::External(command.to_string(), parts)),
        }
    }

    fn parse_command(input: &str) -> Option<ShellCommand> {
        let command = CommandParser::parse_type(input)?;
        let mut parts = SplitArgs::new(input.trim());

        let mut redirect_to_file: Option<String> = None;
        let mut redirect_to_stderr = false;
        let mut append = false;

        while let Some(arg) = parts.next() {
            match arg.as_str() {
                ">" | "1>" | "2>" => {
                    if arg == "2>" {
                        redirect_to_stderr = true;
                    }
                    redirect_to_file = parts.next();
                    break;
                }
                ">>" | "1>>" | "2>>" => {
                    if arg == "2>>" {
                        redirect_to_stderr = true;
                    }
                    redirect_to_file = parts.next();
                    append = true;
                    break;
                }
                _ => continue,
            }
        }

        Some(ShellCommand::new(
            command,
            redirect_to_file,
            redirect_to_stderr,
            append,
        ))
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

        if let Some(cmd) = CommandParser::parse_command(&input.trim()) {
            cmd.execute();
        }
    }
}
