use super::cli;
use super::types::*;
use chrono::prelude::*;
use graphql_client::*;
use itertools::Itertools;
use rayon::prelude::*;
use regex::Regex;
mod blame;
pub mod followup;
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
        query_mine(username, options.include_mine, options.only_mine),
        query_include_reviewed_by_me(username, options.include_reviewed_by_me),
        query_labels(&options.label, &options.exclude_label),
        query_repos(&options.repo),
        query_org(&options.org),
        &options.query.as_ref().unwrap_or(&"".to_string())
    )
}

fn query_drafts(include_drafts: bool) -> &'static str {
    if include_drafts {
        ""
    } else {
        "draft:false "
    }
}

fn query_mine(username: &str, include_mine: bool, only_mine: bool) -> String {
    if only_mine {
        format!("author:{} ", username)
    } else if include_mine {
        "".to_string()
    } else {
        format!("-author:{} ", username)
    }
}

fn query_include_reviewed_by_me(username: &str, include_reviewed_by_me: bool) -> String {
    if include_reviewed_by_me {
        "".to_string()
    } else {
        format!("-reviewed-by:{} ", username)
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

pub fn ranked_prs<'a>(
    github_api_token: &str,
    username: &str,
    required_approvals: u8,
    options: &cli::Pr,
    response_data: &'a repo_view::ResponseData,
) -> Vec<ScoredPr<'a>> {
    let re = regex(&options.regex);
    let re_not = regex(&options.regex_not);
    let sprs: Vec<ScoredPr> = prs(
        github_api_token,
        username,
        &re,
        &re_not,
        options,
        &response_data,
    )
    .into_par_iter()
    .map(|pr| scored_pr(required_approvals, pr))
    .collect();
    sprs
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
    Some(Regex::new(&text).unwrap())
}

fn scored_pr(required_approvals: u8, pr: Pr) -> ScoredPr {
    let s = Score::from_pr(required_approvals, &pr);
    ScoredPr { pr, score: s }
}

fn prs<'a>(
    github_api_token: &str,
    username: &str,
    regex: &Option<Regex>,
    regex_not: &Option<Regex>,
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
        .filter(move |i| {
            !is_empty(i)
                && !has_conflicts(i)
                && regex_match(regex, true, i)
                && !regex_match(regex_not, false, i)
        })
        .map(move |i| pr_stats(github_api_token, username, options, &i)) // <-- Refactor
        .flatten() // Extract value from Some(value) and remove the Nones
        .collect()
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
    pr.mergeable == repo_view::MergeableState::CONFLICTING
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
                    name: l.name.as_ref(),
                    color: l.color.as_ref(),
                })
                .collect(),
        ),
        None => Labels(vec![]),
    }
}

fn pr_stats<'a>(
    github_api_token: &str,
    username: &str,
    options: &cli::Pr,
    pr: &'a repo_view::RepoViewSearchEdgesNodeOnPullRequest,
) -> Option<Pr<'a>> {
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
        );
        (Files(files), blame)
    } else {
        (Files(vec![]), false)
    };

    let author = author(&pr);
    let reviews = review_states(&pr.reviews, &author);

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
        codeowner: is_codeowner(&pr.review_requests, username),
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
) -> (Option<DateTime<Utc>>, TestsState) {
    let tests_re = regex(&tests_regex);
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
                    .map(|status| commit_status_state(&status, &tests_re)),
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
            let states: Vec<TestsState> = nodes.iter().map(|node|
                match node {
                  Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestCommitsNodesCommitStatusCheckRollupContextsNodes::StatusContext(status_context)) => {
                      if tests_re.is_match(&status_context.context) {
                          Some(tests_state(&status_context.state))
                      } else {
                          None
                      }
                  },
                  _ => None
              }).flatten().collect();
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
            .filter(|review| review.0 != author) // exclude reviews given by the author of the PR
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
        .filter(|&&state| state == &repo_view::PullRequestReviewState::APPROVED)
        .count() as i64
}

fn pr_num_reviewers(review_states: &[&repo_view::PullRequestReviewState]) -> i64 {
    review_states.len() as i64
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

fn age(date_time: Option<DateTime<Utc>>) -> Option<i64> {
    match date_time {
        Some(date_time) => Some((Utc::now() - date_time).num_minutes()),
        None => None,
    }
}

fn pr_based_on_main_branch(base_branch_name: &str) -> bool {
    base_branch_name == "main" || base_branch_name == "master"
}

fn is_codeowner(
    requests: &std::option::Option<repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewRequests>,
    username: &str,
) -> bool {
    match requests {
        Some(requests) => requests.nodes.iter().flatten().flatten().any(|r| {
            r.as_code_owner
                && match &r.requested_reviewer {
                    Some(repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewRequestsNodesRequestedReviewer::User(reviewer)) =>
                        reviewer.login == username,
                    _ => false,
                }
        }),
        None => false,
    }
}
