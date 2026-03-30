use std::io;
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
                            eprintln!("cp: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            _ => paths.push(arg),
        }
    }

    if paths.len() < 2 {
        eprintln!("cp: missing destination file operand");
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
            if let Err(e) = copy_path(src_path, &target, recursive, force, interactive) {
                eprintln!("cp: cannot copy '{src}': {e}");
                had_error = true;
            }
        }
    } else if sources.len() == 1 {
        if let Err(e) = copy_path(
            Path::new(sources[0]),
            dest_path,
            recursive,
            force,
            interactive,
        ) {
            eprintln!("cp: cannot copy '{}': {e}", sources[0]);
            had_error = true;
        }
    }

    Ok(if had_error { 1 } else { 0 })
}

fn copy_path(
    src: &Path,
    dest: &Path,
    recursive: bool,
    force: bool,
    interactive: bool,
) -> io::Result<()> {
    let src_meta = std::fs::symlink_metadata(src)?;

    if src_meta.is_dir() {
        if !recursive {
            eprintln!("cp: omitting directory '{}'", src.display());
            return Ok(());
        }
        copy_dir(src, dest, force, interactive)
    } else {
        copy_file(src, dest, force, interactive)
    }
}

fn copy_file(src: &Path, dest: &Path, force: bool, interactive: bool) -> io::Result<()> {
    if dest.exists() {
        if interactive && !force {
            eprint!("cp: overwrite '{}'? (y/n) ", dest.display());
            let mut answer = String::new();
            io::stdin().read_line(&mut answer)?;
            if !answer.starts_with('y') && !answer.starts_with('Y') {
                return Ok(());
            }
        }
    }

    std::fs::copy(src, dest)?;
    Ok(())
}

fn copy_dir(src: &Path, dest: &Path, force: bool, interactive: bool) -> io::Result<()> {
    if !dest.exists() {
        std::fs::create_dir(dest)?;
    }

    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let file_name = entry.file_name();
        let dest_path = dest.join(&file_name);

        let src_meta = entry.metadata()?;
        if src_meta.is_dir() {
            copy_dir(&src_path, &dest_path, force, interactive)?;
        } else {
            copy_file(&src_path, &dest_path, force, interactive)?;
        }
    }

    Ok(())
}
