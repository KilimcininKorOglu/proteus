use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut interactive = false;
    let mut paths: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-f" | "--force" => {}
            "-i" | "--interactive" => interactive = true,
            "--" => {
                paths.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'f' => {}
                        'i' => interactive = true,
                        _ => {
                            eprintln!("mv: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            _ => paths.push(arg),
        }
    }

    if paths.len() < 2 {
        eprintln!("mv: missing destination file operand");
        return Ok(1);
    }

    let dest = paths[paths.len() - 1];
    let sources = &paths[..paths.len() - 1];
    let dest_path = Path::new(dest);

    let mut had_error = false;

    if sources.len() > 1 || dest_path.is_dir() {
        for src in sources {
            let src_path = Path::new(src);
            let file_name = src_path.file_name().unwrap_or_default();
            let target = dest_path.join(file_name);
            if let Err(e) = move_path(src_path, &target, interactive) {
                eprintln!("mv: cannot move '{src}': {e}");
                had_error = true;
            }
        }
    } else if sources.len() == 1 {
        if let Err(e) = move_path(Path::new(sources[0]), dest_path, interactive) {
            eprintln!("mv: cannot move '{}': {e}", sources[0]);
            had_error = true;
        }
    }

    Ok(if had_error { 1 } else { 0 })
}

fn move_path(src: &Path, dest: &Path, interactive: bool) -> std::io::Result<()> {
    if dest.exists() && interactive {
        eprint!("mv: overwrite '{}'? (y/n) ", dest.display());
        let mut answer = String::new();
        std::io::stdin().read_line(&mut answer)?;
        if !answer.starts_with('y') && !answer.starts_with('Y') {
            return Ok(());
        }
    }

    if std::fs::rename(src, dest).is_ok() {
        return Ok(());
    }

    std::fs::copy(src, dest)?;
    std::fs::remove_file(src)?;
    Ok(())
}
