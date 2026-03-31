use std::path::Path;

use proteus_core::ProteusResult;

use super::{normalize_path, walk_paths};

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut start_paths = Vec::new();
    let mut name_filter: Option<String> = None;
    let mut type_filter: Option<char> = None;

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-name" => {
                let Some(pattern) = iter.next() else {
                    eprintln!("find: missing argument to -name");
                    return Ok(2);
                };
                name_filter = Some(pattern.clone());
            }
            "-type" => {
                let Some(kind) = iter.next() else {
                    eprintln!("find: missing argument to -type");
                    return Ok(2);
                };
                type_filter = kind.chars().next();
            }
            value if value.starts_with('-') => {
                eprintln!("find: unsupported option '{value}'");
                return Ok(2);
            }
            value => start_paths.push(value.to_string()),
        }
    }

    let paths = walk_paths(&start_paths)?;
    for path in paths {
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                eprintln!("find: {}: {error}", path.display());
                continue;
            }
        };

        if let Some(kind) = type_filter {
            let matches_type = match kind {
                'f' => metadata.is_file(),
                'd' => metadata.is_dir(),
                _ => false,
            };
            if !matches_type {
                continue;
            }
        }

        if let Some(pattern) = &name_filter {
            let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
            if !matches_name(file_name, pattern) {
                continue;
            }
        }

        println!("{}", normalize_path(Path::new(&path)));
    }

    Ok(0)
}

fn matches_name(candidate: &str, pattern: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some(stripped) = pattern.strip_prefix('*') {
        return candidate.ends_with(stripped);
    }
    if let Some(stripped) = pattern.strip_suffix('*') {
        return candidate.starts_with(stripped);
    }
    candidate == pattern
}
