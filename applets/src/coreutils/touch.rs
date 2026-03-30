use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut no_create = false;
    let mut files: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-c" | "--no-create" => no_create = true,
            "--" => continue,
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    if c == 'c' {
                        no_create = true;
                    } else {
                        eprintln!("touch: invalid option -- '{c}'");
                        return Ok(1);
                    }
                }
            }
            _ => files.push(arg),
        }
    }

    if files.is_empty() {
        eprintln!("touch: missing file operand");
        return Ok(1);
    }

    let mut had_error = false;

    for file in &files {
        let path = Path::new(file);
        if !path.exists() {
            if !no_create {
                if std::fs::File::create(path).is_err() {
                    eprintln!("touch: cannot touch '{file}'");
                    had_error = true;
                }
            }
        } else {
            let c_path = std::ffi::CString::new(path.to_string_lossy().into_owned()).unwrap();
            let now = std::time::SystemTime::now();
            let since_epoch = now.duration_since(std::time::UNIX_EPOCH).unwrap_or_default();
            let secs = since_epoch.as_secs() as libc::time_t;
            let nsecs = since_epoch.subsec_nanos() as libc::c_long;

            let times = [
                libc::timeval {
                    tv_sec: secs,
                    tv_usec: (nsecs / 1000) as libc::suseconds_t,
                },
                libc::timeval {
                    tv_sec: secs,
                    tv_usec: (nsecs / 1000) as libc::suseconds_t,
                },
            ];

            unsafe {
                if libc::utimes(c_path.as_ptr(), times.as_ptr()) != 0 {
                    eprintln!("touch: cannot touch '{file}'");
                    had_error = true;
                }
            }
        }
    }

    Ok(if had_error { 1 } else { 0 })
}
