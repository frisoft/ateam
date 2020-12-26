use failure::*;
// use log::*;

mod client;
use client::*;
mod cli;
mod config;
mod print;
mod table;
mod types;

fn main() -> Result<(), failure::Error> {
    let cmd = cli::command();

    match cmd {
        cli::Ateam {
            cmd: cli::Command::Pr(pr),
        } => pr_cmd(&pr),
    }
}

fn pr_cmd(options: &cli::Pr) -> Result<(), failure::Error> {
    let config = config::get_config().context("while reading from environment")?;

    // println!(">> {:?}", options);
    let response_data: repo_view::ResponseData = client::query(&config.github_api_token, &options)?;

    let username = client::username::username(&config.github_api_token);

    let sprs = client::ranked_prs(
        &config.github_api_token,
        &username,
        options.required_approvals,
        &options,
        &response_data,
    );

    println!();
    print::prs(&sprs, options.num, options.debug, options.short);

    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn parse_repo_name_works() {
//         assert_eq!(
//             parse_repo_name("graphql-rust/graphql-client").unwrap(),
//             ("graphql-rust", "graphql-client")
//         );
//         assert!(parse_repo_name("abcd").is_err());
//     }
// }
