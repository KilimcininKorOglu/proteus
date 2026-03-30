use std::io::{self, BufRead, Write};
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut number_lines = false;
    let mut count: usize = 10;
    let mut files: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-n" => {
                if let Some(n) = iter.next() {
                    count = match n.parse() {
                        Ok(v) => v,
                        Err(_) => {
                            eprintln!("head: invalid number of lines: '{n}'");
                            return Ok(1);
                        }
                    };
                } else {
                    eprintln!("head: option requires an argument -- 'n'");
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
                            'n' => number_lines = true,
                            _ => {
                                eprintln!("head: invalid option -- '{c}'");
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
            head_reader(&mut io::stdin().lock(), count, number_lines)
        } else {
            match std::fs::File::open(file) {
                Ok(f) => head_reader(&mut io::BufReader::new(f), count, number_lines),
                Err(e) => {
                    eprintln!("head: cannot open '{file}': {e}");
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

fn head_reader<R: BufRead>(reader: &mut R, count: usize, number_lines: bool) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut line_num = 1;

    let mut line = String::new();
    for _ in 0..count {
        line.clear();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            break;
        }
        if number_lines {
            write!(out, "{:>6}\t{line}", line_num)?;
        } else {
            out.write_all(line.as_bytes())?;
        }
        line_num += 1;
    }

    Ok(())
}
