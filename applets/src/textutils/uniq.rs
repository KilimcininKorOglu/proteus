use proteus_core::ProteusResult;

use super::{for_each_input, strip_line_ending};

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut count = false;
    let mut repeated_only = false;
    let mut unique_only = false;
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-c" => count = true,
            "-d" => repeated_only = true,
            "-u" => unique_only = true,
            value if value.starts_with('-') => {
                eprintln!("uniq: invalid option -- '{value}'");
                return Ok(2);
            }
            value => files.push(value.to_string()),
        }
    }

    let mut previous = String::new();
    let mut seen_any = false;
    let mut occurrences = 0usize;

    for_each_input(&files, |reader, _file_name| {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }

            let current = strip_line_ending(&line);
            if seen_any && current == previous {
                occurrences += 1;
                continue;
            }

            if seen_any {
                print_group(&previous, occurrences, count, repeated_only, unique_only);
            }

            previous.clear();
            previous.push_str(current);
            seen_any = true;
            occurrences = 1;
        }

        Ok(())
    })?;

    if seen_any {
        print_group(&previous, occurrences, count, repeated_only, unique_only);
    }

    Ok(0)
}

fn print_group(line: &str, occurrences: usize, count: bool, repeated_only: bool, unique_only: bool) {
    if repeated_only && occurrences < 2 {
        return;
    }
    if unique_only && occurrences != 1 {
        return;
    }

    if count {
        println!("{:>4} {line}", occurrences);
    } else {
        println!("{line}");
    }
}
