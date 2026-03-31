use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let Some(format) = args.first() else {
        return Ok(0);
    };

    let mut output = String::new();
    let mut arg_index = 1usize;
    let mut chars = format.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            output.push(ch);
            continue;
        }

        let Some(spec) = chars.next() else {
            output.push('%');
            break;
        };
        match spec {
            '%' => output.push('%'),
            's' => {
                let value = args.get(arg_index).map(String::as_str).unwrap_or("");
                output.push_str(value);
                arg_index += 1;
            }
            'd' => {
                let value = args.get(arg_index).and_then(|value| value.parse::<i64>().ok()).unwrap_or(0);
                output.push_str(&value.to_string());
                arg_index += 1;
            }
            'b' => {
                let value = args.get(arg_index).map(String::as_str).unwrap_or("");
                output.push_str(&interpret_escapes(value));
                arg_index += 1;
            }
            other => {
                output.push('%');
                output.push(other);
            }
        }
    }

    print!("{output}");
    Ok(0)
}

fn interpret_escapes(input: &str) -> String {
    let mut output = String::new();
    let mut chars = input.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            output.push(ch);
            continue;
        }

        match chars.next() {
            Some('n') => output.push('\n'),
            Some('t') => output.push('\t'),
            Some('r') => output.push('\r'),
            Some('\\') => output.push('\\'),
            Some(other) => {
                output.push('\\');
                output.push(other);
            }
            None => output.push('\\'),
        }
    }
    output
}
