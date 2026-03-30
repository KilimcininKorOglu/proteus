use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut newline = true;
    let mut interpret_escapes = false;
    let mut iter = args.iter().peekable();

    while let Some(arg) = iter.peek() {
        match arg.as_str() {
            "-n" => {
                newline = false;
                iter.next();
            }
            "-e" => {
                interpret_escapes = true;
                iter.next();
            }
            "-ne" | "-en" => {
                newline = false;
                interpret_escapes = true;
                iter.next();
            }
            _ => break,
        }
    }

    let output: Vec<String> = iter
        .map(|s| {
            if interpret_escapes {
                interpret_escape_sequences(s)
            } else {
                s.clone()
            }
        })
        .collect();

    let result = output.join(" ");
    if newline {
        println!("{result}");
    } else {
        print!("{result}");
    }

    Ok(0)
}

fn interpret_escape_sequences(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('a') => result.push('\u{07}'),
                Some('b') => result.push('\u{08}'),
                Some('c') => break,
                Some('f') => result.push('\u{0C}'),
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('v') => result.push('\u{0B}'),
                Some('\\') => result.push('\\'),
                Some('0') => {
                    let mut octal = String::new();
                    for _ in 0..3 {
                        match chars.peek() {
                            Some(&d) if ('0'..='7').contains(&d) => {
                                octal.push(chars.next().unwrap());
                            }
                            _ => break,
                        }
                    }
                    if let Ok(byte) = u8::from_str_radix(&octal, 8) {
                        result.push(byte as char);
                    }
                }
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}
