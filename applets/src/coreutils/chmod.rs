use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut recursive = false;
    let mut mode: Option<u32> = None;
    let mut files: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-R" | "--recursive" => recursive = true,
            "--" => {
                files.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'R' => recursive = true,
                        _ => {
                            eprintln!("chmod: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            s if mode.is_none() && files.is_empty() => {
                match parse_mode(s) {
                    Some(m) => mode = Some(m),
                    None => {
                        eprintln!("chmod: invalid mode: '{s}'");
                        return Ok(1);
                    }
                }
            }
            _ => files.push(arg),
        }
    }

    let Some(mode) = mode else {
        eprintln!("chmod: missing mode operand");
        return Ok(1);
    };

    if files.is_empty() {
        eprintln!("chmod: missing operand");
        return Ok(1);
    }

    let mut had_error = false;

    for file in &files {
        let path = Path::new(file);
        if let Err(e) = chmod_path(path, mode, recursive) {
            eprintln!("chmod: cannot access '{file}': {e}");
            had_error = true;
        }
    }

    Ok(if had_error { 1 } else { 0 })
}

fn chmod_path(path: &Path, mode: u32, recursive: bool) -> std::io::Result<()> {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode))?;

    if recursive && path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            let meta = entry.metadata()?;
            if meta.is_dir() {
                chmod_path(&entry.path(), mode, true)?;
            } else {
                std::fs::set_permissions(entry.path(), std::fs::Permissions::from_mode(mode))?;
            }
        }
    }

    Ok(())
}

fn parse_mode(s: &str) -> Option<u32> {
    if s.len() >= 3 && s.chars().all(|c| ('0'..='7').contains(&c)) {
        u32::from_str_radix(s, 8).ok()
    } else {
        None
    }
}
