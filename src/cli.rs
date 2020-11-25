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
    #[structopt(long, help = "Filter by label. Can be used multiple times")]
    pub label: Vec<String>,
    #[structopt(long, short, help = "GitHub query")]
    pub query: Option<String>,
    #[structopt(long, help = "Regexp filter on titles")]
    pub regex: Option<String>,
    #[structopt(long, help = "Include pull requests I have reviewed")]
    pub include_reviewed_by_me: bool,
    #[structopt(long, help = "Include my pull requests")]
    pub include_mine: bool,
    #[structopt(long, help = "Include pull requests with tests in progess")]
    pub include_tests_in_progress: bool,
    #[structopt(long, help = "Include pull requests with tests falure")]
    pub include_tests_failure: bool,
    #[structopt(long, help = "Number of required approvals", default_value = "2")]
    pub required_approvals: u8,
}

pub fn command() -> Ateam {
    Ateam::from_args()
}
