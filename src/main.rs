use failure::*;
// use log::*;

mod client;
use client::*;
mod cli;
mod config;
mod print;
mod table;
mod types;
use futures::future::{BoxFuture, FutureExt};
use futures::stream::{self, StreamExt};

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

    let (responses, _) =
        get_responses(vec![], &config.github_api_token, &username, options, None).await?;

    let github_api_token = &config.github_api_token.clone();
    let user = "something".to_string().clone();
    let sprs = stream::iter(responses)
        .then(|response_data| async move {
            client::ranked_prs(
                &github_api_token,
                &user,
                options.required_approvals,
                options,
                response_data,
            )
            .await
        })
        .collect::<Vec<Vec<types::ScoredPr>>>();
    //let sprs = sprs.into_iter().flatten().collect();

    let sprs = sprs.await.into_iter().flatten().collect();

    // .flatten()
    // .collect::<Vec<Result<_, _>>>()
    // .await?;

    // let sprs = sprs.into_iter().flatten().collect::<Vec<types::ScoredPr>>();

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
) -> BoxFuture<'static, Result<(Vec<repo_view::ResponseData>, Option<String>), failure::Error>> {
    async move {
        eprint!(".");
        let (response_data, cursor) =
            client::query(github_api_token, username, options, after).await?;
        list.push(response_data);
        if cursor == None {
            Ok((list, cursor))
        } else {
            get_responses(list, github_api_token, username, options, cursor).await
        }
    }
    .boxed()
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
