use super::cli;
use super::types::*;
use chrono::prelude::{DateTime as DT, Utc};
use graphql_client::*;
use itertools::Itertools;
use regex::Regex;
mod blame;
pub mod followup;
pub mod username;
use futures::stream::{FuturesUnordered, StreamExt};

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/schema.graphql",
    query_path = "src/client/pr.graphql",
    response_derives = "Debug"
)]
pub struct RepoView;

#[allow(clippy::upper_case_acronyms)]
type URI = String;

type DateTime = String;

const AGENT: &str = concat!("ateam/", env!("CARGO_PKG_VERSION"));

pub async fn fetch_scored_prs(
    github_api_token: &str,
    username: &str,
    options: &cli::Pr,
) -> Result<Vec<ScoredPr>, failure::Error> {
    let mut list_prs: Vec<Vec<ScoredPr>> = vec![];
    let mut list_data: Vec<repo_view::ResponseData> = vec![];
    let mut cursor = None;
    let mut first = true;
    loop {
        eprint!(".");

        let o_get_ranked_prs = if !first {
            list_data.pop().map(|data| {
                ranked_prs(
                    github_api_token,
                    username,
                    options.required_approvals,
                    options,
                    data,
                )
            })
        } else {
            None
        };

        let o_get_next_response_data_and_cursor = if first || cursor != None {
            Some(query(github_api_token, username, options, cursor.clone()))
        } else {
            None
        };

        if o_get_ranked_prs.is_some() && o_get_next_response_data_and_cursor.is_some() {
            // Bot future are present, I can do them in parallel
            let (prs, response_and_cursor) = futures::join!(
                o_get_ranked_prs.unwrap(),
                o_get_next_response_data_and_cursor.unwrap()
            );
            list_prs.push(prs);
            let (new_response_data, new_cursor) = response_and_cursor?;
            cursor = new_cursor;
            list_data.push(new_response_data);
        } else if o_get_ranked_prs.is_some() {
            // Only one future to await
            list_prs.push(o_get_ranked_prs.unwrap().await);
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

    Ok(list_prs.into_iter().flatten().collect::<Vec<ScoredPr>>())
}

pub async fn call<V: serde::Serialize>(
    github_api_token: &str,
    q: &QueryBody<V>,
) -> Result<reqwest::Response, failure::Error> {
    let client = reqwest::Client::builder().user_agent(AGENT).build()?;
    let res = client
        .post("https://api.github.com/graphql")
        .json(&q)
        .bearer_auth(github_api_token)
        .send()
        .await?;
    Ok(res)
}

async fn query(
    github_api_token: &str,
    username: &str,
    options: &cli::Pr,
    after: Option<String>,
) -> Result<(repo_view::ResponseData, Option<String>), failure::Error> {
    let query_argument = github_query(username, options);
    if options.debug {
        println!(">> GitHub query: {:?}", query_argument);
    }
    let batch_size = limited_batch_size(options.batch_size);
    let q = RepoView::build_query(repo_view::Variables {
        query: query_argument,
        first: batch_size,
        after,
        num_checks: match options.tests_regex {
            Some(_) => 20,
            None => 0,
        },
    });

    let res = call(github_api_token, &q).await?;

    let response_body: Response<repo_view::ResponseData> = res.json().await?;
    // println!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");
        for error in &errors {
            println!("{:?}", error);
        }
    }
    // println!("{:?}", response_body.data);

    let response_data = response_body.data.expect("missing response data");
    let cursor = last_item_cursor(&response_data, batch_size);

    Ok((response_data, cursor))
}

fn limited_batch_size(batch_size: u8) -> i64 {
    (if batch_size <= 100 { batch_size } else { 100 }) as i64
}

fn last_item_cursor(response_data: &repo_view::ResponseData, batch_size: i64) -> Option<String> {
    match &response_data.search.edges {
        Some(items) => {
            if items.len() < batch_size as usize {
                None
            } else {
                match items.last() {
                    Some(Some(item)) => Some(item.cursor.clone()),
                    _ => None,
                }
            }
        }
        None => None,
    }
}

fn github_query(username: &str, options: &cli::Pr) -> String {
    format!(
        // "is:pr is:open draft:false -status:progess -status:failure {}{}{}{}",
        "is:pr is:open {}{}{}{}{}{}{}",
        query_drafts(options.include_drafts),
        query_mine(username, options.only_mine),
        query_requested(username, options.requested),
        query_labels(&options.label, &options.exclude_label),
        query_repos(&options.repo),
        query_org(&options.org),
        &options.query.join(" ")
    )
}

fn query_drafts(include_drafts: bool) -> &'static str {
    if include_drafts {
        ""
    } else {
        "draft:false "
    }
}

fn query_mine(username: &str, only_mine: bool) -> String {
    if only_mine {
        format!("author:{} ", username)
    } else {
        "".to_string()
    }
}

fn query_requested(username: &str, requested: bool) -> String {
    if requested {
        format!("review-requested:{} ", username)
    } else {
        "".to_string()
    }
}

fn query_labels(labels: &[String], exclude_label: &[String]) -> String {
    format!(
        "{}{}",
        labels
            .iter()
            .map(|label| format!("label:\"{}\" ", label))
            .collect::<String>(),
        exclude_label
            .iter()
            .map(|label| format!("-label:\"{}\" ", label))
            .collect::<String>()
    )
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

async fn ranked_prs(
    github_api_token: &str,
    username: &str,
    required_approvals: u8,
    options: &cli::Pr,
    response_data: repo_view::ResponseData,
) -> Vec<ScoredPr> {
    prs(github_api_token, username, options, response_data)
        .await
        .into_iter()
        .map(|pr| scored_pr(required_approvals, pr))
        .collect::<Vec<ScoredPr>>()
}

pub fn sorted_ranked_prs(mut sprs: Vec<ScoredPr>) -> Vec<ScoredPr> {
    sprs.sort_by_key(|scored_pr| (scored_pr.score.total() * 1000.0) as i64);
    sprs.reverse();
    sprs
}

fn regex(regex_text: &Option<String>) -> Option<Regex> {
    let text = match regex_text {
        Some(text) => text,
        None => return None,
    };
    Some(Regex::new(text).unwrap())
}

fn scored_pr(required_approvals: u8, pr: Pr) -> ScoredPr {
    let s = Score::from_pr(required_approvals, &pr);
    ScoredPr { pr, score: s }
}

async fn prs(
    github_api_token: &str,
    username: &str,
    options: &cli::Pr,
    response_data: repo_view::ResponseData,
) -> Vec<Pr> {
    let re = regex(&options.regex);
    let re_not = regex(&options.regex_not);
    let prs: FuturesUnordered<_> = response_data
        .search
        .edges
        //.into_par_iter()
        .into_iter()
        .flatten()
        .flatten()
        .map(|i| i.node)
        .filter_map(|n| match n {
            Some(repo_view::RepoViewSearchEdgesNode::PullRequest(pull_request)) => {
                Some(pull_request)
            }
            _ => None,
        })
        .filter(|i| {
            include_pr(
                i,
                &re,
                &re_not,
                username,
                options.include_mine,
                options.only_mine,
                options.include_reviewed_by_me,
            )
        })
        .map(|i| async move { pr_stats(github_api_token, username, options, i).await })
        .collect();

    prs.collect::<Vec<Option<Pr>>>()
        .await
        .into_iter()
        .flatten()
        .collect()
}

fn include_pr(
    pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest,
    regex: &Option<Regex>,
    regex_not: &Option<Regex>,
    username: &str,
    include_mine: bool,
    only_mine: bool,
    include_reviewed_by_me: bool,
) -> bool {
    !is_empty(pr)
        && !has_conflicts(pr)
        && regex_match(regex, true, pr)
        && !regex_match(regex_not, false, pr)
        && (include_mine || only_mine || author(pr) != username)
        && (include_reviewed_by_me
            || only_mine
            || review_states(&pr.reviews, username, true).is_empty()) // not reviewed by me
}

fn regex_match(
    regex: &Option<Regex>,
    or: bool,
    pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest,
) -> bool {
    match regex {
        Some(re) => re.is_match(&pr.title),
        None => or,
    }
}

fn is_empty(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> bool {
    pr.additions == 0 && pr.deletions == 0
}

fn has_conflicts(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> bool {
    matches!(pr.mergeable, repo_view::MergeableState::CONFLICTING)
}

fn pr_files(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> Vec<String> {
    match &pr.files {
        Some(files) => files
            .nodes
            .iter()
            .flatten()
            .flatten()
            .map(|f| f.path.clone())
            .collect(),
        None => vec![],
    }
}

fn pr_labels(
    labels: &std::option::Option<repo_view::RepoViewSearchEdgesNodeOnPullRequestLabels>,
) -> Labels {
    match labels {
        Some(labels) => Labels(
            labels
                .nodes
                .iter()
                .flatten()
                .flatten()
                .map(|l| Label {
                    name: l.name.clone(),
                    color: l.color.clone(),
                })
                .collect(),
        ),
        None => Labels(vec![]),
    }
}

async fn pr_stats(
    github_api_token: &str,
    username: &str,
    options: &cli::Pr,
    pr: repo_view::RepoViewSearchEdgesNodeOnPullRequest,
) -> Option<Pr> {
    let (last_commit_pushed_date, tests_result) = last_commit(&pr, &options.tests_regex);

    if !include_by_tests_state(&tests_result, options) {
        return None;
    }

    let (files, blame) = if options.blame {
        let files = pr_files(&pr);
        let blame = blame::blame(
            github_api_token,
            &pr.repository.name,
            &pr.repository.owner.login,
            &files,
            username,
        )
        .await;
        (Files(files), blame)
    } else {
        (Files(vec![]), false)
    };

    let author = author(&pr);
    let reviews = review_states(&pr.reviews, &author, false);
    let review_requested = review_requested(&pr.review_requests, username);

    Some(Pr {
        title: pr.title.clone(),
        url: pr.url.clone(),
        last_commit_pushed_date,
        last_commit_age_min: age(last_commit_pushed_date),
        tests_result,
        open_conversations: pr_open_conversations(&pr.review_threads),
        num_approvals: pr_num_approvals(&reviews),
        num_reviewers: pr_num_reviewers(&reviews),
        additions: pr.additions,
        deletions: pr.deletions,
        based_on_main_branch: pr_based_on_main_branch(&pr.base_ref_name),
        files,
        blame,
        labels: pr_labels(&pr.labels),
        requested: matches!(review_requested, ReviewRequested::RequestedNotAsCodeOwner),
        codeowner: matches!(review_requested, ReviewRequested::RequestedAsCodeOwner),
    })
}

fn author(pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest) -> String {
    match &pr.author {
        Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestAuthor { login, on: _ }) => {
            login.to_string()
        }
        _ => "".to_string(),
    }
}

fn include_by_tests_state(state: &TestsState, options: &cli::Pr) -> bool {
    match state {
        TestsState::Success => !options.exclude_tests_success,
        TestsState::Failure => options.include_tests_failure,
        TestsState::Pending => options.include_tests_pending,
        TestsState::None => !options.exclude_tests_none,
    }
}

fn last_commit(
    pr: &repo_view::RepoViewSearchEdgesNodeOnPullRequest,
    tests_regex: &Option<String>,
) -> (Option<DT<Utc>>, TestsState) {
    let tests_re = regex(tests_regex);
    if let Some((pushed_date, state)) = pr
        .commits
        .nodes
        .as_ref()
        .and_then(|nodes| nodes[0].as_ref())
        .map(|node| {
            (
                &node.commit.pushed_date,
                node.commit
                    .status_check_rollup
                    .as_ref()
                    .map(|status| commit_status_state(status, &tests_re)),
            )
        })
    {
        (parse_date(pushed_date), state.unwrap_or(TestsState::None))
    } else {
        (None, TestsState::None)
    }
}

fn commit_status_state(
    status: &repo_view::RepoViewSearchEdgesNodeOnPullRequestCommitsNodesCommitStatusCheckRollup,
    tests_re: &Option<Regex>,
) -> TestsState {
    match tests_re {
        Some(tests_re) => commit_tests_state_from_contexts(&status.contexts.nodes, tests_re),
        None => tests_state(&status.state),
    }
}

fn commit_tests_state_from_contexts(
    contexts_nodes: &Option<Vec<Option<repo_view::RepoViewSearchEdgesNodeOnPullRequestCommitsNodesCommitStatusCheckRollupContextsNodes>>>,
    tests_re: &Regex,
) -> TestsState {
    match contexts_nodes {
        Some(nodes) => {
            let states: Vec<TestsState> = nodes.iter().filter_map(|node|
                match node {
                  Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestCommitsNodesCommitStatusCheckRollupContextsNodes::StatusContext(status_context)) => {
                      if tests_re.is_match(&status_context.context) {
                          Some(tests_state(&status_context.state))
                      } else {
                          None
                      }
                  },
                  _ => None
              }).collect();
            match states {
                v if v.iter().any(|state| matches!(state, TestsState::Failure)) => {
                    TestsState::Failure
                }
                v if v.iter().any(|state| matches!(state, TestsState::Pending)) => {
                    TestsState::Pending
                }
                v if v.iter().all(|state| matches!(state, TestsState::Success)) => {
                    TestsState::Success
                }
                _ => TestsState::None,
            }
        }
        None => TestsState::None,
    }
}

fn tests_state(state: &repo_view::StatusState) -> TestsState {
    match state {
        repo_view::StatusState::SUCCESS => TestsState::Success,
        repo_view::StatusState::PENDING | repo_view::StatusState::EXPECTED => TestsState::Pending,
        repo_view::StatusState::FAILURE | repo_view::StatusState::ERROR => TestsState::Failure,
        repo_view::StatusState::Other(_) => TestsState::None,
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

fn review_states<'a>(
    reviews: &'a std::option::Option<repo_view::RepoViewSearchEdgesNodeOnPullRequestReviews>,
    author: &str,
    reviewed_by_author: bool,
) -> Vec<&'a repo_view::PullRequestReviewState> {
    // println!("{:?}", reviews);
    if let Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestReviews {
        total_count: _,
        nodes: Some(nodes),
    }) = reviews
    {
        nodes
            .iter()
            .flat_map(|review| {
                // println!("{:?}", review);
                match review {
                    Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewsNodes {
                        author:
                            Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewsNodesAuthor {
                                login,
                                on: _,
                            }),
                        state,
                    }) => {
                        // println!("{} {:?}", login, state);
                        Some((login, state))
                    }
                    _ => None,
                }
            })
            .rev() // reverse order: from the newest to the oldest
            .filter(|review| {
                if reviewed_by_author {
                    review.0 == author // include only reviews given by the author of the PR
                } else {
                    review.0 != author // exclude reviews given by the author of the PR
                }
            })
            .unique_by(|review| review.0) // unique by user
            .map(|review| review.1) // take only the state
            .collect()
    } else {
        vec![]
    }
}

fn pr_num_approvals(review_states: &[&repo_view::PullRequestReviewState]) -> i64 {
    review_states
        .iter()
        .filter(|&&state| matches!(state, &repo_view::PullRequestReviewState::APPROVED))
        .count() as i64
}

fn pr_num_reviewers(review_states: &[&repo_view::PullRequestReviewState]) -> i64 {
    review_states.len() as i64
}

fn parse_date(date: &Option<String>) -> Option<DT<Utc>> {
    match date {
        Some(s) => match s.parse::<DT<Utc>>() {
            Ok(date_time) => Some(date_time),
            Err(_) => None,
        },
        None => None,
    }
}

fn age(date_time: Option<DT<Utc>>) -> Option<i64> {
    date_time.map(|date_time| (Utc::now() - date_time).num_minutes())
}

fn pr_based_on_main_branch(base_branch_name: &str) -> bool {
    base_branch_name == "main" || base_branch_name == "master"
}

fn review_requested(
    requests: &std::option::Option<repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewRequests>,
    username: &str,
) -> ReviewRequested {
    match requests {
        Some(requests) => {
            requests.nodes.iter().flatten().flatten().find(|r|
            match &r.requested_reviewer {
                Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewRequestsNodesRequestedReviewer::User(reviewer)) =>
reviewer.login == username,
                Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewRequestsNodesRequestedReviewer::Team(team)) =>
                    team.members.nodes.iter().flatten().flatten().any(|member| member.login == username),
                Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewRequestsNodesRequestedReviewer::Mannequin) => false, // Just ignore Mannequins
                None => panic!("Something is wrong with the GitHub token! Have you added the read:org scope?"),
            })
            .map_or(
                ReviewRequested::NotRequested,
                |r|
                 if r.as_code_owner {
                       ReviewRequested::RequestedAsCodeOwner
                 } else {
                       ReviewRequested::RequestedNotAsCodeOwner
                 }
            )
        }
        None => ReviewRequested::NotRequested,
    }
}
