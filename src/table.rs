use super::types::*;
use prettytable::*;

pub fn from(sprs: &[ScoredPr], limit: usize, debug: bool) -> prettytable::Table {
    let mut table = prettytable::Table::new();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_titles(row![
        "Title", "URL", "Age", "CI", "O.C.", "Appr.", "Diff", "On Main", "Blame", "Score"
    ]);

    for spr in sprs.iter().take(limit) {
        table.add_row(pr_row(spr, debug));
    }

    table
}

fn pr_row(spr: &ScoredPr, debug: bool) -> prettytable::row::Row {
    let debug_info = if debug {
        format!(
            "\nAge:{:.1} T:{:.1} OC:{:.1} Ap:{:.1} R:{:.1} +:{:.1} -:{:.1} M.br:{:.1} Bl:{:.1} Tot:{:.1}{}\n",
            spr.score.age,
            spr.score.tests_result,
            spr.score.open_conversations,
            spr.score.num_approvals,
            spr.score.num_reviewers,
            spr.score.additions,
            spr.score.deletions,
            spr.score.based_on_main_branch,
            spr.score.blame,
            spr.score.total(),
            show_files(&spr.pr.files)
        )
    } else {
        "".to_string()
    };
    row!(
        format!("{:.60}{}", spr.pr.title, debug_info),
        spr.pr.url,
        show_duration(spr.pr.last_commit_age_min),
        tests_result_label(spr.pr.tests_result),
        spr.pr.open_conversations.to_string(),
        format!("{}/{}", spr.pr.num_approvals, spr.pr.num_reviewers),
        format!("+{} -{}", spr.pr.additions, spr.pr.deletions),
        show_bool(spr.pr.based_on_main_branch).to_string(),
        show_bool(spr.pr.blame).to_string(),
        format!("{:.1}", spr.score.total()),
    )
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

fn tests_result_label(tests_result: i64) -> char {
    match tests_result {
        0 => 'S',
        1 => 'P',
        2 => 'F',
        _ => '?',
    }
}

fn show_files(files: &[&str]) -> String {
    if files.is_empty() {
        "".to_string()
    } else {
        format!("\n{}\n", files.join("\n"))
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
                    format!("{} d ", d)
                } else {
                    "".to_string()
                },
                if h > 0 {
                    format!("{} h ", h)
                } else {
                    "".to_string()
                },
                // Display minutes only if days is 0
                if d == 0 && m > 0 {
                    format!("{} m ", m)
                } else {
                    "".to_string()
                }
            )
        }
        // d.format("%d-%m-%Y %H:%M").to_string(),
        None => String::from("-"),
    }
}
