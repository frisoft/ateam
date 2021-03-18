use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub struct Ateam {
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Pr(Pr),
    Followup(Followup),
}

#[derive(StructOpt, Debug)]
pub struct Pr {
    #[structopt(long, short, help = "Add debug information")]
    pub debug: bool,
    #[structopt(
        long,
        short,
        name = "repository",
        help = "Repositiy. Can be used multiple times to select more than one"
    )]
    pub repo: Vec<String>,
    #[structopt(
        long,
        name = "organization",
        help = "Selest all the repositoris of the organization"
    )]
    pub org: Option<String>,
    #[structopt(long, short, help = "Number of pull requests to display")]
    pub num: Option<usize>,
    #[structopt(long, short, help = "Short version. No table")]
    pub short: bool,
    #[structopt(long, help = "Output in JSON")]
    pub json: bool,
    #[structopt(long, help = "Filter by label. Can be used multiple times")]
    pub label: Vec<String>,
    #[structopt(
        long,
        help = "Exclude pull requests with this label. Can be used multiple times"
    )]
    pub exclude_label: Vec<String>,
    #[structopt(long, short, help = "GitHub query")]
    pub query: Option<String>,
    #[structopt(long, help = "Regexp filter on titles")]
    pub regex: Option<String>,
    #[structopt(long, help = "Regexp filter on titles to exclude pull requests")]
    pub regex_not: Option<String>,
    #[structopt(long, help = "Include pull requests I have reviewed")]
    pub include_reviewed_by_me: bool,
    #[structopt(long, help = "Include my pull requests")]
    pub include_mine: bool,
    #[structopt(long, help = "select only my pull requests")]
    pub only_mine: bool,
    #[structopt(long, help = "Include pull requests with pending tests")]
    pub include_tests_pending: bool,
    #[structopt(long, help = "Include pull requests with tests failure")]
    pub include_tests_failure: bool,
    #[structopt(
        long,
        help = "Include pull requests with no tests executed (usually because of conflicts)"
    )]
    pub include_tests_none: bool,
    #[structopt(long, help = "Exclide pull requests with tests successful")]
    pub exclude_tests_success: bool,
    #[structopt(long, help = "Select tests via regexp. The others are ignored")]
    pub tests_regex: Option<String>,
    #[structopt(long, help = "Number of required approvals", default_value = "2")]
    pub required_approvals: u8,
    #[structopt(long, help = "Look if I changed the same files in the past (SLOW)")]
    pub blame: bool,
    #[structopt(long, help = "Query for another user")]
    pub user: Option<String>,
}

#[derive(StructOpt, Debug)]
pub struct Followup {
    #[structopt(long, help = "Output in JSON")]
    pub json: bool,
    #[structopt(long, help = "Query for another user")]
    pub user: Option<String>,
}

pub fn command() -> Ateam {
    Ateam::from_args()
}
