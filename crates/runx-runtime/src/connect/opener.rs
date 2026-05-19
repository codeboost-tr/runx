use std::collections::BTreeMap;
use std::process::{Command, Stdio};

use super::client::ConnectError;

pub trait ConnectOpener {
    fn open(&self, url: &str) -> Result<(), ConnectError>;
}

#[derive(Clone, Debug, Default)]
pub struct ProcessConnectOpener {
    pub command: Option<String>,
    pub env: BTreeMap<String, String>,
}

impl ProcessConnectOpener {
    pub fn new(command: Option<String>, env: BTreeMap<String, String>) -> Self {
        Self { command, env }
    }
}

impl ConnectOpener for ProcessConnectOpener {
    fn open(&self, url: &str) -> Result<(), ConnectError> {
        if let Some(command) = self
            .command
            .as_deref()
            .filter(|command| !command.is_empty())
        {
            return run_shell_command(command, url, &self.env);
        }
        run_platform_opener(url, &self.env)
    }
}

fn run_shell_command(
    command: &str,
    url: &str,
    env: &BTreeMap<String, String>,
) -> Result<(), ConnectError> {
    let mut process = if cfg!(windows) {
        let mut process = Command::new("cmd");
        process.args(["/C", command]);
        process
    } else {
        let mut process = Command::new("sh");
        process.args(["-c", command]);
        process
    };
    run_process(&mut process, url, env, "Connect opener command failed")
}

fn run_platform_opener(url: &str, env: &BTreeMap<String, String>) -> Result<(), ConnectError> {
    if cfg!(target_os = "macos") {
        let mut process = Command::new("open");
        process.arg(url);
        return run_process(
            &mut process,
            url,
            env,
            "Connect opener process 'open' failed",
        );
    }
    if cfg!(windows) {
        let mut process = Command::new("cmd");
        process.args(["/C", "start", "", url]);
        return run_process(
            &mut process,
            url,
            env,
            "Connect opener process 'cmd' failed",
        );
    }
    let mut process = Command::new("xdg-open");
    process.arg(url);
    run_process(
        &mut process,
        url,
        env,
        "Connect opener process 'xdg-open' failed",
    )
}

fn run_process(
    process: &mut Command,
    url: &str,
    env: &BTreeMap<String, String>,
    label: &str,
) -> Result<(), ConnectError> {
    process
        .envs(env)
        .env("RUNX_CONNECT_URL", url)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    let status = process
        .status()
        .map_err(|error| ConnectError::OpenerFailed {
            message: format!("{label}: {error}"),
        })?;
    if status.success() {
        return Ok(());
    }
    Err(ConnectError::OpenerFailed {
        message: format!(
            "{label} with exit code {}.",
            status
                .code()
                .map_or_else(|| "unknown".to_owned(), |code| code.to_string())
        ),
    })
}
