use proteus_core::utf8::Utf8Chars;
use proteus_core::ProteusResult;

use super::{for_each_input, strip_line_ending};

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut mode: Option<CutMode> = None;
    let mut files = Vec::new();

    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "-c" => {
                let Some(spec) = iter.next() else {
                    eprintln!("cut: option requires an argument -- 'c'");
                    return Ok(2);
                };
                mode = parse_selection(spec).map(CutMode::Characters);
            }
            "-f" => {
                let Some(spec) = iter.next() else {
                    eprintln!("cut: option requires an argument -- 'f'");
                    return Ok(2);
                };
                mode = parse_selection(spec).map(|selection| CutMode::Fields {
                    selection,
                    delimiter: '\t',
                });
            }
            "-d" => {
                let Some(value) = iter.next() else {
                    eprintln!("cut: option requires an argument -- 'd'");
                    return Ok(2);
                };
                let Some(CutMode::Fields { selection, .. }) = mode.take() else {
                    eprintln!("cut: -d requires -f");
                    return Ok(2);
                };
                let delimiter = value.chars().next().unwrap_or('\t');
                mode = Some(CutMode::Fields { selection, delimiter });
            }
            value if value.starts_with('-') => {
                eprintln!("cut: invalid option -- '{value}'");
                return Ok(2);
            }
            value => files.push(value.to_string()),
        }
    }

    let Some(mode) = mode else {
        eprintln!("cut: one of -c or -f must be specified");
        return Ok(2);
    };

    for_each_input(&files, |reader, _file_name| {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }

            let content = strip_line_ending(&line);
            match &mode {
                CutMode::Characters(selection) => println!("{}", cut_characters(content, selection)),
                CutMode::Fields { selection, delimiter } => {
                    println!("{}", cut_fields(content, selection, *delimiter))
                }
            }
        }
        Ok(())
    })?;

    Ok(0)
}

#[derive(Debug, Clone)]
enum CutMode {
    Characters(Selection),
    Fields { selection: Selection, delimiter: char },
}

#[derive(Debug, Clone)]
struct Selection {
    start: usize,
    end: Option<usize>,
}

fn parse_selection(spec: &str) -> Option<Selection> {
    if let Some((start, end)) = spec.split_once('-') {
        let start = if start.is_empty() { 1 } else { start.parse().ok()? };
        let end = if end.is_empty() { None } else { Some(end.parse().ok()?) };
        return Some(Selection { start, end });
    }

    let index = spec.parse().ok()?;
    Some(Selection {
        start: index,
        end: Some(index),
    })
}

fn cut_characters(input: &str, selection: &Selection) -> String {
    let mut chars = Utf8Chars::new(input.as_bytes());
    let mut index = 1usize;
    let mut output = String::new();

    while let Some(ch) = chars.next_char() {
        if index >= selection.start && selection.end.is_none_or(|end| index <= end) {
            output.push(ch);
        }
        index += 1;
    }

    output
}

fn cut_fields(input: &str, selection: &Selection, delimiter: char) -> String {
    let mut output = Vec::new();
    for (index, field) in input.split(delimiter).enumerate() {
        let field_index = index + 1;
        if field_index >= selection.start && selection.end.is_none_or(|end| field_index <= end) {
            output.push(field);
        }
    }
    output.join(&delimiter.to_string())
}
