use super::table;
use super::types::Review;
use super::types::ScoredPr;

pub fn prs(sprs: &[ScoredPr], num: Option<usize>, debug: bool, short: bool, json: bool) -> String {
    let limit = num.unwrap_or(10000);
    if json {
        json_prs(sprs, limit)
    } else if short {
        short_prs(sprs, limit, debug)
    } else {
        format!("{}", table::from(sprs, limit, debug))
    }
}

fn short_prs(sprs: &[ScoredPr], limit: usize, debug: bool) -> String {
    sprs.iter()
        .take(limit)
        .map(|spr| pr(spr, debug))
        .collect::<Vec<String>>()
        .join("\n")
}

fn pr(spr: &ScoredPr, _debug: bool) -> String {
    format!("{}", &spr.pr)
}

pub fn reviews(reviews: &[Review], json: bool) -> String {
    if json {
        json_reviews(reviews)
    } else {
        format!("{}", table::from_reviews(reviews))
    }
}

fn json_prs(sprs: &[ScoredPr], limit: usize) -> String {
    let len = sprs.len();
    let l = if limit > len { len } else { limit };
    match serde_json::to_string(&sprs[..l]) {
        Ok(json) => json,
        Err(error) => error.to_string(),
    }
}

fn json_reviews(reviews: &[Review]) -> String {
    match serde_json::to_string(&reviews) {
        Ok(json) => json,
        Err(error) => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::types::*;
    use super::*;

    fn make_scored_pr(title: &str, url: &str) -> ScoredPr {
        let pr = Pr {
            title: title.to_string(),
            url: url.to_string(),
            last_commit_pushed_date: None,
            last_commit_age_min: None,
            tests_result: TestsState::Success,
            open_conversations: 0,
            num_approvals: 0,
            num_reviewers: 0,
            additions: 0,
            deletions: 0,
            based_on_main_branch: false,
            files: Files(vec![]),
            blame: false,
            labels: Labels(vec![]),
            requested: false,
            codeowner: false,
        };
        let score = Score::from_pr(1, &pr);
        ScoredPr { pr, score }
    }

    #[test]
    fn test_prs_json_format() {
        let prs_data = vec![
            make_scored_pr("Add feature", "https://example.com/1"),
            make_scored_pr("Fix bug", "https://example.com/2"),
        ];
        let result = prs(&prs_data, None, false, false, true);
        assert!(result.contains("Add feature"));
        assert!(result.contains("Fix bug"));
    }

    #[test]
    fn test_prs_short_format() {
        let prs_data = vec![
            make_scored_pr("Add feature", "https://example.com/1"),
            make_scored_pr("Fix bug", "https://example.com/2"),
        ];
        let result = prs(&prs_data, None, false, true, false);
        assert!(result.contains("https://example.com/1"));
        assert!(result.contains("https://example.com/2"));
    }

    #[test]
    fn test_prs_limit() {
        let prs_data = vec![
            make_scored_pr("Add feature", "https://example.com/1"),
            make_scored_pr("Fix bug", "https://example.com/2"),
            make_scored_pr("Update docs", "https://example.com/3"),
        ];
        let result = prs(&prs_data, Some(2), false, true, false);
        assert!(result.contains("https://example.com/1"));
        assert!(result.contains("https://example.com/2"));
        assert!(!result.contains("https://example.com/3"));
    }

    #[test]
    fn test_prs_empty() {
        let prs_data: Vec<ScoredPr> = vec![];
        let result = prs(&prs_data, None, false, true, false);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_prs_limit_higher_than_count() {
        let prs_data = vec![
            make_scored_pr("Add feature", "https://example.com/1"),
            make_scored_pr("Fix bug", "https://example.com/2"),
        ];
        let result = prs(&prs_data, Some(10), false, true, false);
        assert!(result.contains("https://example.com/1"));
        assert!(result.contains("https://example.com/2"));
    }

    #[test]
    fn test_prs_short_limit() {
        let prs_data = vec![
            make_scored_pr("Add feature", "https://example.com/1"),
            make_scored_pr("Fix bug", "https://example.com/2"),
            make_scored_pr("Update docs", "https://example.com/3"),
        ];
        let result = prs(&prs_data, Some(2), false, true, false);
        let count = result.matches("https://example.com/").count();
        assert_eq!(count, 2);
    }

    #[test]
    fn test_prs_default_format() {
        let prs_data = vec![make_scored_pr("Add feature", "https://example.com/1")];
        let result = prs(&prs_data, None, false, false, false);
        assert!(result.contains("Add feature"));
    }

    // Tests for render::reviews function
    fn make_review(state: ReviewState, url: &str, title: &str) -> Review {
        Review {
            state,
            url: url.to_string(),
            pr_title: title.to_string(),
        }
    }

    #[test]
    fn test_reviews_json_format() {
        let reviews_data = vec![
            make_review(ReviewState::Dismissed, "https://example.com/1", "Fix bug"),
            make_review(
                ReviewState::WithAddressedConversations,
                "https://example.com/2",
                "Add feature",
            ),
        ];
        let result = reviews(&reviews_data, true);
        assert!(result.contains("Dismissed"));
        assert!(result.contains("Add feature"));
    }

    #[test]
    fn test_reviews_table_format() {
        let reviews_data = vec![make_review(
            ReviewState::Dismissed,
            "https://example.com/1",
            "Fix bug",
        )];
        let result = reviews(&reviews_data, false);
        assert!(result.contains("Fix bug"));
    }

    #[test]
    fn test_reviews_empty() {
        let reviews_data: Vec<Review> = vec![];
        let result = reviews(&reviews_data, true);
        assert!(result.starts_with('['));
        assert!(result.ends_with(']'));
        assert_eq!(result, "[]");
    }

    #[test]
    fn test_reviews_single() {
        let reviews_data = vec![make_review(
            ReviewState::Dismissed,
            "https://example.com/pr/123",
            "Update code",
        )];
        let result = reviews(&reviews_data, false);
        assert!(result.contains("https://example.com/pr/123"));
        assert!(result.contains("Update code"));
    }
}
