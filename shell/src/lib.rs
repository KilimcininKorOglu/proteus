pub mod lexer;
pub mod parser;
pub mod interpreter;
pub mod builtins;

use proteus_core::ProteusResult;

pub fn run_shell(args: &[String]) -> ProteusResult<i32> {
    let mut script_mode = false;
    let mut command_string = None;
    let mut positional_args = Vec::new();

    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "-c" => {
                script_mode = true;
                i += 1;
                if i < args.len() {
                    command_string = Some(args[i].clone());
                } else {
                    eprintln!("sh: -c: option requires an argument");
                    return Ok(2);
                }
            }
            "-s" => {
                i += 1;
            }
            "--" => {
                i += 1;
                positional_args.extend(args[i..].iter().cloned());
                break;
            }
            _ => {
                positional_args.push(args[i].clone());
            }
        }
        i += 1;
    }

    if script_mode {
        let cmd = command_string.unwrap_or_default();
        let mut interp = interpreter::Interpreter::new();
        interp.set_args(positional_args);
        return interp.execute_script(&cmd);
    }

    // Interactive mode not yet implemented for v0.1
    eprintln!("sh: interactive mode not yet implemented");
    Ok(2)
}
