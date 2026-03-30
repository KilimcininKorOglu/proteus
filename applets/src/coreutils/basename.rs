use std::path::Path;
use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut paths: Vec<&str> = Vec::new();

    for arg in args {
        match arg.as_str() {
            "--" => continue,
            s if s.starts_with('-') && s.len() > 1 => {
                for _c in s[1..].chars() {
                    eprintln!("basename: invalid option -- '{}'", &s[1..]);
                    return Ok(1);
                }
            }
            _ => paths.push(arg),
        }
    }

    if paths.is_empty() {
        eprintln!("basename: missing operand");
        return Ok(1);
    }

    let path = paths[0];
    let suffix = paths.get(1).copied();

    let p = Path::new(path);
    let name = p
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| path.to_string());

    let result = match suffix {
        Some(suf) if name.ends_with(suf) && name != suf => &name[..name.len() - suf.len()],
        _ => &name,
    };

    println!("{result}");
    Ok(0)
}
