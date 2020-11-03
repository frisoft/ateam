use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub enum Command {
    Pr {
        #[structopt(long, short, name = "repository", help = "Repositiy, can be used multiple times to select more than one")]
        repo: Vec<String>,
        #[structopt(long, short, help = "Add debug information")]
        debug: bool,
        #[structopt(long, short, help = "Number of pull requests to display")]
        num: Option<usize>,
        #[structopt(long, short, help = "Short version. No table")]
        short: bool,
        #[structopt(long, short, help = "GitHub query")]
        query: Option<String>,
    },
}

pub fn command() -> Command {
    Command::from_args()
}
