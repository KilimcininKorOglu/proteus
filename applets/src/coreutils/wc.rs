use std::io::{self, Read};
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut count_lines = false;
    let mut count_words = false;
    let mut count_bytes = false;
    let mut count_chars = false;
    let mut files: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-l" | "--lines" => count_lines = true,
            "-w" | "--words" => count_words = true,
            "-c" | "--bytes" => count_bytes = true,
            "-m" | "--chars" => count_chars = true,
            "--" => {
                files.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'l' => count_lines = true,
                        'w' => count_words = true,
                        'c' => count_bytes = true,
                        'm' => count_chars = true,
                        _ => {
                            eprintln!("wc: invalid option -- '{c}'");
                            return Ok(1);
                        }
                    }
                }
            }
            _ => files.push(arg),
        }
    }

    let show_lines = !count_bytes && !count_chars && !count_words && !count_lines || count_lines;
    let show_words = !count_bytes && !count_chars && !count_words && !count_lines || count_words;
    let show_bytes_chars = if count_chars {
        (false, true)
    } else if count_bytes {
        (true, false)
    } else {
        (true, false)
    };

    if files.is_empty() {
        files.push("-");
    }

    let mut total_lines = 0usize;
    let mut total_words = 0usize;
    let mut total_bytes = 0usize;
    let mut total_chars = 0usize;
    let mut had_error = false;

    for file in &files {
        let result = if *file == "-" {
            wc_reader(&mut io::stdin().lock())
        } else {
            match std::fs::File::open(file) {
                Ok(f) => wc_reader(&mut io::BufReader::new(f)),
                Err(e) => {
                    eprintln!("wc: {file}: {e}");
                    had_error = true;
                    continue;
                }
            }
        };

        match result {
            Ok((lines, words, bytes, chars)) => {
                total_lines += lines;
                total_words += words;
                total_bytes += bytes;
                total_chars += chars;
                print_counts(
                    lines,
                    words,
                    bytes,
                    chars,
                    show_lines,
                    show_words,
                    show_bytes_chars,
                    if *file == "-" { None } else { Some(file) },
                );
            }
            Err(_) => {
                had_error = true;
            }
        }
    }

    if files.len() > 1 {
        print_counts(
            total_lines,
            total_words,
            total_bytes,
            total_chars,
            show_lines,
            show_words,
            show_bytes_chars,
            Some("total"),
        );
    }

    Ok(if had_error { 1 } else { 0 })
}

fn wc_reader<R: Read>(reader: &mut R) -> io::Result<(usize, usize, usize, usize)> {
    let mut buf = [0u8; 8192];
    let mut lines = 0usize;
    let mut words = 0usize;
    let mut bytes = 0usize;
    let mut chars = 0usize;
    let mut in_word = false;

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        bytes += n;

        for &byte in &buf[..n] {
            if byte < 0x80 {
                chars += 1;
            }
            let ch = byte as char;
            if ch.is_whitespace() {
                if in_word {
                    words += 1;
                    in_word = false;
                }
                if ch == '\n' {
                    lines += 1;
                }
            } else {
                in_word = true;
            }
        }
    }

    if in_word {
        words += 1;
    }

    Ok((lines, words, bytes, chars))
}

fn print_counts(
    lines: usize,
    words: usize,
    bytes: usize,
    chars: usize,
    show_lines: bool,
    show_words: bool,
    (show_bytes, show_chars): (bool, bool),
    file_name: Option<&str>,
) {
    let mut parts = Vec::new();
    if show_lines {
        parts.push(format!("{:>8}", lines));
    }
    if show_words {
        parts.push(format!("{:>8}", words));
    }
    if show_bytes {
        parts.push(format!("{:>8}", bytes));
    }
    if show_chars {
        parts.push(format!("{:>8}", chars));
    }
    let mut output = parts.join(" ");
    if let Some(name) = file_name {
        output.push_str(&format!(" {name}"));
    }
    println!("{output}");
}
