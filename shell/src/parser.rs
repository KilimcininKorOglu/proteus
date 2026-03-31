use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct Redirect {
    pub fd: i32,
    pub target: String,
    pub append: bool,
    pub input: bool,
}

#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub argv: Vec<String>,
    pub redirects: Vec<Redirect>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionalOperator {
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum Command {
    Simple(SimpleCommand),
    Pipeline(Vec<SimpleCommand>),
    Conditional {
        left: Box<Command>,
        operator: ConditionalOperator,
        right: Box<Command>,
    },
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Command>, String> {
    let mut commands = Vec::new();
    let mut current_tokens = Vec::new();

    for token in tokens {
        match token {
            Token::Semicolon | Token::Newline => {
                if !current_tokens.is_empty() {
                    commands.push(parse_command_sequence(&current_tokens)?);
                    current_tokens.clear();
                }
            }
            token => current_tokens.push(token.clone()),
        }
    }

    if !current_tokens.is_empty() {
        commands.push(parse_command_sequence(&current_tokens)?);
    }

    Ok(commands)
}

fn parse_command_sequence(tokens: &[Token]) -> Result<Command, String> {
    let mut segments = Vec::new();
    let mut operators = Vec::new();
    let mut start = 0usize;

    for (index, token) in tokens.iter().enumerate() {
        match token {
            Token::AndIf | Token::OrIf => {
                let segment = parse_pipeline(&tokens[start..index])?;
                segments.push(segment);
                operators.push(match token {
                    Token::AndIf => ConditionalOperator::And,
                    Token::OrIf => ConditionalOperator::Or,
                    _ => unreachable!(),
                });
                start = index + 1;
            }
            _ => {}
        }
    }

    segments.push(parse_pipeline(&tokens[start..])?);

    let mut command = segments.remove(0);
    for (operator, right) in operators.into_iter().zip(segments.into_iter()) {
        command = Command::Conditional {
            left: Box::new(command),
            operator,
            right: Box::new(right),
        };
    }

    Ok(command)
}

fn parse_pipeline(tokens: &[Token]) -> Result<Command, String> {
    let mut pipeline = Vec::new();
    let mut current = Vec::new();

    for token in tokens {
        match token {
            Token::Pipe => {
                if current.is_empty() {
                    return Err("sh: syntax error near unexpected token `|'".to_string());
                }
                pipeline.push(parse_simple_command(&current)?);
                current.clear();
            }
            token => current.push(token.clone()),
        }
    }

    if !current.is_empty() {
        pipeline.push(parse_simple_command(&current)?);
    }

    if pipeline.is_empty() {
        return Err("sh: empty command".to_string());
    }

    Ok(if pipeline.len() == 1 {
        Command::Simple(pipeline.remove(0))
    } else {
        Command::Pipeline(pipeline)
    })
}

fn parse_simple_command(tokens: &[Token]) -> Result<SimpleCommand, String> {
    let mut argv = Vec::new();
    let mut redirects = Vec::new();
    let mut iter = tokens.iter().peekable();

    while let Some(token) = iter.next() {
        match token {
            Token::Word(word) => argv.push(word.clone()),
            Token::RedirectIn | Token::RedirectOut | Token::RedirectAppend => {
                let Some(Token::Word(target)) = iter.next() else {
                    return Err("sh: redirection requires a target".to_string());
                };
                redirects.push(Redirect {
                    fd: if matches!(token, Token::RedirectIn) { 0 } else { 1 },
                    target: target.clone(),
                    append: matches!(token, Token::RedirectAppend),
                    input: matches!(token, Token::RedirectIn),
                });
            }
            _ => return Err("sh: unsupported token in command".to_string()),
        }
    }

    Ok(SimpleCommand { argv, redirects })
}
