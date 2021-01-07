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

    let username = client::username::username(&config.github_api_token);

    // println!(">> {:?}", options);
    
    let (responses, _) = get_responses(vec![], &config.github_api_token, &options, None)?;
    let sprs = responses.iter().map(|response_data|
        client::ranked_prs(
            &config.github_api_token,
            &username,
            options.required_approvals,
            &options,
            response_data,
        )
    ).flatten().collect::<Vec<types::ScoredPr>>();

    println!();
    print::prs(&client::sorted_ranked_prs(sprs), options.num, options.debug, options.short);

    Ok(())
}

pub fn get_responses(
    mut list: Vec<repo_view::ResponseData>,
    github_api_token: &str,
    options: &cli::Pr,
    after: Option<String>
) -> Result<(Vec<repo_view::ResponseData>, Option<String>), failure::Error> {
    eprint!(".");
    let (response_data, cursor) = client::query(github_api_token, options, after.clone())?;
    list.push(response_data);
    if cursor == None {
        Ok((list, cursor))
    } else {
        get_responses(list, github_api_token, options, cursor) 
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
