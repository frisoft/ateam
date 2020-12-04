use super::cli;
use super::types::*;
use chrono::prelude::*;
use graphql_client::*;
use rayon::prelude::*;
use regex::Regex;
use std::collections::HashSet;
mod blame;
pub mod username;

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
    options: &cli::Pr,
) -> Result<repo_view::ResponseData, failure::Error> {
    let query_argument = github_query(options);
    if options.debug {
        println!(">> GitHub query: {:?}", query_argument);
    }
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
        // "is:pr is:open draft:false -status:progess -status:failure {}{}{}{}",
        "is:pr is:open draft:false {}{}{}{}{}{}{}{}",
        query_include_mine(options.include_mine),
        query_include_tests_in_progress(options.include_tests_in_progress),
        query_include_tests_failure(options.include_tests_failure),
        query_include_reviewed_by_me(options.include_reviewed_by_me),
        query_labels(&options.label),
        query_repos(&options.repo),
        query_org(&options.org),
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

fn query_include_tests_in_progress(include_tests_in_progress: bool) -> &'static str {
    if include_tests_in_progress {
        ""
    } else {
        "-status:progess "
    }
}

fn query_include_tests_failure(include_tests_failure: bool) -> &'static str {
    if include_tests_failure {
        ""
    } else {
        "-status:failure "
    }
}

fn query_include_reviewed_by_me(include_reviewed_by_me: bool) -> &'static str {
    if include_reviewed_by_me {
        ""
    } else {
        "-reviewed-by:@me "
    }
}

fn query_labels(labels: &[String]) -> String {
    labels
        .iter()
        .map(|label| format!("label:{} ", label))
        .collect()
}

fn query_repos(repos: &[String]) -> String {
    repos.iter().map(|repo| format!("repo:{} ", repo)).collect()
}

fn query_org(org: &Option<String>) -> String {
    if let Some(org) = org {
        format!("org:{} ", org)
    } else {
        "".to_string()
    }
}

pub fn ranked_prs<'a>(
    github_api_token: &str,
    username: &Option<String>,
    required_approvals: u8,
    options: &cli::Pr,
    response_data: &'a repo_view::ResponseData,
) -> Vec<ScoredPr<'a>> {
    let re = regex(&options.regex);
    let mut sprs: Vec<ScoredPr> = prs(github_api_token, username, &re, options, &response_data)
        .into_par_iter()
        .map(|pr| scored_pr(required_approvals, pr))
        .collect();
    sprs.sort_by_key(|scored_pr| (scored_pr.score.total() * 1000.0) as i64);
    sprs.reverse();
    sprs
}

fn regex(regex_text: &Option<String>) -> Option<Regex> {
    let text = match regex_text {
        Some(text) => text,
        None => return None,
    };
    Some(Regex::new(&text).unwrap())
}

fn scored_pr(required_approvals: u8, pr: Pr) -> ScoredPr {
    let s = Score::from_pr(required_approvals, &pr);
    ScoredPr { pr, score: s }
}

fn prs<'a>(
    github_api_token: &str,
    username: &Option<String>,
    regex: &Option<Regex>,
    options: &cli::Pr,
    response_data: &'a repo_view::ResponseData,
) -> Vec<Pr<'a>> {
    response_data
        .search
        .edges
        .par_iter()
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
        .filter(move |i| !is_empty(i) && regex_match(regex, i))
        .map(move |i| pr_stats(github_api_token, username, options, &i)) // <-- Refactor
        .collect()
}

fn regex_match(
    regex: &Option<Regex>,
    pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest,
) -> bool {
    match regex {
        Some(re) => re.is_match(&pr.title),
        None => true,
    }
}

fn is_empty(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> bool {
    pr.additions == 0 && pr.deletions == 0
}

fn pr_files(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> Vec<&str> {
    match &pr.files {
        Some(files) => files
            .nodes
            .iter()
            .flatten()
            .flatten()
            .map(|f| f.path.as_ref())
            .collect(),
        None => vec![],
    }
}

fn pr_stats<'a>(
    github_api_token: &str,
    username: &Option<String>,
    options: &cli::Pr,
    pr: &'a repo_view::RepoViewSearchEdgesNodeOnPullRequest,
) -> Pr<'a> {
    let (last_commit_pushed_date, last_commit_state) = last_commit(&pr);
    let (files, is_author) = if options.blame {
        let files = pr_files(&pr);
        let is_author = blame::is_author(
            github_api_token,
            &pr.repository.name,
            &pr.repository.owner.login,
            &files,
            &username.as_ref().unwrap_or(&"".to_string()),
        );
        (files, is_author)
    } else {
        (vec![], false)
    };
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
        based_on_main_branch: pr_based_on_main_branch(&pr.base_ref_name),
        files,
        is_author,
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

fn pr_open_conversations(
    review_threads: &repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewThreads,
) -> i64 {
    review_threads
        .nodes
        .as_ref()
        .map(|nodes| {
            nodes.iter().filter(|review_thread| {
                review_thread
                    .as_ref()
                    .map(|review_thread| !review_thread.is_resolved && !review_thread.is_outdated)
                    .unwrap_or(false)
            })
        })
        .map(|list| list.count())
        .unwrap_or(0) as i64
}

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

fn pr_based_on_main_branch(base_branch_name: &str) -> bool {
    base_branch_name == "main" || base_branch_name == "master"
}
