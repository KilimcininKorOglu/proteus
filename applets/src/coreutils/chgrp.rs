use std::ffi::CString;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut recursive = false;
    let mut group_spec: Option<&str> = None;
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
                            eprintln!("chgrp: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            s if group_spec.is_none() && files.is_empty() => {
                group_spec = Some(s);
            }
            _ => files.push(arg),
        }
    }

    let Some(spec) = group_spec else {
        eprintln!("chgrp: missing operand");
        return Ok(1);
    };

    if files.is_empty() {
        eprintln!("chgrp: missing operand after '{spec}'");
        return Ok(1);
    }

    let gid: u32 = match spec.parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("chgrp: invalid group: '{spec}'");
            return Ok(1);
        }
    };

    let mut had_error = false;

    for file in &files {
        let path = Path::new(file);
        if let Err(e) = chgrp_path(path, gid, recursive) {
            eprintln!("chgrp: cannot access '{file}': {e}");
            had_error = true;
        }
    }

    Ok(if had_error { 1 } else { 0 })
}

fn chgrp_path(path: &Path, gid: u32, recursive: bool) -> std::io::Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    let uid = meta.uid();

    let c_path = CString::new(path.to_string_lossy().into_owned())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid path"))?;

    unsafe {
        if libc::chown(c_path.as_ptr(), uid, gid) != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    if recursive && path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            chgrp_path(&entry.path(), gid, true)?;
        }
    }

    Ok(())
}
