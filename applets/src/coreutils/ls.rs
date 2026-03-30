use std::io::{self, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut show_all = false;
    let mut long_format = false;
    let mut recursive = false;
    let mut paths: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-a" | "--all" => show_all = true,
            "-l" | "--format=long" => long_format = true,
            "-R" | "--recursive" => recursive = true,
            "--" => {
                paths.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'a' => show_all = true,
                        'l' => long_format = true,
                        'R' => recursive = true,
                        _ => {
                            eprintln!("ls: invalid option -- '{c}'");
                            return Ok(2);
                        }
                    }
                }
            }
            _ => paths.push(arg),
        }
    }

    if paths.is_empty() {
        paths.push(".");
    }

    let mut had_error = false;

    for (i, path) in paths.iter().enumerate() {
        if paths.len() > 1 {
            if i > 0 {
                println!();
            }
            println!("{path}:");
        }

        if let Err(e) = list_dir(path, show_all, long_format, recursive) {
            eprintln!("ls: cannot access '{path}': {e}");
            had_error = true;
        }
    }

    Ok(if had_error { 2 } else { 0 })
}

fn list_dir(
    path: &str,
    show_all: bool,
    long_format: bool,
    recursive: bool,
) -> io::Result<()> {
    let entries = std::fs::read_dir(path)?;
    let mut names: Vec<String> = entries
        .filter_map(|e| e.ok())
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|n| show_all || !n.starts_with('.'))
        .collect();

    names.sort();

    if show_all {
        names.insert(0, ".".to_string());
        names.insert(1, "..".to_string());
    }

    if long_format {
        let mut total = 0u64;
        let mut entries_info: Vec<(String, std::fs::Metadata)> = Vec::new();
        for name in &names {
            let full_path = format!("{path}/{name}");
            match std::fs::symlink_metadata(&full_path) {
                Ok(meta) => {
                    total += (meta.len() + 1023) / 1024;
                    entries_info.push((name.clone(), meta));
                }
                Err(_) => continue,
            }
        }
        println!("total {total}");
        for (name, meta) in &entries_info {
            print_long_entry(path, name, meta);
        }
    } else {
        let stdout = io::stdout();
        let mut out = stdout.lock();
        for name in &names {
            writeln!(out, "{name}")?;
        }
    }

    if recursive {
        for name in &names {
            if name == "." || name == ".." {
                continue;
            }
            let full_path = format!("{path}/{name}");
            if std::fs::symlink_metadata(&full_path)
                .map(|m| m.is_dir())
                .unwrap_or(false)
            {
                println!();
                println!("{full_path}:");
                list_dir(&full_path, show_all, long_format, recursive)?;
            }
        }
    }

    Ok(())
}

fn print_long_entry(path: &str, name: &str, meta: &std::fs::Metadata) {
    let file_type = if meta.is_dir() {
        'd'
    } else if meta.file_type().is_symlink() {
        'l'
    } else {
        '-'
    };

    let mode = meta.permissions().mode();
    let perms = format_permissions(mode);

    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    let size = meta.len();

    let time_str = format_mtime(meta.modified().ok());

    let link_target = if meta.file_type().is_symlink() {
        std::fs::read_link(format!("{path}/{name}"))
            .map(|t| format!(" -> {}", t.display()))
            .unwrap_or_default()
    } else {
        String::new()
    };

    println!(
        "{file_type}{perms} {nlink:>3} {uid:>8} {gid:>8} {size:>8} {time_str} {name}{link_target}"
    );
}

fn format_permissions(mode: u32) -> String {
    let user = triplet(mode, 6);
    let group = triplet(mode, 3);
    let other = triplet(mode, 0);
    format!("{user}{group}{other}")
}

fn triplet(mode: u32, shift: u32) -> String {
    let r = if mode & (0o4 << shift) != 0 { 'r' } else { '-' };
    let w = if mode & (0o2 << shift) != 0 { 'w' } else { '-' };
    let x = if mode & (0o1 << shift) != 0 { 'x' } else { '-' };
    format!("{r}{w}{x}")
}

fn format_mtime(modified: Option<std::time::SystemTime>) -> String {
    let Some(time) = modified else {
        return "???".to_string();
    };

    let secs = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let days_since_epoch = secs as i64 / 86400;
    let time_of_day = secs % 86400;
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;

    let (year, month, day) = days_to_date(days_since_epoch);
    let month_name = match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    };

    let now_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64;

    let current_year = now_secs / 86400 / 365 + 1970;

    if year == current_year {
        format!("{month_name} {day:>2} {hour:02}:{minute:02}")
    } else {
        format!("{month_name} {day:>2}  {year}")
    }
}

fn days_to_date(days: i64) -> (i64, i64, i64) {
    let mut y = 1970;
    let mut remaining = days;

    loop {
        let year_len = if is_leap_year(y) { 366 } else { 365 };
        if remaining < year_len {
            break;
        }
        remaining -= year_len;
        y += 1;
    }

    let leap = is_leap_year(y);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];

    let mut m = 0;
    for &md in &month_days {
        if remaining < md {
            break;
        }
        remaining -= md;
        m += 1;
    }

    (y, (m + 1) as i64, (remaining + 1) as i64)
}

fn is_leap_year(y: i64) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}
