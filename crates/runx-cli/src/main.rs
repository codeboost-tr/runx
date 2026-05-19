use std::env;
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{Command, ExitCode, Stdio};

use runx_cli::launcher::{LauncherAction, shim_help};

fn main() -> ExitCode {
    let args: Vec<OsString> = env::args_os().skip(1).collect();

    match runx_cli::launcher::plan_launcher_with_native_options(
        args,
        env::var_os("RUNX_NPM_PACKAGE"),
        env::var_os("RUNX_JS_BIN"),
        runx_cli::launcher::NativeLauncherOptions {
            rust_cli: env::var_os("RUNX_RUST_CLI"),
            rust_harness: env::var_os("RUNX_RUST_HARNESS"),
        },
    ) {
        LauncherAction::Error(message) => {
            let _ignored = write_stderr_line(&format!("runx: {message}"));
            ExitCode::from(2)
        }
        LauncherAction::PrintHelp => write_stdout(&shim_help()),
        LauncherAction::PrintVersion => {
            write_stdout_line(&format!("runx-cli {}", env!("CARGO_PKG_VERSION")))
        }
        LauncherAction::RunInit(plan) => runx_cli::scaffold::run_native_init(plan),
        LauncherAction::RunNew(plan) => runx_cli::scaffold::run_native_new(plan),
        LauncherAction::RunHistory(plan) => run_native_history(plan.args),
        LauncherAction::RunHarness(plan) => run_native_harness(PathBuf::from(plan.fixture_path)),
        LauncherAction::RunConnect(plan) => run_native_connect(plan),
        LauncherAction::RunTool(plan) => runx_cli::tool::run_native_tool(plan),
        LauncherAction::Delegate(command) => match run_command(command) {
            Ok(code) => ExitCode::from(code),
            Err(error) => {
                let _ignored = write_stderr_line(&format!("runx: {error}"));
                ExitCode::from(1)
            }
        },
    }
}

fn run_native_connect(plan: runx_cli::connect::ConnectPlan) -> ExitCode {
    let env_map = env::vars().collect::<std::collections::BTreeMap<_, _>>();
    let options = match runx_runtime::load_connect_options_from_env(&env_map) {
        Ok(options) => options,
        Err(error) => {
            let _ignored = write_stderr_line(&format!("runx: {error}"));
            return ExitCode::from(1);
        }
    };
    let client = match runx_runtime::ConnectClient::new(options, env_map) {
        Ok(client) => client,
        Err(error) => {
            let _ignored = write_stderr_line(&format!("runx: {error}"));
            return ExitCode::from(1);
        }
    };
    let result = match execute_connect_plan(&client, &plan) {
        Ok(result) => result,
        Err(error) => {
            let _ignored = write_stderr_line(&format!("runx: {error}"));
            return ExitCode::from(1);
        }
    };
    if plan.json {
        return write_connect_json(&result);
    }
    write_stdout(&render_connect_result(&plan, &result))
}

fn execute_connect_plan(
    client: &runx_runtime::ConnectClient,
    plan: &runx_cli::connect::ConnectPlan,
) -> Result<serde_json::Value, runx_runtime::ConnectError> {
    match plan.action {
        runx_cli::connect::ConnectAction::List => {
            serde_json::to_value(client.list()?).map_err(|error| {
                runx_runtime::ConnectError::Serialize {
                    message: error.to_string(),
                }
            })
        }
        runx_cli::connect::ConnectAction::Revoke => {
            let grant_id = plan.grant_id.as_deref().unwrap_or_default();
            serde_json::to_value(client.revoke(grant_id)?).map_err(|error| {
                runx_runtime::ConnectError::Serialize {
                    message: error.to_string(),
                }
            })
        }
        runx_cli::connect::ConnectAction::Preprovision => {
            let request = runx_runtime::HttpConnectPreprovisionRequest {
                provider: plan.provider.clone().unwrap_or_default(),
                scopes: plan.scopes.clone(),
                scope_family: plan.scope_family.clone(),
                authority_kind: plan.authority_kind.map(runtime_authority_kind),
                target_repo: plan.target_repo.clone(),
                target_locator: plan.target_locator.clone(),
            };
            serde_json::to_value(client.preprovision(&request)?).map_err(|error| {
                runx_runtime::ConnectError::Serialize {
                    message: error.to_string(),
                }
            })
        }
    }
}

fn runtime_authority_kind(
    kind: runx_cli::connect::ConnectAuthorityKind,
) -> runx_runtime::connect::ConnectAuthorityKind {
    match kind {
        runx_cli::connect::ConnectAuthorityKind::ReadOnly => {
            runx_runtime::connect::ConnectAuthorityKind::ReadOnly
        }
        runx_cli::connect::ConnectAuthorityKind::Constructive => {
            runx_runtime::connect::ConnectAuthorityKind::Constructive
        }
        runx_cli::connect::ConnectAuthorityKind::Destructive => {
            runx_runtime::connect::ConnectAuthorityKind::Destructive
        }
    }
}

fn write_connect_json(result: &serde_json::Value) -> ExitCode {
    match serde_json::to_string_pretty(&serde_json::json!({
        "status": "success",
        "connect": result,
    })) {
        Ok(json) => write_stdout_line(&json),
        Err(error) => {
            let _ignored = write_stderr_line(&format!(
                "runx: failed to serialize connect result: {error}"
            ));
            ExitCode::from(1)
        }
    }
}

fn render_connect_result(
    plan: &runx_cli::connect::ConnectPlan,
    result: &serde_json::Value,
) -> String {
    if plan.action == runx_cli::connect::ConnectAction::List {
        return render_connect_list(result);
    }
    let grant = result.get("grant").and_then(serde_json::Value::as_object);
    let title = if plan.action == runx_cli::connect::ConnectAction::Revoke {
        "connection revoked"
    } else {
        "connection ready"
    };
    let status = result
        .get("status")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("success");
    let next = if plan.action == runx_cli::connect::ConnectAction::Revoke {
        "runx connect github"
    } else {
        "runx connect list"
    };
    let mut rows = vec![
        ("provider", grant_string(grant, "provider")),
        ("grant", grant_string(grant, "grant_id")),
    ];
    if let Some(scopes) = connect_scopes(grant)
        && !scopes.is_empty()
    {
        rows.push(("scopes", scopes));
    }
    for (label, field) in [
        ("family", "scope_family"),
        ("authority", "authority_kind"),
        ("repo", "target_repo"),
        ("locator", "target_locator"),
    ] {
        let value = grant_string(grant, field);
        if !value.is_empty() {
            rows.push((label, value));
        }
    }
    rows.push(("next", next.to_owned()));

    let mut lines = vec![String::new(), format!("  ✓  {title}  {status}")];
    lines.extend(render_key_value_rows(&rows));
    lines.push(String::new());
    lines.push(String::new());
    lines.join("\n")
}

fn render_connect_list(result: &serde_json::Value) -> String {
    let Some(grants) = result.get("grants").and_then(serde_json::Value::as_array) else {
        return "\n  No connections yet.\n  start  runx connect github\n\n".to_owned();
    };
    if grants.is_empty() {
        return "\n  No connections yet.\n  start  runx connect github\n\n".to_owned();
    }
    let mut lines = vec![
        String::new(),
        format!("  connections  {} grant(s)", grants.len()),
        String::new(),
    ];
    for grant in grants {
        let grant = grant.as_object();
        lines.push(format!(
            "  {}  {}  {}",
            connect_status_icon(grant),
            grant_string(grant, "provider"),
            grant_string(grant, "grant_id")
        ));
        if let Some(scopes) = connect_scopes(grant) {
            if !scopes.is_empty() {
                lines.push(format!("  scopes  {scopes}"));
            }
        }
        for (label, field) in [
            ("family", "scope_family"),
            ("authority", "authority_kind"),
            ("repo", "target_repo"),
            ("locator", "target_locator"),
        ] {
            let value = grant_string(grant, field);
            if !value.is_empty() {
                lines.push(format!("  {label}  {value}"));
            }
        }
        lines.push(String::new());
    }
    lines.join("\n")
}

fn render_key_value_rows(rows: &[(&str, String)]) -> Vec<String> {
    let width = rows
        .iter()
        .filter(|(_label, value)| !value.is_empty())
        .map(|(label, _value)| label.len())
        .max()
        .unwrap_or(0);
    rows.iter()
        .filter(|(_label, value)| !value.is_empty())
        .map(|(label, value)| format!("  {label:<width$}  {value}"))
        .collect()
}

fn connect_status_icon(grant: Option<&serde_json::Map<String, serde_json::Value>>) -> &'static str {
    if grant_string(grant, "status") == "revoked" {
        "✗"
    } else {
        "✓"
    }
}

fn connect_scopes(grant: Option<&serde_json::Map<String, serde_json::Value>>) -> Option<String> {
    grant
        .and_then(|grant| grant.get("scopes"))
        .and_then(serde_json::Value::as_array)
        .map(|scopes| {
            scopes
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(", ")
        })
}

fn grant_string(grant: Option<&serde_json::Map<String, serde_json::Value>>, field: &str) -> String {
    grant
        .and_then(|grant| grant.get(field))
        .and_then(serde_json::Value::as_str)
        .unwrap_or_default()
        .to_owned()
}

fn run_native_history(args: Vec<OsString>) -> ExitCode {
    let cwd = match env::current_dir() {
        Ok(cwd) => cwd,
        Err(error) => {
            let _ignored = write_stderr_line(&format!("runx: failed to resolve cwd: {error}"));
            return ExitCode::from(1);
        }
    };
    match runx_cli::history::run_history_command(&args, &runx_cli::history::env_map(), &cwd) {
        Ok(output) => write_stdout(&output.output),
        Err(runx_cli::history::HistoryCliError::InvalidArgs(message)) => {
            let _ignored = write_stderr_line(&format!("runx: {message}"));
            ExitCode::from(2)
        }
        Err(error) => {
            let _ignored = write_stderr_line(&format!("runx: {error}"));
            ExitCode::from(1)
        }
    }
}

fn run_native_harness(fixture_path: PathBuf) -> ExitCode {
    match runx_runtime::run_harness_fixture(&fixture_path) {
        Ok(output) => match serde_json::to_string_pretty(&output.receipt) {
            Ok(json) => write_stdout_line(&json),
            Err(error) => {
                let _ignored = write_stderr_line(&format!(
                    "runx: failed to serialize harness receipt: {error}"
                ));
                ExitCode::from(1)
            }
        },
        Err(error) => {
            let _ignored = write_stderr_line(&format!(
                "runx: native harness replay failed for {}: {error}",
                fixture_path.display()
            ));
            ExitCode::from(1)
        }
    }
}

fn write_stdout(message: &str) -> ExitCode {
    let mut stdout = io::stdout().lock();
    if stdout.write_all(message.as_bytes()).is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn write_stdout_line(message: &str) -> ExitCode {
    let mut stdout = io::stdout().lock();
    if writeln!(stdout, "{message}").is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn write_stderr_line(message: &str) -> ExitCode {
    let mut stderr = io::stderr().lock();
    if writeln!(stderr, "{message}").is_ok() {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

fn run_command(plan: runx_cli::launcher::CommandPlan) -> Result<u8, String> {
    let status = Command::new(&plan.program)
        .args(plan.args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| {
            format!(
                "failed to launch {}: {error}",
                plan.program.to_string_lossy()
            )
        })?;

    Ok(exit_code(status))
}

fn exit_code(status: std::process::ExitStatus) -> u8 {
    if let Some(code) = status.code() {
        return code.clamp(0, 255) as u8;
    }

    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if let Some(signal) = status.signal() {
            return (128 + signal).clamp(1, 255) as u8;
        }
    }

    1
}
