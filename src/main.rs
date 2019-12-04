use failure::*;
// use log::*;

mod client;
use client::*;
mod cli;
mod config;
mod print;
mod types;

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), failure::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

fn main() -> Result<(), failure::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config: config::Config = config::get_config().context("while reading from environment")?;

    let args = cli::command();

    let repo = args.repo;
    let (owner, name) = parse_repo_name(&repo).unwrap_or(("tomhoule", "graphql-client"));

    let response_data: repo_view::ResponseData =
        client::query(&config.github_api_token, owner, name)?;

    // println!("{}/{} - 🌟 {}", owner, name, stars.unwrap_or(0),);
    // let mut table = prettytable::Table::new();
    // table.add_row(row!(b => "issue", "comments"));

    for spr in client::ranked_prs(&response_data).iter() {
        // table.add_row(row!(
        //     pr.title,
        //     // pr.commits.total_count // ,
        //     // pr.                       // pr.comments.total_count
        // ));

        // let review_threads = &pr
        //     .review_threads
        //     .nodes
        //     .expect("review threads nodes is null");

        // println!("Review threads nodes: {:?}", review_threads);

        // let review_threads = &pr.review_threads
        // for review_thread in &pr.review_threads.nodes {
        //     // if let Some(review_thread) = review_thread {
        //     //     println!("Review thread: {:?}", review_thread);
        //     // }
        // };

        // if let Some(reviews) = &pr.reviews {
        //     for review in reviews.nodes.as_ref().expect("reviews nodes is null") {
        //         if let Some(review) = review {
        //             println!("{:?}", review);
        //         }
        //     }
        // };

        // println!("{:?}", pr.reviews.first.comments);
        // let last_commit = last_commit(&pr);
        // println!("Last commit (pushed date, state): {:?}", last_commit(pr));
        // println!("Review threads count: {:?}", pr.review_threads.total_count);
        print::pr(spr);
    }

    // table.printstd();
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
