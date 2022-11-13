use super::super::types::{Review, ReviewState};
use anyhow::Result;
use graphql_client::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/schema.graphql",
    query_path = "src/client/followup.graphql",
    response_derives = "Debug"
)]
pub struct Followup;

#[allow(clippy::upper_case_acronyms)]
type URI = String;

pub async fn followup(github_api_token: &str, login: &str) -> Vec<Review> {
    let response_data: followup::ResponseData = match girhub_followup(github_api_token, login).await
    {
        Ok(data) => data,
        Err(_) => panic!("Can't get the follow up actions"),
    };

    parse(&response_data, login)
}

async fn girhub_followup(github_api_token: &str, login: &str) -> Result<followup::ResponseData> {
    let q = Followup::build_query(followup::Variables {
        login: login.to_string(),
        query: format!(
            "is:pr is:open draft:false reviewed-by:{} -author:{}",
            login, login
        ),
    });

    let res = super::call(github_api_token, &q).await?;

    let response_body: Response<followup::ResponseData> = res.json().await?;
    // println!("{:?}", response_body);

    if let Some(errors) = response_body.errors {
        println!("there are errors:");
        for error in &errors {
            println!("{:?}", error);
        }
    }
    Ok(response_body.data.expect("missing response data"))
}

fn parse(response_data: &followup::ResponseData, login: &str) -> Vec<Review> {
    match response_data {
        followup::ResponseData {
            search: followup::FollowupSearch { nodes: Some(prs) },
        } => prs
            .iter()
            .flatten()
            .filter_map(|pr| parse_pr(pr, login))
            .collect(),
        _ => vec![],
    }
}

fn parse_pr(pr: &followup::FollowupSearchNodes, login: &str) -> Option<Review> {
    match pr {
        followup::FollowupSearchNodes::PullRequest(
            followup::FollowupSearchNodesOnPullRequest {
                title,
                // url: _,
                mergeable,
                reviews:
                    Some(followup::FollowupSearchNodesOnPullRequestReviews {
                        nodes: Some(reviews),
                    }),
                review_threads,
            },
        ) => {
            if matches!(*mergeable, followup::MergeableState::CONFLICTING) {
                // the PR has conflicts, let's exclude it
                None
            } else {
                last_dismissed_or_addressed_review(reviews, review_threads, login, title)
            }
        }
        _ => None,
    }
}

fn last_dismissed_or_addressed_review(
    reviews: &[Option<followup::FollowupSearchNodesOnPullRequestReviewsNodes>],
    review_threads: &followup::FollowupSearchNodesOnPullRequestReviewThreads,
    login: &str,
    title: &str,
) -> Option<Review> {
    let has_unaddressed_review_threads = has_unaddressed_review_threads(review_threads, login);
    reviews
        .iter()
        .flatten()
        .filter_map(|review| parse_review(review, has_unaddressed_review_threads, title))
        .next()
}

fn parse_review(
    review: &followup::FollowupSearchNodesOnPullRequestReviewsNodes,
    has_unaddressed_review_threads: bool,
    pr_title: &str,
) -> Option<Review> {
    let followup::FollowupSearchNodesOnPullRequestReviewsNodes { state, url } = review;
    // println!("{:?}", url);
    match state {
        followup::PullRequestReviewState::DISMISSED => Some(Review {
            state: ReviewState::Dismissed,
            url: url.to_string(),
            pr_title: pr_title.to_string(),
        }),
        followup::PullRequestReviewState::COMMENTED => {
            if !has_unaddressed_review_threads {
                Some(Review {
                    state: ReviewState::WithAddressedConversations,
                    url: url.to_string(),
                    pr_title: pr_title.to_string(),
                })
            } else {
                None
            }
        }
        _ => None,
    }
}

fn has_unaddressed_review_threads(
    review_threads: &followup::FollowupSearchNodesOnPullRequestReviewThreads,
    login: &str,
) -> bool {
    match review_threads {
        followup::FollowupSearchNodesOnPullRequestReviewThreads { nodes: Some(nodes) } => nodes
            .iter()
            .flatten()
            .any(|review_thread| is_unaddressed_review_thread(review_thread, login)),
        _ => false,
    }
}

fn is_unaddressed_review_thread(
    review_thread: &followup::FollowupSearchNodesOnPullRequestReviewThreadsNodes,
    login: &str,
) -> bool {
    match review_thread {
        followup::FollowupSearchNodesOnPullRequestReviewThreadsNodes {
            // is_collapsed,
            // is_unresolved,
            is_outdated,
            comments:
                followup::FollowupSearchNodesOnPullRequestReviewThreadsNodesComments {
                    nodes: Some(comments),
                },
        } => {
            if *is_outdated {
                false
            } else {
                let first_comment_author = comment_author(comments.first());
                let last_comment_author = comment_author(comments.last());
                first_comment_author == login && last_comment_author == login
            }
        }
        _ => false,
    }
}

fn comment_author(
    comment: Option<
        &Option<followup::FollowupSearchNodesOnPullRequestReviewThreadsNodesCommentsNodes>,
    >,
) -> String {
    match comment {
        Some(Some(followup::FollowupSearchNodesOnPullRequestReviewThreadsNodesCommentsNodes {
            author:
                Some(followup::FollowupSearchNodesOnPullRequestReviewThreadsNodesCommentsNodesAuthor {
                    login,
                    on: _,
                }),
        })) => login.to_string(),
        _ => "".to_string(),
    }
}
