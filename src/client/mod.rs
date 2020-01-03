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

pub fn query(
    github_api_token: &str,
    owner: &str,
    name: &str,
) -> Result<repo_view::ResponseData, failure::Error> {
    let q = RepoView::build_query(repo_view::Variables {
        // ) -> Result<repo_view::ResponseData, failure::Error> {
        //     let q = RepoView::build_query(repo_view::Variables {
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
    // info!("{:?}", response_body);
    if let Some(errors) = response_body.errors {
        println!("there are errors:");
        for error in &errors {
            println!("{:?}", error);
        }
    }
    // println!("{:?}", response_body.data);
    Ok(response_body.data.expect("missing response data"))
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
        .repository
        .as_ref()
        .expect("missing repository")
        .pull_requests
        .nodes
        .as_ref()
        .expect("pull request nodes is null")
        .iter()
        .flatten() // Extract value from Some(value) and remove the Nones
        .filter(|i| !has_wip_label(i))
        .map(|i| pr_stats(i)) // <-- Refactor
}

fn has_wip_label(pr: &repo_view::RepoViewRepositoryPullRequestsNodes) -> bool {
    pr_labels(pr).iter().any(|l| l == &"WIP")
}

fn pr_labels(pr: &repo_view::RepoViewRepositoryPullRequestsNodes) -> Vec<&str> {
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

fn pr_stats(pr: &repo_view::RepoViewRepositoryPullRequestsNodes) -> Pr {
    let (last_commit_pushed_date, last_commit_state) = last_commit(&pr);
    Pr {
        title: pr.title.clone(),
        url: pr.url.clone(),
        last_commit_pushed_date,
        tests_result: status_state_to_i(last_commit_state),
        open_conversations: pr_open_conversations(&pr.review_threads),
        num_approvals: pr_num_approvals(&pr.reviews),
        num_reviewers: pr_num_reviewers(&pr.reviews),
        additions: pr.additions,
        deletions: pr.deletions,
    }
}

fn last_commit(
    pr: &repo_view::RepoViewRepositoryPullRequestsNodes,
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

fn pr_open_conversations(
    review_threads: &repo_view::RepoViewRepositoryPullRequestsNodesReviewThreads,
) -> i64 {
    review_threads
        .nodes
        .as_ref()
        .map(|nodes| {
            nodes.iter().filter(|review_thread| {
                review_thread
                    .as_ref()
                    .map(|review_thread| review_thread_resolved_or_outdated(review_thread))
                    .unwrap_or(false)
            })
        })
        .map(|list| list.count())
        .unwrap_or(0) as i64
}

fn pr_num_approvals(
    reviews: &std::option::Option<repo_view::RepoViewRepositoryPullRequestsNodesReviews>,
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
    reviews: &std::option::Option<repo_view::RepoViewRepositoryPullRequestsNodesReviews>,
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

fn parse_date(date: &str) -> Option<DateTime<Utc>> {
    match date.parse::<DateTime<Utc>>() {
        Ok(date_time) => Some(date_time),
        Err(_) => None,
    }
}

fn review_thread_resolved_or_outdated(
    review_thread: &repo_view::RepoViewRepositoryPullRequestsNodesReviewThreadsNodes,
) -> bool {
    review_thread.is_resolved
        || review_thread
            .comments
            .nodes
            .as_ref()
            .map(|comments| {
                comments.iter().all(|comment| {
                    comment
                        .as_ref()
                        .map(|comment| comment.outdated)
                        .unwrap_or(false)
                })
            })
            .unwrap_or(false)
}
