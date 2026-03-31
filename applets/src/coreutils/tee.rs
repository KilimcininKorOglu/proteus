use std::io::{self, Read, Write};

use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let append = args.iter().any(|arg| arg == "-a");
    let files: Vec<&String> = args.iter().filter(|arg| arg.as_str() != "-a").collect();

    let mut input = Vec::new();
    io::stdin().read_to_end(&mut input)?;
    io::stdout().write_all(&input)?;

    for file in files {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create(true);
        if append {
            options.append(true);
        } else {
            options.truncate(true);
        }
        let mut handle = options.open(file)?;
        handle.write_all(&input)?;
    }

    Ok(0)
}
