use super::types::*;
use prettytable::*;

pub fn from(sprs: &[ScoredPr], limit: usize, debug: bool) -> prettytable::Table {
    let mut table = prettytable::Table::new();
    // let format = format::FormatBuilder::new()
    //     .column_separator('|')
    //     .borders('|')
    //     // .separators(
    //     //     &[format::LinePosition::Top, format::LinePosition::Bottom],
    //     //     format::LineSeparator::new('-', '+', '+', '+'),
    //     // )
    //     .padding(1, 1)
    //     .build();
    table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    // table.set_format(format);
    table.set_titles(row![
        "Title",
        "URL",
        "Last commit",
        "CI",
        "O.C.",
        "Rev.",
        "Diff",
        "Score"
    ]);

    for spr in sprs.iter().take(limit) {
        table.add_row(pr_row(spr, debug));
    }

    table
}

fn pr_row(spr: &ScoredPr, debug: bool) -> prettytable::row::Row {
    let debug_info = if debug {
        format!(
            "\nAge:{:.1} T:{:.1} OC:{:.1} Ap:{:.1} R:{:.1} +:{:.1} -:{:.1} Tot:{:.1}",
            spr.score.age,
            spr.score.tests_result,
            spr.score.open_conversations,
            spr.score.num_approvals,
            spr.score.num_reviewers,
            spr.score.additions,
            spr.score.deletions,
            spr.score.total()
        )
    } else {
        "".to_string()
    };
    row!(
        format!("{}{}", spr.pr.title, debug_info),
        spr.pr.url,
        match spr.pr.last_commit_pushed_date {
            Some(d) => d.format("%d-%m-%Y %H:%M").to_string(),
            None => String::from("-"),
        },
        spr.pr.tests_result.to_string(),
        spr.pr.open_conversations.to_string(),
        format!("{}/{}", spr.pr.num_approvals, spr.pr.num_reviewers),
        format!("+{} -{}", spr.pr.additions, spr.pr.deletions),
        format!("{:.1}", spr.score.total()),
    )
}
