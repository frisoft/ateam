use failure::*;
// use log::*;

mod client;
use client::*;
mod cli;
mod config;
mod filter;
mod print;
mod table;
mod types;

fn main() -> Result<(), failure::Error> {
    let cmd = cli::command();

    match cmd {
        cli::Ateam {
            debug,
            cmd: cli::Command::Pr(pr),
        } => pr_cmd(&pr, debug),
    }
}

fn pr_cmd(options: &cli::Pr, debug: bool) -> Result<(), failure::Error> {
    let config = config::get_config().context("while reading from environment")?;

    // println!(">> {:?}", options);
    let response_data: repo_view::ResponseData = client::query(&config.github_api_token, &options)?;

    let sprs = client::ranked_prs(options.required_approvals, &response_data);
    let sprs = filter::regex(&options.regex, sprs);
    print::prs(&sprs, options.num, debug, options.short);

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
