pub mod cli;
pub mod util;
use clap::Parser;
#[tokio::main]
async fn main() -> std::process::ExitCode {
    match cli::Cli::parse().execute().await {
        Ok(code) => code,
        Err(error) => {
            eprintln!("tcomp: {error:#}");
            std::process::ExitCode::FAILURE
        }
    }
}
