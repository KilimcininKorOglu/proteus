use std::fs::OpenOptions;
use std::os::fd::AsRawFd;
use std::path::Path;
use std::process::{Command as ProcessCommand, Stdio};

use proteus_core::ProteusResult;

use crate::builtins::{run_builtin, ShellState};
use crate::parser::{Command, ConditionalOperator, Redirect, SimpleCommand};

pub struct Interpreter {
    state: ShellState,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            state: ShellState::new(),
        }
    }

    pub fn set_args(&mut self, args: Vec<String>) {
        self.state.args = args;
        unsafe {
            std::env::set_var("PROTEUS_SHELL_ARG_0", "sh");
        }
        for (index, arg) in self.state.args.iter().enumerate() {
            unsafe {
                std::env::set_var(format!("PROTEUS_SHELL_ARG_{}", index + 1), arg);
            }
        }
    }

    pub fn execute_script(&mut self, script: &str) -> ProteusResult<i32> {
        let tokens = crate::lexer::tokenize(script).map_err(proteus_core::ProteusError::Other)?;
        let commands = crate::parser::parse(&tokens).map_err(proteus_core::ProteusError::Other)?;

        let mut exit_code = 0;
        for command in &commands {
            exit_code = self.execute_command(command)?;
            self.state.last_exit_code = exit_code;
            unsafe {
                std::env::set_var("PROTEUS_SHELL_LAST_EXIT", exit_code.to_string());
            }
        }

        Ok(exit_code)
    }

    fn execute_command(&mut self, command: &Command) -> ProteusResult<i32> {
        match command {
            Command::Simple(simple) => self.execute_simple(simple),
            Command::Pipeline(pipeline) => self.execute_pipeline(pipeline),
            Command::Conditional {
                left,
                operator,
                right,
            } => {
                let left_code = self.execute_command(left)?;
                self.state.last_exit_code = left_code;
                unsafe {
                    std::env::set_var("PROTEUS_SHELL_LAST_EXIT", left_code.to_string());
                }

                let should_run_right = match operator {
                    ConditionalOperator::And => left_code == 0,
                    ConditionalOperator::Or => left_code != 0,
                };
                if should_run_right {
                    self.execute_command(right)
                } else {
                    Ok(left_code)
                }
            }
        }
    }

    fn execute_simple(&mut self, command: &SimpleCommand) -> ProteusResult<i32> {
        if command.argv.is_empty() {
            return Ok(0);
        }

        let name = &command.argv[0];
        let args = &command.argv[1..];

        if let Some(code) = with_builtin_redirects(&command.redirects, || {
            run_builtin(&mut self.state, name, args)
        })? {
            return Ok(code);
        }

        let path = resolve_command(name).ok_or_else(|| {
            proteus_core::ProteusError::Other(format!("sh: {name}: command not found"))
        })?;

        let mut process = ProcessCommand::new(path);
        process.args(args);
        apply_redirects(&mut process, &command.redirects)?;
        let status = process.status()?;
        Ok(status.code().unwrap_or(1))
    }

    fn execute_pipeline(&mut self, pipeline: &[SimpleCommand]) -> ProteusResult<i32> {
        if pipeline.is_empty() {
            return Ok(0);
        }

        if pipeline.len() == 1 {
            return self.execute_simple(&pipeline[0]);
        }

        let mut children = Vec::new();
        let mut previous_stdout = None;

        for (index, command) in pipeline.iter().enumerate() {
            if command.argv.is_empty() {
                continue;
            }

            let name = &command.argv[0];
            let args = &command.argv[1..];
            let path = resolve_command(name).ok_or_else(|| {
                proteus_core::ProteusError::Other(format!("sh: {name}: command not found"))
            })?;

            let mut process = ProcessCommand::new(path);
            process.args(args);
            apply_redirects(&mut process, &command.redirects)?;

            if let Some(stdout) = previous_stdout.take() {
                process.stdin(Stdio::from(stdout));
            }

            if index < pipeline.len() - 1 {
                process.stdout(Stdio::piped());
            }

            let mut child = process.spawn()?;
            previous_stdout = child.stdout.take();
            children.push(child);
        }

        let mut last_code = 0;
        for mut child in children {
            let status = child.wait()?;
            last_code = status.code().unwrap_or(1);
        }

        Ok(last_code)
    }
}

fn apply_redirects(process: &mut ProcessCommand, redirects: &[Redirect]) -> ProteusResult<()> {
    for redirect in redirects {
        let file = if redirect.input {
            OpenOptions::new().read(true).open(&redirect.target)?
        } else {
            let mut options = OpenOptions::new();
            options.write(true).create(true);
            if redirect.append {
                options.append(true);
            } else {
                options.truncate(true);
            }
            options.open(&redirect.target)?
        };

        match redirect.fd {
            0 => {
                process.stdin(Stdio::from(file));
            }
            1 => {
                process.stdout(Stdio::from(file));
            }
            _ => {}
        }
    }

    Ok(())
}

fn with_builtin_redirects<F>(redirects: &[Redirect], operation: F) -> ProteusResult<Option<i32>>
where
    F: FnOnce() -> Option<i32>,
{
    let mut saved_stdin = None;
    let mut saved_stdout = None;

    for redirect in redirects {
        let file = if redirect.input {
            OpenOptions::new().read(true).open(&redirect.target)?
        } else {
            let mut options = OpenOptions::new();
            options.write(true).create(true);
            if redirect.append {
                options.append(true);
            } else {
                options.truncate(true);
            }
            options.open(&redirect.target)?
        };

        unsafe {
            match redirect.fd {
                0 => {
                    if saved_stdin.is_none() {
                        saved_stdin = Some(libc::dup(0));
                    }
                    libc::dup2(file.as_raw_fd(), 0);
                }
                1 => {
                    if saved_stdout.is_none() {
                        saved_stdout = Some(libc::dup(1));
                    }
                    libc::dup2(file.as_raw_fd(), 1);
                }
                _ => {}
            }
        }
    }

    let result = operation();

    unsafe {
        if let Some(fd) = saved_stdin {
            libc::dup2(fd, 0);
            libc::close(fd);
        }
        if let Some(fd) = saved_stdout {
            libc::dup2(fd, 1);
            libc::close(fd);
        }
    }

    Ok(result)
}

fn resolve_command(name: &str) -> Option<String> {
    if name.contains('/') {
        return Path::new(name).exists().then(|| name.to_string());
    }

    let path_var = std::env::var("PATH").unwrap_or_else(|_| "/usr/bin:/bin".to_string());
    for dir in path_var.split(':') {
        if dir.is_empty() {
            continue;
        }
        let candidate = format!("{dir}/{name}");
        if Path::new(&candidate).exists() {
            return Some(candidate);
        }
    }
    None
}
