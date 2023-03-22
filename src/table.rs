use super::types::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::*;
use terminal_size::{terminal_size, Height, Width};

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
        "".to_string()
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
    if value {
        YES
    } else {
        NO
    }
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
        "".to_string()
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
                    "".to_string()
                },
                if h > 0 {
                    format!("{h}h ")
                } else {
                    "".to_string()
                },
                // Display minutes only if days is 0
                if d == 0 && m > 0 {
                    format!("{m}m ")
                } else {
                    "".to_string()
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

    reviews.iter().for_each(|review| {
        table.add_row(review_row(review));
    });

    table
}

fn review_row(review: &Review) -> Vec<String> {
    vec![
        review.url.to_string(),
        review.state.to_string(),
        review.pr_title.to_string(),
    ]
}
