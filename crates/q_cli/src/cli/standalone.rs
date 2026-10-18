use crate::util::desktop::{LaunchArgs, launch_fig_desktop};
use eyre::{Result, bail};
use fig_integrations::{Integration, shell::ShellExt};
use fig_util::{Shell, directories};
use std::process::ExitCode;
pub async fn dashboard() -> Result<ExitCode> {
    launch_fig_desktop(LaunchArgs {
        wait_for_socket: true,
        open_dashboard: true,
        immediate_update: false,
        verbose: false,
    })?;
    fig_ipc::local::open_ui_element(fig_proto::local::UiElement::MissionControl, None).await?;
    Ok(ExitCode::SUCCESS)
}
pub async fn install(uninstall: bool, shell_only: bool, input_method_only: bool) -> Result<ExitCode> {
    let env = fig_os_shim::Env::new();
    if !input_method_only {
        for shell in [Shell::Bash, Shell::Zsh] {
            for script in shell.get_script_integrations()? {
                if uninstall {
                    script.uninstall().await?;
                } else {
                    script.install().await?;
                }
            }
            for integration in shell.get_shell_integrations(&env)? {
                if uninstall {
                    integration.uninstall().await?;
                } else {
                    integration.install().await?;
                }
            }
        }
    }
    if !shell_only {
        if directories::desktop_socket_path().is_ok_and(|path| path.exists()) {
            use fig_proto::local::{
                InputMethodAction, InputMethodCommand, command::Command, command_response::Response,
            };
            let action = if uninstall {
                InputMethodAction::Uninstall
            } else {
                InputMethodAction::Install
            };
            let response = fig_ipc::local::send_recv_command_to_socket_with_timeout(
                Command::InputMethod(InputMethodCommand {
                    actions: Some(action.into()),
                }),
                std::time::Duration::from_secs(70),
            )
            .await?;
            match response.and_then(|response| response.response) {
                Some(Response::Success(_)) => {}
                Some(Response::Error(error)) => bail!(
                    "{}",
                    error.message.unwrap_or_else(|| "Cursor helper setup failed".into())
                ),
                _ => bail!("The desktop returned an invalid cursor-helper response"),
            }
        } else {
            let input = fig_integrations::input_method::InputMethod::default();
            if uninstall {
                input.uninstall().await?;
            } else {
                input.install().await?;
            }
        }
    }
    println!(
        "{} Termcomplete integration. Restart your terminal.",
        if uninstall { "Removed" } else { "Installed" }
    );
    Ok(ExitCode::SUCCESS)
}
pub async fn doctor() -> Result<ExitCode> {
    let env = fig_os_shim::Env::new();
    let mut healthy = true;
    for shell in [Shell::Bash, Shell::Zsh] {
        for integration in shell.get_shell_integrations(&env)? {
            let installed = integration.is_installed().await.is_ok();
            println!("{}: {}", integration, if installed { "installed" } else { "missing" });
            healthy &= installed;
        }
    }
    // Permissions belong to the desktop app, not the terminal running this CLI.
    let response = fig_ipc::local::send_recv_command_to_socket(fig_proto::local::command::Command::Diagnostics(
        Default::default(),
    ))
    .await;
    let accessibility = match response {
        Ok(Some(fig_proto::local::CommandResponse {
            response: Some(fig_proto::local::command_response::Response::Diagnostics(diagnostic)),
            ..
        })) => diagnostic.accessibility == "true",
        _ => false,
    };
    let input = fig_integrations::input_method::InputMethod::default()
        .is_enabled()
        .unwrap_or(false);
    println!(
        "Accessibility: {accessibility}\nInput method: {input}\nData: {}",
        directories::fig_data_dir()?.display()
    );
    Ok(if healthy && accessibility && input {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}
pub fn settings(key: Option<String>, value: Option<String>, delete: bool) -> Result<ExitCode> {
    let path = directories::settings_path()?;
    let mut data: serde_json::Value = if path.exists() {
        serde_json::from_slice(&std::fs::read(&path)?)?
    } else {
        serde_json::json!({})
    };
    if let Some(key) = key {
        if delete {
            data.as_object_mut()
                .ok_or_else(|| eyre::eyre!("Invalid settings file"))?
                .remove(&key);
        } else if let Some(value) = value {
            data[&key] = serde_json::from_str(&value).unwrap_or(serde_json::Value::String(value));
        } else {
            println!("{}", data.get(&key).unwrap_or(&serde_json::Value::Null));
            return Ok(ExitCode::SUCCESS);
        }
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(path, serde_json::to_vec_pretty(&data)?)?;
    } else {
        if value.is_some() || delete {
            bail!("A setting key is required");
        }
        println!("{}", serde_json::to_string_pretty(&data)?);
    }
    Ok(ExitCode::SUCCESS)
}
