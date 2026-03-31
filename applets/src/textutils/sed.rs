use proteus_core::ProteusResult;

use super::{for_each_input, strip_line_ending};

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut script: Option<String> = None;
    let mut files = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-e" => {
                let Some(value) = iter.next() else {
                    eprintln!("sed: option requires an argument -- 'e'");
                    return Ok(2);
                };
                script = Some(value.clone());
            }
            value if script.is_none() => script = Some(value.to_string()),
            value => files.push(value.to_string()),
        }
    }

    let Some(script) = script else {
        eprintln!("sed: missing script");
        return Ok(2);
    };

    let command = match parse_substitute_command(&script) {
        Some(command) => command,
        None => {
            eprintln!("sed: only simple substitute scripts are currently supported");
            return Ok(2);
        }
    };

    for_each_input(&files, |reader, _file_name| {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }

            let had_newline = line.ends_with('\n');
            let content = strip_line_ending(&line);
            let replaced = apply_substitute(content, &command);
            if had_newline {
                println!("{replaced}");
            } else {
                print!("{replaced}");
            }
        }
        Ok(())
    })?;

    Ok(0)
}

#[derive(Debug, Clone)]
struct SubstituteCommand {
    pattern: String,
    replacement: String,
    global: bool,
}

fn parse_substitute_command(script: &str) -> Option<SubstituteCommand> {
    let mut chars = script.chars();
    if chars.next()? != 's' {
        return None;
    }

    let delimiter = chars.next()?;
    let rest: String = chars.collect();
    let mut parts = rest.split(delimiter);
    let pattern = parts.next()?.to_string();
    let replacement = parts.next()?.to_string();
    let flags = parts.next().unwrap_or_default();

    if parts.next().is_some() {
        return None;
    }

    Some(SubstituteCommand {
        pattern,
        replacement,
        global: flags.contains('g'),
    })
}

fn apply_substitute(input: &str, command: &SubstituteCommand) -> String {
    if command.pattern.is_empty() {
        return input.to_string();
    }

    if command.global {
        input.replace(&command.pattern, &command.replacement)
    } else {
        input.replacen(&command.pattern, &command.replacement, 1)
    }
}
