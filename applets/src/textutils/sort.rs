use proteus_core::ProteusResult;

use super::read_all_lines;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut reverse = false;
    let mut unique = false;
    let mut files = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-r" => reverse = true,
            "-u" => unique = true,
            "--" => continue,
            value if value.starts_with('-') => {
                eprintln!("sort: invalid option -- '{value}'");
                return Ok(2);
            }
            value => files.push(value.to_string()),
        }
    }

    let mut lines = read_all_lines(&files)?;
    lines.sort();

    if reverse {
        lines.reverse();
    }

    let mut previous: Option<&str> = None;
    for line in &lines {
        if unique && previous == Some(line.as_str()) {
            continue;
        }
        println!("{line}");
        previous = Some(line.as_str());
    }

    Ok(0)
}
