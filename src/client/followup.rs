use super::super::types::{Review, ReviewState};
use graphql_client::*;
// use rayon::prelude::*;

#[derive(GraphQLQuery)]
#[graphql(
    schema_path = "src/client/schema.graphql",
    query_path = "src/client/followup.graphql",
    response_derives = "Debug"
)]
pub struct Followup;

type URI = String;

pub fn followup(github_api_token: &str, login: &str) {
    let response_data: followup::ResponseData = match girhub_followup(github_api_token, login) {
        Ok(data) => data,
        Err(_) => panic!("Can't get the follow up actions"),
    };

    let reviews = parse(&response_data, login);

    reviews.iter().for_each(|review| {
        println!("{:?}", review);
    })

    // println!(">> {:?}", files);
}

fn girhub_followup(
    github_api_token: &str,
    login: &str,
) -> Result<followup::ResponseData, failure::Error> {
    let q = Followup::build_query(followup::Variables {
        login: login.to_string(),
    });
    let client = reqwest::Client::new();
    let mut res = client
        .post("https://api.github.com/graphql")
        .bearer_auth(github_api_token)
        .json(&q)
        .send()?;

    let response_body: Response<followup::ResponseData> = res.json()?;
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
            .map(|pr| parse_pr(&pr, login))
            .flatten()
            .collect(),
        _ => vec![],
    }
}

fn parse_pr(pr: &followup::FollowupSearchNodes, login: &str) -> Option<Review> {
    match pr {
        followup::FollowupSearchNodes::PullRequest(
            followup::FollowupSearchNodesOnPullRequest {
                title,
                url: _,
                reviews:
                    Some(followup::FollowupSearchNodesOnPullRequestReviews {
                        nodes: Some(reviews),
                    }),
                review_threads,
            },
        ) => {
            let has_unresolved_review_threads =
                has_unresolved_review_threads(review_threads, login);
            reviews
                .iter()
                .flatten()
                .map(|review| parse_review(&review, has_unresolved_review_threads, &title))
                .flatten()
                .next()
        }
        _ => None,
    }
}

fn parse_review(
    review: &followup::FollowupSearchNodesOnPullRequestReviewsNodes,
    has_unresolved_review_threads: bool,
    pr_title: &str,
) -> Option<Review> {
    let followup::FollowupSearchNodesOnPullRequestReviewsNodes { state, url } = review;
    match state {
        followup::PullRequestReviewState::DISMISSED => Some(Review {
            state: ReviewState::Dismissed,
            url: url.to_string(),
            pr_title: pr_title.to_string(),
        }),
        followup::PullRequestReviewState::COMMENTED => {
            if !has_unresolved_review_threads {
                Some(Review {
                    state: ReviewState::WithResolvedComments,
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

fn has_unresolved_review_threads(
    review_threads: &followup::FollowupSearchNodesOnPullRequestReviewThreads,
    login: &str,
) -> bool {
    match review_threads {
        followup::FollowupSearchNodesOnPullRequestReviewThreads { nodes: Some(nodes) } => nodes
            .iter()
            .flatten()
            .any(|review_thread| is_unresolved_review_thread(review_thread, login)),
        _ => false,
    };
    // println!("{:?}", review_threads);
    // println!("---------------------------------");
    true
}

fn is_unresolved_review_thread(
    review_thread: &followup::FollowupSearchNodesOnPullRequestReviewThreadsNodes,
    login: &str,
) -> bool {
    match review_thread {
        followup::FollowupSearchNodesOnPullRequestReviewThreadsNodes {
            is_collapsed,
            comments:
                followup::FollowupSearchNodesOnPullRequestReviewThreadsNodesComments {
                    nodes: Some(comments),
                },
        } => {
            if *is_collapsed {
                false
            } else {
                let first_comment_author = comment_author(comments.first());
                let last_comment_author = comment_author(comments.last());
                // println!(
                //     "{:?} - {:?} - {:?}",
                //     is_collapsed, first_comment_author, last_comment_author
                // );
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
