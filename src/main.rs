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

    let sprs: Vec<types::ScoredPr> =
        fetch_scored_prs(&config.github_api_token, &username, options).await?;
    // let sprs = responses
    //     .into_iter()
    //     .flatten()
    //     .collect::<Vec<types::ScoredPr>>();
    // let sprs = responses
    //     .iter()
    //     .flat_map(|response_data| {
    //         client::ranked_prs(
    //             &config.github_api_token,
    //             &username,
    //             options.required_approvals,
    //             options,
    //             response_data,
    //         )
    //     })
    //     .collect::<Vec<types::ScoredPr>>();

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

pub async fn fetch_scored_prs(
    github_api_token: &str,
    username: &str,
    options: &cli::Pr,
    //) -> Result<Vec<repo_view::ResponseData>, failure::Error> {
) -> Result<Vec<types::ScoredPr>, failure::Error> {
    // let (response_data, cursor) = client::query(github_api_token, username, options, after)?;
    // list.push(response_data);
    // if cursor == None {
    //     Ok((list, cursor))
    // } else {
    //     fetch_scored_prs(list, github_api_token, username, options, cursor)
    // }
    let mut list: Vec<Vec<types::ScoredPr>> = vec![];
    let mut list_data: Vec<repo_view::ResponseData> = vec![];
    let mut cursor = None;
    let mut first = true;
    loop {
        // while let Some(cursor_value) = cursor {
        eprint!(".");

        let o_get_ranked_prs = if !first {
            match list_data.pop() {
                Some(data) => Some(async {
                    ranked_prs(
                        github_api_token,
                        username,
                        options.required_approvals,
                        options,
                        data,
                    )
                }),
                None => None,
            }
        } else {
            None
        };

        let o_get_next_response_data_and_cursor = if first || cursor != None {
            Some(client::query(
                github_api_token,
                username,
                options,
                cursor.clone(),
            ))
        } else {
            None
        };

        if o_get_ranked_prs.is_some() && o_get_next_response_data_and_cursor.is_some() {
            // Bot future are present, I can do them in parallel
            let (prs, response_and_cursor) = futures::join!(
                o_get_ranked_prs.unwrap(),
                o_get_next_response_data_and_cursor.unwrap()
            );
            list.push(prs);
            let (new_response_data, new_cursor) = response_and_cursor?;
            cursor = new_cursor;
            list_data.push(new_response_data);
        } else if o_get_ranked_prs.is_some() {
            // Only one future to await
            list.push(o_get_ranked_prs.unwrap().await);
        } else if o_get_next_response_data_and_cursor.is_some() {
            // Only one future to await
            let (new_response_data, new_cursor) =
                o_get_next_response_data_and_cursor.unwrap().await?;
            list_data.push(new_response_data);
            cursor = new_cursor;
        } else {
            break;
        }

        first = false;
    }

    Ok(list.into_iter().flatten().collect::<Vec<types::ScoredPr>>())
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
