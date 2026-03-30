use std::io::{self, Read, Write};
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut number_lines = false;
    let mut squeeze_blank = false;
    let mut show_ends = false;
    let mut show_tabs = false;
    let mut _show_all = false;
    let mut files: Vec<&str> = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-n" => number_lines = true,
            "-b" => {
                number_lines = true;
                squeeze_blank = true;
            }
            "-s" => squeeze_blank = true,
            "-E" => show_ends = true,
            "-T" => show_tabs = true,
            "-A" => {
                _show_all = true;
                show_ends = true;
                show_tabs = true;
            }
            "-e" => {
                show_ends = true;
            }
            "-t" => {
                show_tabs = true;
            }
            "--" => {
                files.extend(iter.by_ref().map(|s| s.as_str()));
            }
            s if s.starts_with('-') && s.len() > 1 => {
                for c in s[1..].chars() {
                    match c {
                        'n' => number_lines = true,
                        'b' => {
                            number_lines = true;
                            squeeze_blank = true;
                        }
                        's' => squeeze_blank = true,
                        'E' => show_ends = true,
                        'T' => show_tabs = true,
                        'A' => {
                            _show_all = true;
                            show_ends = true;
                            show_tabs = true;
                        }
                        'e' => show_ends = true,
                        't' => show_tabs = true,
                        _ => {
                            eprintln!("cat: invalid option -- '{c}'");
                            return Ok(1);
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

    let mut line_number = 1;
    let mut last_was_blank = false;
    let mut had_error = false;

    for file in &files {
        let result = if *file == "-" {
            cat_file(
                &mut io::stdin().lock(),
                number_lines,
                squeeze_blank,
                show_ends,
                show_tabs,
                &mut line_number,
                &mut last_was_blank,
            )
        } else {
            match std::fs::File::open(file) {
                Ok(mut f) => cat_file(
                    &mut f,
                    number_lines,
                    squeeze_blank,
                    show_ends,
                    show_tabs,
                    &mut line_number,
                    &mut last_was_blank,
                ),
                Err(e) => {
                    eprintln!("cat: {file}: {e}");
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

fn cat_file<R: Read>(
    reader: &mut R,
    number_lines: bool,
    squeeze_blank: bool,
    show_ends: bool,
    show_tabs: bool,
    line_number: &mut usize,
    last_was_blank: &mut bool,
) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let mut buf = [0u8; 8192];
    let mut line_buf = Vec::new();

    loop {
        let n = reader.read(&mut buf)?;
        if n == 0 {
            if !line_buf.is_empty() {
                output_line(
                    &line_buf,
                    &mut out,
                    number_lines,
                    squeeze_blank,
                    show_ends,
                    show_tabs,
                    line_number,
                    last_was_blank,
                )?;
            }
            break;
        }

        for &byte in &buf[..n] {
            if byte == b'\n' {
                output_line(
                    &line_buf,
                    &mut out,
                    number_lines,
                    squeeze_blank,
                    show_ends,
                    show_tabs,
                    line_number,
                    last_was_blank,
                )?;
                line_buf.clear();
            } else {
                line_buf.push(byte);
            }
        }
    }

    Ok(())
}

fn output_line<W: Write>(
    line: &[u8],
    out: &mut W,
    number_lines: bool,
    squeeze_blank: bool,
    show_ends: bool,
    show_tabs: bool,
    line_number: &mut usize,
    last_was_blank: &mut bool,
) -> io::Result<()> {
    let is_blank = line.is_empty();

    if squeeze_blank && is_blank && *last_was_blank {
        return Ok(());
    }

    *last_was_blank = is_blank;

    if number_lines && !(is_blank && squeeze_blank) {
        write!(out, "{:6}\t", line_number)?;
        *line_number += 1;
    }

    for &byte in line {
        if show_tabs && byte == b'\t' {
            write!(out, "^I")?;
        } else if byte.is_ascii_control() && byte != b'\t' {
            write!(out, "^{}", (byte + 64) as char)?;
        } else {
            out.write_all(&[byte])?;
        }
    }

    if show_ends {
        write!(out, "$")?;
    }

    writeln!(out)?;
    Ok(())
}
