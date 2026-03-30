use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut dirs: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--" => continue,
            s if s.starts_with('-') && s.len() > 1 => {
                eprintln!("rmdir: invalid option -- '{}'", &s[1..]);
                return Ok(1);
            }
            _ => dirs.push(arg),
        }
    }

    if dirs.is_empty() {
        eprintln!("rmdir: missing operand");
        return Ok(1);
    }

    let mut had_error = false;

    for dir in &dirs {
        let path = Path::new(dir);
        if let Err(e) = std::fs::remove_dir(path) {
            eprintln!("rmdir: failed to remove '{dir}': {e}");
            had_error = true;
        }
    }

    Ok(if had_error { 1 } else { 0 })
}
