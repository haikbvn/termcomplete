use fig_integrations::{Integration, shell::ShellExt};
use fig_os_shim::Context;
use fig_util::{APP_PROCESS_NAME, CLI_BINARY_NAME, PTY_BINARY_NAME, Shell, directories};
use std::sync::Arc;

// Prepare only our helper links and generated integration scripts. Dotfiles are
// changed only by an explicit install action in the CLI or onboarding.
pub async fn run_install(_ctx: Arc<Context>, _ignore_update: bool) {
    if let Err(error) = initialize_fig_dir(&fig_os_shim::Env::new()).await {
        tracing::error!(%error, "Unable to prepare Termcomplete helpers");
    }
}
pub async fn initialize_fig_dir(_env: &fig_os_shim::Env) -> anyhow::Result<()> {
    let bin = directories::home_local_bin()?;
    std::fs::create_dir_all(&bin)?;
    let executable = std::env::current_exe()?;
    let parent = executable
        .parent()
        .ok_or_else(|| anyhow::anyhow!("Missing bundle binary directory"))?;
    for name in [CLI_BINARY_NAME, PTY_BINARY_NAME, APP_PROCESS_NAME] {
        let source = parent.join(name);
        let destination = bin.join(name);
        if source.exists() && !destination.exists() && destination.symlink_metadata().is_err() {
            std::os::unix::fs::symlink(source, destination)?;
        }
    }
    for shell in [Shell::Bash, Shell::Zsh] {
        for integration in shell.get_script_integrations()? {
            integration.install().await?;
        }
    }
    Ok(())
}
