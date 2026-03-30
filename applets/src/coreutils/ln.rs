use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut symbolic = false;
    let mut force = false;
    let mut targets: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-s" | "--symbolic" => symbolic = true,
            "-f" | "--force" => force = true,
            "--" => {
                targets.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        's' => symbolic = true,
                        'f' => force = true,
                        _ => {
                            eprintln!("ln: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            _ => targets.push(arg),
        }
    }

    if targets.len() < 2 {
        eprintln!("ln: missing file operand");
        return Ok(1);
    }

    let dest = targets[targets.len() - 1];
    let sources = &targets[..targets.len() - 1];
    let dest_path = Path::new(dest);

    let mut had_error = false;

    for src in sources {
        let src_path = Path::new(src);
        let target = if dest_path.is_dir() {
            let file_name = src_path.file_name().unwrap_or_default();
            dest_path.join(file_name)
        } else {
            dest_path.to_path_buf()
        };

        let result = if symbolic {
            std::os::unix::fs::symlink(src_path, &target)
        } else {
            std::fs::hard_link(src_path, &target)
        };

        if let Err(e) = result {
            if !force {
                eprintln!("ln: cannot create link '{}': {e}", target.display());
                had_error = true;
            }
        }
    }

    Ok(if had_error { 1 } else { 0 })
}
