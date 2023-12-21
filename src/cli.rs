use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(version, propagate_version = true)]
pub struct Ateam {
    #[command(subcommand)]
    pub cmd: Command,
}

#[derive(Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    /// It shows which pull requests should be reviewed next.
    Pr(PrArgs),
    /// It gives you some information about the reviews you already submitted and need your attention.
    Followup(FollowupArgs),
}

#[derive(Args, Debug)]
pub struct PrArgs {
    /// Repositiy. Can be used multiple times to select more than one
    #[arg(long, short, name = "repository")]
    pub repo: Vec<String>,
    /// Select all the repositories of the organization
    #[arg(long, name = "organization")]
    pub org: Option<String>,
    /// GitHub query. Can be used multiple times
    #[arg(long, short)]
    pub query: Vec<String>,
    /// Number of pull requests to display
    #[arg(long, short)]
    pub num: Option<usize>,
    /// Short version. No table
    #[arg(long, short)]
    pub short: bool,
    /// Output in JSON
    #[arg(long)]
    pub json: bool,
    /// Filter by label. Can be used multiple times
    #[arg(long)]
    pub label: Vec<String>,
    /// Exclude pull requests with this label. Can be used multiple times
    #[arg(long)]
    pub exclude_label: Vec<String>,
    /// Regexp filter on titles
    #[arg(long)]
    pub regex: Option<String>,
    /// Regexp filter on titles to exclude pull requests
    #[arg(long)]
    pub regex_not: Option<String>,
    /// Include pull requests I have reviewed
    #[arg(long)]
    pub include_reviewed_by_me: bool,
    /// Include my pull requests
    #[arg(long)]
    pub include_mine: bool,
    /// select only my pull requests (enables --include-reviewed-by-me automatically)
    #[arg(long)]
    pub only_mine: bool,
    /// Select pull requests I have been requested to review, explicitly or as a code owner
    #[arg(long)]
    pub requested: bool,
    /// Include draft pull requests
    #[arg(long)]
    pub include_drafts: bool,
    /// Include pull requests with pending tests
    #[arg(long)]
    pub include_tests_pending: bool,
    /// Include pull requests with tests failure
    #[arg(long)]
    pub include_tests_failure: bool,
    /// Exclude pull requests without tests
    #[arg(long)]
    pub exclude_tests_none: bool,
    /// Exclude pull requests with tests pending
    #[arg(long)]
    pub exclude_tests_success: bool,
    /// Select tests via regexp. The others are ignored
    #[arg(long)]
    pub tests_regex: Option<String>,
    /// Number of required approvals
    #[arg(long, default_value = "2")]
    pub required_approvals: u8,
    /// Look if I changed the same files in the past (slower)
    #[arg(long)]
    pub blame: bool,
    /// Query for another user
    #[arg(long)]
    pub user: Option<String>,
    /// Mumber of pull requests requested per batch
    #[arg(long, default_value = "30")]
    pub batch_size: u8,
    /// Add debug information
    #[arg(long, short)]
    pub debug: bool,
}

#[derive(Args, Debug)]
pub struct FollowupArgs {
    /// Output in JSON
    #[arg(long)]
    pub json: bool,
    /// Query for another user
    #[arg(long)]
    pub user: Option<String>,
}

pub fn command() -> Ateam {
    Ateam::parse()
}
