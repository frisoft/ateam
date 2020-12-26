fn pr_requested(
    requests: &std::option::Option<repo_view::RepoViewSearchEdgesNodeOnPullRequestReviewRequests>,
    username: &str,
) -> bool {
    match requests {
        Some(requests) => requests
            .nodes
            .iter()
            .flatten()
            .flatten()
            .any(|r| r.as_code_owner && r.requested_reviewer.login == username),
        None => false,
    }
}
