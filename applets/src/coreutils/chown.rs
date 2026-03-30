use std::ffi::CString;
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut recursive = false;
    let mut owner_spec: Option<&str> = None;
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
                            eprintln!("chown: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            s if owner_spec.is_none() && files.is_empty() => {
                owner_spec = Some(s);
            }
            _ => files.push(arg),
        }
    }

    let Some(spec) = owner_spec else {
        eprintln!("chown: missing operand");
        return Ok(1);
    };

    if files.is_empty() {
        eprintln!("chown: missing operand after '{spec}'");
        return Ok(1);
    }

    let (uid, gid) = parse_owner_spec(spec);
    let mut had_error = false;

    for file in &files {
        let path = Path::new(file);
        if let Err(e) = chown_path(path, uid, gid, recursive) {
            eprintln!("chown: cannot access '{file}': {e}");
            had_error = true;
        }
    }

    Ok(if had_error { 1 } else { 0 })
}

fn chown_path(
    path: &Path,
    uid: Option<u32>,
    gid: Option<u32>,
    recursive: bool,
) -> std::io::Result<()> {
    let meta = std::fs::symlink_metadata(path)?;
    let current_uid = meta.uid();
    let current_gid = meta.gid();

    let target_uid = uid.unwrap_or(current_uid);
    let target_gid = gid.unwrap_or(current_gid);

    let c_path = CString::new(path.to_string_lossy().into_owned())
        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidInput, "path contains null"))?;

    unsafe {
        if libc::chown(c_path.as_ptr(), target_uid, target_gid) != 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    if recursive && path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            chown_path(&entry.path(), uid, gid, true)?;
        }
    }

    Ok(())
}

fn parse_owner_spec(spec: &str) -> (Option<u32>, Option<u32>) {
    if let Some(colon_pos) = spec.find(':') {
        let uid_part = &spec[..colon_pos];
        let gid_part = &spec[colon_pos + 1..];
        let uid = if uid_part.is_empty() { None } else { uid_part.parse().ok() };
        let gid = if gid_part.is_empty() { None } else { gid_part.parse().ok() };
        (uid, gid)
    } else {
        (spec.parse().ok(), None)
    }
}
