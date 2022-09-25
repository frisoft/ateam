use clap::Parser;

#[derive(Parser, Debug)]
#[clap(version, author, about)]
pub struct Ateam {
    #[clap(subcommand)]
    pub cmd: Command,
}

#[derive(clap::Subcommand, Debug)]
#[allow(clippy::large_enum_variant)]
pub enum Command {
    Pr(Pr),
    Followup(Followup),
}

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Pr {
    #[clap(long, short, help = "Add debug information")]
    pub debug: bool,
    #[clap(
        long,
        short,
        name = "repository",
        help = "Repositiy. Can be used multiple times to select more than one"
    )]
    pub repo: Vec<String>,
    #[clap(
        long,
        name = "organization",
        help = "Selest all the repositoris of the organization"
    )]
    pub org: Option<String>,
    #[clap(long, short, help = "Number of pull requests to display")]
    pub num: Option<usize>,
    #[clap(long, short, help = "Short version. No table")]
    pub short: bool,
    #[clap(long, help = "Output in JSON")]
    pub json: bool,
    #[clap(long, help = "Filter by label. Can be used multiple times")]
    pub label: Vec<String>,
    #[clap(
        long,
        help = "Exclude pull requests with this label. Can be used multiple times"
    )]
    pub exclude_label: Vec<String>,
    #[clap(long, short, help = "GitHub query. Can be used multiple times")]
    pub query: Vec<String>,
    #[clap(long, help = "Regexp filter on titles")]
    pub regex: Option<String>,
    #[clap(long, help = "Regexp filter on titles to exclude pull requests")]
    pub regex_not: Option<String>,
    #[clap(long, help = "Include pull requests I have reviewed")]
    pub include_reviewed_by_me: bool,
    #[clap(long, help = "Include my pull requests")]
    pub include_mine: bool,
    #[clap(
        long,
        help = "select only my pull requests (enables --include-reviewed-by-me automatically)"
    )]
    pub only_mine: bool,
    #[clap(
        long,
        help = "Select pull requests I have been requested to review, explicitly or as a code owner"
    )]
    pub requested: bool,
    #[clap(long, help = "Include draft pull requests")]
    pub include_drafts: bool,
    #[clap(long, help = "Include pull requests with pending tests")]
    pub include_tests_pending: bool,
    #[clap(long, help = "Include pull requests with tests failure")]
    pub include_tests_failure: bool,
    #[clap(long, help = "Exclude pull requests without tests")]
    pub exclude_tests_none: bool,
    #[clap(long, help = "Exclude pull requests with tests successful")]
    pub exclude_tests_success: bool,
    #[clap(long, help = "Select tests via regexp. The others are ignored")]
    pub tests_regex: Option<String>,
    #[clap(long, help = "Number of required approvals", default_value = "2")]
    pub required_approvals: u8,
    #[clap(long, help = "Look if I changed the same files in the past (SLOW)")]
    pub blame: bool,
    #[clap(long, help = "Query for another user")]
    pub user: Option<String>,
    #[clap(
        long,
        help = "Mumber of pull requests requested per batch",
        default_value = "30"
    )]
    pub batch_size: u8,
}

#[derive(Parser, Debug)]
#[clap(version)]
pub struct Followup {
    #[clap(long, help = "Output in JSON")]
    pub json: bool,
    #[clap(long, help = "Query for another user")]
    pub user: Option<String>,
}

pub fn command() -> Ateam {
    Ateam::from_args()
}
