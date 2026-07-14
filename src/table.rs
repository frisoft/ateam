use super::types::{Files, Review, ScoredPr, TestsState};
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{ColumnConstraint, ContentArrangement, Table};
use terminal_size::{Height, Width, terminal_size};

#[cfg(test)]
use super::types::{Labels, Pr, ReviewState, Score};

pub fn from(sprs: &[ScoredPr], limit: usize, debug: bool) -> Table {
    let mut table = build_table();
    table.set_header(vec![
        "Pull request",
        "Age",
        "CI",
        "O.C.",
        "Appr.",
        "Diff",
        "On Main",
        "Blame",
        "Req.",
        "C. Owner",
        "Score",
    ]);

    for c in 1..8 {
        table
            .column_mut(c)
            .expect("a column")
            .set_constraint(ColumnConstraint::ContentWidth);
    }

    for spr in sprs.iter().take(limit) {
        table.add_row(pr_row(spr, debug));
    }

    table
}

fn pr_row(spr: &ScoredPr, debug: bool) -> Vec<String> {
    let debug_info = if debug {
        format!(
            "\nAge:{:.1} T:{:.1} OC:{:.1} Ap:{:.1} R:{:.1} +:{:.1} -:{:.1} M.br:{:.1} Bl:{:.1} Req.:{:.1} C.Owner:{:.1} Tot:{:.1}{}\n",
            spr.score.age,
            spr.score.tests_result,
            spr.score.open_conversations,
            spr.score.num_approvals,
            spr.score.num_reviewers,
            spr.score.additions,
            spr.score.deletions,
            spr.score.based_on_main_branch,
            spr.score.blame,
            spr.score.requested,
            spr.score.codeowner,
            spr.score.total(),
            show_files(&spr.pr.files)
        )
    } else {
        String::new()
    };
    vec![
        format!(
            "{}\n{} {}{}",
            spr.pr.url, spr.pr.title, spr.pr.labels, debug_info
        ),
        show_duration(spr.pr.last_commit_age_min),
        tests_result_label(&spr.pr.tests_result).to_string(),
        spr.pr.open_conversations.to_string(),
        format!("{}/{}", spr.pr.num_approvals, spr.pr.num_reviewers),
        format!("+{} -{}", spr.pr.additions, spr.pr.deletions),
        show_bool(spr.pr.based_on_main_branch).to_string(),
        show_bool(spr.pr.blame).to_string(),
        show_bool(spr.pr.requested).to_string(),
        show_bool(spr.pr.codeowner).to_string(),
        format!("{:.1}", spr.score.total()),
    ]
}

const YES: &str = "yes";
const NO: &str = "no";
fn show_bool(value: bool) -> &'static str {
    if value { YES } else { NO }
}

fn tests_result_label(tests_result: &TestsState) -> &'static str {
    match tests_result {
        TestsState::Success => "OK",
        TestsState::Pending => "..",
        TestsState::Failure => "Fail",
        TestsState::None => "-",
    }
}

fn show_files(files: &Files) -> String {
    if files.0.is_empty() {
        String::new()
    } else {
        format!("\n{}\n", files.0.join("\n"))
    }
}

fn show_duration(minutes: Option<i64>) -> String {
    match minutes {
        Some(min) => {
            let d = min / 60 / 24;
            let h = (min - d * 24 * 60) / 60;
            let m = min - d * 24 * 60 - h * 60;
            format!(
                "{}{}{}",
                if d > 0 {
                    format!("{d}d ")
                } else {
                    String::new()
                },
                if h > 0 {
                    format!("{h}h ")
                } else {
                    String::new()
                },
                if d == 0 && m > 0 {
                    format!("{m}m ")
                } else {
                    String::new()
                }
            )
        }
        // d.format("%d-%m-%Y %H:%M").to_string(),
        None => String::from("-"),
    }
}

fn build_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    if let Some((Width(w), Height(_h))) = terminal_size() {
        table.set_width(w);
    }

    table
}

pub fn from_reviews(reviews: &[Review]) -> Table {
    let mut table = build_table();
    table.set_header(vec!["Review", "State", "Pull request"]);

    for review in reviews {
        table.add_row(review_row(review));
    }

    table
}

fn review_row(review: &Review) -> Vec<String> {
    vec![
        review.url.clone(),
        review.state.to_string(),
        review.pr_title.clone(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(clippy::too_many_arguments, clippy::fn_params_excessive_bools)]
    fn make_scored_pr(
        title: &str,
        url: &str,
        age_min: Option<i64>,
        approvals: i64,
        reviewers: i64,
        additions: i64,
        deletions: i64,
        on_main: bool,
        blame: bool,
        requested: bool,
        codeowner: bool,
    ) -> ScoredPr {
        let pr = Pr {
            title: title.to_string(),
            url: url.to_string(),
            last_commit_pushed_date: None,
            last_commit_age_min: age_min,
            tests_result: TestsState::Success,
            open_conversations: 0,
            num_approvals: approvals,
            num_reviewers: reviewers,
            additions,
            deletions,
            based_on_main_branch: on_main,
            files: Files(vec![]),
            blame,
            labels: Labels(vec![]),
            requested,
            codeowner,
        };
        let score = Score::from_pr(1, &pr);
        ScoredPr { pr, score }
    }

    #[test]
    fn test_table_from_empty() {
        let prs: &[ScoredPr] = &[];
        let result = from(prs, 10, false);
        assert_eq!(result.row_count(), 0);
    }

    #[test]
    fn test_table_from_single() {
        let prs = vec![make_scored_pr(
            "Fix bug",
            "https://example.com/1",
            Some(60),
            2,
            1,
            100,
            50,
            true,
            false,
            false,
            false,
        )];
        let result = from(&prs, 10, false);
        assert_eq!(result.row_count(), 1);
    }

    #[test]
    fn test_table_from_multiple() {
        let prs = vec![
            make_scored_pr(
                "Fix bug",
                "https://example.com/1",
                Some(60),
                2,
                1,
                100,
                50,
                true,
                false,
                false,
                false,
            ),
            make_scored_pr(
                "Add feature",
                "https://example.com/2",
                Some(120),
                1,
                2,
                200,
                100,
                false,
                true,
                false,
                false,
            ),
        ];
        let result = from(&prs, 10, false);
        assert_eq!(result.row_count(), 2);
    }

    #[test]
    fn test_table_from_limit() {
        let prs = vec![
            make_scored_pr(
                "Fix bug",
                "https://example.com/1",
                Some(60),
                2,
                1,
                100,
                50,
                true,
                false,
                false,
                false,
            ),
            make_scored_pr(
                "Add feature",
                "https://example.com/2",
                Some(120),
                1,
                2,
                200,
                100,
                false,
                true,
                false,
                false,
            ),
            make_scored_pr(
                "Update docs",
                "https://example.com/3",
                Some(180),
                0,
                0,
                50,
                10,
                false,
                false,
                false,
                false,
            ),
        ];
        let result = from(&prs, 2, false);
        assert_eq!(result.row_count(), 2);
    }

    #[test]
    fn test_table_from_limit_higher() {
        let prs = vec![make_scored_pr(
            "Fix bug",
            "https://example.com/1",
            Some(60),
            2,
            1,
            100,
            50,
            true,
            false,
            false,
            false,
        )];
        let result = from(&prs, 10, false);
        assert_eq!(result.row_count(), 1);
    }

    #[test]
    fn test_table_from_debug() {
        let prs = vec![make_scored_pr(
            "Fix bug",
            "https://example.com/1",
            Some(60),
            2,
            1,
            100,
            50,
            true,
            false,
            false,
            false,
        )];
        let result = from(&prs, 10, true);
        assert_eq!(result.row_count(), 1);
    }

    // Tests for from_reviews
    fn make_review(state: ReviewState, url: &str, title: &str) -> Review {
        Review {
            state,
            url: url.to_string(),
            pr_title: title.to_string(),
        }
    }

    #[test]
    fn test_table_from_reviews_empty() {
        let reviews: &[Review] = &[];
        let result = from_reviews(reviews);
        assert_eq!(result.row_count(), 0);
    }

    #[test]
    fn test_table_from_reviews_single() {
        let reviews = vec![make_review(
            ReviewState::Dismissed,
            "https://example.com/1",
            "Fix bug",
        )];
        let result = from_reviews(&reviews);
        assert_eq!(result.row_count(), 1);
    }

    #[test]
    fn test_table_from_reviews_multiple() {
        let reviews = vec![
            make_review(ReviewState::Dismissed, "https://example.com/1", "Fix bug"),
            make_review(
                ReviewState::WithAddressedConversations,
                "https://example.com/2",
                "Add feature",
            ),
        ];
        let result = from_reviews(&reviews);
        assert_eq!(result.row_count(), 2);
    }
}
