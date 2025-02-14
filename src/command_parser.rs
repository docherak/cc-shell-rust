use crate::shell_command::{ShellCommand, ShellCommandType};
use crate::tokenizer::SplitArgs;

pub struct CommandParser;

impl CommandParser {
    pub fn parse_type(input: &str) -> Option<ShellCommandType> {
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

    pub fn parse_command(input: &str) -> Option<ShellCommand> {
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
