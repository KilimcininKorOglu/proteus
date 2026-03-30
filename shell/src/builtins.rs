use std::process;

pub struct ShellState {
    pub args: Vec<String>,
    pub last_exit_code: i32,
}

impl ShellState {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            last_exit_code: 0,
        }
    }
}

pub fn run_builtin(state: &mut ShellState, name: &str, args: &[String]) -> Option<i32> {
    match name {
        "cd" => Some(builtin_cd(args)),
        "exit" => Some(builtin_exit(state, args)),
        "export" => Some(builtin_export(args)),
        "unset" => Some(builtin_unset(args)),
        "echo" => Some(builtin_echo(args)),
        "true" => Some(0),
        "false" => Some(1),
        "pwd" => Some(builtin_pwd()),
        _ => None,
    }
}

fn builtin_cd(args: &[String]) -> i32 {
    let target = if args.is_empty() {
        std::env::var("HOME").unwrap_or_else(|_| "/".to_string())
    } else {
        args[0].clone()
    };

    match std::env::set_current_dir(&target) {
        Ok(()) => 0,
        Err(error) => {
            eprintln!("sh: cd: {target}: {error}");
            1
        }
    }
}

fn builtin_exit(state: &mut ShellState, args: &[String]) -> i32 {
    let code = args
        .first()
        .and_then(|value| value.parse::<i32>().ok())
        .unwrap_or(state.last_exit_code);
    process::exit(code);
}

fn builtin_export(args: &[String]) -> i32 {
    for arg in args {
        if let Some((key, value)) = arg.split_once('=') {
            unsafe {
                std::env::set_var(key, value);
            }
        }
    }
    0
}

fn builtin_unset(args: &[String]) -> i32 {
    for arg in args {
        unsafe {
            std::env::remove_var(arg);
        }
    }
    0
}

fn builtin_echo(args: &[String]) -> i32 {
    let mut newline = true;
    let mut start = 0;

    if args.first().map(|value| value.as_str()) == Some("-n") {
        newline = false;
        start = 1;
    }

    let output = args[start..].join(" ");
    if newline {
        println!("{output}");
    } else {
        print!("{output}");
    }
    0
}

fn builtin_pwd() -> i32 {
    match std::env::current_dir() {
        Ok(path) => {
            println!("{}", path.display());
            0
        }
        Err(error) => {
            eprintln!("sh: pwd: {error}");
            1
        }
    }
}
