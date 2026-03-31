use std::process::Command;

use proteus_core::ProteusResult;

use super::for_each_input_path;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    let mut null_delimited = false;
    let mut command = Vec::new();

    for arg in args {
        match arg.as_str() {
            "-0" => null_delimited = true,
            value => command.push(value.to_string()),
        }
    }

    if command.is_empty() {
        command.push("echo".to_string());
    }

    let mut inputs = Vec::new();
    for_each_input_path(&[], |reader, _path| {
        let mut buffer = String::new();
        loop {
            buffer.clear();
            let bytes = if null_delimited {
                let mut bytes = Vec::new();
                let read = reader.read_until(0, &mut bytes)?;
                if read == 0 {
                    0
                } else {
                    let item = String::from_utf8_lossy(&bytes).trim_end_matches('\0').trim().to_string();
                    if !item.is_empty() {
                        inputs.push(item);
                    }
                    read
                }
            } else {
                let read = reader.read_line(&mut buffer)?;
                if read > 0 {
                    let item = buffer.trim();
                    if !item.is_empty() {
                        inputs.push(item.to_string());
                    }
                }
                read
            };

            if bytes == 0 {
                break;
            }
        }
        Ok(())
    })?;

    let program = &command[0];
    let status = Command::new(program)
        .args(&command[1..])
        .args(&inputs)
        .status()
        .map_err(|error| format!("xargs: failed to execute {program}: {error}"))?;

    Ok(status.code().unwrap_or(1))
}
