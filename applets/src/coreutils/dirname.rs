use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut paths: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--" => continue,
            s if s.starts_with('-') && s.len() > 1 => {
                eprintln!("dirname: invalid option -- '{}'", &s[1..]);
                return Ok(1);
            }
            _ => paths.push(arg),
        }
    }

    if paths.is_empty() {
        eprintln!("dirname: missing operand");
        return Ok(1);
    }

    for path in &paths {
        let p = Path::new(path);
        let dir = p
            .parent()
            .map(|d| d.to_string_lossy().into_owned())
            .unwrap_or_else(|| ".".to_string());
        let result = if dir.is_empty() { "." } else { &dir };
        println!("{result}");
    }

    Ok(0)
}
