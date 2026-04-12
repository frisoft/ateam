use super::types::ScoredPr;
use regex::Regex;

#[allow(dead_code)]
pub fn filter_prs(regex_text: Option<&String>, prs: Vec<ScoredPr>) -> Vec<ScoredPr> {
    if let Some(regex_text) = regex_text {
        let re = Regex::new(regex_text).unwrap();
        prs.into_iter()
            .filter(|pr| re.is_match(&pr.pr.title))
            .collect()
    } else {
        prs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::types::*;

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
    fn test_regex_none_returns_all() {
        let prs = vec![
            make_scored_pr("Add feature", "https://example.com/1"),
            make_scored_pr("Fix bug", "https://example.com/2"),
        ];
        let regex_text: Option<String> = None;
        let result = filter_prs(regex_text.as_ref(), prs);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn test_regex_matching() {
        let prs = vec![
            make_scored_pr("Add feature", "https://example.com/1"),
            make_scored_pr("Fix bug", "https://example.com/2"),
            make_scored_pr("Add tests", "https://example.com/3"),
        ];
        let regex_text = Some("Add".to_string());
        let result = filter_prs(regex_text.as_ref(), prs);
        assert_eq!(result.len(), 2);
        assert!(result.iter().any(|pr| pr.pr.url == "https://example.com/1"));
        assert!(result.iter().any(|pr| pr.pr.url == "https://example.com/3"));
    }

    #[test]
    fn test_regex_no_match() {
        let prs = vec![
            make_scored_pr("Add feature", "https://example.com/1"),
            make_scored_pr("Fix bug", "https://example.com/2"),
        ];
        let regex_text = Some("delete".to_string());
        let result = filter_prs(regex_text.as_ref(), prs);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_regex_case_sensitive() {
        let prs = vec![
            make_scored_pr("Add Feature", "https://example.com/1"),
            make_scored_pr("add feature", "https://example.com/2"),
        ];
        let regex_text = Some("Add".to_string());
        let result = filter_prs(regex_text.as_ref(), prs);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].pr.url, "https://example.com/1");
    }

    #[test]
    fn test_regex_empty_result() {
        let prs: Vec<ScoredPr> = vec![];
        let regex_text = Some("feature".to_string());
        let result = filter_prs(regex_text.as_ref(), prs);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_regex_complex_pattern() {
        let prs = vec![
            make_scored_pr("FIX-123: Fix bug", "https://example.com/1"),
            make_scored_pr("FEAT-456: Add feature", "https://example.com/2"),
            make_scored_pr("Update docs", "https://example.com/3"),
        ];
        let regex_text = Some("(FIX|FEAT)".to_string());
        let result = filter_prs(regex_text.as_ref(), prs);
        assert_eq!(result.len(), 2);
    }
}