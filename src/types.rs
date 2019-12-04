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
    pub score: f64,
}
