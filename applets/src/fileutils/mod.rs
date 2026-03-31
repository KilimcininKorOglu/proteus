use std::fs::File;
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};

use proteus_core::{ProteusError, ProteusResult};

#[cfg(feature = "find")]
pub mod find;
#[cfg(feature = "xargs")]
pub mod xargs;
#[cfg(feature = "tar")]
pub mod tar;
#[cfg(feature = "gzip")]
pub mod gzip;

pub fn for_each_input_path<F>(paths: &[String], mut callback: F) -> ProteusResult<()>
where
    F: FnMut(&mut dyn BufRead, Option<&str>) -> ProteusResult<()>,
{
    if paths.is_empty() {
        let stdin = io::stdin();
        let mut reader = stdin.lock();
        callback(&mut reader, None)?;
        return Ok(());
    }

    for path in paths {
        if path == "-" {
            let stdin = io::stdin();
            let mut reader = stdin.lock();
            callback(&mut reader, None)?;
            continue;
        }

        let file = File::open(path).map_err(|error| ProteusError::Other(format!("{path}: {error}")))?;
        let mut reader = BufReader::new(file);
        callback(&mut reader, Some(path.as_str()))?;
    }

    Ok(())
}

pub fn read_all_bytes(paths: &[String]) -> ProteusResult<Vec<u8>> {
    let mut output = Vec::new();
    for_each_input_path(paths, |reader, _path| {
        reader.read_to_end(&mut output)?;
        Ok(())
    })?;
    Ok(output)
}

pub fn walk_paths(start_paths: &[String]) -> ProteusResult<Vec<PathBuf>> {
    let mut pending: Vec<PathBuf> = if start_paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        start_paths.iter().map(PathBuf::from).collect()
    };
    let mut visited = Vec::new();

    while let Some(path) = pending.pop() {
        visited.push(path.clone());
        let metadata = match std::fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) => {
                return Err(ProteusError::Other(format!("{}: {error}", path.display())));
            }
        };

        if metadata.is_dir() {
            let mut entries = std::fs::read_dir(&path)
                .map_err(|error| ProteusError::Other(format!("{}: {error}", path.display())))?
                .filter_map(Result::ok)
                .map(|entry| entry.path())
                .collect::<Vec<_>>();
            entries.sort();
            entries.reverse();
            pending.extend(entries);
        }
    }

    Ok(visited)
}

pub fn copy_stream<R: Read, W: Write>(reader: &mut R, writer: &mut W) -> io::Result<u64> {
    let mut total = 0u64;
    let mut buffer = [0u8; 8192];
    loop {
        let read = reader.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        writer.write_all(&buffer[..read])?;
        total += read as u64;
    }
    Ok(total)
}

pub fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}
