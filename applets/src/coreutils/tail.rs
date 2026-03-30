use std::io::{self, BufRead, Write};
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut follow = false;
    let mut count: usize = 10;
    let mut files: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-f" | "--follow" => follow = true,
            "-n" => {
                if let Some(n) = iter.next() {
                    count = match n.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            eprintln!("tail: invalid number of lines: '{n}'");
                            return Ok(1);
                        }
                    };
                } else {
                    eprintln!("tail: option requires an argument -- 'n'");
                    return Ok(1);
                }
            }
            "--" => {
                files.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                if let Ok(n) = s[1..].parse::<usize>() {
                    count = n;
                } else {
                    for c in s[1..].chars() {
                        match c {
                            'f' => follow = true,
                            _ => {
                                eprintln!("tail: invalid option -- '{c}'");
                                return Ok(1);
                            }
                        }
                    }
                }
            }
            _ => files.push(arg),
        }
    }

    if files.is_empty() {
        files.push("-");
    }

    let mut had_error = false;

    for (i, file) in files.iter().enumerate() {
        if files.len() > 1 {
            if i > 0 {
                println!();
            }
            println!("==> {file} <==");
        }

        let result = if *file == "-" {
            tail_reader(&mut io::stdin().lock(), count, follow)
        } else {
            match std::fs::File::open(file) {
                Ok(f) => tail_reader(&mut io::BufReader::new(f), count, follow),
                Err(e) => {
                    eprintln!("tail: cannot open '{file}': {e}");
                    had_error = true;
                    continue;
                }
            }
        };

        if result.is_err() {
            had_error = true;
        }
    }

    Ok(if had_error { 1 } else { 0 })
}

fn tail_reader<R: BufRead>(reader: &mut R, count: usize, follow: bool) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();

    let mut lines: Vec<String> = Vec::new();
    let mut line = String::new();

    loop {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        lines.push(line.clone());
        if lines.len() > count {
            lines.remove(0);
        }
    }

    for l in &lines {
        out.write_all(l.as_bytes())?;
    }

    if follow {
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                continue;
            }
            out.write_all(line.as_bytes())?;
            out.flush()?;
        }
    }

    Ok(())
}
