use chrono::prelude::*;
use core::option::Iter;
use failure::*;
use graphql_client::*;
use log::*;
use prettytable::*;
use serde::*;
use structopt::StructOpt;

type URI = String;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/schema.graphql",
    query_path = "src/query_1.graphql",
    response_derives = "Debug"
)]
struct RepoView;

#[derive(StructOpt)]
#[structopt(author, about)]
struct Command {
    #[structopt(name = "repository")]
    repo: String,
}

#[derive(Deserialize, Debug)]
struct Env {
    github_api_token: String,
}

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), failure::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

fn query(
    github_api_token: &str,
    owner: &str,
    name: &str,
) -> Result<repo_view::ResponseData, failure::Error> {
    let q = RepoView::build_query(repo_view::Variables {
        owner: owner.to_string(),
        name: name.to_string(),
    });

    let client = reqwest::Client::new();

    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(github_api_token)
        .json(&q)
        .send()?;

    let response_body: Response<repo_view::ResponseData> = res.json()?;
    info!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");

        for error in &errors {
            println!("{:?}", error);
        }
    }

    // println!("{:?}", response_body.data);
    Ok(response_body.data.expect("missing response data"))
}

// fn last_commit_age(pushed_date: String) {
//     let t = pushed_date.parse::<DateTime<Utc>>;
// }

fn last_commit(
    pr: &repo_view::RepoViewRepositoryPullRequestsNodes,
) -> (Option<&String>, Option<&repo_view::StatusState>) {
    if let Some((pushed_date, state)) = pr
        .commits
        .nodes
        .as_ref()
        .and_then(|nodes| nodes[0].as_ref())
        .map(|node| {
            (
                &node.commit.pushed_date,
                node.commit.status.as_ref().map(|status| &status.state),
            )
        })
    {
        (Some(pushed_date), state)
    } else {
        (None, None)
    }
}

fn prs(response_data: &repo_view::ResponseData) -> impl Iterator<Item = Pr> + '_ {
    response_data
        .repository
        .as_ref()
        .expect("missing repository")
        .pull_requests
        .nodes
        .as_ref()
        .expect("pull request nodes is null")
        .iter()
        .filter(|i| i.is_some()) // <-- Refactor
        .map(|i| pr_stats(i.as_ref().unwrap())) // <-- Refactor
}

struct Pr {
    title: String,
    additions: i64,
    deletions: i64,
    last_commit_pushed_date: Option<String>,
    last_commit_state: i64,
}

fn pr_stats(pr: &repo_view::RepoViewRepositoryPullRequestsNodes) -> Pr {
    let (last_commit_pushed_date, last_commit_state) = last_commit(&pr);
    Pr {
        title: pr.title.clone(),
        additions: pr.additions,
        deletions: pr.deletions,
        last_commit_pushed_date: last_commit_pushed_date.cloned(),
        last_commit_state: match last_commit_state {
            Some(repo_view::StatusState::SUCCESS) => 0,
            Some(repo_view::StatusState::ERROR) => 3,
            Some(repo_view::StatusState::EXPECTED) => 3,
            Some(repo_view::StatusState::FAILURE) => 2,
            Some(repo_view::StatusState::PENDING) => 1,
            None => 3,
        },
    }
}

fn main() -> Result<(), failure::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config: Env = envy::from_env().context("while reading from environment")?;

    let args = Command::from_args();

    let repo = args.repo;
    let (owner, name) = parse_repo_name(&repo).unwrap_or(("tomhoule", "graphql-client"));

    let response_data: repo_view::ResponseData = query(&config.github_api_token, owner, name)?;

    // println!("{}/{} - 🌟 {}", owner, name, stars.unwrap_or(0),);
    // let mut table = prettytable::Table::new();
    // table.add_row(row!(b => "issue", "comments"));

    for pr in prs(&response_data) {
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
        println!("===============================================================");
        println!("PR title: {:?}", pr.title);
        println!("PR additions: {:?}", pr.additions);
        println!("PR deletions: {:?}", pr.deletions);
        println!("Last commit pushed date {:?}", pr.last_commit_pushed_date);
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
