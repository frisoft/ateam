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
        cli::Command::Pr {
            repo,
            num,
            debug,
            short,
            query,
        } => pr_cmd(&repo, num, debug, short, &query),
    }
}

fn pr_cmd(
    repo: &[String],
    num: Option<usize>,
    debug: bool,
    short: bool,
    query: &Option<String>,
) -> Result<(), failure::Error> {
    let config = config::get_config().context("while reading from environment")?;

    let response_data: repo_view::ResponseData =
        client::query(&config.github_api_token, repo, &query)?;

    let sprs = client::ranked_prs(&response_data);
    print::prs(&sprs, num, debug, short);

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
