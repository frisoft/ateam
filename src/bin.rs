use anyhow::{Context, Result};
// use log::*;

mod config;
use ateam::{cli, followup_render, pr_render};

#[tokio::main]
async fn main() -> Result<()> {
    let cmd = cli::command();

    let config = config::get_config().context("while reading from environment")?;

    match cmd {
        cli::Ateam {
            cmd: cli::Command::Pr(pr),
        } => {
            print!("{}", pr_render(&pr, &config.github_api_token).await?);
            Ok(())
        }
        cli::Ateam {
            cmd: cli::Command::Followup(followup),
        } => {
            print!(
                "{}",
                followup_render(&followup, &config.github_api_token).await
            );
            Ok(())
        }
    }
}
