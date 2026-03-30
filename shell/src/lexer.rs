#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Word(String),
    Pipe,
    Semicolon,
    Newline,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();
    let mut in_single = false;
    let mut in_double = false;

    while let Some(ch) = chars.next() {
        match ch {
            '\'' if !in_double => {
                in_single = !in_single;
            }
            '"' if !in_single => {
                in_double = !in_double;
            }
            '|' if !in_single && !in_double => {
                flush_word(&mut current, &mut tokens);
                tokens.push(Token::Pipe);
            }
            ';' if !in_single && !in_double => {
                flush_word(&mut current, &mut tokens);
                tokens.push(Token::Semicolon);
            }
            '\n' if !in_single && !in_double => {
                flush_word(&mut current, &mut tokens);
                tokens.push(Token::Newline);
            }
            ' ' | '\t' if !in_single && !in_double => {
                flush_word(&mut current, &mut tokens);
            }
            '\\' if !in_single => {
                if let Some(next) = chars.next() {
                    current.push(next);
                }
            }
            '$' if !in_single => {
                let mut name = String::new();
                while let Some(next) = chars.peek().copied() {
                    if next.is_ascii_alphanumeric() || next == '_' {
                        name.push(next);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if name.is_empty() {
                    current.push('$');
                } else {
                    current.push_str(&std::env::var(&name).unwrap_or_default());
                }
            }
            _ => current.push(ch),
        }
    }

    if in_single || in_double {
        return Err("sh: unclosed quote".to_string());
    }

    flush_word(&mut current, &mut tokens);
    Ok(tokens)
}

fn flush_word(current: &mut String, tokens: &mut Vec<Token>) {
    if !current.is_empty() {
        tokens.push(Token::Word(std::mem::take(current)));
    }
}
