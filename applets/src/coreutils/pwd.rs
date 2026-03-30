use std::env;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut physical = false;

    for arg in args {
        match arg.as_str() {
            "-P" | "--physical" => physical = true,
            "--" => break,
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'P' => physical = true,
                        _ => {
                            eprintln!("pwd: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            _ => {}
        }
    }

    let cwd = if physical {
        env::current_dir()
    } else {
        env::var("PWD")
            .map(std::path::PathBuf::from)
            .or_else(|_| env::current_dir())
    };

    match cwd {
        Ok(path) => {
            println!("{}", path.display());
            Ok(0)
        }
        Err(e) => {
            eprintln!("pwd: {e}");
            Ok(1)
        }
    }
}
