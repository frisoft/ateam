use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub enum Command {
    Pr {
        #[structopt(name = "repository")]
        repo: String,
        #[structopt(long, short, help = "Add debug information")]
        debug: bool,
        #[structopt(long, short, help = "Number of pull requests to display")]
        num: Option<usize>,
        #[structopt(long, short, help = "Short version. No table")]
        short: bool,
    },
}

pub fn command() -> Command {
    Command::from_args()
}
