use anyhow::Result;
// use log::*;

mod client;
use client::*;
pub mod cli;
use cli::{FollowupArgs, PrArgs};
mod render;
mod table;
mod types;

pub async fn pr(options: &PrArgs, github_api_token: &str) -> Result<Vec<types::ScoredPr>> {
    let username = get_username(&options.user, github_api_token).await;

    fetch_scored_prs(github_api_token, &username, options).await
}

pub async fn pr_render(options: &PrArgs, github_api_token: &str) -> Result<String> {
    let sprs: Vec<types::ScoredPr> = pr(options, github_api_token).await?;

    Ok(render::prs(
        &sorted_ranked_prs(sprs),
        options.num,
        options.debug,
        options.short,
        options.json,
    ))
}

pub async fn followup(options: &FollowupArgs, github_api_token: &str) -> Vec<types::Review> {
    let username = get_username(&options.user, github_api_token).await;

    followup::followup(github_api_token, &username).await
}

pub async fn followup_render(options: &FollowupArgs, github_api_token: &str) -> String {
    let reviews = followup(options, github_api_token).await;

    render::reviews(&reviews, options.json)
}

pub async fn get_username(user: &Option<String>, github_api_token: &str) -> String {
    match user {
        Some(username) => username.to_string(),
        None => username::username(github_api_token).await,
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
