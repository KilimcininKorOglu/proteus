use std::path::Path;
use std::process::{Command as ProcessCommand, Stdio};

use proteus_core::ProteusResult;

use crate::builtins::{run_builtin, ShellState};
use crate::parser::{Command, SimpleCommand};

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
    }

    pub fn execute_script(&mut self, script: &str) -> ProteusResult<i32> {
        let tokens = crate::lexer::tokenize(script).map_err(proteus_core::ProteusError::Other)?;
        let commands = crate::parser::parse(&tokens).map_err(proteus_core::ProteusError::Other)?;

        let mut exit_code = 0;
        for command in &commands {
            exit_code = self.execute_command(command)?;
            self.state.last_exit_code = exit_code;
        }

        Ok(exit_code)
    }

    fn execute_command(&mut self, command: &Command) -> ProteusResult<i32> {
        match command {
            Command::Simple(simple) => self.execute_simple(simple),
            Command::Pipeline(pipeline) => self.execute_pipeline(pipeline),
        }
    }

    fn execute_simple(&mut self, command: &SimpleCommand) -> ProteusResult<i32> {
        if command.argv.is_empty() {
            return Ok(0);
        }

        let name = &command.argv[0];
        let args = &command.argv[1..];

        if let Some(code) = run_builtin(&mut self.state, name, args) {
            return Ok(code);
        }

        let path = resolve_command(name).ok_or_else(|| {
            proteus_core::ProteusError::Other(format!("sh: {name}: command not found"))
        })?;

        let status = ProcessCommand::new(path).args(args).status()?;
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
