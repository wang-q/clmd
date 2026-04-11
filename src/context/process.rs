//! Process management utilities for clmd.
//!
//! This module provides utilities for running external processes,
//! inspired by Pandoc's Process module.

use std::io::{self, Write};
use std::process::{Command, ExitStatus, Stdio};

/// Error type for process operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProcessError {
    /// IO error occurred.
    Io(String),
    /// Process exited with non-zero status.
    ExitError(ExitStatus, String),
    /// Process timed out.
    Timeout,
    /// Process was killed.
    Killed,
    /// Invalid command or arguments.
    InvalidCommand(String),
}

impl std::fmt::Display for ProcessError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProcessError::Io(msg) => write!(f, "IO error: {}", msg),
            ProcessError::ExitError(status, stderr) => {
                write!(f, "Process exited with status: {:?}", status)?;
                if !stderr.is_empty() {
                    write!(f, ", stderr: {}", stderr)?;
                }
                Ok(())
            }
            ProcessError::Timeout => write!(f, "Process timed out"),
            ProcessError::Killed => write!(f, "Process was killed"),
            ProcessError::InvalidCommand(msg) => write!(f, "Invalid command: {}", msg),
        }
    }
}

impl std::error::Error for ProcessError {}

impl From<io::Error> for ProcessError {
    fn from(err: io::Error) -> Self {
        ProcessError::Io(err.to_string())
    }
}

/// Result type for process operations.
pub type ProcessResult<T> = Result<T, ProcessError>;

/// Run a process with input and capture output.
pub fn pipe_process(
    cmd: &str,
    args: &[&str],
    input: &[u8],
) -> ProcessResult<(ExitStatus, Vec<u8>)> {
    let mut command = Command::new(cmd);
    command.args(args);

    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn()?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input)?;
        drop(stdin);
    }

    let output = child.wait_with_output()?;

    Ok((output.status, output.stdout))
}
