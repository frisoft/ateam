use anyhow::Result;

mod client;
use client::{fetch_scored_prs, followup, sorted_ranked_prs, username};
pub mod cli;
use cli::{FollowupArgs, PrArgs};
mod filter;
mod render;
mod table;
mod types;

#[allow(clippy::missing_errors_doc)]
pub async fn pr(options: &PrArgs, github_api_token: &str) -> Result<Vec<types::ScoredPr>> {
    let username = get_username(&options.user, github_api_token).await;

    fetch_scored_prs(github_api_token, &username, options).await
}

#[allow(clippy::missing_errors_doc)]
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
        Some(username) => username.clone(),
        None => username::username(github_api_token).await,
    }
}
