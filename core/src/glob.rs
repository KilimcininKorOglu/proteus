use std::path::Path;

#[derive(Debug, Clone, PartialEq)]
pub enum GlobError {
    InvalidPattern(String),
}

pub fn matches(pattern: &str, path: &str) -> Result<bool, GlobError> {
    let name = Path::new(path)
        .file_name()
        .map(|n| n.to_str().unwrap_or(""))
        .unwrap_or(path);

    do_match(pattern.as_bytes(), name.as_bytes())
}

pub fn matches_path(pattern: &str, path: &str) -> Result<bool, GlobError> {
    do_match(pattern.as_bytes(), path.as_bytes())
}

fn do_match(pattern: &[u8], input: &[u8]) -> Result<bool, GlobError> {
    if pattern.is_empty() {
        return Ok(input.is_empty());
    }

    let p = pattern[0];

    match p {
        b'*' => {
            let rest = &pattern[1..];
            if rest.is_empty() {
                return Ok(true);
            }
            for i in 0..=input.len() {
                if do_match(rest, &input[i..])? {
                    return Ok(true);
                }
            }
            Ok(false)
        }
        b'?' => {
            if input.is_empty() {
                return Ok(false);
            }
            do_match(&pattern[1..], &input[1..])
        }
        b'[' => {
            if input.is_empty() {
                return Ok(false);
            }
            let close = find_close_bracket(&pattern[1..]);
            let Some(close_pos) = close else {
                return Err(GlobError::InvalidPattern("unterminated bracket expression".to_string()));
            };

            let bracket_pattern = &pattern[1..close_pos + 1];
            let c = input[0];
            let matched = match_bracket(bracket_pattern, c);

            if matched {
                do_match(&pattern[close_pos + 2..], &input[1..])
            } else {
                Ok(false)
            }
        }
        b'\\' => {
            if pattern.len() < 2 {
                return Err(GlobError::InvalidPattern("trailing backslash".to_string()));
            }
            if input.is_empty() || input[0] != pattern[1] {
                return Ok(false);
            }
            do_match(&pattern[2..], &input[1..])
        }
        _ => {
            if input.is_empty() || input[0] != p {
                return Ok(false);
            }
            do_match(&pattern[1..], &input[1..])
        }
    }
}

fn find_close_bracket(pattern: &[u8]) -> Option<usize> {
    for (i, &b) in pattern.iter().enumerate() {
        if b == b']' {
            return Some(i);
        }
    }
    None
}

fn match_bracket(pattern: &[u8], c: u8) -> bool {
    if pattern.is_empty() {
        return false;
    }

    let negate = pattern[0] == b'!' || pattern[0] == b'^';
    let start = if negate { 1 } else { 0 };

    let mut matched = false;
    let mut i = start;

    while i < pattern.len() {
        if i + 2 < pattern.len() && pattern[i + 1] == b'-' {
            let lo = pattern[i];
            let hi = pattern[i + 2];
            if c >= lo && c <= hi {
                matched = true;
                break;
            }
            i += 3;
        } else {
            if pattern[i] == c {
                matched = true;
                break;
            }
            i += 1;
        }
    }

    if negate { !matched } else { matched }
}
