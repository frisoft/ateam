use chrono::prelude::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize, Debug, Clone)]
#[allow(clippy::struct_excessive_bools)]
pub struct Pr {
    pub title: String,
    pub url: String,
    pub last_commit_pushed_date: Option<DateTime<Utc>>,
    pub last_commit_age_min: Option<i64>,
    pub tests_result: TestsState,
    pub open_conversations: i64,
    pub num_approvals: i64,
    pub num_reviewers: i64,
    pub additions: i64,
    pub deletions: i64,
    pub based_on_main_branch: bool,
    pub files: Files,
    pub blame: bool,
    pub labels: Labels,
    pub requested: bool,
    pub codeowner: bool,
}

#[derive(Serialize, Debug, Clone)]
pub enum TestsState {
    Pending,
    Success,
    Failure,
    None,
}

#[derive(Serialize)]
pub enum ReviewRequested {
    RequestedAsCodeOwner,
    RequestedNotAsCodeOwner,
    NotRequested,
}

#[derive(Serialize, Debug, Clone)]
pub struct Files(pub Vec<String>);

#[derive(Serialize, Debug, Clone)]
pub struct Labels(pub Vec<Label>);

#[derive(Serialize, Debug, Clone)]
pub struct Label {
    pub name: String,
    pub color: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ScoredPr {
    pub pr: Pr,
    pub score: Score,
}

#[derive(Debug, Serialize, Clone)]
pub struct Score {
    pub age: f64,
    pub tests_result: f64,
    pub open_conversations: f64,
    pub num_approvals: f64,
    pub num_reviewers: f64,
    pub additions: f64,
    pub deletions: f64,
    pub based_on_main_branch: f64,
    pub blame: f64,
    pub requested: f64,
    pub codeowner: f64,
}

impl Score {
    pub fn from_pr(required_approvals: u8, pr: &Pr) -> Score {
        let tests_result_i = match pr.tests_result {
            TestsState::Pending => 1,
            TestsState::Failure => 2,
            TestsState::Success | TestsState::None => 0, // a repo without CI is treated as successful
        };
        #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
        Score {
            age: pr.last_commit_age_min.unwrap_or(0) as f64 / 60.0 * 2.0,
            tests_result: f64::from(tests_result_i - 1) * -200.0,
            open_conversations: pr.open_conversations as f64 * -30.0,
            num_approvals: (pr.num_approvals - i64::from(required_approvals)) as f64 * -80.0,
            num_reviewers: (pr.num_reviewers - i64::from(required_approvals)) as f64 * -50.0,
            additions: pr.additions as f64 * -0.5,
            deletions: pr.deletions as f64 * -0.1,
            based_on_main_branch: u8::from(pr.based_on_main_branch) as f64 * 200.0,
            blame: u8::from(pr.blame) as f64 * 400.0,
            requested: u8::from(pr.requested) as f64 * 800.0,
            codeowner: u8::from(pr.codeowner) as f64 * 400.0,
        }
    }

    pub fn total(&self) -> f64 {
        self.age
            + self.tests_result
            + self.open_conversations
            + self.num_approvals
            + self.num_reviewers
            + self.additions
            + self.deletions
            + self.based_on_main_branch
            + self.blame
            + self.requested
            + self.codeowner
    }
}

impl std::fmt::Display for Pr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} - {} {}", self.url, self.title, self.labels,)
    }
}

impl std::fmt::Display for Labels {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<String>>()
                .join(" "),
        )
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "({})", self.name,)
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct Review {
    pub state: ReviewState,
    pub url: String,
    pub pr_title: String,
}

#[derive(Debug, Serialize, Clone)]
pub enum ReviewState {
    Dismissed,
    WithAddressedConversations,
}

impl std::fmt::Display for ReviewState {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let text = match self {
            ReviewState::Dismissed => "Dismissed",
            ReviewState::WithAddressedConversations => "With addressed conversations",
        };
        write!(f, "{text}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pr_fmt_method() {
        let pr = Pr {
            title: "Some important changes".to_string(),
            url: "https://github.com/frisoft/ateam/pull/1".to_string(),
            last_commit_pushed_date: None,
            last_commit_age_min: None,
            tests_result: TestsState::Success,
            open_conversations: 0,
            num_approvals: 1,
            num_reviewers: 2,
            additions: 1000,
            deletions: 999,
            based_on_main_branch: true,
            files: Files(vec![]),
            blame: false,
            labels: Labels(vec![]),
            requested: true,
            codeowner: false,
        };

        assert_eq!(
            format!("{pr}"),
            "https://github.com/frisoft/ateam/pull/1 - Some important changes ",
        );
    }
}
