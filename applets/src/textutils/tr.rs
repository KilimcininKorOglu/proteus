use proteus_core::utf8::{to_lowercase, to_uppercase};
use proteus_core::ProteusResult;

use super::read_all_text;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    if args.is_empty() {
        eprintln!("tr: missing operand");
        return Ok(2);
    }

    let mut delete_mode = false;
    let mut operand_index = 0usize;
    if args[0] == "-d" {
        delete_mode = true;
        operand_index = 1;
    }

    if operand_index >= args.len() {
        eprintln!("tr: missing operand");
        return Ok(2);
    }

    let set1 = &args[operand_index];
    let set2 = if delete_mode {
        None
    } else {
        args.get(operand_index + 1).map(String::as_str)
    };
    let files = if delete_mode {
        args[(operand_index + 1)..].to_vec()
    } else if operand_index + 1 < args.len() {
        args[(operand_index + 2)..].to_vec()
    } else {
        Vec::new()
    };

    if !delete_mode && set2.is_none() {
        eprintln!("tr: missing replacement set");
        return Ok(2);
    }

    let text = read_all_text(&files)?;
    let source_chars: Vec<char> = expand_set(set1).chars().collect();

    let output = if delete_mode {
        text.chars()
            .filter(|ch| !source_chars.contains(ch))
            .collect::<String>()
    } else {
        let replacement_chars: Vec<char> = expand_set(set2.unwrap()).chars().collect();
        text.chars()
            .map(|ch| translate_char(ch, &source_chars, &replacement_chars))
            .collect::<String>()
    };

    print!("{output}");
    Ok(0)
}

fn expand_set(spec: &str) -> String {
    match spec {
        "[:lower:]" => ('a'..='z').collect(),
        "[:upper:]" => ('A'..='Z').collect(),
        _ => spec.to_string(),
    }
}

fn translate_char(ch: char, source: &[char], replacement: &[char]) -> char {
    if source == expand_set("[:lower:]").chars().collect::<Vec<_>>()
        && replacement == expand_set("[:upper:]").chars().collect::<Vec<_>>()
    {
        return to_uppercase(ch);
    }

    if source == expand_set("[:upper:]").chars().collect::<Vec<_>>()
        && replacement == expand_set("[:lower:]").chars().collect::<Vec<_>>()
    {
        return to_lowercase(ch);
    }

    if let Some(index) = source.iter().position(|candidate| *candidate == ch) {
        return replacement
            .get(index)
            .copied()
            .or_else(|| replacement.last().copied())
            .unwrap_or(ch);
    }

    ch
}
