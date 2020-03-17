use failure::*;
// use log::*;

mod client;
use client::*;
mod cli;
mod config;
mod print;
mod table;
mod types;

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), failure::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

fn main() -> Result<(), failure::Error> {
    let cmd = cli::command();

    match cmd {
        cli::Command::Pr {
            repos,
            num,
            debug,
            short,
        } => pr_cmd(&repos, num, debug, short),
    }
}

fn pr_cmd(
    repos: &Vec<String>,
    num: Option<usize>,
    debug: bool,
    short: bool,
) -> Result<(), failure::Error> {
    let config = config::get_config().context("while reading from environment")?;

    let mut dataset: Vec<repo_view::ResponseData> = vec![];
    for repo in repos.iter() {
        let (owner, name) = parse_repo_name(repo).unwrap_or(("-", "-"));
        let data_or_error = client::query(&config.github_api_token, owner, name);
        match data_or_error {
            Ok(data) => dataset.push(data),
            _ => {
                return data_or_error;
            }
        }
    }

    //     let data: Vec<repo_view::ResponseData> = repos
    //         .iter()
    //         .map(|repo| {
    //             let (owner, name) = parse_repo_name(repo).unwrap_or(("-", "-"));
    //             let res = client::query(&config.github_api_token, owner, name)?;
    //             res
    //         })
    //         .collect();

    let sprs = client::ranked_prs(&dataset);
    print::prs(&sprs, num, debug, short);

    // for repo in repos {
    //     let (owner, name) = parse_repo_name(repo).unwrap_or(("-", "-"));

    //     let response_data: repo_view::ResponseData =
    //         client::query(&config.github_api_token, owner, name)?;

    //     let sprs = client::ranked_prs(&response_data);
    //     print::prs(&sprs, num, debug, short);
    // }

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
