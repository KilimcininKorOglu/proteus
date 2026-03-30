use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut recursive = false;
    let mut force = false;
    let mut interactive = false;
    let mut paths: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-r" | "-R" | "--recursive" => recursive = true,
            "-f" | "--force" => force = true,
            "-i" | "--interactive" => interactive = true,
            "--" => {
                paths.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'r' | 'R' => recursive = true,
                        'f' => force = true,
                        'i' => interactive = true,
                        _ => {
                            eprintln!("rm: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            _ => paths.push(arg),
        }
    }

    if paths.is_empty() && !force {
        eprintln!("rm: missing operand");
        return Ok(1);
    }

    let mut had_error = false;

    for path_str in &paths {
        let path = Path::new(path_str);
        if let Err(e) = remove_path(path, recursive, force, interactive) {
            if !force {
                eprintln!("rm: cannot remove '{}': {e}", path.display());
                had_error = true;
            }
        }
    }

    Ok(if had_error { 1 } else { 0 })
}

fn remove_path(
    path: &Path,
    recursive: bool,
    force: bool,
    interactive: bool,
) -> std::io::Result<()> {
    let meta = std::fs::symlink_metadata(path)?;

    if meta.is_dir() {
        if !recursive {
            eprintln!("rm: cannot remove '{}': Is a directory", path.display());
            return Err(std::io::Error::new(
                std::io::ErrorKind::IsADirectory,
                "is a directory",
            ));
        }
        remove_dir_recursive(path, force, interactive)
    } else {
        if interactive && !force {
            eprint!("rm: remove file '{}'? (y/n) ", path.display());
            let mut answer = String::new();
            std::io::stdin().read_line(&mut answer)?;
            if !answer.starts_with('y') && !answer.starts_with('Y') {
                return Ok(());
            }
        }
        std::fs::remove_file(path)
    }
}

fn remove_dir_recursive(
    dir: &Path,
    force: bool,
    interactive: bool,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let meta = entry.metadata()?;

        if meta.is_dir() {
            remove_dir_recursive(&path, force, interactive)?;
        } else {
            if interactive && !force {
                eprint!("rm: remove file '{}'? (y/n) ", path.display());
                let mut answer = String::new();
                std::io::stdin().read_line(&mut answer)?;
                if !answer.starts_with('y') && !answer.starts_with('Y') {
                    continue;
                }
            }
            std::fs::remove_file(&path)?;
        }
    }

    std::fs::remove_dir(dir)
}
