use crate::lexer::Token;

#[derive(Debug, Clone)]
pub struct SimpleCommand {
    pub argv: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum Command {
    Simple(SimpleCommand),
    Pipeline(Vec<SimpleCommand>),
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Command>, String> {
    let mut commands = Vec::new();
    let mut current_pipeline: Vec<SimpleCommand> = Vec::new();
    let mut current_argv: Vec<String> = Vec::new();

    for token in tokens {
        match token {
            Token::Word(word) => current_argv.push(word.clone()),
            Token::Pipe => {
                if current_argv.is_empty() {
                    return Err("sh: syntax error near unexpected token `|'".to_string());
                }
                current_pipeline.push(SimpleCommand {
                    argv: std::mem::take(&mut current_argv),
                });
            }
            Token::Semicolon | Token::Newline => {
                if !current_argv.is_empty() {
                    current_pipeline.push(SimpleCommand {
                        argv: std::mem::take(&mut current_argv),
                    });
                }
                if !current_pipeline.is_empty() {
                    commands.push(finalize_pipeline(std::mem::take(&mut current_pipeline)));
                }
            }
        }
    }

    if !current_argv.is_empty() {
        current_pipeline.push(SimpleCommand { argv: current_argv });
    }
    if !current_pipeline.is_empty() {
        commands.push(finalize_pipeline(current_pipeline));
    }

    Ok(commands)
}

fn finalize_pipeline(mut pipeline: Vec<SimpleCommand>) -> Command {
    if pipeline.len() == 1 {
        Command::Simple(pipeline.remove(0))
    } else {
        Command::Pipeline(pipeline)
    }
}
