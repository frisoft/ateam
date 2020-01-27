use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(author, about)]
pub enum Command {
    Pr {
        #[structopt(name = "repository")]
        repo: String,
        #[structopt(long, short)]
        debug: bool,
        #[structopt(long, short)]
        num: Option<usize>,
    },
}

pub fn command() -> Command {
    Command::from_args()
}
