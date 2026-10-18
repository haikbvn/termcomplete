mod hook;
mod init;
pub mod internal;
mod standalone;
mod theme;
use clap::{Parser, Subcommand, ValueEnum};
use eyre::Result;
use std::process::ExitCode;
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, ValueEnum)]
pub enum OutputFormat {
    #[default]
    Plain,
    Json,
    JsonPretty,
}
impl OutputFormat {
    pub fn print<T, TFn, J, JFn>(&self, text_fn: TFn, json_fn: JFn)
    where
        T: std::fmt::Display,
        TFn: FnOnce() -> T,
        J: serde::Serialize,
        JFn: FnOnce() -> J,
    {
        match self {
            Self::Plain => println!("{}", text_fn()),
            Self::Json => println!("{}", serde_json::to_string(&json_fn()).unwrap()),
            Self::JsonPretty => println!("{}", serde_json::to_string_pretty(&json_fn()).unwrap()),
        }
    }
}
#[derive(Debug, Parser)]
#[command(name = "tcomp", version, about = "Standalone terminal autocomplete")]
pub struct Cli {
    #[arg(short,long,action=clap::ArgAction::Count)]
    pub verbose: u8,
    #[command(subcommand)]
    command: Option<Commands>,
}
#[derive(Debug, Subcommand)]
enum Commands {
    /// Open the settings window
    Dashboard,
    /// Start the desktop application
    Launch,
    /// Install shell integration and the input-method helper
    Install {
        #[arg(long)]
        shell_only: bool,
    },
    /// Remove Termcomplete shell integration and input-method registration
    Uninstall {
        #[arg(long)]
        shell_only: bool,
    },
    /// Inspect local installation and permissions
    Doctor,
    /// Read, write, or delete a setting (values accept JSON)
    Settings {
        key: Option<String>,
        value: Option<String>,
        #[arg(short, long)]
        delete: bool,
    },
    Theme(theme::ThemeArgs),
    #[command(hide = true)]
    Init(init::InitArgs),
    #[command(hide = true, subcommand)]
    Hook(hook::HookSubcommand),
    #[command(name = "internal", alias = "_", hide = true, subcommand)]
    Internal(internal::InternalSubcommand),
}
impl Cli {
    pub async fn execute(self) -> Result<ExitCode> {
        fig_settings::settings::init_global().ok();
        match self.command {
            None | Some(Commands::Dashboard) | Some(Commands::Launch) => standalone::dashboard().await,
            Some(Commands::Install { shell_only }) => standalone::install(false, shell_only).await,
            Some(Commands::Uninstall { shell_only }) => standalone::install(true, shell_only).await,
            Some(Commands::Doctor) => standalone::doctor().await,
            Some(Commands::Settings { key, value, delete }) => standalone::settings(key, value, delete),
            Some(Commands::Theme(args)) => args.execute().await,
            Some(Commands::Init(args)) => args.execute().await,
            Some(Commands::Hook(args)) => args.execute().await,
            Some(Commands::Internal(args)) => args.execute().await,
        }
    }
}
#[cfg(test)]
mod tests {
    #[test]
    fn cli_contract() {
        use clap::CommandFactory;
        super::Cli::command().debug_assert();
    }
}
