use super::cli;
use super::types::*;
use chrono::prelude::*;
use graphql_client::*;
use std::collections::HashSet;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/schema.graphql",
    query_path = "src/client/pr.graphql",
    response_derives = "Debug"
)]
pub struct RepoView;

type URI = String;

// pub fn quer2(github_api_token: &str, owner: &str, ticket: &str) -> Result<Vec<Pr>, failure::Error> {
//     let q = PullRequests::build_query(pull_requests::Variables {
//         query: format!("org:{} is:pr [{}] in:title", owner, ticket),
//     });
//     let client = reqwest::Client::new();
//     let mut res = client
//         .post("https://api.github.com/graphql")
//         .bearer_auth(github_api_token)
//         .json(&q)
//         .send()?;
//     let response_body: Response<pull_requests::ResponseData> = res.json()?;
//     if let Some(errors) = response_body.errors {
//         println!("there are errors:");
//         for error in &errors {
//             println!("{:?}", error);
//         }
//     }
//     // Ok(prs(&response_body.data.expect("missing response data")).collect())
//     // println!("{:?}", response_body.data);
//     let prs: Vec<Pr> = prs(&response_body.data, &ticket);
//     // println!("{:?}", prs);
//     Ok(prs)
// }

pub fn query(
    github_api_token: &str,
    options: &cli::Pr,
) -> Result<repo_view::ResponseData, failure::Error> {
    let query_argument = github_query(options);
    // println!(">> {:?}", query_argument);
    let q = RepoView::build_query(repo_view::Variables {
        query: query_argument,
    });
    let client = reqwest::Client::new();
    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(github_api_token)
        .json(&q)
        .send()?;

    // println!(
    // ">>-----------------------------------\n{}\n-------------------------------\n",
    // res.text()?
    // );
    // println!(">> {:?}", res.json()?);
    // println!("{:?}", res);

    let response_body: Response<repo_view::ResponseData> = res.json()?;
    // println!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");
        for error in &errors {
            println!("{:?}", error);
        }
    }
    // println!("{:?}", response_body.data);
    Ok(response_body.data.expect("missing response data"))
}

fn github_query(options: &cli::Pr) -> String {
    format!(
        "is:pr is:open draft:false -status:progess -status:failure {}{}{}{}",
        query_include_mine(options.include_mine),
        query_excluse_reciewed_by_me(options.exclude_reviewed_by_me),
        query_repos(&options.repo),
        &options.query.as_ref().unwrap_or(&"".to_string())
    )
}

fn query_include_mine(include_mine: bool) -> &'static str {
    if include_mine {
        ""
    } else {
        "-author:@me "
    }
}

fn query_excluse_reciewed_by_me(exclude_reviewed_by_me: bool) -> &'static str {
    if exclude_reviewed_by_me {
        "-reviewed-by:@me "
    } else {
        ""
    }
}

fn query_repos(repos: &[String]) -> String {
    repos.iter().map(|repo| format!("repo:{} ", repo)).collect()
}

pub fn ranked_prs(response_data: &repo_view::ResponseData) -> Vec<ScoredPr> {
    let mut sprs: Vec<ScoredPr> = prs(&response_data).map(scored_pr).collect();
    sprs.sort_by_key(|scored_pr| (scored_pr.score.total() * 1000.0) as i64);
    sprs.reverse();
    sprs
}

fn scored_pr(pr: Pr) -> ScoredPr {
    let s = Score::from_pr(&pr);
    ScoredPr { pr, score: s }
}

fn prs(response_data: &repo_view::ResponseData) -> impl Iterator<Item = Pr> + '_ {
    response_data
        .search
        .edges
        .iter()
        .flatten()
        .flatten()
        .map(|i| i.node.as_ref()) // <-- Refactor
        .map(|n| match n {
            Some(repo_view::RepoViewSearchEdgesNode::PullRequest(pull_request)) => {
                Some(pull_request)
            }
            _ => None,
        }) // <-- Refactor
        .flatten() // Extract value from Some(value) and remove the Nones
        .filter(|i| !has_wip_label(i) && !is_empty(i))
        .map(|i| pr_stats(&i)) // <-- Refactor
}

fn has_wip_label(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> bool {
    pr_labels(pr).iter().any(|l| l == &"WIP")
}

fn is_empty(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> bool {
    pr.additions == 0 && pr.deletions == 0
}

fn pr_labels(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> Vec<&str> {
    match &pr.labels {
        Some(labels) => labels
            .nodes
            .iter()
            .flatten()
            .flatten()
            .map(|l| l.name.as_ref())
            .collect(),
        None => vec![],
    }
}

fn pr_stats(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> Pr {
    let (last_commit_pushed_date, last_commit_state) = last_commit(&pr);
    Pr {
        title: pr.title.clone(),
        url: pr.url.clone(),
        last_commit_pushed_date,
        tests_result: status_state_to_i(last_commit_state),
        open_conversations: 0, // pr_open_conversations(&pr.review_threads), <-- reviewThreads is no more provided by GitHub
        num_approvals: pr_num_approvals(&pr.reviews),
        num_reviewers: pr_num_reviewers(&pr.reviews),
        additions: pr.additions,
        deletions: pr.deletions,
    }
}

fn last_commit(
    pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest,
) -> (Option<DateTime<Utc>>, Option<&repo_view::StatusState>) {
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
        (parse_date(pushed_date), state)
    } else {
        (None, None)
    }
}

fn status_state_to_i(state: Option<&repo_view::StatusState>) -> i64 {
    match state {
        Some(state) => match state {
            repo_view::StatusState::SUCCESS => 0,
            repo_view::StatusState::PENDING => 1,
            repo_view::StatusState::FAILURE => 2,
            repo_view::StatusState::ERROR => 3,
            repo_view::StatusState::EXPECTED => 3,
            repo_view::StatusState::Other(_) => 3,
        },
        None => 3,
    }
}

// fn pr_open_conversations(
//     review_threads: &repo_view::RepoViewRepositoryPullRequestsNodesReviewThreads,
// ) -> i64 {
//     review_threads
//         .nodes
//         .as_ref()
//         .map(|nodes| {
//             nodes.iter().filter(|review_thread| {
//                 review_thread
//                     .as_ref()
//                     .map(|review_thread| review_thread_resolved_or_outdated(review_thread))
//                     .unwrap_or(false)
//             })
//         })
//         .map(|list| list.count())
//         .unwrap_or(0) as i64
// }

fn pr_num_approvals(
    reviews: &std::option::Option<repo_view::RepoViewSearchEdgesNodeOnPullRequestReviews>,
) -> i64 {
    reviews
        .as_ref()
        .and_then(|reviews| reviews.nodes.as_ref())
        .map(|nodes| {
            nodes
                .iter()
                .map(|review| review.as_ref().map(|review| &review.state))
                .filter(|state| state == &Some(&repo_view::PullRequestReviewState::APPROVED))
                .count()
        })
        .unwrap_or(0) as i64
}

fn pr_num_reviewers(
    reviews: &std::option::Option<repo_view::RepoViewSearchEdgesNodeOnPullRequestReviews>,
) -> i64 {
    let reviewers = reviews
        .as_ref()
        .and_then(|reviews| reviews.nodes.as_ref())
        .map(|nodes| {
            nodes
                .iter()
                .map(|review| {
                    review
                        .as_ref()
                        .and_then(|review| review.author.as_ref())
                        .map(|author| &author.login)
                })
                .flatten()
        });

    let s: HashSet<&String> = match reviewers {
        Some(reviewers) => reviewers.collect(),
        None => HashSet::new(),
    };

    s.len() as i64
}

fn parse_date(date: &Option<String>) -> Option<DateTime<Utc>> {
    match date {
        Some(s) => match s.parse::<DateTime<Utc>>() {
            Ok(date_time) => Some(date_time),
            Err(_) => None,
        },
        None => None,
    }
}

// fn review_thread_resolved_or_outdated(
//     review_thread: &repo_view::RepoViewRepositoryPullRequestsNodesReviewThreadsNodes,
// ) -> bool {
//     review_thread.is_resolved
//         || review_thread
//             .comments
//             .nodes
//             .as_ref()
//             .map(|comments| {
//                 comments.iter().all(|comment| {
//                     comment
//                         .as_ref()
//                         .map(|comment| comment.outdated)
//                         .unwrap_or(false)
//                 })
//             })
//             .unwrap_or(false)
// }
