use failure::*;
// use log::*;

mod client;
use client::*;
mod cli;
mod config;
mod print;
mod table;
mod types;

#[tokio::main]
async fn main() -> Result<(), failure::Error> {
    let cmd = cli::command();

    match cmd {
        cli::Ateam {
            cmd: cli::Command::Pr(pr),
        } => pr_cmd(&pr).await,
        cli::Ateam {
            cmd: cli::Command::Followup(followup),
        } => followup_cmd(&followup).await,
    }
}

async fn pr_cmd(options: &cli::Pr) -> Result<(), failure::Error> {
    let config = config::get_config().context("while reading from environment")?;

    let username = get_username(&options.user, &config.github_api_token).await;

    let (responses, _) = get_responses(vec![], &config.github_api_token, &username, options, None)?;
    let sprs = responses
        .iter()
        .flat_map(|response_data| {
            client::ranked_prs(
                &config.github_api_token,
                &username,
                options.required_approvals,
                options,
                response_data,
            )
        })
        .collect::<Vec<types::ScoredPr>>();

    eprintln!(".");
    print::prs(
        &client::sorted_ranked_prs(sprs),
        options.num,
        options.debug,
        options.short,
        options.json,
    );

    Ok(())
}

async fn followup_cmd(options: &cli::Followup) -> Result<(), failure::Error> {
    let config = config::get_config().context("while reading from environment")?;

    let username = get_username(&options.user, &config.github_api_token).await;

    let reviews = client::followup::followup(&config.github_api_token, &username).await;

    print::reviews(&reviews, options.json);

    Ok(())
}

async fn get_username(user: &Option<String>, github_api_token: &str) -> String {
    match user {
        Some(username) => username.to_string(),
        None => client::username::username(github_api_token).await,
    }
}

pub fn get_responses(
    mut list: Vec<repo_view::ResponseData>,
    github_api_token: &str,
    username: &str,
    options: &cli::Pr,
    after: Option<String>,
) -> Result<(Vec<repo_view::ResponseData>, Option<String>), failure::Error> {
    eprint!(".");
    let (response_data, cursor) = client::query(github_api_token, username, options, after)?;
    list.push(response_data);
    if cursor == None {
        Ok((list, cursor))
    } else {
        get_responses(list, github_api_token, username, options, cursor)
    }
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
