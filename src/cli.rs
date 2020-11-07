use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub struct Ateam {
    #[structopt(long, short, help = "Add debug information")]
    pub debug: bool,
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(StructOpt, Debug)]
pub enum Command {
    Pr(Pr),
}

#[derive(StructOpt, Debug)]
pub struct Pr {
    #[structopt(
        long,
        short,
        name = "repository",
        help = "Repositiy, can be used multiple times to select more than one"
    )]
    pub repo: Vec<String>,
    #[structopt(long, short, help = "Number of pull requests to display")]
    pub num: Option<usize>,
    #[structopt(long, short, help = "Short version. No table")]
    pub short: bool,
    #[structopt(long, short, help = "GitHub query")]
    pub query: Option<String>,
    #[structopt(long, help = "Exclude PRs I have reviewed")]
    pub exclude_reviewed_by_me: bool,
}

pub fn command() -> Ateam {
    Ateam::from_args()
}
