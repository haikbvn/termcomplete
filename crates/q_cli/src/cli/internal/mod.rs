pub mod local_state;
pub mod should_figterm_launch;
use clap::Subcommand;
use eyre::Result;
use fig_ipc::SendMessage;
use fig_proto::{
    figterm::{FigtermRequestMessage, UpdateShellContextRequest, figterm_request_message::Request as FigtermRequest},
    local::EnvironmentVariable,
};
use fig_util::{directories, directories::figterm_socket_path, env_var::TCOMP_PTY_SESSION_ID};
use std::{path::PathBuf, process::ExitCode};
use tracing::error;
#[derive(Debug, PartialEq, Subcommand)]
pub enum InternalSubcommand {
    PreCmd {
        #[arg(long, allow_hyphen_values = true)]
        alias: Option<String>,
    },
    LocalState(local_state::LocalStateArgs),
    GetShell,
    ShouldFigtermLaunch,
    SocketsDir,
    FigtermSocketPath {
        session_id: String,
    },
    AttemptToFinishInputMethodInstallation {
        bundle_path: Option<PathBuf>,
        #[arg(long)]
        enable: bool,
    },
}
impl InternalSubcommand {
    pub async fn execute(self) -> Result<ExitCode> {
        match self {
            Self::PreCmd { alias } => return Ok(pre_cmd(alias).await),
            Self::LocalState(args) => {
                args.execute().await?;
            }
            Self::GetShell => println!(
                "{}",
                fig_util::get_parent_process_exe()
                    .unwrap_or_else(|| "/bin/zsh".into())
                    .display()
            ),
            Self::ShouldFigtermLaunch => {
                return Ok(should_figterm_launch::should_figterm_launch_exit_status(
                    &fig_os_shim::Context::new(),
                    true,
                )
                .into());
            }
            Self::SocketsDir => println!("{}", directories::sockets_dir()?.display()),
            Self::FigtermSocketPath { session_id } => println!("{}", figterm_socket_path(&session_id)?.display()),
            Self::AttemptToFinishInputMethodInstallation { bundle_path, enable } => {
                if let Err(error) =
                    fig_integrations::input_method::InputMethod::finish_input_method_installation(bundle_path, enable)
                {
                    println!("{}", serde_json::to_string(&error)?);
                    return Ok(ExitCode::FAILURE);
                }
            }
        }
        Ok(ExitCode::SUCCESS)
    }
}
pub async fn pre_cmd(alias: Option<String>) -> ExitCode {
    let Ok(session_id) = std::env::var(TCOMP_PTY_SESSION_ID) else {
        return ExitCode::FAILURE;
    };

    match figterm_socket_path(&session_id) {
        Ok(figterm_path) => match fig_ipc::socket_connect(figterm_path).await {
            Ok(mut figterm_stream) => {
                let message = FigtermRequestMessage {
                    request: Some(FigtermRequest::UpdateShellContext(UpdateShellContextRequest {
                        update_environment_variables: true,
                        environment_variables: std::env::vars()
                            .map(|(key, value)| EnvironmentVariable {
                                key,
                                value: Some(value),
                            })
                            .collect(),
                        update_alias: true,
                        alias,
                    })),
                };
                if let Err(err) = figterm_stream.send_message(message).await {
                    error!(%err, %session_id, "Failed to send UpdateShellContext to Figterm");
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(err) => {
                error!(%err, %session_id, "Failed to connect to Figterm socket");
                ExitCode::FAILURE
            }
        },
        Err(err) => {
            error!(%err, %session_id, "Failed to get Figterm socket path");
            ExitCode::FAILURE
        }
    }
}
