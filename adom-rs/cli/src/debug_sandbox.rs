use std::path::PathBuf;

use adom_common::CliConfigOverrides;
use adom_core::config::Config;
use adom_core::config::ConfigOverrides;
use adom_core::exec_env::create_env;
use adom_core::landlock::spawn_command_under_linux_sandbox;
#[cfg(target_os = "macos")]
use adom_core::seatbelt::spawn_command_under_seatbelt;
use adom_core::spawn::StdioPolicy;
use adom_protocol::config_types::SandboxMode;

use crate::LandlockCommand;
use crate::SeatbeltCommand;
use crate::WindowsCommand;
use crate::exit_status::handle_exit_status;

#[cfg(target_os = "macos")]
pub async fn run_command_under_seatbelt(
    command: SeatbeltCommand,
    adom_linux_sandbox_exe: Option<PathBuf>,
) -> anyhow::Result<()> {
    let SeatbeltCommand {
        full_auto,
        config_overrides,
        command,
    } = command;
    run_command_under_sandbox(
        full_auto,
        command,
        config_overrides,
        adom_linux_sandbox_exe,
        SandboxType::Seatbelt,
    )
    .await
}

#[cfg(not(target_os = "macos"))]
pub async fn run_command_under_seatbelt(
    _command: SeatbeltCommand,
    _adom_linux_sandbox_exe: Option<PathBuf>,
) -> anyhow::Result<()> {
    anyhow::bail!("Seatbelt sandbox is only available on macOS");
}

pub async fn run_command_under_landlock(
    command: LandlockCommand,
    adom_linux_sandbox_exe: Option<PathBuf>,
) -> anyhow::Result<()> {
    let LandlockCommand {
        full_auto,
        config_overrides,
        command,
    } = command;
    run_command_under_sandbox(
        full_auto,
        command,
        config_overrides,
        adom_linux_sandbox_exe,
        SandboxType::Landlock,
    )
    .await
}

pub async fn run_command_under_windows(
    command: WindowsCommand,
    adom_linux_sandbox_exe: Option<PathBuf>,
) -> anyhow::Result<()> {
    let WindowsCommand {
        full_auto,
        config_overrides,
        command,
    } = command;
    run_command_under_sandbox(
        full_auto,
        command,
        config_overrides,
        adom_linux_sandbox_exe,
        SandboxType::Windows,
    )
    .await
}

enum SandboxType {
    #[cfg(target_os = "macos")]
    Seatbelt,
    Landlock,
    Windows,
}

async fn run_command_under_sandbox(
    full_auto: bool,
    command: Vec<String>,
    config_overrides: CliConfigOverrides,
    adom_linux_sandbox_exe: Option<PathBuf>,
    sandbox_type: SandboxType,
) -> anyhow::Result<()> {
    let sandbox_mode = create_sandbox_mode(full_auto);
    let config = Config::load_with_cli_overrides(
        config_overrides
            .parse_overrides()
            .map_err(anyhow::Error::msg)?,
        ConfigOverrides {
            sandbox_mode: Some(sandbox_mode),
            adom_linux_sandbox_exe,
            ..Default::default()
        },
    )
    .await?;

    // In practice, this should be `std::env::current_dir()` because this CLI
    // does not support `--cwd`, but let's use the config value for consistency.
    let cwd = config.cwd.clone();
    // For now, we always use the same cwd for both the command and the
    // sandbox policy. In the future, we could add a CLI option to set them
    // separately.
    let sandbox_policy_cwd = cwd.clone();

    let stdio_policy = StdioPolicy::Inherit;
    let env = create_env(&config.shell_environment_policy);

    // Special-case Windows sandbox: execute and exit the process to emulate inherited stdio.
    if let SandboxType::Windows = sandbox_type {
        #[cfg(target_os = "windows")]
        {
            use adom_windows_sandbox::run_windows_sandbox_capture;

            let policy_str = match &config.sandbox_policy {
                adom_core::protocol::SandboxPolicy::DangerFullAccess => "workspace-write",
                adom_core::protocol::SandboxPolicy::ReadOnly => "read-only",
                adom_core::protocol::SandboxPolicy::WorkspaceWrite { .. } => "workspace-write",
            };

            let sandbox_cwd = sandbox_policy_cwd.clone();
            let cwd_clone = cwd.clone();
            let env_map = env.clone();
            let command_vec = command.clone();
            let base_dir = config.adom_home.clone();
            let res = tokio::task::spawn_blocking(move || {
                run_windows_sandbox_capture(
                    policy_str,
                    &sandbox_cwd,
                    command_vec,
                    &cwd_clone,
                    env_map,
                    None,
                    Some(base_dir.as_path()),
                )
            })
            .await;

            let capture = match res {
                Ok(Ok(v)) => v,
                Ok(Err(err)) => {
                    eprintln!("windows sandbox failed: {err}");
                    std::process::exit(1);
                }
                Err(join_err) => {
                    eprintln!("windows sandbox join error: {join_err}");
                    std::process::exit(1);
                }
            };

            if !capture.stdout.is_empty() {
                use std::io::Write;
                let _ = std::io::stdout().write_all(&capture.stdout);
            }
            if !capture.stderr.is_empty() {
                use std::io::Write;
                let _ = std::io::stderr().write_all(&capture.stderr);
            }

            std::process::exit(capture.exit_code);
        }
        #[cfg(not(target_os = "windows"))]
        {
            anyhow::bail!("Windows sandbox is only available on Windows");
        }
    }

    let mut child = match sandbox_type {
        #[cfg(target_os = "macos")]
        SandboxType::Seatbelt => {
            spawn_command_under_seatbelt(
                command,
                cwd,
                &config.sandbox_policy,
                sandbox_policy_cwd.as_path(),
                stdio_policy,
                env,
            )
            .await?
        }
        SandboxType::Landlock => {
            #[expect(clippy::expect_used)]
            let adom_linux_sandbox_exe = config
                .adom_linux_sandbox_exe
                .expect("adom-linux-sandbox executable not found");
            spawn_command_under_linux_sandbox(
                adom_linux_sandbox_exe,
                command,
                cwd,
                &config.sandbox_policy,
                sandbox_policy_cwd.as_path(),
                stdio_policy,
                env,
            )
            .await?
        }
        SandboxType::Windows => {
            unreachable!("Windows sandbox should have been handled above");
        }
    };
    let status = child.wait().await?;

    handle_exit_status(status);
}

pub fn create_sandbox_mode(full_auto: bool) -> SandboxMode {
    if full_auto {
        SandboxMode::WorkspaceWrite
    } else {
        SandboxMode::ReadOnly
    }
}
