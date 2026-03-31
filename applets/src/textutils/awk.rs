use proteus_core::ProteusResult;

use super::{for_each_input, strip_line_ending};

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let Some(program) = args.first() else {
        eprintln!("awk: missing program");
        return Ok(2);
    };
    let files = args[1..].to_vec();

    let action = match parse_program(program) {
        Some(action) => action,
        None => {
            eprintln!("awk: only print-oriented programs are currently supported");
            return Ok(2);
        }
    };

    for_each_input(&files, |reader, _file_name| {
        let mut line = String::new();
        let mut line_number = 0usize;
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            line_number += 1;
            let content = strip_line_ending(&line);
            execute_action(content, line_number, &action);
        }
        Ok(())
    })?;

    Ok(0)
}

#[derive(Debug, Clone)]
enum AwkAction {
    PrintWholeLine,
    PrintField(usize),
    PrintLineNumber,
}

fn parse_program(program: &str) -> Option<AwkAction> {
    let normalized = program.trim();
    match normalized {
        "{print}" => Some(AwkAction::PrintWholeLine),
        "{print NR}" => Some(AwkAction::PrintLineNumber),
        _ if normalized.starts_with("{print $") && normalized.ends_with('}') => {
            let field = normalized
                .trim_start_matches("{print $")
                .trim_end_matches('}')
                .trim()
                .parse()
                .ok()?;
            Some(AwkAction::PrintField(field))
        }
        _ => None,
    }
}

fn execute_action(line: &str, line_number: usize, action: &AwkAction) {
    match action {
        AwkAction::PrintWholeLine => println!("{line}"),
        AwkAction::PrintField(field_index) => {
            let field = line.split_whitespace().nth(field_index.saturating_sub(1)).unwrap_or_default();
            println!("{field}");
        }
        AwkAction::PrintLineNumber => println!("{line_number}"),
    }
}
