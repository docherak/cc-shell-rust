#[derive(Debug)]
pub struct SplitArgs<'a> {
    input: &'a str,
}

impl<'a> SplitArgs<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input: input.trim(),
        }
    }
}

impl<'a> Iterator for SplitArgs<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        let mut chars = self.input.chars().peekable();
        let mut result = String::new();
        let mut in_quotes: Option<char> = None;
        let mut consumed = 0;
        let redirection_operators = ["2>>", "1>>", ">>", "2>", "1>", ">"];

        while let Some(&c) = chars.peek() {
            match c {
                '\\' if in_quotes.is_none() => {
                    // Outside quotes: Escape the next character (but only if it's not a space)
                    chars.next();
                    consumed += 1;
                    if let Some(&next_c) = chars.peek() {
                        // Skip the backslash and add the next character
                        result.push(next_c);
                        chars.next();
                        consumed += 1;
                    }
                }
                '\\' if in_quotes == Some('"') => {
                    // Inside double quotes: handle escape sequences for quotes and backslashes
                    chars.next();
                    consumed += 1;
                    if let Some(&next_c) = chars.peek() {
                        match next_c {
                            '"' => {
                                result.push('"');
                                chars.next();
                                consumed += 1;
                            }
                            '\\' => {
                                result.push('\\');
                                chars.next();
                                consumed += 1;
                            }
                            _ => {
                                result.push('\\');
                                result.push(next_c);
                                chars.next();
                                consumed += 1;
                            }
                        }
                    }
                }
                '\\' if in_quotes == Some('\'') => {
                    // Inside single quotes: treat backslashes literally
                    chars.next();
                    consumed += 1;
                    result.push('\\');
                }
                '"' | '\'' => {
                    // Handle quotes
                    chars.next();
                    consumed += 1;

                    if in_quotes == Some(c) {
                        in_quotes = None; // Closing the quote
                    } else if in_quotes.is_none() {
                        in_quotes = Some(c); // Opening a quote
                    } else {
                        result.push(c);
                    }
                }
                _ if c.is_whitespace() && in_quotes.is_none() => {
                    // Split on spaces outside quotes
                    break;
                }
                _ if in_quotes.is_none() => {
                    // Check for redirection operators
                    let mut is_redirection = false;
                    for &op in &redirection_operators {
                        if self.input[consumed..].starts_with(op) {
                            if !result.is_empty() {
                                // Special handling for `>` and `>>` to separate them from the previous token
                                if op == ">" || op == ">>" {
                                    self.input = self.input[consumed..].trim_start();
                                    return Some(result);
                                }
                                break;
                            }
                            result.push_str(op);
                            chars.nth(op.len() - 1); // Consume the operator
                            consumed += op.len();
                            is_redirection = true;
                            break;
                        }
                    }
                    if is_redirection {
                        break;
                    } else {
                        result.push(c);
                        chars.next();
                        consumed += 1;
                    }
                }
                _ => {
                    result.push(c);
                    chars.next();
                    consumed += 1;
                }
            }
        }

        self.input = self.input[consumed..].trim_start();

        if result.is_empty() {
            None
        } else {
            Some(result)
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_splitting() {
        let input = "echo hello world";
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_quotes() {
        let input = r#"echo "hello world" 'test case'"#;
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["echo", "hello world", "test case"]);
    }

    #[test]
    fn test_mixed_spaces() {
        let input = "  echo   'hello   world'  \"test\"  ";
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["echo", "hello   world", "test"]);
    }

    #[test]
    fn test_nested_quotes() {
        let input = r#"echo "it's a test" 'Rust "rocks"'"#;
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["echo", "it's a test", r#"Rust "rocks""#]);
    }

    #[test]
    fn test_empty_input() {
        let input = "    ";
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, Vec::<String>::new());
    }

    #[test]
    fn test_no_quotes() {
        let input = "ls -l /home/user";
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["ls", "-l", "/home/user"]);
    }

    #[test]
    fn test_backslash_in_double_quotes() {
        let input = r#"echo "Hello \"world\" Backslash \\ test""#;
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["echo", "Hello \"world\" Backslash \\ test"]);
    }

    #[test]
    fn test_backslash_outside_quotes() {
        let input = r#"echo \hello world"#;
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["echo", "hello", "world"]);
    }

    #[test]
    fn test_escape_quotes_inside_quotes() {
        let input = r#"echo \"test\""#;
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["echo", "\"test\""]);
    }

    #[test]
    fn test_mixed_escaped_and_unescaped_quotes() {
        let input = r#"echo "hello\"world""#;
        let words: Vec<String> = SplitArgs::new(input).collect();
        assert_eq!(words, vec!["echo", "hello\"world"]);
    }
}
