use failure::*;
// use log::*;

mod client;
use client::*;
mod cli;
mod config;
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
        cli::Command::Pr { repo, num, debug } => pr_cmd(&repo, num, debug),
    }
}

fn pr_cmd(repo: &str, num: Option<usize>, debug: bool) -> Result<(), failure::Error> {
    let config = config::get_config().context("while reading from environment")?;

    let (owner, name) = parse_repo_name(repo).unwrap_or(("tomhoule", "graphql-client"));

    let response_data: repo_view::ResponseData =
        client::query(&config.github_api_token, owner, name)?;

    let sprs = client::ranked_prs(&response_data);
    let table = table::from(&sprs, num.unwrap_or(10000), debug);

    table.printstd();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repo_name_works() {
        assert_eq!(
            parse_repo_name("graphql-rust/graphql-client").unwrap(),
            ("graphql-rust", "graphql-client")
        );
        assert!(parse_repo_name("abcd").is_err());
    }
}
