use proteus_core::ProteusResult;

pub fn run(args: &[String]) -> ProteusResult<i32> {
    if args.is_empty() {
        let mut vars = std::env::vars().collect::<Vec<_>>();
        vars.sort_by(|a, b| a.0.cmp(&b.0));
        for (key, value) in vars {
            println!("{key}={value}");
        }
        return Ok(0);
    }

    let mut command_index = 0usize;
    while command_index < args.len() {
        if let Some((key, value)) = args[command_index].split_once('=') {
            unsafe {
                std::env::set_var(key, value);
            }
            command_index += 1;
        } else {
            break;
        }
    }

    if command_index >= args.len() {
        return Ok(0);
    }

    let command = &args[command_index];
    let status = std::process::Command::new(command)
        .args(&args[command_index + 1..])
        .status()?;
    Ok(status.code().unwrap_or(1))
}
