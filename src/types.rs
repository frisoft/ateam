use chrono::prelude::{DateTime, Utc};

pub struct Pr {
    pub title: String,
    pub url: String,
    pub last_commit_pushed_date: Option<DateTime<Utc>>,
    pub tests_result: i64,
    pub open_conversations: i64,
    pub num_approvals: i64,
    pub num_reviewers: i64,
    pub additions: i64,
    pub deletions: i64,
}

pub struct ScoredPr {
    pub pr: Pr,
    pub score: Score,
}

#[derive(Debug)]
pub struct Score {
    pub age: f64,
    pub tests_result: f64,
    pub open_conversations: f64,
    pub num_approvals: f64,
    pub num_reviewers: f64,
    pub additions: f64,
    pub deletions: f64,
}

fn age(date_time: Option<DateTime<Utc>>) -> i64 {
    match date_time {
        Some(date_time) => (Utc::now() - date_time).num_hours(),
        None => 0,
    }
}

impl Score {
    pub fn from_pr(pr: &Pr) -> Score {
        Score {
            age: age(pr.last_commit_pushed_date) as f64 * 10.0,
            tests_result: (pr.tests_result - 1) as f64 * -200.0,
            open_conversations: pr.open_conversations as f64 * -20.0,
            num_approvals: (pr.num_approvals ^ 2) as f64 * -50.0,
            num_reviewers: (pr.num_reviewers ^ 2) as f64 * -20.0,
            additions: pr.additions as f64 * -0.5,
            deletions: pr.deletions as f64 * -0.1,
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
    }
}
