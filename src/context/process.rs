//! Process management utilities for clmd.
//!
//! This module provides utilities for running external processes,
//! inspired by Pandoc's Process module. It includes support for
//! piping data to processes and capturing output.
//!
//! # Example
//!
//! ```
//! use clmd::process::pipe_process;
//! use std::process::Stdio;
//!
//! // Run a simple command
//! let result = pipe_process("echo", &["hello"], b"");
//! ```

use std::collections::HashMap;
use std::io::{self, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::time::Duration;

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

/// Options for running a process.
#[derive(Debug, Clone)]
pub struct ProcessOptions {
    /// Environment variables to set.
    pub env: HashMap<String, String>,
    /// Working directory for the process.
    pub cwd: Option<std::path::PathBuf>,
    /// Timeout for the process.
    pub timeout: Option<Duration>,
    /// Whether to capture stdout.
    pub capture_stdout: bool,
    /// Whether to capture stderr.
    pub capture_stderr: bool,
    /// Whether to inherit stdin from parent.
    pub inherit_stdin: bool,
}

impl Default for ProcessOptions {
    fn default() -> Self {
        Self {
            env: HashMap::new(),
            cwd: None,
            timeout: None,
            capture_stdout: true,
            capture_stderr: true,
            inherit_stdin: false,
        }
    }
}

impl ProcessOptions {
    /// Create new default options.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the working directory.
    pub fn cwd(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    /// Set the timeout.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set whether to capture stdout.
    pub fn capture_stdout(mut self, capture: bool) -> Self {
        self.capture_stdout = capture;
        self
    }

    /// Set whether to capture stderr.
    pub fn capture_stderr(mut self, capture: bool) -> Self {
        self.capture_stderr = capture;
        self
    }

    /// Set whether to inherit stdin.
    pub fn inherit_stdin(mut self, inherit: bool) -> Self {
        self.inherit_stdin = inherit;
        self
    }
}

/// Output from a process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessOutput {
    /// The exit status of the process.
    pub status: ExitStatus,
    /// The stdout output (if captured).
    pub stdout: Vec<u8>,
    /// The stderr output (if captured).
    pub stderr: Vec<u8>,
}

impl ProcessOutput {
    /// Returns true if the process exited successfully.
    pub fn success(&self) -> bool {
        self.status.success()
    }

    /// Returns the stdout as a string (lossy conversion).
    pub fn stdout_str(&self) -> String {
        String::from_utf8_lossy(&self.stdout).into_owned()
    }

    /// Returns the stderr as a string (lossy conversion).
    pub fn stderr_str(&self) -> String {
        String::from_utf8_lossy(&self.stderr).into_owned()
    }

    /// Unwrap the output, returning stdout if successful.
    ///
    /// # Errors
    ///
    /// Returns an error if the process did not exit successfully.
    pub fn unwrap(self) -> ProcessResult<Vec<u8>> {
        if self.success() {
            Ok(self.stdout)
        } else {
            Err(ProcessError::ExitError(self.status, self.stderr_str()))
        }
    }
}

/// Run a process with input and capture output.
///
/// This is similar to Pandoc's `pipeProcess` function. It runs a command
/// with the given arguments, pipes input to stdin, and captures stdout.
///
/// # Arguments
///
/// * `cmd` - The command to run.
/// * `args` - Arguments for the command.
/// * `input` - Input data to pipe to stdin.
///
/// # Returns
///
/// A tuple of (exit status, stdout) on success.
///
/// # Example
///
/// ```ignore
/// use clmd::process::pipe_process;
///
/// let (status, output) = pipe_process("echo", &["hello"], b"").unwrap();
/// assert!(status.success());
/// assert_eq!(String::from_utf8_lossy(&output).trim(), "hello");
/// ```ignore
pub fn pipe_process(
    cmd: &str,
    args: &[&str],
    input: &[u8],
) -> ProcessResult<(ExitStatus, Vec<u8>)> {
    let mut command = Command::new(cmd);
    command.args(args);

    // Set up stdin/stdout
    command.stdin(Stdio::piped());
    command.stdout(Stdio::piped());
    command.stderr(Stdio::piped());

    let mut child = command.spawn()?;

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(input)?;
        // Close stdin to signal EOF
        drop(stdin);
    }

    // Wait for output
    let output = child.wait_with_output()?;

    Ok((output.status, output.stdout))
}

/// Run a process with options.
///
/// This is a more flexible version of `pipe_process` that allows
/// setting environment variables, working directory, and timeout.
///
/// # Arguments
///
/// * `cmd` - The command to run.
/// * `args` - Arguments for the command.
/// * `input` - Input data to pipe to stdin.
/// * `options` - Process options.
///
/// # Example
///
/// ```ignore
/// use clmd::process::{run_process, ProcessOptions};
/// use std::time::Duration;
///
/// let options = ProcessOptions::new()
///     .timeout(Duration::from_secs(30));
///
/// let output = run_process("echo", &["hello"], b"", &options);
/// ```ignore
pub fn run_process(
    cmd: &str,
    args: &[&str],
    input: &[u8],
    options: &ProcessOptions,
) -> ProcessResult<ProcessOutput> {
    let mut command = Command::new(cmd);
    command.args(args);

    // Set environment variables
    for (key, value) in &options.env {
        command.env(key, value);
    }

    // Set working directory
    if let Some(ref cwd) = options.cwd {
        command.current_dir(cwd);
    }

    // Set up stdin
    if options.inherit_stdin {
        command.stdin(Stdio::inherit());
    } else {
        command.stdin(Stdio::piped());
    }

    // Set up stdout/stderr
    command.stdout(if options.capture_stdout {
        Stdio::piped()
    } else {
        Stdio::inherit()
    });
    command.stderr(if options.capture_stderr {
        Stdio::piped()
    } else {
        Stdio::inherit()
    });

    let mut child = command.spawn()?;

    // Write input to stdin if not inheriting
    if !options.inherit_stdin {
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(input)?;
            drop(stdin);
        }
    }

    // Wait with optional timeout
    let output = if let Some(timeout) = options.timeout {
        match wait_with_timeout(child, timeout) {
            Some(result) => result?,
            None => return Err(ProcessError::Timeout),
        }
    } else {
        child.wait_with_output()?
    };

    Ok(ProcessOutput {
        status: output.status,
        stdout: output.stdout,
        stderr: output.stderr,
    })
}

/// Wait for a child process with a timeout.
fn wait_with_timeout(
    mut child: std::process::Child,
    timeout: Duration,
) -> Option<io::Result<std::process::Output>> {
    let start = std::time::Instant::now();

    loop {
        if start.elapsed() >= timeout {
            // Kill the process
            let _ = child.kill();
            return None;
        }

        match child.try_wait() {
            Ok(Some(_)) => return Some(child.wait_with_output()),
            Ok(None) => {
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(e) => return Some(Err(e)),
        }
    }
}

/// Run a simple command without input.
///
/// # Arguments
///
/// * `cmd` - The command to run.
/// * `args` - Arguments for the command.
///
/// # Example
///
/// ```ignore
/// use clmd::process::run_command;
///
/// let output = run_command("echo", &["hello"]).unwrap();
/// assert!(output.success());
/// ```ignore
pub fn run_command(cmd: &str, args: &[&str]) -> ProcessResult<ProcessOutput> {
    run_process(cmd, args, b"", &ProcessOptions::default())
}

/// Check if a command is available in PATH.
///
/// # Example
///
/// ```ignore
/// use clmd::process::command_exists;
///
/// assert!(command_exists("echo"));
/// assert!(!command_exists("nonexistent_command_12345"));
/// ```ignore
pub fn command_exists(cmd: &str) -> bool {
    Command::new(cmd)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Get the path to a command if it exists.
///
/// # Example
///
/// ```ignore
/// use clmd::process::which;
///
/// let echo_path = which("echo");
/// assert!(echo_path.is_some());
/// ```ignore
pub fn which(cmd: &str) -> Option<std::path::PathBuf> {
    if let Ok(output) = Command::new("which")
        .arg(cmd)
        .stdout(Stdio::piped())
        .output()
    {
        if output.status.success() {
            let path = String::from_utf8_lossy(&output.stdout);
            return Some(std::path::PathBuf::from(path.trim()));
        }
    }

    // Try `where` on Windows
    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = Command::new("where")
            .arg(cmd)
            .stdout(Stdio::piped())
            .output()
        {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout);
                return path
                    .lines()
                    .next()
                    .map(|p| std::path::PathBuf::from(p.trim()));
            }
        }
    }

    None
}

/// Run a process and return stdout as a string.
///
/// # Arguments
///
/// * `cmd` - The command to run.
/// * `args` - Arguments for the command.
/// * `input` - Input data to pipe to stdin.
///
/// # Example
///
/// ```ignore
/// use clmd::process::pipe_process_string;
///
/// let output = pipe_process_string("echo", &["hello"], "").unwrap();
/// assert_eq!(output.trim(), "hello");
/// ```ignore
pub fn pipe_process_string(
    cmd: &str,
    args: &[&str],
    input: &str,
) -> ProcessResult<String> {
    let (status, stdout) = pipe_process(cmd, args, input.as_bytes())?;
    if status.success() {
        Ok(String::from_utf8_lossy(&stdout).into_owned())
    } else {
        Err(ProcessError::ExitError(status, String::new()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pipe_process() {
        let (status, output) = pipe_process("echo", &["hello"], b"").unwrap();
        assert!(status.success());
        assert_eq!(String::from_utf8_lossy(&output).trim(), "hello");
    }

    #[test]
    fn test_pipe_process_with_input() {
        // Use cat to echo back input
        let (status, output) = pipe_process("cat", &[], b"hello world").unwrap();
        assert!(status.success());
        assert_eq!(String::from_utf8_lossy(&output), "hello world");
    }

    #[test]
    fn test_run_command() {
        let output = run_command("echo", &["test"]).unwrap();
        assert!(output.success());
        assert_eq!(output.stdout_str().trim(), "test");
    }

    #[test]
    fn test_run_process_with_options() {
        let options = ProcessOptions::new()
            .capture_stdout(true)
            .capture_stderr(true);

        let output = run_process("echo", &["hello"], b"", &options).unwrap();
        assert!(output.success());
        assert_eq!(output.stdout_str().trim(), "hello");
    }

    #[test]
    fn test_process_options_builder() {
        let options = ProcessOptions::new()
            .env("TEST_VAR", "test_value")
            .capture_stdout(true);

        assert_eq!(options.env.get("TEST_VAR"), Some(&"test_value".to_string()));
        assert!(options.capture_stdout);
    }

    #[test]
    fn test_command_exists() {
        assert!(command_exists("echo"));
        assert!(!command_exists("nonexistent_command_12345"));
    }

    #[test]
    fn test_which() {
        let echo_path = which("echo");
        assert!(echo_path.is_some());
    }

    #[test]
    fn test_pipe_process_string() {
        let output = pipe_process_string("echo", &["hello"], "").unwrap();
        assert_eq!(output.trim(), "hello");
    }

    #[test]
    fn test_process_output_methods() {
        let output = run_command("echo", &["test"]).unwrap();
        assert!(output.success());
        assert!(!output.stdout.is_empty());
    }

    #[test]
    fn test_process_error_display() {
        let err = ProcessError::Io("test error".to_string());
        assert!(err.to_string().contains("IO error"));

        let status = ExitStatus::default();
        let err = ProcessError::ExitError(status, "stderr".to_string());
        assert!(err.to_string().contains("exited"));

        let err = ProcessError::Timeout;
        assert!(err.to_string().contains("timed out"));

        let err = ProcessError::Killed;
        assert!(err.to_string().contains("killed"));

        let err = ProcessError::InvalidCommand("bad cmd".to_string());
        assert!(err.to_string().contains("Invalid command"));
    }

    #[test]
    fn test_invalid_command() {
        let result = pipe_process("nonexistent_command_12345", &[], b"");
        assert!(result.is_err());
    }

    #[test]
    fn test_run_process_with_env() {
        let options = ProcessOptions::new().env("TEST_ENV_VAR", "test_value");

        // Use printenv or env to check environment variable
        let output = if command_exists("printenv") {
            run_process("printenv", &["TEST_ENV_VAR"], b"", &options)
        } else {
            // Skip test if printenv not available
            return;
        };

        if let Ok(output) = output {
            assert!(output.success());
            assert!(output.stdout_str().contains("test_value"));
        }
    }
}
