use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut _mode: u32 = 0o755;
    let mut parents = false;
    let mut dirs: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-p" | "--parents" => parents = true,
            "-m" | "--mode" => {
                if let Some(m) = iter.next() {
                    _mode = match u32::from_str_radix(m.trim_start_matches('0'), 8) {
                        Ok(v) => v,
                        Err(_) => {
                            eprintln!("mkdir: invalid mode: '{m}'");
                            return Ok(1);
                        }
                    };
                } else {
                    eprintln!("mkdir: option requires an argument -- 'm'");
                    return Ok(1);
                }
            }
            "--" => {
                dirs.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'p' => parents = true,
                        _ => {
                            eprintln!("mkdir: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            _ => dirs.push(arg),
        }
    }

    if dirs.is_empty() {
        eprintln!("mkdir: missing operand");
        return Ok(1);
    }

    let mut had_error = false;

    for dir in &dirs {
        let path = Path::new(dir);
        let result = if parents {
            std::fs::create_dir_all(path)
        } else {
            std::fs::create_dir(path)
        };

        if let Err(e) = result {
            eprintln!("mkdir: cannot create directory '{dir}': {e}");
            had_error = true;
        }
    }

    Ok(if had_error { 1 } else { 0 })
}
