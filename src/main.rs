use crate::types::ScoredPr;
use failure::*;
// use log::*;
use prettytable::*;

mod client;
use client::*;
mod cli;
mod config;
mod types;

fn parse_repo_name(repo_name: &str) -> Result<(&str, &str), failure::Error> {
    let mut parts = repo_name.split('/');
    match (parts.next(), parts.next()) {
        (Some(owner), Some(name)) => Ok((owner, name)),
        _ => Err(format_err!("wrong format for the repository name param (we expect something like facebook/graphql)"))
    }
}

fn pr_row(spr: &ScoredPr, debug: bool) -> prettytable::row::Row {
    let title = if debug {
        let debug_info = format!(
            "SCORES - Age:{} Tests:{} OC:{} Appr:{} Rev:{} Add:{} Del:{} Total:{}",
            spr.score.age,
            spr.score.tests_result,
            spr.score.open_conversations,
            spr.score.num_approvals,
            spr.score.num_reviewers,
            spr.score.additions,
            spr.score.deletions,
            spr.score.total()
        );
        format!("{}\n{}", spr.pr.title, debug_info)
    } else {
        spr.pr.title.clone()
    };
    row!(
        title.to_string(),
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

fn main() -> Result<(), failure::Error> {
    dotenv::dotenv().ok();
    env_logger::init();

    let config: config::Config = config::get_config().context("while reading from environment")?;

    let args = cli::command();

    let repo = args.repo;
    let (owner, name) = parse_repo_name(&repo).unwrap_or(("tomhoule", "graphql-client"));

    let response_data: repo_view::ResponseData =
        client::query(&config.github_api_token, owner, name)?;

    let mut table = prettytable::Table::new();
    let format = format::FormatBuilder::new()
        // .column_separator('|')
        // .borders('|')
        // .separators(
        //     &[format::LinePosition::Top, format::LinePosition::Bottom],
        //     format::LineSeparator::new('-', '+', '+', '+'),
        // )
        .padding(1, 0)
        .build();
    // table.set_format(*format::consts::FORMAT_NO_BORDER_LINE_SEPARATOR);
    table.set_format(format);

    let sprs = client::ranked_prs(&response_data);
    for spr in sprs.iter().take(args.num.unwrap_or(10000)) {
        table.add_row(pr_row(spr, args.debug));
    }

    table.printstd();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_repo_name_works() {
        assert_eq!(
            parse_repo_name("graphql-rust/graphql-client").unwrap(),
            ("graphql-rust", "graphql-client")
        );
        assert!(parse_repo_name("abcd").is_err());
    }
}
